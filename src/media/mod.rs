//! Media handling and streaming for crab-dlna
//!
//! This module provides comprehensive media functionality including:
//! - Media file streaming over HTTP
//! - Playlist management for multiple files
//! - Subtitle synchronization and display

pub mod playlist;
pub mod streaming;
pub mod subtitle_sync;

// Re-export main types and functions for backward compatibility
pub use playlist::Playlist;
pub use streaming::{MediaStreamingServer, STREAMING_PORT_DEFAULT, get_local_ip};
pub use subtitle_sync::SubtitleSyncer;
