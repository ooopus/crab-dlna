//! Utility functions and helpers for crab-dlna

use crate::{config::MAX_NETWORK_RETRIES, types::SubtitleType};
use log::{debug, warn};
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

/// Converts time string to milliseconds
///
/// Supports two formats:
/// - HH:MM:SS (for DLNA position info)
/// - HH:MM:SS,mmm (for subtitle timestamps)
///
/// # Arguments
/// * `time_str` - Time string to convert
///
/// # Returns
/// Returns time in milliseconds, or 0 if parsing fails
pub fn time_str_to_milliseconds(time_str: &str) -> u64 {
    // Try HH:MM:SS format first (DLNA format)
    if let Ok(ms) = parse_dlna_time_format(time_str) {
        return ms;
    }

    // Try HH:MM:SS,mmm format (subtitle format)
    if let Ok(ms) = parse_subtitle_time_format(time_str) {
        return ms;
    }

    // Return 0 if both formats fail
    0
}

/// Parses DLNA time format (HH:MM:SS or HH:MM:SS.mmm)
fn parse_dlna_time_format(time_str: &str) -> Result<u64, ()> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return Err(());
    }

    let hours: u64 = parts[0].parse().map_err(|_| ())?;
    let minutes: u64 = parts[1].parse().map_err(|_| ())?;

    // Handle seconds with optional decimal part
    let seconds: f64 = parts[2].parse().map_err(|_| ())?;

    Ok((((hours as f64) * 3600.0 + (minutes as f64) * 60.0 + seconds) * 1000.0) as u64)
}

/// Parses subtitle time format (HH:MM:SS,mmm)
fn parse_subtitle_time_format(time_str: &str) -> Result<u64, ()> {
    let parts: Vec<&str> = time_str.split(&[',', ':']).collect();
    if parts.len() != 4 {
        return Err(());
    }

    let hours: u64 = parts[0].parse().map_err(|_| ())?;
    let minutes: u64 = parts[1].parse().map_err(|_| ())?;
    let seconds: u64 = parts[2].parse().map_err(|_| ())?;
    let milliseconds: u64 = parts[3].parse().map_err(|_| ())?;

    Ok(hours * 3600000 + minutes * 60000 + seconds * 1000 + milliseconds)
}

/// Infers subtitle file path from video file path
///
/// Tries different subtitle formats in order of preference and returns
/// the first one that exists.
///
/// # Arguments
/// * `video_path` - Path to the video file
///
/// # Returns
/// Returns the path to the subtitle file if found, None otherwise
pub fn infer_subtitle_from_video(video_path: &Path) -> Option<std::path::PathBuf> {
    // Try different subtitle formats in order of preference using SubtitleType enum
    for subtitle_type in SubtitleType::all() {
        let inferred_subtitle_path = video_path.with_extension(subtitle_type.extension());

        if inferred_subtitle_path.exists() {
            return Some(inferred_subtitle_path);
        }
    }

    None
}

/// Detects subtitle type from file extension
///
/// # Arguments
/// * `path` - Path to the subtitle file
///
/// # Returns
/// Returns the detected SubtitleType if the extension is supported, None otherwise
pub fn detect_subtitle_type(path: &Path) -> Option<SubtitleType> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    for subtitle_type in SubtitleType::all() {
        if subtitle_type.extension() == extension {
            return Some(subtitle_type);
        }
    }

    None
}

/// Validates if a file path has a supported video extension
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// Returns true if the file has a supported video extension
pub fn is_supported_video_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            let ext_lower = ext_str.to_lowercase();
            return crate::config::SUPPORTED_VIDEO_EXTENSIONS.contains(&ext_lower.as_str());
        }
    }
    false
}

/// Validates if a file path has a supported audio extension
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// Returns true if the file has a supported audio extension
pub fn is_supported_audio_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            let ext_lower = ext_str.to_lowercase();
            return crate::config::SUPPORTED_AUDIO_EXTENSIONS.contains(&ext_lower.as_str());
        }
    }
    false
}

