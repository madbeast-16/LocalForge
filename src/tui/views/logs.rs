use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::tui::theme;

/// Render the log viewer screen.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    log_lines: &[String],
    scroll_offset: usize,
    filter: &str,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Length(3),  // Filter bar
        Constraint::Min(6),    // Log lines
        Constraint::Length(2),  // Footer
    ])
    .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" ◆ ", Style::default().fg(theme::ACCENT)),
        Span::styled("Log Viewer", theme::title()),
        Span::styled(
            format!("  ({} entries)", log_lines.len()),
            theme::dim(),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(header, chunks[0]);

    // Filter bar
    let filter_text = if filter.is_empty() {
        "All levels".to_string()
    } else {
        format!("Filter: {}", filter)
    };
    let filter_widget = Paragraph::new(Line::from(vec![
        Span::styled("  ", theme::normal()),
        Span::styled(filter_text, Style::default().fg(theme::ACCENT)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(filter_widget, chunks[1]);

    // Log lines
    let filtered: Vec<&String> = if filter.is_empty() {
        log_lines.iter().collect()
    } else {
        log_lines
            .iter()
            .filter(|l| l.to_lowercase().contains(&filter.to_lowercase()))
            .collect()
    };

    let total = filtered.len();
    let visible_height = chunks[2].height.saturating_sub(2) as usize;
    let max_scroll = total.saturating_sub(visible_height);
    let actual_offset = scroll_offset.min(max_scroll);

    let visible: Vec<Line> = filtered
        .iter()
        .skip(actual_offset)
        .take(visible_height)
        .map(|line| {
            let style = if line.contains("ERROR") || line.contains("error") {
                theme::error()
            } else if line.contains("WARN") || line.contains("warn") {
                theme::warn()
            } else if line.contains("INFO") || line.contains("info") {
                Style::default().fg(theme::ACCENT)
            } else if line.contains("DEBUG") || line.contains("debug") {
                theme::dim()
            } else {
                theme::normal()
            };
            Line::from(Span::styled(format!("  {}", line), style))
        })
        .collect();

    let log_widget = Paragraph::new(visible).block(
        Block::default()
            .title(" Logs ")
            .title_style(theme::title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(log_widget, chunks[2]);

    // Scrollbar
    if total > visible_height {
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(actual_offset);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        frame.render_stateful_widget(scrollbar, chunks[2], &mut scrollbar_state);
    }

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " ↑/↓ ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("scroll  |  ", theme::dim()),
        Span::styled(
            "e/w/i/d ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("filter (Error/Warn/Info/Debug)  |  ", theme::dim()),
        Span::styled(
            "c ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("clear filter  |  ", theme::dim()),
        Span::styled(
            "Esc ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("back", theme::dim()),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[3]);
}
