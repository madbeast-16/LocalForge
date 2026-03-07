use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::config::BackendName;
use crate::tui::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    backends: &[BackendName],
    selected: usize,
    recommended: Option<BackendName>,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(6),
        Constraint::Length(3),
        Constraint::Length(2),
    ])
    .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" ◆ ", Style::default().fg(theme::ACCENT)),
        Span::styled("Backend Selection", theme::title()),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(header, chunks[0]);

    // Backend list
    let items: Vec<ListItem> = backends
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let is_recommended = recommended.map_or(false, |r| r == *b);
            let marker = if is_recommended { " (recommended)" } else { "" };
            let label = format!("  {}{}", b.as_str(), marker);

            let style = if i == selected {
                theme::highlight()
            } else if is_recommended {
                Style::default().fg(theme::SUCCESS)
            } else {
                theme::normal()
            };

            ListItem::new(Line::from(Span::styled(label, style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Available Backends ")
            .title_style(theme::title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );

    let mut state = ListState::default();
    state.select(Some(selected));
    frame.render_stateful_widget(list, chunks[1], &mut state);

    // Description of selected backend
    let desc = if selected < backends.len() {
        backend_description(backends[selected])
    } else {
        ""
    };
    let desc_widget = Paragraph::new(Line::from(vec![
        Span::styled("  ", theme::dim()),
        Span::styled(desc, theme::normal()),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(desc_widget, chunks[2]);

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
    frame.render_widget(footer, chunks[3]);
}

fn backend_description(b: BackendName) -> &'static str {
    match b {
        BackendName::Cuda => {
            "NVIDIA GPU acceleration via CUDA. Best for NVIDIA cards with compute capability >= 6.0"
        }
        BackendName::Hip => "AMD GPU acceleration via ROCm/HIP. Requires ROCm driver stack",
        BackendName::Metal => "Apple Silicon GPU via Metal. Native on macOS with M-series chips",
        BackendName::Vulkan => "Cross-platform GPU via Vulkan. Works with Intel, AMD, and NVIDIA",
        BackendName::OpenBlas => "CPU with BLAS acceleration. Requires OpenBLAS library installed",
        BackendName::CpuOnly => {
            "Pure CPU build with native instruction optimisation. No dependencies"
        }
    }
}
