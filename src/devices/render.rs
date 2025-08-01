//! DLNA render device implementation for crab-dlna
//!
//! This module contains the main Render struct and its implementation
//! for interacting with DLNA devices.

use crate::{
    config::{
        DLNA_ACTION_GET_POSITION_INFO, DLNA_ACTION_GET_TRANSPORT_INFO, DLNA_POSITION_INFO_PAYLOAD,
        DLNA_TRANSPORT_INFO_PAYLOAD, NO_DEVICES_DISCOVERED_MSG, RENDER_NOT_FOUND_MSG,
    },
    error::{Error, Result},
    utils::{format_device_with_service_description, retry_with_backoff},
};
use http::Uri;
use log::{debug, info};

use super::types::{PositionInfo, RenderSpec, TransportInfo};

/// A DLNA device which is capable of AVTransport actions.
#[derive(Debug, Clone)]
pub struct Render {
    /// The UPnP device
    pub device: rupnp::Device,
    /// The AVTransport service
    pub service: rupnp::Service,
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

    /// Returns the host of the render
    pub fn host(&self) -> String {
        self.device.url().authority().unwrap().host().to_string()
    }

    /// Selects a device by URL
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
