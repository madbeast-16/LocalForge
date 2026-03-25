use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

use crate::inference::session::{ChatMessage, ChatRole};
use crate::tui::theme;

/// Render the chat screen.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    messages: &[ChatMessage],
    input: &str,
    model_name: &str,
    scroll_offset: usize,
    context_pct: f64,
    is_generating: bool,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  // Header bar
        Constraint::Min(6),    // Message area
        Constraint::Length(3),  // Input area
        Constraint::Length(2),  // Status bar
    ])
    .split(area);

    // ── Header bar ──
    let ctx_bar = format!("{}%", (context_pct * 100.0) as u16);
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" ◆ ", Style::default().fg(theme::ACCENT)),
        Span::styled("Chat", theme::title()),
        Span::styled("  │  ", theme::dim()),
        Span::styled("Model: ", theme::dim()),
        Span::styled(
            model_name,
            Style::default()
                .fg(theme::SUCCESS)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  Ctx: ", theme::dim()),
        Span::styled(
            ctx_bar,
            Style::default().fg(if context_pct > 0.8 {
                theme::WARN
            } else {
                theme::SUCCESS
            }),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(header, chunks[0]);

    // ── Message area ──
    let msg_lines = render_messages(messages, is_generating);
    let total_lines = msg_lines.len();
    let visible_height = chunks[1].height as usize;

    // Calculate scroll: show latest messages by default
    let max_scroll = total_lines.saturating_sub(visible_height);
    let actual_offset = scroll_offset.min(max_scroll);

    let visible_lines: Vec<Line> = msg_lines
        .into_iter()
        .skip(actual_offset)
        .take(visible_height)
        .collect();

    let msg_widget = Paragraph::new(visible_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::DIM)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(msg_widget, chunks[1]);

    // Scrollbar
    if total_lines > visible_height {
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(actual_offset);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        frame.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state);
    }

    // ── Input area ──
    let cursor_char = if is_generating { "" } else { "▌" };
    let input_text = format!(" {} {}{}", "›", input, cursor_char);
    let input_widget = Paragraph::new(Line::from(Span::styled(
        input_text,
        Style::default()
            .fg(theme::TEXT)
            .add_modifier(Modifier::BOLD),
    )))
    .block(
        Block::default()
            .title(if is_generating {
                " Generating... "
            } else {
                " Message "
            })
            .title_style(if is_generating {
                theme::warn()
            } else {
                theme::title()
            })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if is_generating {
                theme::WARN
            } else {
                theme::ACCENT
            })),
    );
    frame.render_widget(input_widget, chunks[2]);

    // ── Status bar ──
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " Enter ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("send  |  ", theme::dim()),
        Span::styled(
            "↑/↓ ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("scroll  |  ", theme::dim()),
        Span::styled(
            "Ctrl+N ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("new chat  |  ", theme::dim()),
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

/// Convert a chat message list into styled lines for rendering.
fn render_messages(messages: &[ChatMessage], is_generating: bool) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    if messages.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Start a conversation by typing a message below.",
            theme::dim(),
        )));
        lines.push(Line::from(""));
        return lines;
    }

    for msg in messages {
        match msg.role {
            ChatRole::System => {
                lines.push(Line::from(vec![
                    Span::styled("  ⚙ System: ", Style::default().fg(theme::DIM)),
                    Span::styled(msg.content.clone(), theme::dim()),
                ]));
                lines.push(Line::from(""));
            }
            ChatRole::User => {
                lines.push(Line::from(Span::styled(
                    format!("  ▸ You"),
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                )));
                for text_line in msg.content.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", text_line),
                        theme::normal(),
                    )));
                }
                lines.push(Line::from(""));
            }
            ChatRole::Assistant => {
                lines.push(Line::from(Span::styled(
                    format!("  ◆ Assistant"),
                    Style::default()
                        .fg(theme::SUCCESS)
                        .add_modifier(Modifier::BOLD),
                )));
                for text_line in msg.content.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", text_line),
                        theme::normal(),
                    )));
                }
                lines.push(Line::from(""));
            }
        }
    }

    if is_generating {
        lines.push(Line::from(Span::styled(
            "  ◆ Assistant ▍",
            Style::default()
                .fg(theme::WARN)
                .add_modifier(Modifier::BOLD),
        )));
    }

    lines
}
