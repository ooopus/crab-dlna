//! Types used in crab-dlna

/// Supported subtitle types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtitleType {
    /// SubRip subtitle format
    Srt,
    /// Advanced SubStation Alpha subtitle format
    Ass,
    /// SubStation Alpha subtitle format
    Ssa,
}

impl SubtitleType {
    /// Returns the file extension for the subtitle type
    pub fn extension(&self) -> &'static str {
        match self {
            SubtitleType::Srt => "srt",
            SubtitleType::Ass => "ass",
            SubtitleType::Ssa => "ssa",
        }
    }

    /// Returns the MIME type for the subtitle type
    pub fn mime_type(&self) -> &'static str {
        match self {
            SubtitleType::Srt => "text/srt",
            SubtitleType::Ass => "text/x-ass",
            SubtitleType::Ssa => "text/x-ssa",
        }
    }

    /// Returns all supported subtitle types in order of preference
    pub fn all() -> Vec<SubtitleType> {
        vec![SubtitleType::Srt, SubtitleType::Ass, SubtitleType::Ssa]
    }
}

impl std::fmt::Display for SubtitleType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtitle_type_extension() {
        assert_eq!(SubtitleType::Srt.extension(), "srt");
        assert_eq!(SubtitleType::Ass.extension(), "ass");
        assert_eq!(SubtitleType::Ssa.extension(), "ssa");
    }

    #[test]
    fn test_subtitle_type_display() {
        assert_eq!(SubtitleType::Srt.to_string(), "srt");
        assert_eq!(SubtitleType::Ass.to_string(), "ass");
        assert_eq!(SubtitleType::Ssa.to_string(), "ssa");
    }

    #[test]
    fn test_subtitle_type_all() {
        let all_types = SubtitleType::all();
        assert_eq!(all_types.len(), 3);
        assert_eq!(all_types[0], SubtitleType::Srt);
        assert_eq!(all_types[1], SubtitleType::Ass);
        assert_eq!(all_types[2], SubtitleType::Ssa);
    }

    #[test]
    fn test_subtitle_type_equality() {
        assert_eq!(SubtitleType::Srt, SubtitleType::Srt);
        assert_ne!(SubtitleType::Srt, SubtitleType::Ass);
    }
}
