//! Terminal User Interface for crab-dlna
//!
//! This module provides a comprehensive TUI using Ratatui for interactive media control,
//! playlist management, and real-time status display.

pub mod app;
pub mod events;
pub mod ui;

use app::AppState;
use events::handle_key_event;
use ui::draw_ui;

use crate::{
    devices::Render,
    error::{Error, Result},
    media::Playlist,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use log::info;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::interval};

/// Main TUI application
pub struct TuiApp {
    /// Application state
    state: Arc<Mutex<AppState>>,
    /// Terminal instance
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TuiApp {
    /// Creates a new TUI application
    pub fn new(render: Render, playlist: Playlist) -> Result<Self> {
        // Setup terminal
        enable_raw_mode().map_err(|e| Error::KeyboardError {
            message: format!("Failed to enable raw mode: {e}"),
        })?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
            Error::KeyboardError {
                message: format!("Failed to setup terminal: {e}"),
            }
        })?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).map_err(|e| Error::KeyboardError {
            message: format!("Failed to create terminal: {e}"),
        })?;

        let state = Arc::new(Mutex::new(AppState::new(render, playlist)));

        Ok(Self { state, terminal })
    }

    /// Runs the TUI application
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting TUI application");

        // Start status update task
        let state_clone = Arc::clone(&self.state);
        let update_handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(1000));
            loop {
                interval.tick().await;
                if let Ok(mut state) = state_clone.try_lock() {
                    if state.should_quit {
                        break;
                    }
                    state.update_status().await;
                }
            }
        });

        // Main event loop
        let result = self.event_loop().await;

        // Cleanup
        update_handle.abort();
        self.cleanup()?;

        result
    }

    /// Main event loop
    async fn event_loop(&mut self) -> Result<()> {
        loop {
            // Check if we should quit
            {
                let state = self.state.lock().await;
                if state.should_quit {
                    break;
                }
            }

            // Draw the UI
            let state = self.state.lock().await.clone();
            self.terminal
                .draw(|f| draw_ui(f, &state))
                .map_err(|e| Error::KeyboardError {
                    message: format!("Failed to draw UI: {e}"),
                })?;

            // Handle events
            if event::poll(Duration::from_millis(50)).map_err(|e| Error::KeyboardError {
                message: format!("Failed to poll for events: {e}"),
            })? {
                match event::read().map_err(|e| Error::KeyboardError {
                    message: format!("Failed to read event: {e}"),
                })? {
                    Event::Key(key_event) => {
                        if key_event.kind == KeyEventKind::Press {
                            handle_key_event(Arc::clone(&self.state), key_event.code).await?;
                        }
                    }
                    Event::Resize(_, _) => {
                        // Terminal was resized, will be handled on next draw
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Cleanup terminal state
    fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode().map_err(|e| Error::KeyboardError {
            message: format!("Failed to disable raw mode: {e}"),
        })?;

        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .map_err(|e| Error::KeyboardError {
            message: format!("Failed to cleanup terminal: {e}"),
        })?;

        self.terminal
            .show_cursor()
            .map_err(|e| Error::KeyboardError {
                message: format!("Failed to show cursor: {e}"),
            })?;

        Ok(())
    }
}

/// Starts the TUI application
pub async fn start_tui(render: Render, playlist: Playlist) -> Result<()> {
    let mut app = TuiApp::new(render, playlist)?;
    app.run().await
}
