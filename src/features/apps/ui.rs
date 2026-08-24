use crate::features::apps::AppsState;
use crate::tui::theme::Theme;
use crate::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppsState) {
    let title = if state.is_scanning {
        " Scanning Applications... "
    } else {
        " Installed Applications "
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .title_style(Theme::block_title())
        .border_style(Theme::border())
        .style(Style::default().bg(Theme::current().surface));

    if state.is_scanning {
        frame.render_widget(Paragraph::new("Finding apps...").block(block), area);
        return;
    }

    let rows = state.items.iter().enumerate().map(|(i, app)| {
        let style = if i == state.selected {
            Style::default()
                .bg(Theme::current().accent)
                .fg(Theme::current().background)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::current().text)
        };
        Row::new(vec![
            Cell::from(app.name.clone()),
            Cell::from(app.path.to_string_lossy().to_string())
                .style(Style::default().fg(Theme::current().muted)),
            Cell::from(format_bytes(app.size))
                .style(Style::default().fg(Theme::current().secondary)),
        ])
        .style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(50),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["App Name", "Location", "Size"]).style(
            Style::default()
                .fg(Theme::current().muted)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("➤ ");

    let visible_rows = area.height.saturating_sub(3) as usize;
    let offset = if state.items.len() > visible_rows {
        state.selected.min(state.items.len() - visible_rows)
    } else {
        0
    };
    let mut table_state = TableState::new()
        .with_selected(Some(state.selected))
        .with_offset(offset);
    frame.render_stateful_widget(table, area, &mut table_state);
}
