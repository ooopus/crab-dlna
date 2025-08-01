//! DLNA protocol implementation for crab-dlna
//!
//! This module provides comprehensive DLNA functionality including:
//! - Media playback control (play, pause, resume)
//! - Metadata generation for media files
//! - Transport state management
//! - Subtitle synchronization support

pub mod actions;
pub mod metadata;
pub mod playback;

// Re-export main functions for backward compatibility
pub use actions::{pause, resume, toggle_play_pause};
pub use playback::play;
