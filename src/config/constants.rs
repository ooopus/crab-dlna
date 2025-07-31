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

/// Supported subtitle file extensions in order of preference
pub const SUPPORTED_SUBTITLE_EXTENSIONS: &[&str] = &["srt", "ass", "ssa"];

// =============================================================================
// Subtitle and Synchronization Constants
// =============================================================================

/// Default interval for subtitle synchronization checks in milliseconds
pub const DEFAULT_SUBTITLE_SYNC_INTERVAL_MS: u64 = 500;

/// Default subtitle file name when no subtitle is available
pub const DEFAULT_SUBTITLE_FILENAME: &str = "dummy.srt";

// =============================================================================
// Logging Constants
// =============================================================================

/// Environment variable name for custom log level
pub const LOG_LEVEL_ENV_VAR: &str = "CRABDLNA_LOG";

// =============================================================================
// Device Discovery Constants
// =============================================================================

/// Default number of SSDP search attempts
pub const DEFAULT_SSDP_SEARCH_ATTEMPTS: u32 = 3;

/// Default TTL for SSDP discovery packets
pub const DEFAULT_SSDP_TTL: u32 = 4;

/// SSDP search attempts used in upnp_discover function
pub const SSDP_SEARCH_ATTEMPTS: usize = 3;

// =============================================================================
// Error and Status Messages
// =============================================================================

/// Error message when no devices are discovered
pub const NO_DEVICES_DISCOVERED_MSG: &str = "No devices discovered in the network";

/// Error message for render device not found
pub const RENDER_NOT_FOUND_MSG: &str = "No render specified, selecting first one";

/// Error message for failed clipboard initialization
pub const CLIPBOARD_INIT_FAILED_MSG: &str = "Warning: Failed to initialize clipboard";

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

/// DLNA action name for getting position info
pub const DLNA_ACTION_GET_POSITION_INFO: &str = "GetPositionInfo";

/// DLNA action name for getting transport info
pub const DLNA_ACTION_GET_TRANSPORT_INFO: &str = "GetTransportInfo";

// =============================================================================
// Logging Messages
// =============================================================================

/// Log message for device discovery start
pub const LOG_MSG_DISCOVERING_DEVICES: &str =
    "Discovering devices in the network, waiting {} seconds...";

/// Log message for setting video URI
pub const LOG_MSG_SETTING_VIDEO_URI: &str = "Setting Video URI";

/// Log message for playing video
pub const LOG_MSG_PLAYING_VIDEO: &str = "Playing video";

/// Log message for video file serving
pub const LOG_MSG_VIDEO_FILE: &str = "Video file: {}";

/// Log message for subtitle file serving
pub const LOG_MSG_SUBTITLE_FILE: &str = "Subtitle file: {}";

/// Log message for no subtitle file
pub const LOG_MSG_NO_SUBTITLE_FILE: &str = "No subtitle file";

/// Log message for list devices command
pub const LOG_MSG_LIST_DEVICES: &str = "List devices";

// =============================================================================
// DLNA Metadata Templates
// =============================================================================

/// DLNA metadata template for video with subtitles
pub const DLNA_VIDEO_METADATA_TEMPLATE: &str = r###"
<DIDL-Lite xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/"
            xmlns:dc="http://purl.org/dc/elements/1.1/" 
            xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/" 
            xmlns:dlna="urn:schemas-dlna-org:metadata-1-0/" 
            xmlns:sec="http://www.sec.co.kr/" 
            xmlns:xbmc="urn:schemas-xbmc-org:metadata-1-0/">
    <item id="0" parentID="-1" restricted="1">
        <dc:title>crab-dlna Video</dc:title>
        <res protocolInfo="http-get:*:video/{type_video}:" xmlns:pv="http://www.pv.com/pvns/" pv:subtitleFileUri="{uri_sub}" pv:subtitleFileType="{type_sub}">{uri_video}</res>
        <res protocolInfo="http-get:*:text/srt:*">{uri_sub}</res>
        <res protocolInfo="http-get:*:smi/caption:*">{uri_sub}</res>
        <sec:CaptionInfoEx sec:type="{type_sub}">{uri_sub}</sec:CaptionInfoEx>
        <sec:CaptionInfo sec:type="{type_sub}">{uri_sub}</sec:CaptionInfo>
        <upnp:class>object.item.videoItem.movie</upnp:class>
    </item>
</DIDL-Lite>
"###;

/// Default DLNA video title
pub const DEFAULT_DLNA_VIDEO_TITLE: &str = "crab-dlna Video";

/// DLNA UPnP class for video items
pub const DLNA_VIDEO_UPNP_CLASS: &str = "object.item.videoItem.movie";
