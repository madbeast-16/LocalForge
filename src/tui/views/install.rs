use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::build::BuildPhase;
use crate::tui::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    phase: BuildPhase,
    log_lines: &[String],
    progress: f64,
    is_done: bool,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(6),
        Constraint::Length(2),
    ])
    .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" ◆ ", Style::default().fg(theme::ACCENT)),
        Span::styled("Installing llama.cpp", theme::title()),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(header, chunks[0]);

    // Progress bar
    let phase_label = phase.label();
    let pct = (progress * 100.0).min(100.0) as u16;
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!(" {} ", phase_label))
                .title_style(theme::title())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::DIM)),
        )
        .gauge_style(Style::default().fg(theme::ACCENT))
        .percent(pct)
        .label(format!("{}%", pct));
    frame.render_widget(gauge, chunks[1]);

    // Build log
    let visible = if log_lines.len() > 20 {
        &log_lines[log_lines.len() - 20..]
    } else {
        log_lines
    };
    let lines: Vec<Line> = visible
        .iter()
        .map(|l| Line::from(Span::styled(format!("  {}", l), theme::dim())))
        .collect();

    let log_widget = Paragraph::new(lines).block(
        Block::default()
            .title(" Build Log ")
            .title_style(theme::title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(log_widget, chunks[2]);

    // Footer
    let footer_text = if is_done {
        vec![
            Span::styled(
                " Enter ",
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("continue", theme::dim()),
        ]
    } else {
        vec![Span::styled(" Building... please wait ", theme::dim())]
    };
    let footer = Paragraph::new(Line::from(footer_text)).alignment(Alignment::Center);
    frame.render_widget(footer, chunks[3]);
}
