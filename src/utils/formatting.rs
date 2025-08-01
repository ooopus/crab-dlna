//! Formatting utilities for crab-dlna
//!
//! This module provides functions for formatting text and display strings,
//! particularly for device information and user interface elements.

/// Formats a device description for display
///
/// # Arguments
/// * `device_type` - The device type
/// * `friendly_name` - The friendly name of the device
/// * `url` - The device URL
///
/// # Returns
/// Returns a formatted string describing the device
pub fn format_device_description(device_type: &str, friendly_name: &str, url: &str) -> String {
    format!("[{device_type}] {friendly_name} @ {url}")
}

/// Formats a device description with service type for display
///
/// # Arguments
/// * `device_type` - The device type
/// * `service_type` - The service type
/// * `friendly_name` - The friendly name of the device
/// * `url` - The device URL
///
/// # Returns
/// Returns a formatted string describing the device with service information
pub fn format_device_with_service_description(
    device_type: &str,
    service_type: &str,
    friendly_name: &str,
    url: &str,
) -> String {
    format!(
        "[{device_type}][{service_type}] {friendly_name} @ {url}"
    )
}
