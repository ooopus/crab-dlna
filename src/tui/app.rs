//! TUI application state management for crab-dlna
//!
//! This module contains the application state structure and related
//! functionality for the TUI interface.

use crate::{
    devices::{PositionInfo, Render, TransportInfo},
    media::Playlist,
};
use log::{debug, warn};
use std::{path::PathBuf, time::Instant};

/// Application state for the TUI
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current playlist
    pub playlist: Playlist,
    /// Current playing file index
    pub current_file_index: Option<usize>,
    /// Current playing file path
    pub current_file: Option<PathBuf>,
    /// Transport information
    pub transport_info: Option<TransportInfo>,
    /// Position information
    pub position_info: Option<PositionInfo>,
    /// DLNA render device
    pub render: Render,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Status message to display
    pub status_message: String,
    /// Error message to display
    pub error_message: Option<String>,
    /// Last update time
    pub last_update: Instant,
    /// Selected playlist item
    pub selected_playlist_item: usize,
    /// Whether help dialog is shown
    pub show_help: bool,
    /// Whether device info dialog is shown
    pub show_device_info: bool,
}

impl AppState {
    /// Creates a new application state
    pub fn new(render: Render, playlist: Playlist) -> Self {
        Self {
            playlist,
            current_file_index: None,
            current_file: None,
            transport_info: None,
            position_info: None,
            render,
            should_quit: false,
            status_message: "Ready".to_string(),
            error_message: None,
            last_update: Instant::now(),
            selected_playlist_item: 0,
            show_help: false,
            show_device_info: false,
        }
    }

    /// Updates the transport and position information
    pub async fn update_status(&mut self) {
        // Update transport info
        match self.render.get_transport_info().await {
            Ok(info) => {
                self.transport_info = Some(info);
                self.error_message = None;
            }
            Err(e) => {
                warn!("Failed to get transport info: {e}");
                self.error_message = Some(format!("Transport error: {e}"));
            }
        }

        // Update position info
        match self.render.get_position_info().await {
            Ok(info) => {
                self.position_info = Some(info);
            }
            Err(e) => {
                debug!("Failed to get position info: {e}");
            }
        }

        self.last_update = Instant::now();
    }

    /// Moves to the next playlist item
    pub fn next_playlist_item(&mut self) {
        if !self.playlist.is_empty() {
            self.selected_playlist_item = (self.selected_playlist_item + 1) % self.playlist.len();
        }
    }

    /// Moves to the previous playlist item
    pub fn previous_playlist_item(&mut self) {
        if !self.playlist.is_empty() {
            self.selected_playlist_item = if self.selected_playlist_item == 0 {
                self.playlist.len() - 1
            } else {
                self.selected_playlist_item - 1
            };
        }
    }

    /// Gets the currently selected playlist file
    pub fn get_selected_file(&self) -> Option<&PathBuf> {
        self.playlist.get_file(self.selected_playlist_item)
    }

    /// Sets the current playing file
    pub fn set_current_file(&mut self, file_path: PathBuf, index: usize) {
        self.current_file = Some(file_path);
        self.current_file_index = Some(index);
    }

    /// Clears the current playing file
    pub fn clear_current_file(&mut self) {
        self.current_file = None;
        self.current_file_index = None;
    }

    /// Sets a status message
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = message;
    }

    /// Sets an error message
    pub fn set_error_message(&mut self, message: Option<String>) {
        self.error_message = message;
    }

    /// Toggles the help dialog
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Toggles the device info dialog
    pub fn toggle_device_info(&mut self) {
        self.show_device_info = !self.show_device_info;
    }

    /// Closes all dialogs
    pub fn close_dialogs(&mut self) {
        self.show_help = false;
        self.show_device_info = false;
    }

    /// Marks the app for quitting
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

/// Parses a time string (HH:MM:SS) to seconds
pub fn parse_time_string(time_str: &str) -> f64 {
    let parts: Vec<&str> = time_str.split(':').collect();
    match parts.len() {
        3 => {
            let hours: f64 = parts[0].parse().unwrap_or(0.0);
            let minutes: f64 = parts[1].parse().unwrap_or(0.0);
            let seconds: f64 = parts[2].parse().unwrap_or(0.0);
            hours * 3600.0 + minutes * 60.0 + seconds
        }
        2 => {
            let minutes: f64 = parts[0].parse().unwrap_or(0.0);
            let seconds: f64 = parts[1].parse().unwrap_or(0.0);
            minutes * 60.0 + seconds
        }
        1 => parts[0].parse().unwrap_or(0.0),
        _ => 0.0,
    }
}
