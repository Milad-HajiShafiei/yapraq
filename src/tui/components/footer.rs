use crate::{
    app::{App, Tab},
    tui::theme::Theme,
};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub struct Footer;

impl Footer {
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let mut hints = vec![
            ("↑↓", "navigate"),
            ("j/k", "select"),
            ("Ctrl+S", "settings"),
            ("?", "help"),
            ("q", "quit"),
        ];
        if app.current_tab == Tab::Files {
            hints = vec![
                ("↑↓", "tabs"),
                ("j/k", "select"),
                ("Enter", "open"),
                ("n", "folder"),
                ("f", "file"),
                ("e", "rename"),
                ("x", "delete"),
                ("r", "refresh"),
                ("?", "help"),
            ];
        } else if app.current_tab == Tab::Apps || app.current_tab == Tab::Packages || app.current_tab == Tab::Junk {
            hints.insert(1, ("s", "scan"));
        }

        let mut spans = Vec::new();
        for (index, (key, label)) in hints.into_iter().enumerate() {
            if index > 0 {
                spans.push(Span::styled("  ·  ", Style::default().fg(Theme::current().muted)));
            }
            spans.push(Span::styled(
                format!(" {key} "),
                Style::default()
                    .fg(Theme::current().background)
                    .bg(Theme::current().accent)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!(" {label}"),
                Style::default().fg(Theme::current().text),
            ));
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Shortcuts ")
            .title_style(Theme::block_title())
            .border_style(Theme::border())
            .style(Style::default().bg(Theme::current().surface));

        frame.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
    }
}
