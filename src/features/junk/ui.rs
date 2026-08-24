use crate::features::junk::JunkState;
use crate::tui::theme::Theme;
use crate::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
};

pub fn render(frame: &mut Frame, area: Rect, state: &JunkState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Summary bar
            Constraint::Min(0),    // Table
        ])
        .split(area);

    // Summary Bar
    let summary_text = if state.is_scanning {
        " Scanning for junk files... (This may take a minute) ".to_string()
    } else {
        format!(
            " Found {} junk items | Total Reclaimable: {} | [s]can | [d]elete selected | [D]elete ALL ",
            state.items.len(),
            format_bytes(state.total_size)
        )
    };

    let summary = Paragraph::new(summary_text)
        .style(
            Style::default()
                .fg(Theme::current().background)
                .bg(Theme::current().accent)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
    frame.render_widget(summary, chunks[0]);

    // Table
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Junk Files ")
        .title_style(Theme::block_title())
        .border_style(Theme::border())
        .style(Style::default().bg(Theme::current().surface));

    if state.items.is_empty() && !state.is_scanning {
        let empty = Paragraph::new("No junk files found. Press 's' to run a scan.")
            .style(Style::default().fg(Theme::current().muted))
            .block(block);
        frame.render_widget(empty, chunks[1]);
        return;
    }

    let rows = state.items.iter().enumerate().map(|(i, item)| {
        // FIXED: Map to String first, then unwrap_or_default
        let name = item
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let path = item
            .path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let style = if i == state.selected {
            Style::default()
                .bg(Theme::current().accent)
                .fg(Theme::current().background)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::current().text)
        };

        Row::new(vec![
            Cell::from(name.to_string()),
            Cell::from(path.to_string()).style(Style::default().fg(Theme::current().muted)),
            Cell::from(item.reason.clone()),
            Cell::from(format_bytes(item.size))
                .style(Style::default().fg(Theme::current().secondary)),
        ])
        .style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(
        Row::new(vec!["File Name", "Directory", "Reason", "Size"]).style(
            Style::default()
                .fg(Theme::current().muted)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("➤ ");

    let visible_rows = chunks[1].height.saturating_sub(3) as usize;
    let offset = if state.items.len() > visible_rows {
        state.selected.min(state.items.len() - visible_rows)
    } else {
        0
    };
    let mut table_state = TableState::new()
        .with_selected(Some(state.selected))
        .with_offset(offset);
    frame.render_stateful_widget(table, chunks[1], &mut table_state);
}
