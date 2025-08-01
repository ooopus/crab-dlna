use crate::devices::RenderSpec;
use std::fmt;

/// Errors that can happen inside crab-dlna
#[derive(Debug)]
pub enum Error {
    // Device discovery and management errors
    /// Failed to discover DLNA devices on the network
    DeviceDiscoveryFailed {
        /// The underlying UPnP error
        source: rupnp::Error,
        /// Additional context about the discovery attempt
        context: String,
    },
    /// Failed to parse a device URL
    DeviceUrlParseError {
        /// The invalid URL that failed to parse
        url: String,
        /// Additional context about why parsing failed
        reason: String,
    },
    /// Failed to create a device from URL
    DeviceCreationError {
        /// The URL that failed to create a device
        url: String,
        /// The underlying UPnP error
        source: rupnp::Error,
    },
    /// The specified render device was not found
    RenderNotFound {
        /// The render specification that was searched for
        spec: RenderSpec,
        /// Additional context about the search
        context: String,
    },

    // Streaming and network errors
    /// Failed to parse host or IP address
    NetworkAddressParseError {
        /// The address that failed to parse
        address: String,
        /// The reason for the parsing failure
        reason: String,
    },
    /// Media file does not exist or is not accessible
    MediaFileNotFound {
        /// Path to the missing file
        path: String,
        /// Additional context about the file access attempt
        context: String,
    },
    /// Failed to connect to remote render device
    RenderConnectionFailed {
        /// The host that failed to connect
        host: String,
        /// The underlying I/O error
        source: std::io::Error,
    },
    /// Failed to identify local IP address
    LocalAddressResolutionFailed {
        /// The underlying error from local IP detection
        source: local_ip_address::Error,
        /// Additional context about the resolution attempt
        context: String,
    },

    // DLNA protocol errors
    /// Failed to set AV transport URI on the render
    DlnaSetTransportUriFailed {
        /// The underlying UPnP error
        source: rupnp::Error,
        /// The URI that failed to be set
        uri: String,
    },
    /// Failed to start playback on the render
    DlnaPlaybackFailed {
        /// The underlying UPnP error
        source: rupnp::Error,
        /// Additional context about the playback attempt
        context: String,
    },
    /// Failed to execute a DLNA action
    DlnaActionFailed {
        /// The action that failed
        action: String,
        /// The underlying UPnP error
        source: rupnp::Error,
    },
    /// Failed to parse response from DLNA device
    DlnaResponseParseError {
        /// The action that generated the response
        action: String,
        /// The parsing error message
        error: String,
    },

    // Streaming server errors
    /// Media streaming server encountered an error
    StreamingServerError {
        /// The underlying task join error
        source: tokio::task::JoinError,
        /// Additional context about the streaming failure
        context: String,
    },

    // Subtitle synchronization errors
    /// Subtitle synchronization encountered an error
    SubtitleSyncError {
        /// The error message
        message: String,
        /// Additional context about the subtitle operation
        context: String,
    },

    // Keyboard input errors
    /// Keyboard input handling encountered an error
    KeyboardError {
        /// The error message
        message: String,
    },

