use crate::error::{Error, Result};
use futures_util::stream::{Stream, StreamExt, TryStreamExt};
use http::Uri;
use log::{debug, info, warn};
use rupnp::ssdp::{SearchTarget, URN};
use std::collections::HashSet;
use std::time::Duration;

const AV_TRANSPORT: URN = URN::service("schemas-upnp-org", "AVTransport", 1);

macro_rules! format_device {
    ($device:expr) => {{
        format!(
            "[{}] {} @ {}",
            $device.device_type(),
            $device.friendly_name(),
            $device.url()
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
                    .ok_or(Error::DevicesRenderNotFound(render_spec))
            }
            RenderSpec::Query(timeout, device_query) => {
                info!("Render specified by query: {device_query}");
                Self::select_by_query(*timeout, device_query)
                    .await?
                    .ok_or(Error::DevicesRenderNotFound(render_spec))
            }
            RenderSpec::First(timeout) => {
                info!("No render specified, selecting first one");
                Ok(Self::discover(*timeout)
                    .await?
                    .first()
                    .ok_or(Error::DevicesRenderNotFound(render_spec))?
                    .to_owned())
            }
        }
    }

    /// Discovers DLNA device with AVTransport on the network.
    pub async fn discover(duration_secs: u64) -> Result<Vec<Self>> {
        info!("Discovering devices in the network, waiting {duration_secs} seconds...");
        let search_target = SearchTarget::URN(AV_TRANSPORT);
        let devices =
            upnp_discover(&search_target, Duration::from_secs(duration_secs), Some(4)).await?;

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
        let uri: Uri = url
            .parse()
            .map_err(|_| Error::DevicesUrlParseError(url.to_owned()))?;

        let device = rupnp::Device::from_url(uri)
            .await
            .map_err(|err| Error::DevicesCreateError(url.to_owned(), err))?;

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

    /// 获取当前播放位置信息
    ///
    /// 此方法调用DLNA AVTransport服务的GetPositionInfo操作，
    /// 返回当前播放位置的详细信息，包括时间位置和轨道信息
    pub async fn get_position_info(&self) -> Result<PositionInfo> {
        let payload = r#"<InstanceID>0</InstanceID>"#;

        let response = self
            .service
            .action(self.device.url(), "GetPositionInfo", payload)
            .await
            .map_err(Error::DLNAActionError)?;

        PositionInfo::from_map(&response).map_err(Error::ParsingError)
    }

    /// 获取传输信息（播放状态等）
    ///
    /// 此方法调用DLNA AVTransport服务的GetTransportInfo操作，
    /// 返回当前传输状态，如播放、暂停等状态
    pub async fn get_transport_info(&self) -> Result<TransportInfo> {
        let payload = r#"<InstanceID>0</InstanceID>"#;

        let response = self
            .service
            .action(self.device.url(), "GetTransportInfo", payload)
            .await
            .map_err(Error::DLNAActionError)?;

        TransportInfo::from_map(&response).map_err(Error::ParsingError)
    }
}

impl std::fmt::Display for Render {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}][{}] {} @ {}",
            self.device.device_type(),
            self.service.service_type(),
            self.device.friendly_name(),
            self.device.url()
        )
    }
}

async fn upnp_discover(
    search_target: &SearchTarget,
    timeout: Duration,
    ttl: Option<u32>,
) -> Result<impl Stream<Item = Result<rupnp::Device, rupnp::Error>>> {
    Ok(ssdp_client::search(search_target, timeout, 3, ttl)
        .await?
        .map_err(rupnp::Error::SSDPError)
        .map(|res| Ok(res?.location().parse()?))
        .and_then(rupnp::Device::from_url))
}

/// 播放位置信息
///
/// 包含GetPositionInfo操作返回的所有信息
#[derive(Debug)]
pub struct PositionInfo {
    /// 当前轨道编号
    pub track: u32,
    /// 当前轨道的总时长 (格式: HH:MM:SS)
    pub track_duration: String,
    /// 当前轨道的元数据
    pub track_meta_data: String,
    /// 当前轨道的URI
    pub track_uri: String,
    /// 相对时间位置 (格式: HH:MM:SS)
    pub rel_time: String,
    /// 绝对时间位置
    pub abs_time: String,
    /// 相对计数位置
    pub rel_count: i32,
    /// 绝对计数位置
    pub abs_count: i32,
}

impl PositionInfo {
    /// 从HashMap响应解析PositionInfo
    pub fn from_map(map: &std::collections::HashMap<String, String>) -> Result<Self, String> {
        Ok(PositionInfo {
            track: map
                .get("Track")
                .unwrap_or(&"0".to_string())
                .parse()
                .map_err(|e| format!("Failed to parse Track: {}", e))?,
            track_duration: map.get("TrackDuration").unwrap_or(&"".to_string()).clone(),
            track_meta_data: map.get("TrackMetaData").unwrap_or(&"".to_string()).clone(),
            track_uri: map.get("TrackURI").unwrap_or(&"".to_string()).clone(),
            rel_time: map.get("RelTime").unwrap_or(&"".to_string()).clone(),
            abs_time: map.get("AbsTime").unwrap_or(&"".to_string()).clone(),
            rel_count: map
                .get("RelCount")
                .unwrap_or(&"-1".to_string())
                .parse()
                .map_err(|e| format!("Failed to parse RelCount: {}", e))?,
            abs_count: map
                .get("AbsCount")
                .unwrap_or(&"-1".to_string())
                .parse()
                .map_err(|e| format!("Failed to parse AbsCount: {}", e))?,
        })
    }
}

/// 传输信息
///
/// 包含GetTransportInfo操作返回的信息
#[derive(Debug)]
pub struct TransportInfo {
    /// 传输状态 (如: PLAYING, PAUSED_PLAYBACK, STOPPED)
    pub transport_state: String,
    /// 传输状态详细信息
    pub transport_status: String,
    /// 播放速度
    pub speed: String,
}

impl TransportInfo {
    /// 从HashMap响应解析TransportInfo
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
