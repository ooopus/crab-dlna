use crate::{
    config::{
        DLNA_ACTION_GET_POSITION_INFO, DLNA_ACTION_GET_TRANSPORT_INFO, DLNA_POSITION_INFO_PAYLOAD,
        DLNA_TRANSPORT_INFO_PAYLOAD, NO_DEVICES_DISCOVERED_MSG, RENDER_NOT_FOUND_MSG,
        SSDP_SEARCH_ATTEMPTS, SSDP_TTL,
    },
    error::{Error, Result},
    utils::{
        format_device_description, format_device_with_service_description, retry_with_backoff,
    },
};
use futures_util::stream::{Stream, StreamExt, TryStreamExt};
use http::Uri;
use log::{debug, info, warn};
use rupnp::ssdp::{SearchTarget, URN};
use std::collections::HashSet;
use std::time::Duration;

const AV_TRANSPORT: URN = URN::service("schemas-upnp-org", "AVTransport", 1);

macro_rules! format_device {
    ($device:expr) => {{
        format_device_description(
            &$device.device_type().to_string(),
            $device.friendly_name(),
            &$device.url().to_string(),
        )
    }};
}

/// A DLNA device which is capable of AVTransport actions.
#[derive(Debug, Clone)]
pub struct Render {
    /// The UPnP device
    pub device: rupnp::Device,
    /// The AVTransport service
    pub service: rupnp::Service,
}

/// An specification of a DLNA render device.
#[derive(Debug, Clone)]
pub enum RenderSpec {
    /// Render specified by a location URL
    Location(String),
    /// Render specified by a query string
    Query(u64, String),
    /// The first render found
    First(u64),
}

impl Render {
    /// Create a new render from render device specification.
    pub async fn new(render_spec: RenderSpec) -> Result<Self> {
        match &render_spec {
            RenderSpec::Location(device_url) => {
                info!("Render specified by location: {device_url}");
                Self::select_by_url(device_url)
                    .await?
                    .ok_or(Error::RenderNotFound {
                        spec: render_spec.clone(),
                        context: "Device not found at specified URL".to_string(),
                    })
            }
            RenderSpec::Query(timeout, device_query) => {
                info!("Render specified by query: {device_query}");
                Self::select_by_query(*timeout, device_query)
                    .await?
                    .ok_or(Error::RenderNotFound {
                        spec: render_spec.clone(),
                        context: format!("No device found matching query '{device_query}'"),
                    })
            }
            RenderSpec::First(timeout) => {
                info!("{RENDER_NOT_FOUND_MSG}");
                Ok(Self::discover(*timeout)
                    .await?
                    .first()
                    .ok_or(Error::RenderNotFound {
                        spec: render_spec.clone(),
                        context: NO_DEVICES_DISCOVERED_MSG.to_string(),
                    })?
                    .to_owned())
            }
        }
    }

    /// Discovers DLNA device with AVTransport on the network.
    pub async fn discover(duration_secs: u64) -> Result<Vec<Self>> {
        Self::discover_with_config(duration_secs, SSDP_SEARCH_ATTEMPTS, SSDP_TTL).await
    }

    /// Discovers DLNA devices with configurable SSDP parameters
    pub async fn discover_with_config(
        duration_secs: u64,
        search_attempts: usize,
        ttl: Option<u32>,
    ) -> Result<Vec<Self>> {
        info!("Discovering devices in the network, waiting {duration_secs} seconds...");
        let search_target = SearchTarget::URN(AV_TRANSPORT);
        let devices = upnp_discover_with_config(
            &search_target,
            Duration::from_secs(duration_secs),
            search_attempts,
            ttl,
        )
        .await?;

        pin_utils::pin_mut!(devices);

        let mut renders = Vec::new();
        let mut discovered_urls = HashSet::new();

        while let Some(result) = devices.next().await {
            match result {
                Ok(device) => {
                    let device_url = device.url().to_string();
                    if discovered_urls.contains(&device_url) {
                        debug!("Skipping duplicate device: {}", format_device!(device));
                        continue;
                    }

                    debug!("Found device: {}", format_device!(device));
                    if let Some(render) = Self::from_device(device).await {
                        discovered_urls.insert(device_url);
                        renders.push(render);
                    };
                }
                Err(e) => {
                    debug!("A device returned error while discovering it: {e}");
                }
            }
        }

        Ok(renders)
    }

    /// Returns the host of the render
    pub fn host(&self) -> String {
        self.device.url().authority().unwrap().host().to_string()
    }

    async fn select_by_url(url: &String) -> Result<Option<Self>> {
        debug!("Selecting device by url: {url}");
        let uri: Uri = url.parse().map_err(|e| Error::DeviceUrlParseError {
            url: url.to_owned(),
            reason: format!("Invalid URL format: {e}"),
        })?;

        let device = retry_with_backoff(
            || async { rupnp::Device::from_url(uri.clone()).await },
            &format!("Device creation from URL {url}"),
        )
        .await
        .map_err(|err| Error::DeviceCreationError {
            url: url.to_owned(),
            source: err,
        })?;

        Ok(Self::from_device(device).await)
    }

    async fn select_by_query(duration_secs: u64, query: &String) -> Result<Option<Self>> {
        debug!("Selecting device by query: '{query}'");
        for render in Self::discover(duration_secs).await? {
            let render_str = render.to_string();
            if render_str.contains(query.as_str()) {
                return Ok(Some(render));
            }
        }
        Ok(None)
    }

    async fn from_device(device: rupnp::Device) -> Option<Self> {
        debug!(
            "Retrieving AVTransport service from device '{}'",
            format_device!(device)
        );
        match device.find_service(&AV_TRANSPORT) {
            Some(service) => Some(Self {
                device: device.clone(),
                service: service.clone(),
            }),
            None => {
                warn!("No AVTransport service found on {}", device.friendly_name());
                None
            }
        }
    }

