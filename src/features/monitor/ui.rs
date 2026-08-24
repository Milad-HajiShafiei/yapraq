use crate::features::monitor::MonitorData;
use crate::tui::theme::Theme;
use crate::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Sparkline},
};

pub fn render(frame: &mut Frame, area: Rect, data: &MonitorData, scroll_offset: usize) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    let top = split_columns(rows[0]);
    let middle = split_columns(rows[1]);
    let bottom = split_columns(rows[2]);

    let cpu = Sparkline::default()
        .block(panel(format!(
            " CPU signal  ·  {}% ",
            data.cpu_history.last().unwrap_or(&0)
        )))
        .data(&data.cpu_history)
        .style(Style::default().fg(Theme::current().accent));
    frame.render_widget(cpu, top[0]);

    let memory = Gauge::default()
        .block(panel(format!(
            " Memory  ·  {} / {} ",
            format_bytes(data.mem_used),
            format_bytes(data.mem_total)
        )))
        .gauge_style(Style::default().fg(Theme::current().secondary))
        .label(format!("{}% used", data.mem_usage_percent.round() as u16))
        .percent(data.mem_usage_percent.round().clamp(0.0, 100.0) as u16);
    frame.render_widget(memory, top[1]);

    let memory_trend = Sparkline::default()
        .block(panel(" Memory trend "))
        .data(&data.memory_history)
        .style(Style::default().fg(Theme::current().secondary));
    frame.render_widget(memory_trend, middle[0]);

    let receive = Sparkline::default()
        .block(panel(" Network in  ·  KiB/s "))
        .data(&data.net_rx_history)
        .style(Style::default().fg(Theme::current().success));
    frame.render_widget(receive, middle[1]);

    let transmit = Sparkline::default()
        .block(panel(" Network out  ·  KiB/s "))
        .data(&data.net_tx_history)
        .style(Style::default().fg(Theme::current().accent_soft));
    frame.render_widget(transmit, bottom[0]);

    let peak_core = data.core_usage.iter().copied().max().unwrap_or(0);
    let core_activity = Sparkline::default()
        .block(panel(format!(" Core activity  ·  peak {}% ", peak_core)))
        .data(&data.core_usage)
        .style(Style::default().fg(Theme::current().accent));
    frame.render_widget(core_activity, bottom[1]);

    if rows.len() > 3 && rows[3].height > 1 {
        let snapshot = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!(" Live snapshot  ·  {} entries ", data.snapshot_history.len()))
            .title_style(Theme::block_title())
            .border_style(Theme::border())
            .style(Style::default().bg(Theme::current().surface_alt));

        let snapshot_text: Vec<Line> = data
            .snapshot_history
            .iter()
            .map(|entry| {
                Line::from(vec![
                    Span::styled(
                        " CPU ",
                        Style::default()
                            .fg(Theme::current().background)
                            .bg(Theme::current().accent),
                    ),
                    Span::styled(
                        format!(" {:>3}%  ", entry.cpu_percent),
                        Style::default()
                            .fg(Theme::current().text)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "RAM ",
                        Style::default()
                            .fg(Theme::current().background)
                            .bg(Theme::current().secondary),
                    ),
                    Span::styled(
                        format!(" {:>3}%  ", entry.ram_percent),
                        Style::default()
                            .fg(Theme::current().text)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(
                            "↓ {}  ↑ {}",
                            format_bytes(entry.net_rx),
                            format_bytes(entry.net_tx)
                        ),
                        Style::default().fg(Theme::current().muted),
                    ),
                ])
            })
            .collect();

        let max_scroll = snapshot_text.len().saturating_sub(rows[3].height as usize);
        let clamped_offset = scroll_offset.min(max_scroll);
        frame.render_widget(
            Paragraph::new(snapshot_text).block(snapshot).scroll((clamped_offset as u16, 0)),
            rows[3],
        );
    }
}

fn split_columns(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area)
        .to_vec()
}

fn panel(title: impl Into<String>) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title.into())
        .title_style(Theme::block_title())
        .border_style(Theme::border())
        .style(Style::default().bg(Theme::current().surface))
}
