//! Subtitle synchronization module
//!
//! This module provides subtitle synchronization functionality, including parsing subtitle files,
//! retrieving current subtitle content based on playback time, and copying subtitle content to clipboard.

use crate::error::{Error, Result};
use arboard::Clipboard;
use aspasia::{Subtitle, TimedEventInterface, TimedSubtitleFile};
use std::path::Path;

/// Subtitle entry containing timing and text information
#[derive(Debug, Clone)]
pub struct SubtitleEntry {
    /// Start time in milliseconds
    pub start_time: u64,
    /// End time in milliseconds
    pub end_time: u64,
    /// Subtitle text content
    pub text: String,
}

/// Subtitle synchronizer for managing subtitle display and clipboard integration
pub struct SubtitleSyncer {
    /// List of parsed subtitle entries
    entries: Vec<SubtitleEntry>,
    /// Clipboard instance for copying subtitle text
    clipboard: Option<Clipboard>,
}

impl SubtitleSyncer {
    /// Creates a new subtitle synchronizer
    ///
    /// # Arguments
    /// * `subtitle_path` - Path to the subtitle file
    ///
    /// # Returns
    /// Returns a new subtitle synchronizer instance
    pub fn new(subtitle_path: &Path) -> Result<Self> {
        // Parse subtitle file
        let entries = parse_subtitle_file(subtitle_path)?;

        // Initialize clipboard
        let clipboard = match Clipboard::new() {
            Ok(clipboard) => Some(clipboard),
            Err(e) => {
                eprintln!("Warning: Failed to initialize clipboard: {e}");
                None
            }
        };

        Ok(SubtitleSyncer { entries, clipboard })
    }

    /// Gets the current subtitle text for the given time
    ///
    /// # Arguments
    /// * `current_time_ms` - Current playback time in milliseconds
    ///
    /// # Returns
    /// Returns the subtitle text if available at the current time
    pub fn get_current_subtitle(&self, current_time_ms: u64) -> Option<&str> {
        for entry in &self.entries {
            if current_time_ms >= entry.start_time && current_time_ms <= entry.end_time {
                return Some(&entry.text);
            }
        }
        None
    }

    /// Copies the current subtitle text to clipboard
    ///
    /// # Arguments
    /// * `current_time_ms` - Current playback time in milliseconds
    ///
    /// # Returns
    /// Returns true if subtitle was copied to clipboard, false otherwise
    pub fn copy_current_subtitle_to_clipboard(&mut self, current_time_ms: u64) -> bool {
        if let Some(subtitle_text) = self.get_current_subtitle(current_time_ms) {
            let subtitle_text = subtitle_text.to_string(); // Clone the text to avoid borrow issues
            if let Some(ref mut clipboard) = self.clipboard {
                match clipboard.set_text(subtitle_text.clone()) {
                    Ok(_) => {
                        println!("Copied to clipboard: {subtitle_text}");
                        return true;
                    }
                    Err(e) => {
                        eprintln!("Failed to copy subtitle to clipboard: {e}");
                    }
                }
            }
        }
        false
    }

    /// Updates clipboard with current subtitle (alias for copy_current_subtitle_to_clipboard)
    ///
    /// # Arguments
    /// * `current_time_ms` - Current playback time in milliseconds
    ///
    /// # Returns
    /// Returns Ok(()) if successful, Err with error message if failed
    pub fn update_clipboard(&mut self, current_time_ms: u64) -> Result<(), String> {
        if self.copy_current_subtitle_to_clipboard(current_time_ms) {
            Ok(())
        } else {
            Err("No subtitle found or clipboard update failed".to_string())
        }
    }

    /// Gets all subtitle entries
    pub fn entries(&self) -> &[SubtitleEntry] {
        &self.entries
    }

    /// Gets the total number of subtitle entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Checks if there are no subtitle entries
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Parses a subtitle file and returns a list of subtitle entries
///
/// # Arguments
/// * `subtitle_path` - Path to the subtitle file
///
/// # Returns
/// Returns a list of parsed subtitle entries
fn parse_subtitle_file(subtitle_path: &Path) -> Result<Vec<SubtitleEntry>> {
    // Parse subtitle file using aspasia
    let subtitle_file =
        TimedSubtitleFile::new(subtitle_path).map_err(|e| Error::SubtitleSyncError {
            message: format!("Failed to parse subtitle file: {e}"),
            context: format!("Parsing file: {}", subtitle_path.display()),
        })?;

    // Convert to unified subtitle entry format
    let mut entries = Vec::new();

    match subtitle_file {
        TimedSubtitleFile::SubRip(srt) => {
            for event in srt.events() {
                let start_time = Into::<i64>::into(event.start()).max(0) as u64;
                let end_time = Into::<i64>::into(event.end()).max(0) as u64;
                let text = clean_subtitle_text(&event.text);

                entries.push(SubtitleEntry {
                    start_time,
                    end_time,
                    text,
                });
            }
        }
        TimedSubtitleFile::WebVtt(vtt) => {
            for event in vtt.events() {
                let start_time = Into::<i64>::into(event.start()).max(0) as u64;
                let end_time = Into::<i64>::into(event.end()).max(0) as u64;
                let text = clean_subtitle_text(&event.text);

                entries.push(SubtitleEntry {
                    start_time,
                    end_time,
                    text,
                });
            }
        }
        TimedSubtitleFile::Ass(ssa) => {
            for event in ssa.events() {
                let start_time = Into::<i64>::into(event.start()).max(0) as u64;
                let end_time = Into::<i64>::into(event.end()).max(0) as u64;
                let text = clean_subtitle_text(&event.text);

                entries.push(SubtitleEntry {
                    start_time,
                    end_time,
                    text,
                });
            }
        }
        TimedSubtitleFile::Ssa(ssa) => {
            for event in ssa.events() {
                let start_time = Into::<i64>::into(event.start()).max(0) as u64;
                let end_time = Into::<i64>::into(event.end()).max(0) as u64;
                let text = clean_subtitle_text(&event.text);

                entries.push(SubtitleEntry {
                    start_time,
                    end_time,
                    text,
                });
            }
        }
        TimedSubtitleFile::MicroDvd(mdvd) => {
            for event in mdvd.events() {
                let start_time = Into::<i64>::into(event.start()).max(0) as u64;
                let end_time = Into::<i64>::into(event.end()).max(0) as u64;
                let text = clean_subtitle_text(&event.text);

                entries.push(SubtitleEntry {
                    start_time,
                    end_time,
                    text,
                });
            }
        }
    }

    Ok(entries)
}

/// Cleans subtitle text by removing formatting tags and extra whitespace
///
/// # Arguments
/// * `text` - Raw subtitle text
///
/// # Returns
/// Returns cleaned subtitle text
fn clean_subtitle_text(text: &str) -> String {
    // Remove subtitle formatting tags (like HTML tags)
    let cleaned = text.replace("<i>", "").replace("</i>", "");
    // Remove extra whitespace
    cleaned.trim().to_string()
}
