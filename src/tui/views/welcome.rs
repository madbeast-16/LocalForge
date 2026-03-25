use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::config::ExpertiseLevel;
use crate::tui::theme;

const LOGO: &[&str] = &[
    r"  ╦  ╔═╗╔═╗╔═╗╦  ╔═╗╔═╗╦═╗╔═╗╔═╗ ",
    r"  ║  ║ ║║  ╠═╣║  ╠╣ ║ ║╠╦╝║ ╦║╣  ",
    r"  ╩═╝╚═╝╚═╝╩ ╩╩═╝╚  ╚═╝╩╚═╚═╝╚═╝ ",
];

pub fn render(
    frame: &mut Frame,
    area: Rect,
    detected_platform: &str,
    expertise_idx: usize,
) {
    let chunks = Layout::vertical([
        Constraint::Length(2),  // Title bar
        Constraint::Length(5),  // Logo
        Constraint::Length(2),  // Subtitle
        Constraint::Length(3),  // Platform info
        Constraint::Length(1),  // Spacer
        Constraint::Min(9),    // Features + Expertise
        Constraint::Length(2),  // Footer
    ])
    .split(area);

    // ── Title bar ──
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" ⚡ ", Style::default().fg(theme::WARN)),
        Span::styled("LocalForge ", theme::title()),
        Span::styled("v0.1.5", Style::default().fg(theme::DIM)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // ── ASCII Logo ──
    let logo_lines: Vec<Line> = LOGO
        .iter()
        .map(|l| {
            Line::from(Span::styled(
                *l,
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    let logo = Paragraph::new(logo_lines).alignment(Alignment::Center);
    frame.render_widget(logo, chunks[1]);

    // ── Subtitle / Description ──
    let subtitle = Paragraph::new(vec![
        Line::from(Span::styled(
            "Local-first TUI for a fully working local LLM stack",
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Zero to inference in minutes — no Python, no Docker, just Rust.",
            Style::default().fg(theme::DIM),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(subtitle, chunks[2]);

    // ── Detected Platform ──
    let platform = Paragraph::new(Line::from(vec![
        Span::styled("  ◆ Platform: ", Style::default().fg(theme::ACCENT)),
        Span::styled(
            detected_platform,
            Style::default()
                .fg(theme::SUCCESS)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM))
            .title(" Detected System ")
            .title_style(theme::title()),
    );
    frame.render_widget(platform, chunks[3]);

    // ── Spacer ──
    frame.render_widget(Paragraph::new(""), chunks[4]);

    // ── Split: Features (left) + Expertise Selector (right) ──
    let inner_chunks = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(chunks[5]);

    // Features list
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
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(theme::SUCCESS)),
            Span::styled("Chat with local inference", theme::normal()),
        ]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(theme::SUCCESS)),
            Span::styled("HTTPS server with API keys", theme::normal()),
        ]),
    ];
    let feature_block = Paragraph::new(features).block(
        Block::default()
            .title(" Features ")
            .title_style(theme::title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(feature_block, inner_chunks[0]);

    // Expertise selector
    let expertise_levels = ExpertiseLevel::all();
    let items: Vec<ListItem> = expertise_levels
        .iter()
        .enumerate()
        .map(|(i, level)| {
            let marker = if i == expertise_idx { "▸ " } else { "  " };
            let label = format!("{}{}", marker, level.as_str());
            let desc = format!("    {}", level.description());

            let style = if i == expertise_idx {
                theme::highlight()
            } else {
                theme::normal()
            };
            let desc_style = if i == expertise_idx {
                Style::default()
                    .fg(ratatui::style::Color::Black)
                    .bg(theme::ACCENT)
            } else {
                theme::dim()
            };

            ListItem::new(vec![
                Line::from(Span::styled(label, style)),
                Line::from(Span::styled(desc, desc_style)),
            ])
        })
        .collect();

    let expertise_list = List::new(items).block(
        Block::default()
            .title(" Expertise Level ")
            .title_style(theme::title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );

    let mut state = ListState::default();
    state.select(Some(expertise_idx));
    frame.render_stateful_widget(expertise_list, inner_chunks[1], &mut state);

    // ── Footer ──
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑/↓ ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("expertise  |  ", theme::dim()),
        Span::styled(
            "Enter",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" begin setup  |  ", theme::dim()),
        Span::styled(
            "q",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit", theme::dim()),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[6]);
}
