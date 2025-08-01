//! Device discovery utilities for crab-dlna
//!
//! This module provides functions for discovering DLNA devices on the network
//! using SSDP (Simple Service Discovery Protocol).

use crate::{
    config::{SSDP_SEARCH_ATTEMPTS, SSDP_TTL},
    error::Result,
    utils::format_device_description,
};
use futures_util::stream::{Stream, StreamExt, TryStreamExt};
use log::{debug, info};
use rupnp::ssdp::{SearchTarget, URN};
use std::{collections::HashSet, time::Duration};

use super::render::Render;

/// UPnP service URN for AVTransport
pub const AV_TRANSPORT: URN = URN::service("schemas-upnp-org", "AVTransport", 1);

/// Macro for formatting device information
macro_rules! format_device {
    ($device:expr) => {{
        format_device_description(
            &$device.device_type().to_string(),
            $device.friendly_name(),
            &$device.url().to_string(),
        )
    }};
}

impl Render {
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

        let mut devices = std::pin::pin!(devices);

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

    /// Selects a device by query string
    pub(super) async fn select_by_query(
        duration_secs: u64,
        query: &String,
    ) -> Result<Option<Self>> {
        debug!("Selecting device by query: '{query}'");
        for render in Self::discover(duration_secs).await? {
            let render_str = render.to_string();
            if render_str.contains(query.as_str()) {
                return Ok(Some(render));
            }
        }
        Ok(None)
    }

    /// Creates a Render from a UPnP device if it has AVTransport service
    pub(super) async fn from_device(device: rupnp::Device) -> Option<Self> {
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
                log::warn!("No AVTransport service found on {}", device.friendly_name());
                None
            }
        }
    }
}

/// Discovers UPnP devices with configurable parameters
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
