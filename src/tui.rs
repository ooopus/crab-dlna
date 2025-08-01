//! Terminal User Interface for crab-dlna
//!
//! This module provides a comprehensive TUI using Ratatui for interactive media control,
//! playlist management, and real-time status display.

use crate::{
    devices::{PositionInfo, Render, TransportInfo},
    dlna::{pause, toggle_play_pause},
    error::{Error, Result},
    playlist::Playlist,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use log::{debug, info, warn};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{
    io,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::Mutex, time::interval};

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
            if self.selected_playlist_item == 0 {
                self.selected_playlist_item = self.playlist.len() - 1;
            } else {
                self.selected_playlist_item -= 1;
            }
        }
    }

    /// Gets the current transport state as a display string
    pub fn transport_state_display(&self) -> String {
        match &self.transport_info {
            Some(info) => match info.transport_state.as_str() {
                "PLAYING" => "▶ Playing".to_string(),
                "PAUSED_PLAYBACK" => "⏸ Paused".to_string(),
                "STOPPED" => "⏹ Stopped".to_string(),
                state => format!("? {state}"),
            },
            None => "? Unknown".to_string(),
        }
    }

    /// Gets the current position as a display string
    pub fn position_display(&self) -> String {
        match &self.position_info {
            Some(info) => {
                if info.track_duration.is_empty() || info.track_duration == "NOT_IMPLEMENTED" {
                    format!("Position: {}", info.rel_time)
                } else {
                    format!("{} / {}", info.rel_time, info.track_duration)
                }
            }
            None => "Position: --:--".to_string(),
        }
    }

    /// Gets the progress percentage for the progress bar
    pub fn progress_percentage(&self) -> f64 {
        match &self.position_info {
            Some(info) => {
                let current = parse_time_string(&info.rel_time);
                let total = parse_time_string(&info.track_duration);
                if total > 0.0 {
                    (current / total * 100.0).min(100.0)
                } else {
                    0.0
                }
            }
            None => 0.0,
        }
    }
}

/// Parses a time string (HH:MM:SS) to seconds
fn parse_time_string(time_str: &str) -> f64 {
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
                            self.handle_key_event(key_event.code).await?;
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

    /// Handles keyboard input
    async fn handle_key_event(&mut self, key_code: KeyCode) -> Result<()> {
        let mut state = self.state.lock().await;

        // Handle global keys first
        match key_code {
            KeyCode::Char('q') | KeyCode::Esc => {
                state.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('h') | KeyCode::F(1) => {
                state.show_help = !state.show_help;
                return Ok(());
            }
            KeyCode::Char('d') => {
                state.show_device_info = !state.show_device_info;
                return Ok(());
            }
            _ => {}
        }

        // If help or device info is shown, handle those keys
        if state.show_help || state.show_device_info {
            match key_code {
                KeyCode::Enter | KeyCode::Char(' ') => {
                    state.show_help = false;
                    state.show_device_info = false;
                }
                _ => {}
            }
            return Ok(());
        }

        // Handle main interface keys
        match key_code {
            KeyCode::Char(' ') | KeyCode::Char('p') => {
                let render = state.render.clone();
                drop(state); // Release lock before async operation

                match toggle_play_pause(&render).await {
                    Ok(_) => {
                        let mut state = self.state.lock().await;
                        state.status_message = "Play/pause toggled".to_string();
                        state.error_message = None;
                    }
                    Err(e) => {
                        let mut state = self.state.lock().await;
                        state.error_message = Some(format!("Failed to toggle play/pause: {e}"));
                    }
                }
            }
            KeyCode::Char('s') => {
                let render = state.render.clone();
                drop(state);

                match pause(&render).await {
                    Ok(_) => {
                        let mut state = self.state.lock().await;
                        state.status_message = "Playback stopped".to_string();
                        state.error_message = None;
                    }
                    Err(e) => {
                        let mut state = self.state.lock().await;
                        state.error_message = Some(format!("Failed to stop: {e}"));
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                state.previous_playlist_item();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.next_playlist_item();
            }
            KeyCode::Enter => {
                state.status_message = "Playing selected item (not implemented)".to_string();
            }
            KeyCode::Char('r') => {
                state.status_message = "Refreshing status...".to_string();
                drop(state);

                let mut state = self.state.lock().await;
                state.update_status().await;
                state.status_message = "Status refreshed".to_string();
            }
            _ => {}
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

/// Draws the main UI
fn draw_ui(f: &mut Frame, state: &AppState) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Draw header
    draw_header(f, chunks[0], state);

    // Draw main content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Playlist
            Constraint::Percentage(40), // Info panel
        ])
        .split(chunks[1]);

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

/// Draws the header with device info and status
fn draw_header(f: &mut Frame, area: Rect, state: &AppState) {
    let device_name = state.render.device.friendly_name();
    let device_url = state.render.device.url().to_string();

    let header_text = format!("🎵 crab-dlna TUI - Device: {device_name} ({device_url})");

    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("DLNA Media Player"),
        )
        .alignment(Alignment::Center);

    f.render_widget(header, area);
}

/// Draws the playlist panel
fn draw_playlist(f: &mut Frame, area: Rect, state: &AppState) {
    let files: Vec<ListItem> = state
        .playlist
        .files()
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let style = if Some(i) == state.current_file_index {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if i == state.selected_playlist_item {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let icon = if Some(i) == state.current_file_index {
                "▶ "
            } else {
                "  "
            };

            ListItem::new(format!(
                "{}{}",
                icon,
                path.file_name().unwrap_or_default().to_string_lossy()
            ))
            .style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_playlist_item));

    let playlist = List::new(files)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Playlist ({}/{})",
            state.selected_playlist_item + 1,
            state.playlist.len()
        )))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("► ");

    f.render_stateful_widget(playlist, area, &mut list_state);
}

