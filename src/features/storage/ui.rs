use crate::features::storage::{StorageData, state::usage_percent};
use crate::tui::theme::Theme;
use crate::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
};

pub fn render(frame: &mut Frame, area: Rect, data: &StorageData) {
    let disks = data.disks.list();
    let totals = disks.iter().fold((0_u64, 0_u64), |(total, free), disk| {
        (
            total.saturating_add(disk.total_space()),
            free.saturating_add(disk.available_space()),
        )
    });
    let used = totals.0.saturating_sub(totals.1);
    let percent = usage_percent(used, totals.0);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    render_summary(frame, layout[0], disks.len(), totals.0, totals.1, used);

    let overview = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Overall capacity ")
                .title_style(Theme::block_title())
                .border_style(Theme::border())
                .style(Style::default().bg(Theme::current().surface)),
        )
        .gauge_style(Style::default().fg(usage_color(percent)))
        .label(format!(
            "{percent}% used  ·  {} free",
            format_bytes(totals.1)
        ))
        .percent(percent);
    frame.render_widget(overview, layout[1]);

    render_disks(frame, layout[2], disks);
}

fn render_summary(
    frame: &mut Frame,
    area: Rect,
    disk_count: usize,
    total: u64,
    free: u64,
    used: u64,
) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    render_metric(
        frame,
        columns[0],
        "Volumes",
        disk_count.to_string(),
        Theme::current().accent,
    );
    render_metric(
        frame,
        columns[1],
        "Capacity",
        format_bytes(total),
        Theme::current().text,
    );
    render_metric(
        frame,
        columns[2],
        "In use",
        format_bytes(used),
        Theme::current().secondary,
    );
    render_metric(
        frame,
        columns[3],
        "Available",
        format_bytes(free),
        Theme::current().success,
    );
}

fn render_metric(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: String,
    color: ratatui::style::Color,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border())
        .style(Style::default().bg(Theme::current().surface));
    let content = Paragraph::new(vec![
        Line::from(Span::styled(
            format!("  {label}"),
            Style::default().fg(Theme::current().muted),
        )),
        Line::from(Span::styled(
            format!("  {value}"),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
    ])
    .block(block);
    frame.render_widget(content, area);
}

fn render_disks(frame: &mut Frame, area: Rect, disks: &[sysinfo::Disk]) {
    if disks.is_empty() {
        let empty = Paragraph::new("No mounted disks were reported by the operating system.")
            .style(Style::default().fg(Theme::current().muted))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Mounted volumes ")
                    .title_style(Theme::block_title())
                    .border_style(Theme::border())
                    .style(Style::default().bg(Theme::current().surface)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let constraints: Vec<Constraint> = disks.iter().map(|_| Constraint::Length(4)).collect();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (index, disk) in disks.iter().enumerate() {
        if index >= rows.len() {
            break;
        }
        let total = disk.total_space();
        let free = disk.available_space();
        let used = total.saturating_sub(free);
        let percent = usage_percent(used, total);
        let name = disk.name().to_string_lossy();
        let mount = disk.mount_point().display();
        let label = format!(
            " {}  ·  {} / {}  ·  {} free",
            name,
            format_bytes(used),
            format_bytes(total),
            format_bytes(free)
        );
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(format!(" {mount} "))
                    .title_style(Theme::block_title())
                    .border_style(Theme::border())
                    .style(Style::default().bg(Theme::current().surface)),
            )
            .gauge_style(Style::default().fg(usage_color(percent)))
            .label(label)
            .percent(percent);
        frame.render_widget(gauge, rows[index]);
    }
}

fn usage_color(percent: u16) -> ratatui::style::Color {
    match percent {
        0..=69 => Theme::current().success,
        70..=89 => Theme::current().accent,
        _ => Theme::current().secondary,
    }
}
