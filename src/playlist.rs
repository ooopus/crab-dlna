//! Playlist management for media files
//!
//! This module provides functionality for creating and managing playlists
//! of media files, including support for playing entire folders.

use crate::{
    error::{Error, Result},
    utils::is_supported_media_file,
};
use log::{debug, info, warn};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

/// Represents a playlist of media files
#[derive(Debug, Clone)]
pub struct Playlist {
    /// List of media files in the playlist
    files: VecDeque<PathBuf>,
    /// Current playing index
    current_index: Option<usize>,
    /// Whether to loop the playlist
    loop_playlist: bool,
    /// Whether to shuffle the playlist
    shuffle: bool,
}

impl Playlist {
    /// Creates a new empty playlist
    pub fn new() -> Self {
        Self {
            files: VecDeque::new(),
            current_index: None,
            loop_playlist: false,
            shuffle: false,
        }
    }

    /// Creates a playlist from a single file
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let path = file_path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(Error::MediaFileNotFound {
                path: path.display().to_string(),
                context: "File does not exist".to_string(),
            });
        }

        if !is_supported_media_file(&path) {
            return Err(Error::MediaFileNotFound {
                path: path.display().to_string(),
                context: "Unsupported media file format".to_string(),
            });
        }

        let mut playlist = Self::new();
        playlist.add_file(path);
        Ok(playlist)
    }

    /// Creates a playlist from a directory, scanning for supported media files
    pub fn from_directory<P: AsRef<Path>>(dir_path: P) -> Result<Self> {
        let path = dir_path.as_ref();

        if !path.exists() {
            return Err(Error::MediaFileNotFound {
                path: path.display().to_string(),
                context: "Directory does not exist".to_string(),
            });
        }

        if !path.is_dir() {
            return Err(Error::MediaFileNotFound {
                path: path.display().to_string(),
                context: "Path is not a directory".to_string(),
            });
        }

        let mut playlist = Self::new();
        playlist.scan_directory(path)?;

        if playlist.is_empty() {
            return Err(Error::MediaFileNotFound {
                path: path.display().to_string(),
                context: "No supported media files found in directory".to_string(),
            });
        }

        info!(
            "Created playlist with {} files from directory: {}",
            playlist.len(),
            path.display()
        );
        Ok(playlist)
    }

    /// Scans a directory for supported media files and adds them to the playlist
    fn scan_directory<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<()> {
        let entries =
            std::fs::read_dir(dir_path.as_ref()).map_err(|e| Error::MediaFileNotFound {
                path: dir_path.as_ref().display().to_string(),
                context: format!("Failed to read directory: {}", e),
            })?;

        let mut files = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| Error::MediaFileNotFound {
                path: dir_path.as_ref().display().to_string(),
                context: format!("Failed to read directory entry: {}", e),
            })?;

            let path = entry.path();

            if path.is_file() && is_supported_media_file(&path) {
                debug!("Found media file: {}", path.display());
                files.push(path);
            } else if path.is_dir() {
                // Recursively scan subdirectories
                if let Err(e) = self.scan_directory(&path) {
                    warn!("Failed to scan subdirectory {}: {}", path.display(), e);
                }
            }
        }

        // Sort files for consistent ordering
        files.sort();

        for file in files {
            self.add_file(file);
        }

        Ok(())
    }

    /// Adds a file to the playlist
    pub fn add_file<P: Into<PathBuf>>(&mut self, file_path: P) {
        self.files.push_back(file_path.into());
    }

    /// Gets the current file in the playlist
    pub fn current_file(&self) -> Option<&PathBuf> {
        self.current_index.and_then(|index| self.files.get(index))
    }

    /// Moves to the next file in the playlist
    pub fn next(&mut self) -> Option<&PathBuf> {
        if self.files.is_empty() {
            return None;
        }

        match self.current_index {
            None => {
                self.current_index = Some(0);
            }
            Some(index) => {
                let next_index = index + 1;
                if next_index >= self.files.len() {
                    if self.loop_playlist {
                        self.current_index = Some(0);
                    } else {
                        return None; // End of playlist
                    }
                } else {
                    self.current_index = Some(next_index);
                }
            }
        }

        self.current_file()
    }

    /// Moves to the previous file in the playlist
    pub fn previous(&mut self) -> Option<&PathBuf> {
        if self.files.is_empty() {
            return None;
        }

        match self.current_index {
            None => {
                self.current_index = Some(self.files.len() - 1);
            }
            Some(0) => {
                if self.loop_playlist {
                    self.current_index = Some(self.files.len() - 1);
                } else {
                    return None; // Beginning of playlist
                }
            }
            Some(index) => {
                self.current_index = Some(index - 1);
            }
        }

        self.current_file()
    }

    /// Resets the playlist to the beginning
    pub fn reset(&mut self) {
        self.current_index = None;
    }

    /// Returns whether the playlist is empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Returns the number of files in the playlist
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Sets whether to loop the playlist
    pub fn set_loop(&mut self, loop_playlist: bool) {
        self.loop_playlist = loop_playlist;
    }

    /// Returns whether the playlist is set to loop
    pub fn is_looping(&self) -> bool {
        self.loop_playlist
    }

    /// Gets all files in the playlist
    pub fn files(&self) -> &VecDeque<PathBuf> {
        &self.files
    }

    /// Gets the current index
    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }
}

impl Default for Playlist {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for Playlist {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        self.next().cloned()
    }
}
