use crate::features::packages::PackagesState;
use crate::tui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
};

pub fn render(frame: &mut Frame, area: Rect, state: &PackagesState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(format!(" System Packages ({}) ", state.items.len()))
        .title_style(Theme::block_title())
        .border_style(Theme::border())
        .style(Style::default().bg(Theme::current().surface));

    let rows = state.items.iter().enumerate().map(|(i, pkg)| {
        let style = if i == state.selected {
            Style::default()
                .bg(Theme::current().accent)
                .fg(Theme::current().background)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::current().text)
        };
        Row::new(vec![
            Cell::from(pkg.name.clone()),
            Cell::from(pkg.version.clone()).style(Style::default().fg(Theme::current().muted)),
            Cell::from(pkg.manager.clone()).style(Style::default().fg(Theme::current().secondary)),
        ])
        .style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["Package", "Version", "Manager"]).style(
            Style::default()
                .fg(Theme::current().muted)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("> ");

    // Auto-scroll: keep selected row visible
    let visible_rows = chunks[0].height.saturating_sub(3) as usize; // borders + header
    let offset = if state.items.len() > visible_rows {
        state.selected.min(state.items.len() - visible_rows)
    } else {
        0
    };
    let mut table_state = TableState::new()
        .with_selected(Some(state.selected))
        .with_offset(offset);

    frame.render_stateful_widget(table, chunks[0], &mut table_state);

    // Uninstall Command Bar
    let cmd_text = if state.uninstall_cmd.is_empty() {
        " [i] to show uninstall command "
    } else {
        state.uninstall_cmd.as_str()
    };
    let cmd_bar = Paragraph::new(cmd_text)
        .style(
            Style::default()
                .fg(Theme::current().background)
                .bg(Theme::current().secondary)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
    frame.render_widget(cmd_bar, chunks[1]);
}
