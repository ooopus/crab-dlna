//! Layout utilities for the TUI interface
//!
//! This module provides layout creation functions for organizing
//! the TUI interface components.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Creates the main application layout
pub fn create_main_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(area)
        .to_vec()
}

/// Creates the content layout (playlist and info panel)
pub fn create_content_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Playlist
            Constraint::Percentage(40), // Info panel
        ])
        .split(area)
        .to_vec()
}

/// Creates the info panel layout
pub fn create_info_panel_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Current track info
            Constraint::Length(3), // Progress bar
            Constraint::Length(6), // Transport controls
            Constraint::Min(0),    // Status/Error messages
        ])
        .split(area)
        .to_vec()
}

/// Helper function to create a centered rectangle
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