/// Validates if a file path has a supported media extension (video or audio)
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// Returns true if the file has a supported media extension
pub fn is_supported_media_file(path: &Path) -> bool {
    is_supported_video_file(path) || is_supported_audio_file(path)
}

/// Retries an async operation with exponential backoff
///
/// # Arguments
/// * `operation` - The async operation to retry
/// * `operation_name` - Name of the operation for logging
///
/// # Returns
/// Returns the result of the operation or the last error if all retries fail
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    operation_name: &str,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error = None;

    for attempt in 1..=MAX_NETWORK_RETRIES {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!("{} succeeded on attempt {}", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(error) => {
                if attempt < MAX_NETWORK_RETRIES {
                    let delay = Duration::from_millis(100 * (1 << (attempt - 1))); // Exponential backoff
                    warn!(
                        "{} failed on attempt {} ({}), retrying in {:?}",
                        operation_name, attempt, error, delay
                    );
                    sleep(delay).await;
                } else {
                    warn!(
                        "{} failed on final attempt {} ({})",
                        operation_name, attempt, error
                    );
                }
                last_error = Some(error);
            }
        }
    }

    Err(last_error.unwrap())
}

/// Sanitizes a filename for use in URLs
///
/// # Arguments
/// * `filename` - The filename to sanitize
///
/// # Returns
/// Returns a URL-safe version of the filename
pub fn sanitize_filename_for_url(filename: &str) -> String {
    use slugify::slugify;
    slugify!(filename, separator = ".")
}

/// Formats a device description for display
///
/// # Arguments
/// * `device_type` - The device type
/// * `friendly_name` - The friendly name of the device
/// * `url` - The device URL
///
/// # Returns
/// Returns a formatted string describing the device
pub fn format_device_description(device_type: &str, friendly_name: &str, url: &str) -> String {
    format!("[{}] {} @ {}", device_type, friendly_name, url)
}

/// Formats a device description with service type for display
///
/// # Arguments
/// * `device_type` - The device type
/// * `service_type` - The service type
/// * `friendly_name` - The friendly name of the device
/// * `url` - The device URL
///
/// # Returns
/// Returns a formatted string describing the device with service information
pub fn format_device_with_service_description(
    device_type: &str,
    service_type: &str,
    friendly_name: &str,
    url: &str,
) -> String {
    format!(
        "[{}][{}] {} @ {}",
        device_type, service_type, friendly_name, url
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_time_str_to_milliseconds_dlna_format() {
        assert_eq!(time_str_to_milliseconds("01:30:45"), 5445000);
        assert_eq!(time_str_to_milliseconds("00:00:30"), 30000);
        assert_eq!(time_str_to_milliseconds("02:15:30.5"), 8130500);
    }

    #[test]
    fn test_time_str_to_milliseconds_subtitle_format() {
        assert_eq!(time_str_to_milliseconds("01:30:45,123"), 5445123);
        assert_eq!(time_str_to_milliseconds("00:00:30,000"), 30000);
    }

    #[test]
    fn test_time_str_to_milliseconds_invalid() {
        assert_eq!(time_str_to_milliseconds("invalid"), 0);
        assert_eq!(time_str_to_milliseconds("1:2"), 0);
    }

    #[test]
    fn test_is_supported_video_file() {
        assert!(is_supported_video_file(&PathBuf::from("test.mp4")));
        assert!(is_supported_video_file(&PathBuf::from("test.avi")));
        assert!(!is_supported_video_file(&PathBuf::from("test.txt")));
    }

    #[test]
    fn test_sanitize_filename_for_url() {
        assert_eq!(
            sanitize_filename_for_url("My Video File.mp4"),
            "my.video.file.mp4"
        );
        assert_eq!(
            sanitize_filename_for_url("Test (2023).avi"),
            "test.2023.avi"
        );
    }
}
