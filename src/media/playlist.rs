//! Playlist management for media files
//!
//! This module provides functionality for creating and managing playlists
//! of media files, including support for playing entire folders.

use crate::{
    error::{Error, Result},
    utils::is_supported_media_file,
};
use log::{debug, info};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

/// Represents a playlist of media files
#[derive(Debug, Clone, Default)]
pub struct Playlist {
    /// List of media files in the playlist
    files: VecDeque<PathBuf>,
    /// Current playing index
    current_index: Option<usize>,
    /// Whether to loop the playlist
    loop_playlist: bool,
}

impl Playlist {
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

        let mut playlist = Self::default();
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

        let mut playlist = Self::default();
        playlist.scan_directory(path)?;

        if playlist.is_empty() {
            return Err(Error::MediaFileNotFound {
                path: path.display().to_string(),
                context: "No supported media files found in directory".to_string(),
            });
        }

        Ok(playlist)
    }

    /// Scans a directory for supported media files and adds them to the playlist
    fn scan_directory(&mut self, dir_path: &Path) -> Result<()> {
        info!("Scanning directory for media files: {}", dir_path.display());

        let entries = std::fs::read_dir(dir_path).map_err(|e| Error::MediaFileNotFound {
            path: dir_path.display().to_string(),
            context: format!("Failed to read directory: {e}"),
        })?;

        let mut media_files = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| Error::MediaFileNotFound {
                path: dir_path.display().to_string(),
                context: format!("Failed to read directory entry: {e}"),
            })?;

            let path = entry.path();

            if path.is_file() && is_supported_media_file(&path) {
                debug!("Found media file: {}", path.display());
                media_files.push(path);
            } else if path.is_dir() {
                debug!("Skipping subdirectory: {}", path.display());
            } else {
                debug!("Skipping unsupported file: {}", path.display());
            }
        }

        // Sort files for consistent ordering
        media_files.sort();

        for file in media_files {
            self.add_file(file);
        }

        info!("Found {} media files in directory", self.files.len());
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
    pub fn next_file(&mut self) -> Option<&PathBuf> {
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
    pub fn previous_file(&mut self) -> Option<&PathBuf> {
        if self.files.is_empty() {
            return None;
        }

        match self.current_index {
            None => {
                self.current_index = Some(self.files.len() - 1);
            }
            Some(index) => {
                if index == 0 {
                    if self.loop_playlist {
                        self.current_index = Some(self.files.len() - 1);
                    } else {
                        return None; // Beginning of playlist
                    }
                } else {
                    self.current_index = Some(index - 1);
                }
            }
        }

        self.current_file()
    }

    /// Resets the playlist to the beginning
    pub fn reset(&mut self) {
        self.current_index = None;
    }

    /// Checks if the playlist is empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Gets the number of files in the playlist
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

    /// Gets a file at the specified index
    pub fn get_file(&self, index: usize) -> Option<&PathBuf> {
        self.files.get(index)
    }
}

impl Iterator for Playlist {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_file().cloned()
    }
}
