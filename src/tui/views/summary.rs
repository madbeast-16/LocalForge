use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::config::Backend;
use crate::hardware::HardwareInfo;
use crate::tui::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    hw: &HardwareInfo,
    backend: &Backend,
    prefix: &str,
    model_name: Option<&str>,
    build_ok: bool,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(6),
        Constraint::Length(2),
    ])
    .split(area);

    // Header
    let status_icon = if build_ok { "✓" } else { "✗" };
    let status_style = if build_ok {
        theme::success()
    } else {
        theme::error()
    };
    let header = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {} ", status_icon), status_style),
        Span::styled("Installation Summary", theme::title()),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(header, chunks[0]);

    // Summary details
    let vram_str = HardwareInfo::format_memory(hw.gpu.vram_mb);
    let ram_str = HardwareInfo::format_memory(hw.memory.total_mb);
    let model_display = model_name.unwrap_or("(none selected)");

    let details = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Hardware    ", Style::default().fg(theme::ACCENT)),
            Span::styled(
                format!("{} | {} VRAM | {} RAM", hw.gpu.name, vram_str, ram_str),
                theme::normal(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Backend     ", Style::default().fg(theme::ACCENT)),
            Span::styled(&backend.description, theme::normal()),
        ]),
        Line::from(vec![
            Span::styled("  Prefix      ", Style::default().fg(theme::ACCENT)),
            Span::styled(prefix, theme::normal()),
        ]),
        Line::from(vec![
            Span::styled("  Model       ", Style::default().fg(theme::ACCENT)),
            Span::styled(model_display, theme::normal()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            if build_ok {
                "  Build completed successfully."
            } else {
                "  Build failed. Check logs."
            },
            if build_ok {
                theme::success()
            } else {
                theme::error()
            },
        )]),
    ];

    let detail_widget = Paragraph::new(details).block(
        Block::default()
            .title(" Results ")
            .title_style(theme::title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(detail_widget, chunks[1]);

    // Quick start guide
    let quick_start = vec![
        Line::from(Span::styled(
            "  Quick Start:",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(
                "    {}/share/llama.cpp/build/bin/llama-cli -m <model.gguf> -p \"Hello\"",
                prefix
            ),
            theme::normal(),
        )),
        Line::from(Span::styled(
            format!(
                "    {}/share/llama.cpp/build/bin/llama-server -m <model.gguf>",
                prefix
            ),
            theme::normal(),
        )),
    ];

    let qs_widget = Paragraph::new(quick_start).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(qs_widget, chunks[2]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " q ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("quit", theme::dim()),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[3]);
}