/// Draws the info panel with playback status and controls
fn draw_info_panel(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Current track info
            Constraint::Length(3), // Progress bar
            Constraint::Length(6), // Transport controls
            Constraint::Min(0),    // Status/Error messages
        ])
        .split(area);

    // Current track info
    draw_current_track_info(f, chunks[0], state);

    // Progress bar
    draw_progress_bar(f, chunks[1], state);

    // Transport controls
    draw_transport_controls(f, chunks[2], state);

    // Status messages
    draw_status_messages(f, chunks[3], state);
}

/// Draws current track information
fn draw_current_track_info(f: &mut Frame, area: Rect, state: &AppState) {
    let current_file_text = match &state.current_file {
        Some(path) => {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            format!("🎵 {filename}")
        }
        None => "No file playing".to_string(),
    };

    let transport_state = state.transport_state_display();
    let position = state.position_display();

    let info_text = vec![
        Line::from(vec![Span::styled(
            "Current Track:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(current_file_text),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Status:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(transport_state),
        Line::from(position),
    ];

    let info = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("Now Playing"))
        .wrap(Wrap { trim: true });

    f.render_widget(info, area);
}

/// Draws the progress bar
fn draw_progress_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let progress = state.progress_percentage();
    let label = format!("{progress:.1}%");

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(progress as u16)
        .label(label);

    f.render_widget(gauge, area);
}

/// Draws transport controls
fn draw_transport_controls(f: &mut Frame, area: Rect, _state: &AppState) {
    let controls_text = vec![
        Line::from("Controls:"),
        Line::from("SPACE/P: Play/Pause  S: Stop"),
        Line::from("↑/↓: Navigate  ENTER: Play Selected"),
        Line::from("R: Refresh  H: Help  D: Device Info"),
    ];

    let controls = Paragraph::new(controls_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .alignment(Alignment::Left);

    f.render_widget(controls, area);
}

/// Draws status and error messages
fn draw_status_messages(f: &mut Frame, area: Rect, state: &AppState) {
    let mut lines = vec![Line::from(vec![
        Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(&state.status_message),
    ])];

    if let Some(ref error) = state.error_message {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "Error: ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(error, Style::default().fg(Color::Red)),
        ]));
    }

    let messages = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Messages"))
        .wrap(Wrap { trim: true });

    f.render_widget(messages, area);
}

/// Draws the footer with keyboard shortcuts
fn draw_footer(f: &mut Frame, area: Rect, _state: &AppState) {
    let footer_text = "Q/ESC: Quit | H/F1: Help | D: Device Info | SPACE/P: Play/Pause | ↑/↓: Navigate | R: Refresh";

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}

/// Draws the help dialog
fn draw_help_dialog(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());

    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Playback Controls:"),
        Line::from("  SPACE / P    - Toggle play/pause"),
        Line::from("  S            - Stop playback"),
        Line::from("  R            - Refresh status"),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  ↑ / K        - Previous item"),
        Line::from("  ↓ / J        - Next item"),
        Line::from("  ENTER        - Play selected item"),
        Line::from(""),
        Line::from("Interface:"),
        Line::from("  H / F1       - Toggle this help"),
        Line::from("  D            - Show device info"),
        Line::from("  Q / ESC      - Quit application"),
        Line::from(""),
        Line::from("Press any key to close this help..."),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(help, area);
}

/// Draws the device info dialog
fn draw_device_info_dialog(f: &mut Frame, state: &AppState) {
    let area = centered_rect(70, 50, f.area());

    f.render_widget(Clear, area);

    let device = &state.render.device;
    let service = &state.render.service;

    let device_info_text = vec![
        Line::from(vec![Span::styled(
            "Device Information",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(format!("Name: {}", device.friendly_name())),
        Line::from(format!("URL: {}", device.url())),
        Line::from(format!("Type: {}", device.device_type())),
        Line::from(format!("Host: {}", state.render.host())),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Service Information",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(format!("Service Type: {}", service.service_type())),
        Line::from(format!("Service ID: {}", service.service_id())),
        Line::from(""),
        Line::from("Press any key to close..."),
    ];

    let device_info = Paragraph::new(device_info_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Device Info")
                .style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(device_info, area);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

/// Starts the TUI application
pub async fn start_tui(render: Render, playlist: Playlist) -> Result<()> {
    let mut app = TuiApp::new(render, playlist)?;
    app.run().await
}
