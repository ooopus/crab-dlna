//! CLI argument parsing for crab-dlna
//!
//! This module contains the CLI argument definitions and parsing logic
//! using the clap crate.

use crate::config::{Config, DEFAULT_DISCOVERY_TIMEOUT};
use crate::media::STREAMING_PORT_DEFAULT;
use clap::{Args, Parser};
use log::LevelFilter;
use std::path::PathBuf;

/// A minimal UPnP/DLNA media streamer
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Time in seconds to search and discover streamer hosts
    #[arg(short, long, default_value_t = DEFAULT_DISCOVERY_TIMEOUT)]
    pub timeout: u64,

    /// Log level
    #[arg(long, value_name = "LEVEL", global = true, default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,

    /// Subtitle synchronization interval in milliseconds
    #[arg(long, default_value_t = 500)]
    pub subtitle_sync_interval: u64,

    /// The command to execute
    #[command(subcommand)]
    pub command: super::Commands,
}

impl Cli {
    /// Build a Config from CLI arguments and Play command
    pub fn build_config(&self, play_cmd: Option<&super::Play>) -> Config {
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

/// List command arguments
#[derive(Args)]
pub struct List;

/// Play command arguments
#[derive(Args)]
pub struct Play {
    /// The hostname or IP to be used to host and serve the files (if not provided we derive it from the local network address)
    #[arg(short = 'H', long = "host")]
    pub host: Option<String>,

    /// The port to be used to host and serve the files
    #[arg(short = 'P', long = "port", default_value_t=STREAMING_PORT_DEFAULT)]
    pub port: u32,

    /// Specify the device where to play through a query (scan devices before playing)
    #[arg(short = 'q', long = "query-device")]
    pub device_query: Option<String>,

    /// Specify the device where to play through its exact location (no scan, faster)
    #[arg(short, long = "device")]
    pub device_url: Option<String>,

    /// The file of the subtitle (if not provided, we derive it from <FILE_VIDEO>)
    #[arg(short, long, value_name = "FILE_SUBTITLE")]
    pub subtitle: Option<PathBuf>,

    /// Disable subtitles
    #[arg(short, long)]
    pub no_subtitle: bool,

    /// Enable subtitle synchronization to clipboard
    #[arg(long)]
    pub subtitle_sync: bool,

    /// Enable interactive keyboard control (space to pause/resume, q to quit)
    #[arg(short, long)]
    pub interactive: bool,

    /// Enable Terminal User Interface (TUI) mode
    #[arg(long)]
    pub tui: bool,

    /// Enable playlist mode (loop through all files)
    #[arg(long)]
    pub playlist: bool,

    /// The file or directory to be played
    #[arg(long)]
    pub path: PathBuf,
}
