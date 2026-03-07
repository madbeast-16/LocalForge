use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::hardware::HardwareInfo;
use crate::tui::theme;

pub fn render(frame: &mut Frame, area: Rect, hw: &HardwareInfo) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(2),
    ])
    .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" ◆ ", Style::default().fg(theme::ACCENT)),
        Span::styled("Hardware Detection", theme::title()),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(header, chunks[0]);

    // Hardware details
    let vram_str = HardwareInfo::format_memory(hw.gpu.vram_mb);
    let ram_str = HardwareInfo::format_memory(hw.memory.total_mb);
    let avail_str = HardwareInfo::format_memory(hw.memory.available_mb);

    let detail_lines = vec![
        format!("  CPU          {}", hw.cpu.name),
        format!("  Arch         {}", hw.cpu.architecture),
        format!(
            "  Cores        {} physical / {} logical",
            hw.cpu.cores_physical, hw.cpu.cores_logical
        ),
        String::new(),
        format!("  GPU          {}", hw.gpu.name),
        format!("  Vendor       {}", hw.gpu.vendor.as_str()),
        format!("  VRAM         {}", vram_str),
        String::new(),
        format!("  RAM          {} ({} available)", ram_str, avail_str),
        format!("  OS           {}", hw.os),
        format!(
            "  CUDA         {}",
            if hw.cuda_available {
                hw.cuda_version.as_deref().unwrap_or("Available")
            } else {
                "Not available"
            }
        ),
        format!(
            "  ROCm         {}",
            if hw.rocm_available {
                "Available"
            } else {
                "Not available"
            }
        ),
    ];

    let lines: Vec<Line> = detail_lines
        .iter()
        .map(|l| {
            if l.is_empty() {
                Line::from("")
            } else {
                Line::from(Span::styled(l.as_str(), theme::normal()))
            }
        })
        .collect();

    let details = Paragraph::new(lines).block(
        Block::default()
            .title(" System Information ")
            .title_style(theme::title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::DIM)),
    );
    frame.render_widget(details, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " Enter ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("continue  |  ", theme::dim()),
        Span::styled(
            "Esc ",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("back", theme::dim()),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}
