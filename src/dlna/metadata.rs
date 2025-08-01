//! DLNA metadata generation for crab-dlna
//!
//! This module handles the creation of DLNA-compatible metadata XML
//! for media files, including subtitle support.

use crate::{
    config::{DEFAULT_DLNA_VIDEO_TITLE, DLNA_INSTANCE_ID},
    error::Result,
    media::MediaStreamingServer,
};
use askama::Template;
use xml::escape::escape_str_attribute;

/// Template context for DIDL-Lite metadata with subtitles
#[derive(Template)]
#[template(path = "didl_lite_with_subtitles.xml")]
struct DidlLiteWithSubtitlesTemplate {
    title: String,
    video_uri: String,
    video_type: String,
    subtitle_uri: String,
    subtitle_type: String,
}

/// Template context for DIDL-Lite metadata without subtitles
#[derive(Template)]
#[template(path = "didl_lite_without_subtitles.xml")]
struct DidlLiteWithoutSubtitlesTemplate {
    title: String,
    video_uri: String,
    video_type: String,
}

/// Template context for SetAVTransportURI payload
#[derive(Template)]
#[template(path = "set_av_transport_uri.xml")]
struct SetAvTransportUriTemplate {
    instance_id: u32,
    current_uri: String,
    current_uri_metadata: String,
}

/// Builds the metadata XML for the media content
pub fn build_metadata(streaming_server: &MediaStreamingServer) -> Result<String> {
    let subtitle_uri = streaming_server.subtitle_uri();

    let metadata = match subtitle_uri {
        Some(subtitle_uri) => {
            let template = DidlLiteWithSubtitlesTemplate {
                title: DEFAULT_DLNA_VIDEO_TITLE.to_string(),
                video_uri: streaming_server.video_uri(),
                video_type: streaming_server.video_type(),
                subtitle_uri,
                subtitle_type: streaming_server
                    .subtitle_type()
                    .unwrap_or_else(|| "unknown".to_string()),
            };
            template
                .render()
                .map_err(|e| crate::error::Error::TemplateRenderError {
                    template_name: "didl_lite_with_subtitles.xml".to_string(),
                    source: e.into(),
                })?
        }
        None => {
            let template = DidlLiteWithoutSubtitlesTemplate {
                title: DEFAULT_DLNA_VIDEO_TITLE.to_string(),
                video_uri: streaming_server.video_uri(),
                video_type: streaming_server.video_type(),
            };
            template
                .render()
                .map_err(|e| crate::error::Error::TemplateRenderError {
                    template_name: "didl_lite_without_subtitles.xml".to_string(),
                    source: e.into(),
                })?
        }
    };

    Ok(escape_str_attribute(metadata.as_str()).to_string())
}

/// Builds the SetAVTransportURI payload
pub fn build_setavtransporturi_payload(
    streaming_server: &MediaStreamingServer,
    metadata: &str,
) -> Result<String> {
    let template = SetAvTransportUriTemplate {
        instance_id: DLNA_INSTANCE_ID,
        current_uri: streaming_server.video_uri(),
        current_uri_metadata: metadata.to_string(),
    };

    template
        .render()
        .map_err(|e| crate::error::Error::TemplateRenderError {
            template_name: "set_av_transport_uri.xml".to_string(),
            source: e.into(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::MediaStreamingServer;
    use std::path::PathBuf;

    /// Create a test MediaStreamingServer for testing
    fn create_test_streaming_server(with_subtitle: bool) -> MediaStreamingServer {
        let video_path = PathBuf::from("test_video.mp4");
        let subtitle_path = if with_subtitle {
            Some(PathBuf::from("test_subtitle.srt"))
        } else {
            None
        };
        let host_ip = "192.168.1.100".to_string();
        let host_port = 9000;

        // Create a temporary video file for testing
        std::fs::write(&video_path, b"fake video content").unwrap();

        if with_subtitle {
            std::fs::write(subtitle_path.as_ref().unwrap(), b"fake subtitle content").unwrap();
        }

        let server =
            MediaStreamingServer::new(&video_path, &subtitle_path, &host_ip, &host_port).unwrap();

        // Clean up test files
        std::fs::remove_file(&video_path).ok();
        if let Some(sub_path) = subtitle_path {
            std::fs::remove_file(&sub_path).ok();
        }

        server
    }

    #[test]
    fn test_metadata_without_subtitles() {
        let streaming_server = create_test_streaming_server(false);
        let result = build_metadata(&streaming_server);

        assert!(result.is_ok());
        let metadata = result.unwrap();

        // Check that the metadata contains expected elements
        assert!(metadata.contains("DIDL-Lite"));
        assert!(metadata.contains("crab-dlna Video"));
        assert!(metadata.contains("192.168.1.100:9000")); // Check for the host/port instead
        assert!(metadata.contains("object.item.videoItem.movie"));

        // Should not contain subtitle-related elements
        assert!(!metadata.contains("CaptionInfo"));
        assert!(!metadata.contains("subtitleFileUri"));
    }

    #[test]
    fn test_metadata_with_subtitles() {
        let streaming_server = create_test_streaming_server(true);
        let result = build_metadata(&streaming_server);

        assert!(result.is_ok());
        let metadata = result.unwrap();

        // Check that the metadata contains expected elements
        assert!(metadata.contains("DIDL-Lite"));
        assert!(metadata.contains("crab-dlna Video"));
        assert!(metadata.contains("192.168.1.100:9000")); // Check for the host/port instead
        assert!(metadata.contains("object.item.videoItem.movie"));

        // Should contain subtitle-related elements
        assert!(metadata.contains("CaptionInfo"));
        assert!(metadata.contains("subtitleFileUri"));
    }

    #[test]
    fn test_setavtransporturi_payload() {
        let streaming_server = create_test_streaming_server(false);
        let metadata = "test metadata";
        let result = build_setavtransporturi_payload(&streaming_server, metadata);

        assert!(result.is_ok());
        let payload = result.unwrap();

        // Check that the payload contains expected elements
        assert!(payload.contains("<InstanceID>0</InstanceID>"));
        assert!(payload.contains("<CurrentURI>"));
        assert!(payload.contains("192.168.1.100:9000")); // Check for the host/port instead
        assert!(payload.contains("<CurrentURIMetaData>test metadata</CurrentURIMetaData>"));
    }

    #[test]
    fn test_xml_escaping() {
        let streaming_server = create_test_streaming_server(false);
        let result = build_metadata(&streaming_server);

        assert!(result.is_ok());
        let metadata = result.unwrap();

        // The result should be XML-escaped
        // Since we use escape_str_attribute, special characters should be escaped
        assert!(!metadata.contains("<DIDL-Lite")); // Should be escaped
        assert!(metadata.contains("&lt;DIDL-Lite")); // Should be escaped
    }
}
