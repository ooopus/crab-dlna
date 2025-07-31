//! Keyboard event handling for interactive media control
//!
//! This module provides keyboard input handling for controlling media playback,
//! including play/pause toggle with the space key and other media controls.

use crate::{devices::Render, dlna::toggle_play_pause, error::Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use log::{debug, info, warn};
use std::time::Duration;
use tokio::time::timeout;

/// Keyboard event handler for media control
pub struct KeyboardHandler {
    /// The DLNA render device to control
    render: Render,
    /// Whether keyboard handling is active
    active: bool,
}

impl KeyboardHandler {
    /// Creates a new keyboard handler for the given render device
    pub fn new(render: Render) -> Self {
        Self {
            render,
            active: false,
        }
    }

    /// Starts the keyboard event loop
    ///
    /// This function enables raw mode for the terminal and starts listening for keyboard events.
    /// It will block until the event loop is stopped or an error occurs.
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting keyboard event handler...");
        info!("Press SPACE to toggle play/pause, 'q' to quit");

        // Enable raw mode to capture key events
        enable_raw_mode().map_err(|e| crate::error::Error::KeyboardError {
            message: format!("Failed to enable raw mode: {}", e),
        })?;

        self.active = true;

        let result = self.event_loop().await;

        // Always disable raw mode when exiting
        if let Err(e) = disable_raw_mode() {
            warn!("Failed to disable raw mode: {}", e);
        }

        self.active = false;
        result
    }

    /// Stops the keyboard event handler
    pub fn stop(&mut self) {
        self.active = false;
        info!("Keyboard event handler stopped");
    }

    /// Main event loop for handling keyboard input
    async fn event_loop(&mut self) -> Result<()> {
        while self.active {
            // Use timeout to allow for graceful shutdown
            match timeout(Duration::from_millis(100), self.read_event()).await {
                Ok(Ok(should_continue)) => {
                    if !should_continue {
                        break;
                    }
                }
                Ok(Err(e)) => {
                    warn!("Error reading keyboard event: {}", e);
                }
                Err(_) => {
                    // Timeout - continue loop to check if we should still be active
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Reads and processes a single keyboard event
    ///
    /// Returns Ok(true) to continue the event loop, Ok(false) to exit
    async fn read_event(&self) -> Result<bool> {
        if event::poll(Duration::from_millis(50)).map_err(|e| {
            crate::error::Error::KeyboardError {
                message: format!("Failed to poll for events: {}", e),
            }
        })? {
            match event::read().map_err(|e| crate::error::Error::KeyboardError {
                message: format!("Failed to read event: {}", e),
            })? {
                Event::Key(key_event) => {
                    return self.handle_key_event(key_event).await;
                }
                Event::Resize(_, _) => {
                    debug!("Terminal resized");
                }
                _ => {
                    debug!("Unhandled event");
                }
            }
        }

        Ok(true)
    }

    /// Handles a keyboard key event
    ///
    /// Returns Ok(true) to continue the event loop, Ok(false) to exit
    async fn handle_key_event(&self, key_event: KeyEvent) -> Result<bool> {
        // Only handle key press events, not release
        if key_event.kind != KeyEventKind::Press {
            return Ok(true);
        }

        match key_event.code {
            KeyCode::Char(' ') => {
                debug!("Space key pressed - toggling play/pause");
                if let Err(e) = toggle_play_pause(&self.render).await {
                    warn!("Failed to toggle play/pause: {}", e);
                } else {
                    info!("Play/pause toggled successfully");
                }
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                info!("Quit key pressed - exiting");
                return Ok(false);
            }
            KeyCode::Esc => {
                info!("Escape key pressed - exiting");
                return Ok(false);
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                debug!("P key pressed - toggling play/pause");
                if let Err(e) = toggle_play_pause(&self.render).await {
                    warn!("Failed to toggle play/pause: {}", e);
                } else {
                    info!("Play/pause toggled successfully");
                }
            }
            KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') => {
                self.show_help();
            }
            _ => {
                debug!("Unhandled key: {:?}", key_event.code);
            }
        }

        Ok(true)
    }

    /// Shows help information for keyboard controls
    fn show_help(&self) {
        println!("\n=== Keyboard Controls ===");
        println!("SPACE / P  : Toggle play/pause");
        println!("Q / ESC    : Quit");
        println!("H / ?      : Show this help");
        println!("========================\n");
    }
}

impl Drop for KeyboardHandler {
    fn drop(&mut self) {
        if self.active {
            // Ensure raw mode is disabled when the handler is dropped
            if let Err(e) = disable_raw_mode() {
                eprintln!("Failed to disable raw mode in drop: {}", e);
            }
        }
    }
}

/// Starts an interactive keyboard control session for the given render device
///
/// This is a convenience function that creates a KeyboardHandler and starts the event loop.
pub async fn start_interactive_control(render: Render) -> Result<()> {
    let mut handler = KeyboardHandler::new(render);
    handler.start().await
}
