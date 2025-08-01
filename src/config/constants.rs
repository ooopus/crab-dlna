//! Configuration constants for crab-dlna
//!
//! This module contains all hardcoded constants used throughout the application,
//! organized by functionality and following Rust naming conventions.

// =============================================================================
// Network and Streaming Constants
// =============================================================================

/// Default port for the media streaming server
pub const DEFAULT_STREAMING_PORT: u32 = 9000;

/// Default timeout for device discovery in seconds
pub const DEFAULT_DISCOVERY_TIMEOUT: u64 = 5;

/// Maximum number of retries for network operations
pub const MAX_NETWORK_RETRIES: u32 = 3;

/// TTL (Time To Live) for SSDP multicast packets
pub const SSDP_TTL: Option<u32> = Some(3);

/// User agent string for HTTP requests
pub const USER_AGENT: &str = concat!("crab-dlna/", env!("CARGO_PKG_VERSION"));

// =============================================================================
// DLNA Protocol Constants
// =============================================================================

/// DLNA payload template for position info action
pub const DLNA_POSITION_INFO_PAYLOAD: &str = r#"<InstanceID>0</InstanceID>"#;

/// DLNA payload template for transport info action
pub const DLNA_TRANSPORT_INFO_PAYLOAD: &str = r#"<InstanceID>0</InstanceID>"#;

/// DLNA instance ID used in payloads
pub const DLNA_INSTANCE_ID: u32 = 0;

/// DLNA default playback speed
pub const DLNA_DEFAULT_SPEED: u32 = 1;

// =============================================================================
// Media File Support Constants
// =============================================================================

/// Supported video file extensions
pub const SUPPORTED_VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "3gp", "ogv",
];

/// Supported audio file extensions
pub const SUPPORTED_AUDIO_EXTENSIONS: &[&str] =
    &["mp3", "wav", "flac", "aac", "ogg", "wma", "m4a", "opus"];

// =============================================================================
// Subtitle and Synchronization Constants
// =============================================================================

/// Default interval for subtitle synchronization checks in milliseconds
pub const DEFAULT_SUBTITLE_SYNC_INTERVAL_MS: u64 = 500;

// =============================================================================
// Logging Constants
// =============================================================================

/// Environment variable name for custom log level
pub const LOG_LEVEL_ENV_VAR: &str = "CRABDLNA_LOG";

// =============================================================================
// Device Discovery Constants
// =============================================================================

/// SSDP search attempts used in upnp_discover function
pub const SSDP_SEARCH_ATTEMPTS: usize = 3;

// =============================================================================
// Error and Status Messages
// =============================================================================

/// Error message when no devices are discovered
pub const NO_DEVICES_DISCOVERED_MSG: &str = "No devices discovered in the network";

/// Error message for render device not found
pub const RENDER_NOT_FOUND_MSG: &str = "No render specified, selecting first one";

/// Error message for invalid socket address format
pub const INVALID_SOCKET_ADDRESS_MSG: &str = "Invalid socket address format";

/// Error message for failed media playback
pub const MEDIA_PLAYBACK_FAILED_MSG: &str = "Failed to start media playback on render device";

// =============================================================================
// DLNA Action Names
// =============================================================================

/// DLNA action name for setting AV transport URI
pub const DLNA_ACTION_SET_AV_TRANSPORT_URI: &str = "SetAVTransportURI";

/// DLNA action name for play
pub const DLNA_ACTION_PLAY: &str = "Play";

/// DLNA action name for pause
pub const DLNA_ACTION_PAUSE: &str = "Pause";

/// DLNA action name for getting position info
pub const DLNA_ACTION_GET_POSITION_INFO: &str = "GetPositionInfo";

/// DLNA action name for getting transport info
pub const DLNA_ACTION_GET_TRANSPORT_INFO: &str = "GetTransportInfo";

// =============================================================================
// Logging Messages
// =============================================================================

/// Log message for setting video URI
pub const LOG_MSG_SETTING_VIDEO_URI: &str = "Setting Video URI";

/// Log message for playing video
pub const LOG_MSG_PLAYING_VIDEO: &str = "Playing video";

/// Log message for no subtitle file
pub const LOG_MSG_NO_SUBTITLE_FILE: &str = "No subtitle file";

/// Log message for list devices command
pub const LOG_MSG_LIST_DEVICES: &str = "List devices";

// =============================================================================
// DLNA Metadata Constants
// =============================================================================

/// Default DLNA video title
pub const DEFAULT_DLNA_VIDEO_TITLE: &str = "crab-dlna Video";
