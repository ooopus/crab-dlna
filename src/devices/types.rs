//! Device-related types for crab-dlna
//!
//! This module contains type definitions for DLNA devices,
//! including render specifications and device information structures.

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

impl Default for PositionInfo {
    fn default() -> Self {
        Self {
            track: 0,
            track_duration: String::new(),
            track_meta_data: String::new(),
            track_uri: String::new(),
            rel_time: String::new(),
            abs_time: String::new(),
            rel_count: -1,
            abs_count: -1,
        }
    }
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
#[derive(Default)]
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
