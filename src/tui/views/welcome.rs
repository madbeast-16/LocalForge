use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::theme;

const LOGO: &[&str] = &[
    r"  ██╗     ██╗      █████╗ ███╗   ███╗ █████╗ ",
    r"  ██║     ██║     ██╔══██╗████╗ ████║██╔══██╗",
    r"  ██║     ██║     ███████║██╔████╔██║███████║",
    r"  ██║     ██║     ██╔══██║██║╚██╔╝██║██╔══██║",
    r"  ███████╗███████╗██║  ██║██║ ╚═╝ ██║██║  ██║",
    r"  ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝",
];

pub fn render(frame: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(8),
        Constraint::Length(3),
        Constraint::Length(2),
        Constraint::Min(4),
        Constraint::Length(2),
    ])
    .split(area);

    // Title bar
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" llama-install ", theme::title()),
        Span::styled("v0.2.0", Style::default().fg(theme::DIM)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // ASCII logo
    let logo_lines: Vec<Line> = LOGO
        .iter()
        .map(|l| Line::from(Span::styled(*l, Style::default().fg(theme::ACCENT))))
        .collect();
    let logo = Paragraph::new(logo_lines).alignment(Alignment::Center);
    frame.render_widget(logo, chunks[1]);

    // Subtitle
    let subtitle = Paragraph::new(Line::from(vec![Span::styled(
        "Zero-overhead installer for llama.cpp",
        Style::default()
            .fg(theme::TEXT)
            .add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Center);
    frame.render_widget(subtitle, chunks[2]);

    // Spacer
    frame.render_widget(Paragraph::new(""), chunks[3]);

    // Feature list
    let features = vec![
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(theme::SUCCESS)),
            Span::styled(
                "Auto-detect hardware & select optimal backend",
                theme::normal(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(theme::SUCCESS)),
            Span::styled("Build llama.cpp with tuned CMake flags", theme::normal()),
        ]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(theme::SUCCESS)),
            Span::styled("Browse & download curated GGUF models", theme::normal()),
        ]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(theme::SUCCESS)),
            Span::styled("Search HuggingFace model hub", theme::normal()),
        ]),
    ];
    let feature_block = Paragraph::new(features)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    frame.render_widget(feature_block, chunks[4]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Press ", theme::dim()),
        Span::styled(
            "Enter",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to begin  |  ", theme::dim()),
        Span::styled(
            "q",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to quit", theme::dim()),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[5]);
}
