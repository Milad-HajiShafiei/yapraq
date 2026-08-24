use crate::features::info::InfoState;
use crate::tui::theme::Theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn render(frame: &mut Frame, area: Rect, state: &InfoState, scroll_offset: usize) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Descriptive Machine Info ")
        .title_style(Theme::block_title())
        .border_style(Theme::border())
        .style(Style::default().bg(Theme::current().surface));

    let label_style = Style::default()
        .fg(Theme::current().accent)
        .add_modifier(Modifier::BOLD);
    let value_style = Style::default().fg(Theme::current().text);

    let text = vec![
        Line::from(vec![
            Span::styled("  Hostname:    ", label_style),
            Span::styled(&state.hostname, value_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  OS Name:     ", label_style),
            Span::styled(&state.os_name, value_style),
        ]),
        Line::from(vec![
            Span::styled("  OS Version:  ", label_style),
            Span::styled(&state.os_version, value_style),
        ]),
        Line::from(vec![
            Span::styled("  Kernel:      ", label_style),
            Span::styled(&state.kernel, value_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  CPU:         ", label_style),
            Span::styled(&state.cpu_brand, value_style),
        ]),
        Line::from(vec![
            Span::styled("  Cores/Threads: ", label_style),
            Span::styled(state.cpu_cores.to_string(), value_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Total RAM:   ", label_style),
            Span::styled(&state.total_ram, value_style),
        ]),
    ];

    let inner_height = block.inner(area).height as usize;
    let content_lines = text.len();
    let max_scroll = content_lines.saturating_sub(inner_height);
    let clamped_offset = scroll_offset.min(max_scroll);
    let paragraph = Paragraph::new(text)
        .block(block)
        .scroll((clamped_offset as u16, 0));
    frame.render_widget(paragraph, area);
}
