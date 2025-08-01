//! CLI command implementations for crab-dlna
//!
//! This module contains the implementation of CLI commands including
//! list and play functionality.

mod list;
mod play;

pub use list::ListCommand;
pub use play::PlayCommand;

use crate::{config::Config, error::Result};
use clap::Subcommand;

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Scan and list devices in the network capable of playing media
    List(super::List),

    /// Play a video file
    Play(super::Play),
}

impl Commands {
    /// Execute the command
    pub async fn run(&self, cli: &super::Cli) -> Result<()> {
        let config = match self {
            Self::List(_) => cli.build_config(None),
            Self::Play(play) => cli.build_config(Some(play)),
        };
        self.setup_log(&config);
        match self {
            Self::List(list) => ListCommand::new(list).run(&config).await?,
            Self::Play(play) => PlayCommand::new(play).run(&config).await?,
        }
        Ok(())
    }

    /// Setup logging configuration
    fn setup_log(&self, _config: &Config) {
        use crate::config::LOG_LEVEL_ENV_VAR;
        use log::LevelFilter;
        use simple_logger::SimpleLogger;
        use std::env;

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
