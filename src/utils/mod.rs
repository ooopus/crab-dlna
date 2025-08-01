//! Utility functions and helpers for crab-dlna
//!
//! This module provides various utility functions organized by functionality:
//! - Time parsing and conversion utilities
//! - Media file handling and validation
//! - Network operations and retry mechanisms
//! - Text formatting and display utilities

pub mod formatting;
pub mod media;
pub mod network;
pub mod time;

// Re-export commonly used functions for backward compatibility
pub use formatting::{format_device_description, format_device_with_service_description};
pub use media::{
    detect_subtitle_type, infer_subtitle_from_video,
    is_supported_media_file, sanitize_filename_for_url,
};
pub use network::retry_with_backoff;
pub use time::time_str_to_milliseconds;
