//! Dialog components for the TUI interface
//!
//! This module contains dialog boxes like help and device info dialogs.

use super::layout::centered_rect;
use crate::tui::app::AppState;
use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

/// Draws the help dialog
pub fn draw_help_dialog(f: &mut Frame) {
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

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(help_paragraph, area);
}

/// Draws the device info dialog
pub fn draw_device_info_dialog(f: &mut Frame, state: &AppState) {
    let area = centered_rect(70, 60, f.area());

    f.render_widget(Clear, area);

    let device = &state.render.device;
    let device_info = vec![
        Line::from(vec![Span::styled(
            "Device Information",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(device.friendly_name()),
        ]),
        Line::from(vec![
            Span::styled("URL: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(device.url().to_string()),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(device.device_type().to_string()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Service Information:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                "Service Type: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(state.render.service.service_type().to_string()),
        ]),
        Line::from(vec![
            Span::styled(
                "Service ID: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(state.render.service.service_id()),
        ]),
        Line::from(""),
        Line::from("Press any key to close this dialog..."),
    ];

    let device_paragraph = Paragraph::new(device_info)
        .block(
            Block::default()
                .title("Device Info")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(device_paragraph, area);
}
