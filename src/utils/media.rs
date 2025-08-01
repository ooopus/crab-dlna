//! Media file utilities for crab-dlna
//!
//! This module provides functions for working with media files,
//! including subtitle detection and file format validation.

use crate::types::SubtitleType;
use std::path::Path;

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

    SubtitleType::all().into_iter().find(|&subtitle_type| subtitle_type.extension() == extension)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
