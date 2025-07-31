use crate::{
    config::{
        DEFAULT_STREAMING_PORT, DEFAULT_SUBTITLE_FILENAME, INVALID_SOCKET_ADDRESS_MSG,
        LOG_MSG_NO_SUBTITLE_FILE, USER_AGENT,
    },
    error::{Error, Result},
    utils::{detect_subtitle_type, sanitize_filename_for_url},
};
use local_ip_address::local_ip;
use log::{debug, info};

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

        debug!("Streaming server address: {server_addr}");

        debug!("Creating video file route in streaming server");
        let video_file = match video_path.exists() {
            true => MediaFile {
                file_path: video_path.to_path_buf(),
                host_uri: format!("http://{server_addr}"),
                file_uri: sanitize_filename_for_url(&video_path.display().to_string()),
            },
            false => {
                return Err(Error::MediaFileNotFound {
                    path: video_path.display().to_string(),
                    context: "Video file does not exist or is not accessible".to_string(),
                });
            }
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

    #[doc(hidden)]
    pub fn video_uri(&self) -> String {
        format!("{}/{}", self.video_file.host_uri, self.video_file.file_uri)
    }

    #[doc(hidden)]
    pub fn video_type(&self) -> String {
        self.video_file
            .file_path
            .as_path()
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string()
    }

    #[doc(hidden)]
    pub fn subtitle_uri(&self) -> Option<String> {
        self.subtitle_file
            .clone()
            .map(|subtitle_file| format!("{}/{}", subtitle_file.host_uri, subtitle_file.file_uri))
    }

    #[doc(hidden)]
    pub fn subtitle_type(&self) -> Option<String> {
        self.subtitle_file.as_ref().and_then(|subtitle_file| {
            detect_subtitle_type(&subtitle_file.file_path)
                .map(|subtitle_type| subtitle_type.to_string())
        })
    }

    /// Gets the subtitle file path
    pub fn subtitle_file_path(&self) -> Option<&std::path::Path> {
        self.subtitle_file.as_ref().map(|f| f.file_path.as_path())
    }

    fn get_routes(
        self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let video_route = warp::path(self.video_file.file_uri.to_string())
            .and(warp::fs::file(self.video_file.file_path.clone()));

        info!("Video file: {}", self.video_file.file_path.display());
        debug!("Serving video file: {}", self.video_file);

        let subtitle_route = match &self.subtitle_file {
            Some(subtitle_file) => {
                info!("Subtitle file: {}", subtitle_file.file_path.display());
                debug!("Serving subtitle file: {subtitle_file}");
                warp::path(subtitle_file.file_uri.to_string())
                    .and(warp::fs::file(subtitle_file.file_path.clone()))
            }
            None => {
                info!("{LOG_MSG_NO_SUBTITLE_FILE}");
                warp::path(DEFAULT_SUBTITLE_FILENAME.to_string())
                    .and(warp::fs::file(self.video_file.file_path.clone()))
            }
        };

        warp::get()
            .and(video_route.or(subtitle_route))
            .with(warp::reply::with::header("Server", USER_AGENT))
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
