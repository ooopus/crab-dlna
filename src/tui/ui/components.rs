//! UI components for the TUI interface
//!
//! This module contains individual UI components like header, footer,
//! playlist, and info panels.

use super::layout::create_info_panel_layout;
use crate::tui::app::{AppState, parse_time_string};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};

/// Draws the header with device info and status
pub fn draw_header(f: &mut Frame, area: Rect, state: &AppState) {
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
pub fn draw_playlist(f: &mut Frame, area: Rect, state: &AppState) {
    let files: Vec<ListItem> = state
        .playlist
        .files()
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let filename = file
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown");

            let style = if Some(i) == state.current_file_index {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if Some(i) == state.current_file_index {
                "♪ "
            } else {
                "  "
            };

            ListItem::new(format!("{prefix}{filename}")).style(style)
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
pub fn draw_info_panel(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = create_info_panel_layout(area);

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
pub fn draw_current_track_info(f: &mut Frame, area: Rect, state: &AppState) {
    let current_track = if let Some(ref current_file) = state.current_file {
        current_file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
    } else {
        "No track selected"
    };

    let transport_state = state
        .transport_info
        .as_ref()
        .map(|info| info.transport_state.as_str())
        .unwrap_or("Unknown");

    let track_info = vec![
        Line::from(vec![
            Span::styled("Track: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(current_track),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("State: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                transport_state,
                match transport_state {
                    "PLAYING" => Style::default().fg(Color::Green),
                    "PAUSED_PLAYBACK" => Style::default().fg(Color::Yellow),
                    "STOPPED" => Style::default().fg(Color::Red),
                    _ => Style::default().fg(Color::Gray),
                },
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Updated: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(
                "{:.1}s ago",
                state.last_update.elapsed().as_secs_f64()
            )),
        ]),
    ];

    let track_widget = Paragraph::new(track_info)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Current Track"),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(track_widget, area);
}

/// Draws the progress bar
pub fn draw_progress_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let (progress, label) = if let Some(ref position_info) = state.position_info {
        let current_time = parse_time_string(&position_info.rel_time);
        let total_time = parse_time_string(&position_info.track_duration);

        let progress = if total_time > 0.0 {
            (current_time / total_time * 100.0) as u16
        } else {
            0
        };

        let label = format!(
            "{} / {}",
            position_info.rel_time, position_info.track_duration
        );
        (progress, label)
    } else {
        (0, "-- / --".to_string())
    };

    let progress_bar = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(progress)
        .label(label);

    f.render_widget(progress_bar, area);
}

/// Draws transport controls
pub fn draw_transport_controls(f: &mut Frame, area: Rect, _state: &AppState) {
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
pub fn draw_status_messages(f: &mut Frame, area: Rect, state: &AppState) {
    let mut lines = vec![Line::from(vec![
        Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(&state.status_message),
    ])];

    if let Some(ref error_msg) = state.error_message {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "Error: ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(error_msg, Style::default().fg(Color::Red)),
        ]));
    }

    let status_widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .wrap(Wrap { trim: true });

    f.render_widget(status_widget, area);
}

/// Draws the footer with keyboard shortcuts
pub fn draw_footer(f: &mut Frame, area: Rect, _state: &AppState) {
    let footer_text = "Q/ESC: Quit | H/F1: Help | D: Device Info | SPACE/P: Play/Pause | ↑/↓: Navigate | R: Refresh";

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}
