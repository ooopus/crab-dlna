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