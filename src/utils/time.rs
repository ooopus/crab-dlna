//! Time parsing utilities for crab-dlna
//!
//! This module provides functions for parsing time strings in various formats
//! used by DLNA devices and subtitle files.

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
