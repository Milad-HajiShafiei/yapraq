use crate::features::files::FilesState;
use crate::tui::theme::Theme;
use crate::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

pub fn render(frame: &mut Frame, area: Rect, state: &FilesState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);
    let title = if state.is_scanning {
        format!(" Scanning: {} ", state.current_dir.display())
    } else {
        format!(" {} ", state.current_dir.display())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .title_style(Theme::block_title())
        .border_style(Theme::border())
        .style(Style::default().bg(Theme::current().surface));

    if state.is_scanning && state.items.is_empty() {
        frame.render_widget(
            Paragraph::new("Calculating folder sizes...")
                .style(Style::default().fg(Theme::current().muted))
                .block(block),
            layout[0],
        );
        render_hint(frame, layout[1], "Scanning folder contents...");
        return;
    }

    if state.items.is_empty() {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "This folder is empty",
                    Style::default()
                        .fg(Theme::current().text)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "Use n to create a folder or f to create a file.",
                    Style::default().fg(Theme::current().muted),
                )),
            ])
            .block(block),
            layout[0],
        );
        render_hint(frame, layout[1], "n folder  ·  f file  ·  ? help");
        return;
    }

    let rows = state.items.iter().enumerate().map(|(index, item)| {
        let icon = if item.is_dir { "▰" } else { "▪" };
        let kind = if item.is_dir { "Folder" } else { "File" };
        let row_style = if index == state.selected {
            Style::default()
                .bg(Theme::current().accent)
                .fg(Theme::current().background)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::current().text)
        };
        Row::new(vec![
            Cell::from(format!(" {icon} {}", item.name)),
            Cell::from(kind).style(Style::default().fg(Theme::current().muted)),
            Cell::from(format_bytes(item.size)).style(Style::default().fg(Theme::current().accent)),
        ])
        .style(row_style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(64),
            Constraint::Percentage(16),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["Name", "Type", "Size"]).style(
            Style::default()
                .fg(Theme::current().muted)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(block)
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("▌");

    frame.render_widget(table, layout[0]);
    render_hint(
        frame,
        layout[1],
        "Enter open  ·  n folder  ·  f file  ·  e rename  ·  x delete",
    );
}

fn render_hint(frame: &mut Frame, area: Rect, text: &str) {
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("  {text}"),
            Style::default().fg(Theme::current().muted),
        )))
        .style(Style::default().bg(Theme::current().background)),
        area,
    );
}
