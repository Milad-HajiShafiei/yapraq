use crate::{
    app::{App, Tab},
    tui::theme::Theme,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

pub struct Help;

impl Help {
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let popup = centered_rect(76, 78, area);
        frame.render_widget(Clear, popup);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Help & keyboard reference ")
            .title_style(Theme::block_title())
            .border_style(Style::default().fg(Theme::current().accent))
            .style(Style::default().bg(Theme::current().surface));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0), Constraint::Length(2)])
            .split(block.inner(popup));

        let mut lines = vec![
            section("Navigation"),
            shortcut("↑ / ↓", "Move between sidebar sections"),
            shortcut("1 - 8 / F1 - F8", "Jump directly to a section"),
            shortcut("j / k", "Move through the current list"),
            shortcut("Auto-scroll", "Content scrolls automatically"),
            shortcut("? / Esc", "Close this help"),
            Line::from(""),
            section("General actions"),
            shortcut("q / Ctrl-C", "Quit"),
            shortcut("Ctrl+S", "Open settings (theme, about)"),
            shortcut("Enter", "Open the selected file or folder"),
            shortcut("Backspace", "Go to the parent folder in Files"),
        ];

        if app.current_tab == Tab::Files {
            lines.extend([
                Line::from(""),
                section("File workspace"),
                shortcut("n", "Create a folder"),
                shortcut("f", "Create a file"),
                shortcut("e", "Rename the selected item"),
                shortcut("x", "Delete the selected item"),
                shortcut("r", "Refresh the current folder"),
            ]);
        }

        lines.extend([
            Line::from(""),
            section("Tools"),
            shortcut("s", "Scan (context-sensitive)"),
            shortcut("d / D", "Delete selected / all junk"),
            shortcut("r", "Refresh (devices/files)"),
        ]);

        let content = Paragraph::new(lines).block(block.clone());
        frame.render_widget(content, popup);
        frame.render_widget(
            Paragraph::new(" Press ? or Esc to close ")
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .fg(Theme::current().muted)
                        .add_modifier(Modifier::ITALIC),
                ),
            chunks[1],
        );
    }
}

fn section(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  {title}"),
        Style::default()
            .fg(Theme::current().secondary)
            .add_modifier(Modifier::BOLD),
    ))
}

fn shortcut(key: &str, description: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {key:<16}"),
            Style::default()
                .fg(Theme::current().accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(description.to_string(), Style::default().fg(Theme::current().text)),
    ])
}

fn centered_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}
