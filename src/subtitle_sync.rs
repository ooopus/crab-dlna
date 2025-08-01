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

    /// Gets the current subtitle content based on playback time
    ///
    /// # Arguments
    /// * `position_ms` - Playback position in milliseconds
    ///
    /// # Returns
    /// Returns the subtitle text that should be displayed at the given time, or empty string if no subtitle
    pub fn get_current_subtitle(&self, position_ms: u64) -> String {
        for entry in &self.entries {
            if position_ms >= entry.start_time && position_ms <= entry.end_time {
                return entry.text.clone();
            }
        }
        String::new()
    }

    /// Copies subtitle content to clipboard
    ///
    /// # Arguments
    /// * `subtitle_text` - The subtitle text to copy to clipboard
    ///
    /// # Returns
    /// Returns Ok if successfully copied to clipboard, otherwise returns Err
    pub fn copy_to_clipboard(&mut self, subtitle_text: &str) -> Result<()> {
        if let Some(clipboard) = &mut self.clipboard {
            clipboard
                .set_text(subtitle_text)
                .map_err(|e| Error::SubtitleSyncError {
                    message: format!("Failed to copy to clipboard: {e}"),
                    context: "Clipboard operation failed".to_string(),
                })?;
        }
        Ok(())
    }

    /// Updates clipboard with subtitle content based on playback position
    ///
    /// # Arguments
    /// * `position_ms` - Playback position in milliseconds
    ///
    /// # Returns
    /// Returns Ok if successfully updated clipboard, otherwise returns Err
    pub fn update_clipboard(&mut self, position_ms: u64) -> Result<()> {
        let subtitle_text = self.get_current_subtitle(position_ms);
        self.copy_to_clipboard(&subtitle_text)
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

    // Get all subtitle events based on the subtitle format
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
        TimedSubtitleFile::Ass(ass) => {
            for event in ass.events() {
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

/// Converts time string to milliseconds
///
/// # Arguments
/// * `time_str` - Time string in format HH:MM:SS,mmm
///
/// # Returns
/// Returns time in milliseconds
#[allow(dead_code)]
fn time_str_to_milliseconds(time_str: &str) -> Result<u64> {
    let parts: Vec<&str> = time_str.split(&[',', ':']).collect();
    if parts.len() != 4 {
        return Err(Error::SubtitleSyncError {
            message: "Invalid time format".to_string(),
            context: format!("Expected HH:MM:SS,mmm format, got: {time_str}"),
        });
    }

    let hours: u64 = parts[0].parse().map_err(|_| Error::SubtitleSyncError {
        message: "Invalid hours".to_string(),
        context: format!("Failed to parse hours from: {}", parts[0]),
    })?;
    let minutes: u64 = parts[1].parse().map_err(|_| Error::SubtitleSyncError {
        message: "Invalid minutes".to_string(),
        context: format!("Failed to parse minutes from: {}", parts[1]),
    })?;
    let seconds: u64 = parts[2].parse().map_err(|_| Error::SubtitleSyncError {
        message: "Invalid seconds".to_string(),
        context: format!("Failed to parse seconds from: {}", parts[2]),
    })?;
    let milliseconds: u64 = parts[3].parse().map_err(|_| Error::SubtitleSyncError {
        message: "Invalid milliseconds".to_string(),
        context: format!("Failed to parse milliseconds from: {}", parts[3]),
    })?;

    Ok(hours * 3600000 + minutes * 60000 + seconds * 1000 + milliseconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_aspasia_direct() {
        // Test aspasia directly with a more complete SRT file
        let mut file = File::create("test_direct.srt").unwrap();
        writeln!(file, "1").unwrap();
        writeln!(file, "00:00:01,000 --> 00:00:03,000").unwrap();
        writeln!(file, "Hello, world!").unwrap();
        writeln!(file).unwrap(); // Empty line between entries
        writeln!(file, "2").unwrap();
        writeln!(file, "00:00:04,000 --> 00:00:06,000").unwrap();
        writeln!(file, "This is a test.").unwrap();
        writeln!(file).unwrap(); // Empty line at end
        drop(file);

        let subtitle_file = TimedSubtitleFile::new("test_direct.srt").unwrap();
        match subtitle_file {
            TimedSubtitleFile::SubRip(srt) => {
                assert_eq!(srt.events().len(), 2);
                assert_eq!(Into::<i64>::into(srt.events()[0].start()), 1000);
                assert_eq!(Into::<i64>::into(srt.events()[0].end()), 3000);
                assert_eq!(srt.events()[0].text, "Hello, world!");
            }
            _ => panic!("Expected SubRip format"),
        }

        std::fs::remove_file("test_direct.srt").unwrap();
    }

    #[test]
    fn test_parse_srt_subtitle() {
        // 创建临时SRT文件用于测试
        let mut file = File::create("test.srt").unwrap();
        writeln!(file, "1").unwrap();
        writeln!(file, "00:00:01,000 --> 00:00:03,000").unwrap();
        writeln!(file, "Hello, world!").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "2").unwrap();
        writeln!(file, "00:00:04,000 --> 00:00:06,000").unwrap();
        writeln!(file, "This is a test.").unwrap();
        drop(file); // Ensure file is closed before reading

        // 解析字幕文件
        let entries = parse_subtitle_file(Path::new("test.srt")).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].start_time, 1000);
        assert_eq!(entries[0].end_time, 3000);
        assert_eq!(entries[0].text, "Hello, world!");
        assert_eq!(entries[1].start_time, 4000);
        assert_eq!(entries[1].end_time, 6000);
        assert_eq!(entries[1].text, "This is a test.");

        // 清理临时文件
        std::fs::remove_file("test.srt").unwrap();
    }

    #[test]
    fn test_get_current_subtitle() {
        let entries = vec![
            SubtitleEntry {
                start_time: 1000,
                end_time: 3000,
                text: "Hello, world!".to_string(),
            },
            SubtitleEntry {
                start_time: 4000,
                end_time: 6000,
                text: "This is a test.".to_string(),
            },
        ];

        let syncer = SubtitleSyncer {
            entries,
            clipboard: None,
        };

        // 测试在字幕时间段内的情况
        assert_eq!(syncer.get_current_subtitle(2000), "Hello, world!");
        assert_eq!(syncer.get_current_subtitle(5000), "This is a test.");

        // 测试在字幕时间段外的情况
        assert_eq!(syncer.get_current_subtitle(0), "");
        assert_eq!(syncer.get_current_subtitle(3500), "");
        assert_eq!(syncer.get_current_subtitle(7000), "");
    }
}
