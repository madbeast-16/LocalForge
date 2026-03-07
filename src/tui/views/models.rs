use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::models::ModelEntry;
use crate::tui::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    models: &[ModelEntry],
    selected: usize,
    search_query: &str,
    is_searching: bool,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(6),
        Constraint::Length(4),
        Constraint::Length(2),
    ])
    .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" ◆ ", Style::default().fg(theme::ACCENT)),
        Span::styled("Model Selection", theme::title()),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(header, chunks[0]);

    // Search bar
    let search_display = if is_searching {
        format!("Search: {}_", search_query)
    } else if !search_query.is_empty() {
        format!(
            "Search results for: \"{}\"  (Press / to search again, Esc to clear)",
            search_query
        )
    } else {
        "Press / to search HuggingFace  ".to_string()
    };
    let search = Paragraph::new(Line::from(Span::styled(
        format!("  {}", search_display),
        if is_searching {
            Style::default().fg(theme::ACCENT)
        } else {
            theme::dim()
        },
    )))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(search, chunks[1]);

    // Model list
    let items: Vec<ListItem> = models
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let style = if i == selected {
                theme::highlight()
            } else {
                theme::normal()
            };
            let line = format!("  {} [{}]  {}", m.name, m.quantization, m.size_label);
            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let title = if search_query.is_empty() {
        " Recommended Models "
    } else {
        " Search Results "
    };

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .title_style(theme::title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );

    let mut state = ListState::default();
    state.select(Some(selected));
    frame.render_stateful_widget(list, chunks[2], &mut state);

    // Detail pane for selected model
    let detail = if selected < models.len() {
        let m = &models[selected];
        vec![
            Line::from(vec![
                Span::styled("  Repo: ", Style::default().fg(theme::ACCENT)),
                Span::styled(&m.repo, theme::normal()),
            ]),
            Line::from(vec![
                Span::styled("  Info: ", Style::default().fg(theme::ACCENT)),
                Span::styled(&m.description, theme::normal()),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "  No models available",
            theme::dim(),
        ))]
    };

    let detail_widget = Paragraph::new(detail).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(detail_widget, chunks[3]);

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
            "/ ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("search  |  ", theme::dim()),
        Span::styled(
            "s ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("skip  |  ", theme::dim()),
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
