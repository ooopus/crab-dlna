//! Event handling for the TUI interface
//!
//! This module handles keyboard input and other events for the TUI application.

use super::app::AppState;
use crate::{
    dlna::{pause, toggle_play_pause},
    error::Result,
};
use crossterm::event::KeyCode;
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handles keyboard input events
pub async fn handle_key_event(state_arc: Arc<Mutex<AppState>>, key_code: KeyCode) -> Result<()> {
    let mut state = state_arc.lock().await;

    // Handle global keys first
    match key_code {
        KeyCode::Char('q') | KeyCode::Esc => {
            state.quit();
            return Ok(());
        }
        KeyCode::Char('h') | KeyCode::F(1) => {
            state.toggle_help();
            return Ok(());
        }
        KeyCode::Char('d') => {
            state.toggle_device_info();
            return Ok(());
        }
        _ => {}
    }

    // If help or device info is shown, handle those keys
    if state.show_help || state.show_device_info {
        match key_code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                state.close_dialogs();
            }
            _ => {}
        }
        return Ok(());
    }

    // Handle main interface keys
    match key_code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.previous_playlist_item();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.next_playlist_item();
        }
        KeyCode::Enter => {
            if let Some(selected_file) = state.get_selected_file().cloned() {
                let index = state.selected_playlist_item;
                state.set_current_file(selected_file.clone(), index);
                state.set_status_message(format!("Playing: {}", selected_file.display()));
                info!("Selected file for playback: {}", selected_file.display());
            }
        }
        KeyCode::Char(' ') | KeyCode::Char('p') => {
            state.set_status_message("Toggling play/pause...".to_string());
            let render = state.render.clone();
            drop(state);

            match toggle_play_pause(&render).await {
                Ok(_) => {
                    let mut state = state_arc.lock().await;
                    state.set_status_message("Play/pause toggled".to_string());
                }
                Err(e) => {
                    let mut state = state_arc.lock().await;
                    state.set_error_message(Some(format!("Failed to toggle play/pause: {e}")));
                }
            }
        }
        KeyCode::Char('s') => {
            state.set_status_message("Stopping playback...".to_string());
            let render = state.render.clone();
            drop(state);

            match pause(&render).await {
                Ok(_) => {
                    let mut state = state_arc.lock().await;
                    state.set_status_message("Playback stopped".to_string());
                    state.clear_current_file();
                }
                Err(e) => {
                    let mut state = state_arc.lock().await;
                    state.set_error_message(Some(format!("Failed to stop playback: {e}")));
                }
            }
        }
        KeyCode::Char('r') => {
            state.set_status_message("Refreshing status...".to_string());
            drop(state);

            let mut state = state_arc.lock().await;
            state.update_status().await;
            state.set_status_message("Status refreshed".to_string());
        }
        _ => {}
    }

    Ok(())
}
