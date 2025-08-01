//! UI rendering components for the TUI interface
//!
//! This module contains all the UI rendering functions and components
//! for the TUI application.

mod components;
mod dialogs;
mod layout;

pub use components::*;
pub use dialogs::*;
pub use layout::*;

use super::app::AppState;
use ratatui::Frame;

/// Draws the main UI
pub fn draw_ui(f: &mut Frame, state: &AppState) {
    // Create main layout
    let chunks = create_main_layout(f.area());

    // Draw header
    draw_header(f, chunks[0], state);

    // Draw main content
    let main_chunks = create_content_layout(chunks[1]);
    draw_playlist(f, main_chunks[0], state);
    draw_info_panel(f, main_chunks[1], state);

    // Draw footer
    draw_footer(f, chunks[2], state);

    // Draw overlays
    if state.show_help {
        draw_help_dialog(f);
    }
    if state.show_device_info {
        draw_device_info_dialog(f, state);
    }
}