    /// Gets current playback position information
    ///
    /// This method calls the DLNA AVTransport service's GetPositionInfo operation,
    /// returning detailed information about the current playback position, including time position and track information
    pub async fn get_position_info(&self) -> Result<PositionInfo> {
        let payload = DLNA_POSITION_INFO_PAYLOAD;

        let response = self
            .service
            .action(self.device.url(), DLNA_ACTION_GET_POSITION_INFO, payload)
            .await
            .map_err(|err| Error::DlnaActionFailed {
                action: DLNA_ACTION_GET_POSITION_INFO.to_string(),
                source: err,
            })?;

        PositionInfo::from_map(&response).map_err(|err| Error::DlnaResponseParseError {
            action: DLNA_ACTION_GET_POSITION_INFO.to_string(),
            error: err,
        })
    }

    /// Gets transport information (playback status, etc.)
    ///
    /// This method calls the DLNA AVTransport service's GetTransportInfo operation,
    /// returning the current transport state such as playing, paused, etc.
    pub async fn get_transport_info(&self) -> Result<TransportInfo> {
        let payload = DLNA_TRANSPORT_INFO_PAYLOAD;

        let response = self
            .service
            .action(self.device.url(), DLNA_ACTION_GET_TRANSPORT_INFO, payload)
            .await
            .map_err(|err| Error::DlnaActionFailed {
                action: DLNA_ACTION_GET_TRANSPORT_INFO.to_string(),
                source: err,
            })?;

        TransportInfo::from_map(&response).map_err(|err| Error::DlnaResponseParseError {
            action: DLNA_ACTION_GET_TRANSPORT_INFO.to_string(),
            error: err,
        })
    }
}

impl std::fmt::Display for Render {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format_device_with_service_description(
                &self.device.device_type().to_string(),
                &self.service.service_type().to_string(),
                self.device.friendly_name(),
                &self.device.url().to_string()
            )
        )
    }
}

async fn upnp_discover(
    search_target: &SearchTarget,
    timeout: Duration,
    ttl: Option<u32>,
) -> Result<impl Stream<Item = Result<rupnp::Device, rupnp::Error>>> {
    upnp_discover_with_config(search_target, timeout, SSDP_SEARCH_ATTEMPTS, ttl).await
}

async fn upnp_discover_with_config(
    search_target: &SearchTarget,
    timeout: Duration,
    search_attempts: usize,
    ttl: Option<u32>,
) -> Result<impl Stream<Item = Result<rupnp::Device, rupnp::Error>>> {
    Ok(
        ssdp_client::search(search_target, timeout, search_attempts, ttl)
            .await?
            .map_err(rupnp::Error::SSDPError)
            .map(|res| Ok(res?.location().parse()?))
            .and_then(rupnp::Device::from_url),
    )
}

/// Playback position information
///
/// Contains all information returned by the GetPositionInfo operation
#[derive(Debug, Clone)]
pub struct PositionInfo {
    /// Current track number
    pub track: u32,
    /// Total duration of current track (format: HH:MM:SS)
    pub track_duration: String,
    /// Metadata of current track
    pub track_meta_data: String,
    /// URI of current track
    pub track_uri: String,
    /// Relative time position (format: HH:MM:SS)
    pub rel_time: String,
    /// Absolute time position
    pub abs_time: String,
    /// Relative count position
    pub rel_count: i32,
    /// Absolute count position
    pub abs_count: i32,
}

impl PositionInfo {
    /// Parses PositionInfo from HashMap response
    pub fn from_map(map: &std::collections::HashMap<String, String>) -> Result<Self, String> {
        Ok(PositionInfo {
            track: map
                .get("Track")
                .unwrap_or(&"0".to_string())
                .parse()
                .map_err(|e| format!("Failed to parse Track: {e}"))?,
            track_duration: map.get("TrackDuration").unwrap_or(&"".to_string()).clone(),
            track_meta_data: map.get("TrackMetaData").unwrap_or(&"".to_string()).clone(),
            track_uri: map.get("TrackURI").unwrap_or(&"".to_string()).clone(),
            rel_time: map.get("RelTime").unwrap_or(&"".to_string()).clone(),
            abs_time: map.get("AbsTime").unwrap_or(&"".to_string()).clone(),
            rel_count: map
                .get("RelCount")
                .unwrap_or(&"-1".to_string())
                .parse()
                .map_err(|e| format!("Failed to parse RelCount: {e}"))?,
            abs_count: map
                .get("AbsCount")
                .unwrap_or(&"-1".to_string())
                .parse()
                .map_err(|e| format!("Failed to parse AbsCount: {e}"))?,
        })
    }
}

/// Transport information
///
/// Contains information returned by the GetTransportInfo operation
#[derive(Debug, Clone)]
pub struct TransportInfo {
    /// Transport state (e.g., PLAYING, PAUSED_PLAYBACK, STOPPED)
    pub transport_state: String,
    /// Detailed transport status information
    pub transport_status: String,
    /// Playback speed
    pub speed: String,
}

impl TransportInfo {
    /// Parses TransportInfo from HashMap response
    pub fn from_map(map: &std::collections::HashMap<String, String>) -> Result<Self, String> {
        Ok(TransportInfo {
            transport_state: map
                .get("CurrentTransportState")
                .unwrap_or(&"".to_string())
                .clone(),
            transport_status: map
                .get("CurrentTransportStatus")
                .unwrap_or(&"".to_string())
                .clone(),
            speed: map.get("CurrentSpeed").unwrap_or(&"".to_string()).clone(),
        })
    }
}
