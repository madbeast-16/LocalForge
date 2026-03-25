use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::theme;

/// Render the settings screen.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    host: &str,
    port: u16,
    tls_enabled: bool,
    parallel_slots: usize,
    ctx_size: usize,
    selected_row: usize,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Min(12),   // Settings grid
        Constraint::Length(2),  // Footer
    ])
    .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" ◆ ", Style::default().fg(theme::ACCENT)),
        Span::styled("Settings", theme::title()),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(header, chunks[0]);

    // Settings list
    let settings = vec![
        ("Server Host", host.to_string(), "Bind address for the API server"),
        ("Server Port", port.to_string(), "Port number (1024-65535)"),
        ("TLS / HTTPS", if tls_enabled { "Enabled".into() } else { "Disabled".into() }, "Auto-generates self-signed cert"),
        ("Parallel Slots", parallel_slots.to_string(), "Concurrent inference slots"),
        ("Context Size", ctx_size.to_string(), "Max tokens per context window"),
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    for (i, (label, value, hint)) in settings.iter().enumerate() {
        let is_selected = i == selected_row;
        let marker = if is_selected { "▸ " } else { "  " };

        let label_style = if is_selected {
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::ACCENT)
        };

        let value_style = if is_selected {
            theme::highlight()
        } else {
            theme::normal()
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {}{:<18}", marker, label), label_style),
            Span::styled(format!("{:<15}", value), value_style),
            Span::styled(format!("  {}", hint), theme::dim()),
        ]));
        lines.push(Line::from(""));
    }

    let settings_widget = Paragraph::new(lines).block(
        Block::default()
            .title(" Configuration ")
            .title_style(theme::title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(settings_widget, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " ↑/↓ ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("navigate  |  ", theme::dim()),
        Span::styled(
            "Enter ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("edit  |  ", theme::dim()),
        Span::styled(
            "Esc ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("back", theme::dim()),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}
