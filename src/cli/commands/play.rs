//! Play command implementation for crab-dlna
//!
//! This module implements the play command which handles media playback
//! including playlist management, TUI mode, and interactive control.

use crate::{
    config::Config,
    devices::{Render, RenderSpec},
    dlna,
    error::{Error, Result},
    infer_subtitle_from_video,
    keyboard::start_interactive_control,
    media::{MediaStreamingServer, Playlist, SubtitleSyncer, get_local_ip},
    start_tui,
    utils::is_supported_media_file,
};
use log::info;
use std::path::Path;

/// Play command implementation
pub struct PlayCommand<'a> {
    args: &'a super::super::Play,
}

impl<'a> PlayCommand<'a> {
    /// Create a new play command
    pub fn new(args: &'a super::super::Play) -> Self {
        Self { args }
    }

    /// Execute the play command
    pub async fn run(&self, config: &Config) -> Result<()> {
        let render = self.select_render(config).await?;

        // Create playlist from path
        let mut playlist = if self.args.path.is_dir() {
            info!(
                "Creating playlist from directory: {}",
                self.args.path.display()
            );
            Playlist::from_directory(&self.args.path)?
        } else {
            info!("Creating playlist from file: {}", self.args.path.display());
            Playlist::from_file(&self.args.path)?
        };

        // Set playlist options
        playlist.set_loop(self.args.playlist);

        // Handle TUI mode
        if self.args.tui {
            info!("Starting TUI mode");
            return start_tui(render, playlist).await;
        }

        // Start interactive control if requested
        let interactive_handle = if self.args.interactive {
            let render_clone = render.clone();
            Some(tokio::spawn(async move {
                if let Err(e) = start_interactive_control(render_clone).await {
                    eprintln!("Interactive control error: {e}");
                }
            }))
        } else {
            None
        };

        // Play all files in the playlist
        let mut play_result = Ok(());
        while let Some(current_file) = playlist.next_file() {
            info!("Playing: {}", current_file.display());

            let media_streaming_server = self
                .build_media_streaming_server_for_file(current_file, config)
                .await?;

            // Create subtitle syncer if subtitle synchronization is enabled and subtitle file exists
            let subtitle_syncer = if self.args.subtitle_sync {
                if let Some(subtitle_path) = media_streaming_server.subtitle_file_path() {
                    match SubtitleSyncer::new(subtitle_path) {
                        Ok(syncer) => {
                            info!("Subtitle synchronization enabled");
                            Some(syncer)
                        }
                        Err(e) => {
                            eprintln!("Failed to create subtitle syncer: {e}");
                            None
                        }
                    }
                } else {
                    eprintln!("Subtitle synchronization requires a subtitle file");
                    None
                }
            } else {
                None
            };

            // Play the current file
            play_result = dlna::play(
                render.clone(),
                media_streaming_server,
                subtitle_syncer,
                config,
            )
            .await;

            if play_result.is_err() {
                eprintln!(
                    "Failed to play {}: {:?}",
                    current_file.display(),
                    play_result
                );
                if !self.args.playlist {
                    break; // Stop on error if not in playlist mode
                }
            }

            // If not in playlist mode, play only one file
            if !self.args.playlist {
                break;
            }
        }

        // Cancel interactive control
        if let Some(handle) = interactive_handle {
            handle.abort();
        }

        play_result
    }

    /// Select the render device based on command arguments
    async fn select_render(&self, config: &Config) -> Result<Render> {
        info!("Selecting render");
        Render::new(if let Some(device_url) = &self.args.device_url {
            RenderSpec::Location(device_url.to_owned())
        } else if let Some(device_query) = &self.args.device_query {
            RenderSpec::Query(config.discovery_timeout, device_query.to_owned())
        } else {
            RenderSpec::First(config.discovery_timeout)
        })
        .await
    }

    /// Build media streaming server for a specific file
    async fn build_media_streaming_server_for_file(
        &self,
        file_path: &Path,
        config: &Config,
    ) -> Result<MediaStreamingServer> {
        info!(
            "Building media streaming server for file: {}",
            file_path.display()
        );

        // Validate that the video file is supported
        if !is_supported_media_file(file_path) {
            return Err(Error::MediaFileNotFound {
                path: file_path.display().to_string(),
                context:
                    "Unsupported media file format. Please use a supported video or audio format."
                        .to_string(),
            });
        }

        let local_host_ip = get_local_ip().await?;
        let host_ip = self.args.host.as_ref().unwrap_or(&local_host_ip);
        let host_port = config.streaming_port;

        let subtitle = match &self.args.no_subtitle {
            false => self
                .args
                .subtitle
                .clone()
                .or_else(|| infer_subtitle_from_video(file_path)),
            true => None,
        };

        MediaStreamingServer::new(file_path, &subtitle, host_ip, &host_port)
    }
}
