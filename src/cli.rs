//! Command line interface for crab-dlna
//!
//! This module provides the CLI argument parsing and command execution
//! for the crab-dlna media streaming application.

mod args;
mod commands;

pub use args::{Cli, List, Play};
pub use commands::Commands;

use crate::error::Result;
use clap::Parser;

/// Run the CLI application
pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    cli.command.run(&cli).await
}
