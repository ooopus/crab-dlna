//! Media streaming server for crab-dlna
//!
//! This module provides functionality for serving media files over HTTP
//! to DLNA devices, including video and subtitle file streaming.

use crate::{
    config::{DEFAULT_STREAMING_PORT, INVALID_SOCKET_ADDRESS_MSG},
    error::{Error, Result},
    utils::{detect_subtitle_type, sanitize_filename_for_url},
};
use local_ip_address::local_ip;
use log::debug;
use std::net::SocketAddr;
use warp::Filter;

/// Default port to use for the streaming server
pub const STREAMING_PORT_DEFAULT: u32 = DEFAULT_STREAMING_PORT;

/// A media file to stream
#[derive(Debug, Clone)]
pub struct MediaFile {
    file_path: std::path::PathBuf,
    host_uri: String,
    file_uri: String,
}

impl std::fmt::Display for MediaFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "'{}' @  {}/{}",
            self.file_path.display(),
            self.host_uri,
            self.file_uri,
        )
    }
}

/// A media streaming server
#[derive(Debug, Clone)]
pub struct MediaStreamingServer {
    video_file: MediaFile,
    subtitle_file: Option<MediaFile>,
    server_addr: SocketAddr,
}

impl MediaStreamingServer {
    /// Create a new media streaming server
    pub fn new(
        video_path: &std::path::Path,
        subtitle_path: &Option<std::path::PathBuf>,
        host_ip: &String,
        host_port: &u32,
    ) -> Result<Self> {
        let server_addr_str = format!("{host_ip}:{host_port}");
        let server_addr: SocketAddr =
            server_addr_str
                .parse()
                .map_err(|e| Error::NetworkAddressParseError {
                    address: server_addr_str.clone(),
                    reason: format!("{INVALID_SOCKET_ADDRESS_MSG}: {e}"),
                })?;

        debug!("Creating video file route in streaming server");
        let video_file = MediaFile {
            file_path: video_path.to_path_buf(),
            host_uri: format!("http://{server_addr}"),
            file_uri: sanitize_filename_for_url(&video_path.display().to_string()),
        };

        debug!("Creating subtitle file route in streaming server");
        let subtitle_file = match subtitle_path {
            Some(subtitle_path) => match subtitle_path.exists() {
                true => Some(MediaFile {
                    file_path: subtitle_path.clone(),
                    host_uri: format!("http://{server_addr}"),
                    file_uri: sanitize_filename_for_url(&subtitle_path.display().to_string()),
                }),
                false => {
                    return Err(Error::MediaFileNotFound {
                        path: subtitle_path.display().to_string(),
                        context: "Subtitle file does not exist or is not accessible".to_string(),
                    });
                }
            },
            None => None,
        };

        Ok(Self {
            video_file,
            subtitle_file,
            server_addr,
        })
    }

    /// Gets the video URI
    #[doc(hidden)]
    pub fn video_uri(&self) -> String {
        format!("{}/{}", self.video_file.host_uri, self.video_file.file_uri)
    }

    /// Gets the subtitle URI if available
    pub fn subtitle_uri(&self) -> Option<String> {
        self.subtitle_file
            .as_ref()
            .map(|subtitle| format!("{}/{}", subtitle.host_uri, subtitle.file_uri))
    }

    /// Gets the subtitle file path if available
    pub fn subtitle_file_path(&self) -> Option<&std::path::Path> {
        self.subtitle_file
            .as_ref()
            .map(|subtitle| subtitle.file_path.as_path())
    }

    /// Gets the video file path
    pub fn video_file_path(&self) -> &std::path::Path {
        &self.video_file.file_path
    }

    /// Gets the server address
    pub fn server_addr(&self) -> SocketAddr {
        self.server_addr
    }

    /// Gets the video file type/MIME type
    pub fn video_type(&self) -> String {
        get_mime_type_from_path(&self.video_file.file_path)
    }

    /// Gets the subtitle file type/MIME type if available
    pub fn subtitle_type(&self) -> Option<String> {
        self.subtitle_file.as_ref().map(|subtitle| {
            let subtitle_type = detect_subtitle_type(&subtitle.file_path);
            match subtitle_type {
                Some(sub_type) => sub_type.mime_type().to_string(),
                None => "text/plain".to_string(),
            }
        })
    }

    /// Creates the warp routes for serving media files
    fn get_routes(
        self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        // For now, just serve the video file - subtitle serving can be added later
        self.get_video_route()
    }

    /// Creates the video file route
    fn get_video_route(
        self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let video_file_path = self.video_file.file_path.clone();
        let video_file_uri = self.video_file.file_uri.clone();

        warp::path(video_file_uri)
            .and(warp::get())
            .and_then(move || {
                let video_file_path = video_file_path.clone();
                async move {
                    debug!("Serving video file: {}", video_file_path.display());
                    serve_full_file(video_file_path).await
                }
            })
    }

    /// Start the media streaming server.
    pub async fn run(self) {
        let streaming_routes = self.clone().get_routes();
        warp::serve(streaming_routes).run(self.server_addr).await;
    }
}

/// Identifies the local serve IP address.
pub async fn get_local_ip() -> Result<String> {
    debug!("Identifying local IP address of host");
    Ok(local_ip()
        .map_err(|err| Error::LocalAddressResolutionFailed {
            source: err,
            context: "Failed to determine local IP address for streaming server".to_string(),
        })?
        .to_string())
}

/// Gets MIME type from file path extension
fn get_mime_type_from_path(path: &std::path::Path) -> String {
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            match ext_str.to_lowercase().as_str() {
                "mp4" => "video/mp4",
                "avi" => "video/x-msvideo",
                "mkv" => "video/x-matroska",
                "mov" => "video/quicktime",
                "wmv" => "video/x-ms-wmv",
                "flv" => "video/x-flv",
                "webm" => "video/webm",
                "m4v" => "video/x-m4v",
                "3gp" => "video/3gpp",
                "mp3" => "audio/mpeg",
                "wav" => "audio/wav",
                "flac" => "audio/flac",
                "aac" => "audio/aac",
                "ogg" => "audio/ogg",
                "m4a" => "audio/mp4",
                _ => "application/octet-stream",
            }
        } else {
            "application/octet-stream"
        }
    } else {
        "application/octet-stream"
    }
    .to_string()
}

/// Serves a file with range support
async fn serve_file_with_range(
    file_path: &std::path::Path,
    _range_header: &str,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Implementation would go here - this is a simplified version
    serve_full_file(file_path.to_path_buf()).await
}

/// Serves a complete file
async fn serve_full_file(
    file_path: std::path::PathBuf,
) -> Result<impl warp::Reply, warp::Rejection> {
    match tokio::fs::read(&file_path).await {
        Ok(contents) => {
            let mime_type = get_mime_type_from_path(&file_path);
            Ok(warp::reply::with_header(
                contents,
                "content-type",
                mime_type,
            ))
        }
        Err(_) => Err(warp::reject::not_found()),
    }
}
