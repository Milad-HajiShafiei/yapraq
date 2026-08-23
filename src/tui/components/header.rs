use crate::{
    app::{App, Tab},
    tui::theme::Theme,
};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub struct Header;

impl Header {
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let title = Line::from(vec![
            Span::styled("  ", Style::default().fg(Theme::current().success)),
            Span::styled(" Yapraq ", Theme::active_tab()),
            Span::styled(
                format!("  {}", app.current_tab.label()),
                Style::default()
                    .fg(Theme::current().text)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        let subtitle = if app.current_tab == Tab::Files {
            format!("  {}", app.files.current_dir.display())
        } else {
            format!("  {}", app.status_message)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" System workspace ")
            .title_style(Theme::block_title())
            .border_style(Theme::border())
            .style(Style::default().bg(Theme::current().surface));

        let content = Paragraph::new(vec![
            title,
            Line::from(Span::styled(subtitle, Style::default().fg(Theme::current().muted))),
        ])
        .alignment(Alignment::Left)
        .block(block);
        frame.render_widget(content, area);
    }
}
