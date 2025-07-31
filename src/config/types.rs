//! Configuration types for crab-dlna
//!
//! This module contains configuration structures and related types
//! used throughout the application.

use log::LevelFilter;

use super::constants::*;

/// Configuration for the application
#[derive(Debug, Clone)]
pub struct Config {
    /// Port for the streaming server
    pub streaming_port: u32,
    /// Timeout for device discovery
    pub discovery_timeout: u64,
    /// Interval for subtitle synchronization
    pub subtitle_sync_interval_ms: u64,
    /// Log level
    pub log_level: LevelFilter,
    /// Number of SSDP search attempts
    pub ssdp_search_attempts: usize,
    /// TTL for SSDP discovery packets
    pub ssdp_ttl: Option<u32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            streaming_port: DEFAULT_STREAMING_PORT,
            discovery_timeout: DEFAULT_DISCOVERY_TIMEOUT,
            subtitle_sync_interval_ms: DEFAULT_SUBTITLE_SYNC_INTERVAL_MS,
            log_level: LevelFilter::Info,
            ssdp_search_attempts: super::constants::SSDP_SEARCH_ATTEMPTS,
            ssdp_ttl: super::constants::SSDP_TTL,
        }
    }
}

impl Config {
    /// Creates a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the streaming port
    pub fn with_streaming_port(mut self, port: u32) -> Self {
        self.streaming_port = port;
        self
    }

    /// Sets the discovery timeout
    pub fn with_discovery_timeout(mut self, timeout: u64) -> Self {
        self.discovery_timeout = timeout;
        self
    }

    /// Sets the subtitle synchronization interval
    pub fn with_subtitle_sync_interval(mut self, interval_ms: u64) -> Self {
        self.subtitle_sync_interval_ms = interval_ms;
        self
    }

    /// Sets the log level
    pub fn with_log_level(mut self, level: LevelFilter) -> Self {
        self.log_level = level;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.streaming_port, DEFAULT_STREAMING_PORT);
        assert_eq!(config.discovery_timeout, DEFAULT_DISCOVERY_TIMEOUT);
        assert_eq!(config.log_level, LevelFilter::Info);
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new()
            .with_streaming_port(8080)
            .with_discovery_timeout(10)
            .with_log_level(LevelFilter::Debug);

        assert_eq!(config.streaming_port, 8080);
        assert_eq!(config.discovery_timeout, 10);
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_STREAMING_PORT, 9000);
        assert_eq!(DEFAULT_DISCOVERY_TIMEOUT, 5);
        assert_eq!(LOG_LEVEL_ENV_VAR, "CRABDLNA_LOG");
    }
}
