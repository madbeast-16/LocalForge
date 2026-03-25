use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::models::ModelEntry;
use crate::models::fit::FitLevel;
use crate::tui::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    models: &[ModelEntry],
    fits: &[FitLevel],
    selected: usize,
    checked: &[bool],
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
    let checked_count = checked.iter().filter(|c| **c).count();
    let header_text = if checked_count > 0 {
        format!("Model Selection  ({} selected)", checked_count)
    } else {
        "Model Selection".into()
    };
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" ◆ ", Style::default().fg(theme::ACCENT)),
        Span::styled(header_text, theme::title()),
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

    // Table header row
    let header_cells = [
        "", "Name", "Params", "Size", "Quant", "Context", "Fit", "Est. tok/s",
    ]
    .iter()
    .map(|h| {
        Cell::from(Span::styled(
            *h,
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ))
    });
    let header_row = Row::new(header_cells)
        .height(1)
        .bottom_margin(0);

    // Data rows
    let rows: Vec<Row> = models
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let is_checked = checked.get(i).copied().unwrap_or(false);
            let checkbox = if is_checked { " ✓ " } else { "   " };
            let fit = fits.get(i).copied().unwrap_or(FitLevel::Marginal);

            let (fit_label, fit_color) = match fit {
                FitLevel::Perfect => ("★ Perfect", theme::SUCCESS),
                FitLevel::Good => ("● Good", theme::ACCENT),
                FitLevel::Marginal => ("◐ Marginal", theme::WARN),
                FitLevel::TooTight => ("✗ Too Tight", theme::ERROR),
            };

            let fit_style = Style::default().fg(fit_color);

            Row::new(vec![
                Cell::from(Span::styled(checkbox, theme::normal())),
                Cell::from(Span::styled(&*m.name, theme::normal())),
                Cell::from(Span::styled(&*m.params, theme::dim())),
                Cell::from(Span::styled(&*m.size_label, theme::dim())),
                Cell::from(Span::styled(&*m.quantization, theme::dim())),
                Cell::from(Span::styled(&*m.context_length, theme::dim())),
                Cell::from(Span::styled(fit_label, fit_style)),
                Cell::from(Span::styled(&*m.estimated_toks, theme::dim())),
            ])
            .height(1)
        })
        .collect();

    let title = if search_query.is_empty() {
        " Recommended Models "
    } else {
        " Search Results "
    };

    let widths = [
        Constraint::Length(4),   // checkbox
        Constraint::Min(20),    // name
        Constraint::Length(7),   // params
        Constraint::Length(9),   // size
        Constraint::Length(8),   // quant
        Constraint::Length(9),   // context
        Constraint::Length(13),  // fit
        Constraint::Length(12),  // est tok/s
    ];

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(
            Block::default()
                .title(title)
                .title_style(theme::title())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::DIM)),
        )
        .row_highlight_style(theme::highlight())
        .highlight_symbol("▸ ");

    let mut state = TableState::default();
    state.select(Some(selected));
    frame.render_stateful_widget(table, chunks[2], &mut state);

    // Detail pane for selected model
    let detail = if selected < models.len() {
        let m = &models[selected];
        let fit = fits.get(selected).copied().unwrap_or(FitLevel::Marginal);
        vec![
            Line::from(vec![
                Span::styled("  Repo: ", Style::default().fg(theme::ACCENT)),
                Span::styled(&m.repo, theme::normal()),
                Span::styled("    Fit: ", Style::default().fg(theme::ACCENT)),
                Span::styled(fit.label(), match fit {
                    FitLevel::Perfect => theme::success(),
                    FitLevel::Good => Style::default().fg(theme::ACCENT),
                    FitLevel::Marginal => theme::warn(),
                    FitLevel::TooTight => theme::error(),
                }),
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
            "Space ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("toggle  |  ", theme::dim()),
        Span::styled(
            "Enter ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("download selected  |  ", theme::dim()),
        Span::styled(
            "/ ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("search  |  ", theme::dim()),
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
