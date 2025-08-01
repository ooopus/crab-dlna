use crate::{
    config::{Config, DEFAULT_DISCOVERY_TIMEOUT, LOG_LEVEL_ENV_VAR, LOG_MSG_LIST_DEVICES},
    devices::{Render, RenderSpec},
    dlna,
    error::{Error, Result},
    infer_subtitle_from_video,
    keyboard::start_interactive_control,
    playlist::Playlist,
    start_tui,
    streaming::{MediaStreamingServer, STREAMING_PORT_DEFAULT, get_local_ip},
    subtitle_sync::SubtitleSyncer,
    utils::is_supported_media_file,
};
use clap::{Args, Parser, Subcommand};
use log::{LevelFilter, info};
use simple_logger::SimpleLogger;
use std::env;

/// A minimal UPnP/DLNA media streamer
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Time in seconds to search and discover streamer hosts
    #[arg(short, long, default_value_t = DEFAULT_DISCOVERY_TIMEOUT)]
    timeout: u64,

    /// Log level
    #[arg(long, value_name = "LEVEL", global = true, default_value_t = LevelFilter::Info)]
    log_level: LevelFilter,

    /// Subtitle synchronization interval in milliseconds
    #[arg(long, default_value_t = 500)]
    subtitle_sync_interval: u64,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    /// Build a Config from CLI arguments and Play command
    fn build_config(&self, play_cmd: Option<&Play>) -> Config {
        let mut config = Config::new()
            .with_discovery_timeout(self.timeout)
            .with_log_level(self.log_level)
            .with_subtitle_sync_interval(self.subtitle_sync_interval);

        if let Some(play) = play_cmd {
            config = config.with_streaming_port(play.port);
        }

        config
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Scan and list devices in the network capable of playing media
    List(List),

    /// Play a video file
    Play(Play),
}

impl Commands {
    pub async fn run(&self, cli: &Cli) -> Result<()> {
        let config = match self {
            Self::List(_) => cli.build_config(None),
            Self::Play(play) => cli.build_config(Some(play)),
        };
        self.setup_log(&config);
        match self {
            Self::List(list) => list.run(&config).await?,
            Self::Play(play) => play.run(&config).await?,
        }
        Ok(())
    }

    fn setup_log(&self, _config: &Config) {
        let log_level = if let Ok(crabldna_log) = env::var(LOG_LEVEL_ENV_VAR) {
            match crabldna_log.as_str() {
                "trace" => LevelFilter::Trace,
                "debug" => LevelFilter::Debug,
                "info" => LevelFilter::Info,
                "warn" => LevelFilter::Warn,
                "error" => LevelFilter::Error,
                _ => LevelFilter::Info,
            }
        } else {
            LevelFilter::Info
        };

        SimpleLogger::new()
            .with_level(log_level)
            .init()
            .unwrap_or_else(|_| eprintln!("Warning: Logger already initialized"));
    }
}

#[derive(Args)]
struct List;

impl List {
    async fn run(&self, config: &Config) -> Result<()> {
        info!("{LOG_MSG_LIST_DEVICES}");
        for render in Render::discover(config.discovery_timeout).await? {
            println!("{render}");
        }
        Ok(())
    }
}

#[derive(Args)]
struct Play {
    /// The hostname or IP to be used to host and serve the files (if not provided we derive it from the local network address)
    #[arg(short = 'H', long = "host")]
    host: Option<String>,

    /// The port to be used to host and serve the files
    #[arg(short = 'P', long = "port", default_value_t=STREAMING_PORT_DEFAULT)]
    port: u32,

    /// Specify the device where to play through a query (scan devices before playing)
    #[arg(short = 'q', long = "query-device")]
    device_query: Option<String>,

    /// Specify the device where to play through its exact location (no scan, faster)
    #[arg(short, long = "device")]
    device_url: Option<String>,

    /// The file of the subtitle (if not provided, we derive it from <FILE_VIDEO>)
    #[arg(short, long, value_name = "FILE_SUBTITLE")]
    subtitle: Option<std::path::PathBuf>,

    /// Disable subtitles
    #[arg(short, long)]
    no_subtitle: bool,

    /// Enable subtitle synchronization to clipboard
    #[arg(long)]
    subtitle_sync: bool,

    /// Enable interactive keyboard control (space to pause/resume, q to quit)
    #[arg(short, long)]
    interactive: bool,

    /// Enable Terminal User Interface (TUI) mode
    #[arg(long)]
    tui: bool,

    /// Enable playlist mode (loop through all files)
    #[arg(long)]
    playlist: bool,

    /// The file or directory to be played
    #[arg(long)]
    path: std::path::PathBuf,
}

impl Play {
    async fn run(&self, config: &Config) -> Result<()> {
        let render = self.select_render(config).await?;

        // Create playlist from path
        let mut playlist = if self.path.is_dir() {
            info!("Creating playlist from directory: {}", self.path.display());
            Playlist::from_directory(&self.path)?
        } else {
            info!("Creating playlist from file: {}", self.path.display());
            Playlist::from_file(&self.path)?
        };

        // Set playlist options
        playlist.set_loop(self.playlist);

        // Handle TUI mode
        if self.tui {
            info!("Starting TUI mode");
            return start_tui(render, playlist).await;
        }

        // Start interactive control if requested
        let interactive_handle = if self.interactive {
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
            let subtitle_syncer = if self.subtitle_sync {
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
                if !self.playlist {
                    break; // Stop on error if not in playlist mode
                }
            }

            // If not in playlist mode, play only one file
            if !self.playlist {
                break;
            }
        }

        // Cancel interactive control
        if let Some(handle) = interactive_handle {
            handle.abort();
        }

        play_result
    }

    async fn select_render(&self, config: &Config) -> Result<Render> {
        info!("Selecting render");
        Render::new(if let Some(device_url) = &self.device_url {
            RenderSpec::Location(device_url.to_owned())
        } else if let Some(device_query) = &self.device_query {
            RenderSpec::Query(config.discovery_timeout, device_query.to_owned())
        } else {
            RenderSpec::First(config.discovery_timeout)
        })
        .await
    }

    async fn build_media_streaming_server_for_file(
        &self,
        file_path: &std::path::Path,
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
        let host_ip = self.host.as_ref().unwrap_or(&local_host_ip);
        let host_port = config.streaming_port;

        let subtitle = match &self.no_subtitle {
            false => self
                .subtitle
                .clone()
                .or_else(|| infer_subtitle_from_video(file_path)),
            true => None,
        };

        MediaStreamingServer::new(file_path, &subtitle, host_ip, &host_port)
    }
}

/// Run the CLI application
pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    cli.command.run(&cli).await
}
