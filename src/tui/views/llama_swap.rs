use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::tui::theme;

/// Render the llama-swap install / skip screen.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    selected: usize,
    installed: bool,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Length(8),  // Explanation
        Constraint::Length(1),  // Spacer
        Constraint::Min(5),    // Options
        Constraint::Length(2),  // Footer
    ])
    .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" ◆ ", Style::default().fg(theme::ACCENT)),
        Span::styled("llama-swap (Optional)", theme::title()),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(header, chunks[0]);

    // Explanation
    let explanation = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  llama-swap enables hot-swapping models without restarting the server.",
            theme::normal(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  This is useful if you:",
            Style::default().fg(theme::ACCENT),
        )),
        Line::from(Span::styled(
            "    • Run multiple models for different tasks",
            theme::normal(),
        )),
        Line::from(Span::styled(
            "    • Use AI agents that need different models",
            theme::normal(),
        )),
        Line::from(Span::styled(
            "    • Want to switch models without downtime",
            theme::normal(),
        )),
    ];
    let explain_widget = Paragraph::new(explanation).block(
        Block::default()
            .title(" What is llama-swap? ")
            .title_style(theme::title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(explain_widget, chunks[1]);

    // Spacer
    frame.render_widget(Paragraph::new(""), chunks[2]);

    // Options
    let options = if installed {
        vec![
            ("✓ Already installed", "llama-swap is ready to use", true),
            ("  Continue", "Proceed to model selection", false),
        ]
    } else {
        vec![
            ("  Install llama-swap", "Download and set up llama-swap", false),
            ("  Skip for now", "You can install later from Settings", false),
        ]
    };

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (label, desc, is_status))| {
            let style = if *is_status {
                theme::success()
            } else if i == selected {
                theme::highlight()
            } else {
                theme::normal()
            };
            let desc_style = if i == selected && !is_status {
                Style::default()
                    .fg(ratatui::style::Color::Black)
                    .bg(theme::ACCENT)
            } else {
                theme::dim()
            };

            ListItem::new(vec![
                Line::from(Span::styled(format!("  {}", label), style)),
                Line::from(Span::styled(format!("    {}", desc), desc_style)),
            ])
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Choose an option ")
            .title_style(theme::title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );

    let mut state = ListState::default();
    state.select(Some(selected));
    frame.render_stateful_widget(list, chunks[3], &mut state);

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
        Span::styled("select  |  ", theme::dim()),
        Span::styled(
            "Esc ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("back", theme::dim()),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[4]);
}
