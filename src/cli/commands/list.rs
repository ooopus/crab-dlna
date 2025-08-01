//! List command implementation for crab-dlna
//!
//! This module implements the list command which discovers and displays
//! available DLNA devices on the network.

use crate::{
    config::{Config, LOG_MSG_LIST_DEVICES},
    devices::Render,
    error::Result,
};
use log::info;

/// List command implementation
pub struct ListCommand<'a> {
    _args: &'a super::super::List,
}

impl<'a> ListCommand<'a> {
    /// Create a new list command
    pub fn new(args: &'a super::super::List) -> Self {
        Self { _args: args }
    }

    /// Execute the list command
    pub async fn run(&self, config: &Config) -> Result<()> {
        info!("{LOG_MSG_LIST_DEVICES}");
        for render in Render::discover(config.discovery_timeout).await? {
            println!("{render}");
        }
        Ok(())
    }
}
