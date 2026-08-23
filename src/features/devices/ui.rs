use crate::features::devices::DevicesState;
use crate::tui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

pub fn render(frame: &mut Frame, area: Rect, state: &DevicesState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(format!(" Connected USB Devices ({}) ", state.devices.len()))
        .title_style(Theme::block_title())
        .border_style(Theme::border())
        .style(Style::default().bg(Theme::current().surface));

    if state.devices.is_empty() {
        let msg = Paragraph::new("No USB devices detected or permission denied.")
            .style(Style::default().fg(Theme::current().muted))
            .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let rows = state.devices.iter().enumerate().map(|(i, dev)| {
        let vid_pid = format!("{:04x}:{:04x}", dev.vendor_id, dev.product_id);

        let style = if i == state.selected {
            Style::default()
                .bg(Theme::current().accent)
                .fg(Theme::current().background)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::current().text)
        };

        Row::new(vec![
            Cell::from(format!("Bus {}", dev.bus)),
            Cell::from(vid_pid).style(Style::default().fg(Theme::current().secondary)),
            Cell::from(dev.manufacturer.clone()),
            Cell::from(dev.product.clone()),
        ])
        .style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(35),
        ],
    )
    .header(
        Row::new(vec!["Bus", "VID:PID", "Manufacturer", "Product"]).style(
            Style::default()
                .fg(Theme::current().muted)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("➤ ");

    frame.render_widget(table, area);
}