    // Template rendering errors
    /// Template rendering encountered an error
    TemplateRenderError {
        /// The name of the template that failed to render
        template_name: String,
        /// The underlying template error
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DeviceDiscoveryFailed { source, context } => {
                write!(f, "Failed to discover devices: {source} ({context})")
            }
            Error::DeviceUrlParseError { url, reason } => {
                write!(f, "Failed to parse URL '{url}': {reason}")
            }
            Error::DeviceCreationError { url, source } => {
                write!(f, "Failed to create device from '{url}': {source}")
            }
            Error::RenderNotFound { spec, context } => match spec {
                RenderSpec::Location(device_url) => {
                    write!(f, "No render found at '{device_url}': {context}")
                }
                RenderSpec::Query(timeout, device_query) => write!(
                    f,
                    "No render found within {timeout} seconds with query '{device_query}': {context}"
                ),
                RenderSpec::First(timeout) => {
                    write!(f, "No render found within {timeout} seconds: {context}")
                }
            },
            Error::NetworkAddressParseError { address, reason } => {
                write!(f, "Failed to parse network address '{address}': {reason}")
            }
            Error::MediaFileNotFound { path, context } => {
                write!(f, "Media file '{path}' not found: {context}")
            }
            Error::RenderConnectionFailed { host, source } => {
                write!(f, "Failed to connect to render '{host}': {source}")
            }
            Error::LocalAddressResolutionFailed { source, context } => {
                write!(f, "Failed to resolve local address: {source} ({context})")
            }
            Error::DlnaSetTransportUriFailed { source, uri } => {
                write!(f, "Failed to set transport URI '{uri}': {source}")
            }
            Error::DlnaPlaybackFailed { source, context } => {
                write!(f, "Failed to start playback: {source} ({context})")
            }
            Error::DlnaActionFailed { action, source } => {
                write!(f, "Failed to execute DLNA action '{action}': {source}")
            }
            Error::DlnaResponseParseError { action, error } => {
                write!(
                    f,
                    "Failed to parse response from action '{action}': {error}"
                )
            }
            Error::StreamingServerError { source, context } => {
                write!(f, "Streaming server error: {source} ({context})")
            }
            Error::SubtitleSyncError { message, context } => {
                write!(f, "Subtitle synchronization error: {message} ({context})")
            }
            Error::KeyboardError { message } => {
                write!(f, "Keyboard input error: {message}")
            }
            Error::TemplateRenderError {
                template_name,
                source,
            } => {
                write!(f, "Failed to render template '{template_name}': {source}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::DeviceDiscoveryFailed { source, .. } => Some(source),
            Error::DeviceCreationError { source, .. } => Some(source),
            Error::RenderConnectionFailed { source, .. } => Some(source),
            Error::LocalAddressResolutionFailed { source, .. } => Some(source),
            Error::DlnaSetTransportUriFailed { source, .. } => Some(source),
            Error::DlnaPlaybackFailed { source, .. } => Some(source),
            Error::DlnaActionFailed { source, .. } => Some(source),
            Error::StreamingServerError { source, .. } => Some(source),
            Error::TemplateRenderError { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<ssdp_client::Error> for Error {
    fn from(err: ssdp_client::Error) -> Self {
        Error::DeviceDiscoveryFailed {
            source: rupnp::Error::SSDPError(err),
            context: "SSDP discovery failed".to_string(),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::RenderSpec;
    use std::error::Error as StdError;

    #[test]
    fn test_error_display() {
        let error = Error::DeviceDiscoveryFailed {
            source: rupnp::Error::ParseError("test"),
            context: "test context".to_string(),
        };
        assert!(error.to_string().contains("Failed to discover devices"));
        assert!(error.to_string().contains("test context"));
    }

    #[test]
    fn test_render_not_found_error() {
        let spec = RenderSpec::Query(5, "test".to_string());
        let error = Error::RenderNotFound {
            spec,
            context: "test context".to_string(),
        };
        assert!(
            error
                .to_string()
                .contains("No render found within 5 seconds")
        );
        assert!(error.to_string().contains("test"));
    }

    #[test]
    fn test_network_address_parse_error() {
        let error = Error::NetworkAddressParseError {
            address: "invalid:address".to_string(),
            reason: "Invalid format".to_string(),
        };
        assert!(
            error
                .to_string()
                .contains("Failed to parse network address")
        );
        assert!(error.to_string().contains("invalid:address"));
    }

    #[test]
    fn test_subtitle_sync_error() {
        let error = Error::SubtitleSyncError {
            message: "Failed to sync".to_string(),
            context: "test context".to_string(),
        };
        assert!(error.to_string().contains("Subtitle synchronization error"));
        assert!(error.to_string().contains("Failed to sync"));
    }

    #[test]
    fn test_error_source() {
        let source_error = rupnp::Error::ParseError("test");
        let error = Error::DeviceDiscoveryFailed {
            source: source_error,
            context: "test".to_string(),
        };
        assert!(StdError::source(&error).is_some());
    }
}
