use crate::app::App;
use crate::app::SettingsSection;
use crate::tui::theme::{Theme, ThemeKind};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

pub struct Settings;

impl Settings {
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let popup = centered_rect(60, 55, area);
        frame.render_widget(Clear, popup);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Settings ")
            .title_style(
                Style::default()
                    .fg(Theme::current().accent)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(Theme::current().accent))
            .style(Style::default().bg(Theme::current().surface));

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .margin(1)
            .split(block.inner(popup));

        frame.render_widget(block, popup);

        // Left panel: sections
        Self::render_sections(frame, chunks[0], app);

        // Right panel: section content
        match app.settings_section {
            SettingsSection::Theme => Self::render_theme_panel(frame, chunks[1], app),
            SettingsSection::About => Self::render_about_panel(frame, chunks[1]),
        }

        // Footer hint
        let footer_area = Rect {
            x: popup.x + 1,
            y: popup.y + popup.height - 2,
            width: popup.width - 2,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new("Esc/s close  ·  ↑↓/hl sections  ·  j/k items  ·  Enter select")
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .fg(Theme::current().muted)
                        .add_modifier(Modifier::ITALIC),
                ),
            footer_area,
        );
    }

    fn render_sections(frame: &mut Frame, area: Rect, app: &App) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Sections ")
            .title_style(Theme::block_title())
            .border_style(Style::default().fg(Theme::current().accent_soft))
            .style(Style::default().bg(Theme::current().surface));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let items: Vec<Line> = SettingsSection::ALL
            .iter()
            .map(|&section| {
                let active = section == app.settings_section;
                let style = if active {
                    Style::default()
                        .fg(Theme::current().background)
                        .bg(Theme::current().accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::current().text)
                };
                Line::from(Span::styled(
                    format!("  {}  ", section.label()),
                    style,
                ))
            })
            .collect();

        frame.render_widget(
            Paragraph::new(items).block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default().bg(Theme::current().surface)),
            ),
            inner,
        );
    }

    fn render_theme_panel(frame: &mut Frame, area: Rect, app: &App) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Theme ")
            .title_style(Theme::block_title())
            .border_style(Style::default().fg(Theme::current().accent_soft))
            .style(Style::default().bg(Theme::current().surface));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let current_kind = Theme::current_kind();
        let items: Vec<Line> = ThemeKind::ALL
            .iter()
            .enumerate()
            .map(|(i, &kind)| {
                let active = kind == current_kind;
                let selected = i == app.settings_selected;
                let indicator = if active { " ● " } else { " ○ " };
                let style = if active {
                    Style::default()
                        .fg(Theme::current().background)
                        .bg(Theme::current().success)
                        .add_modifier(Modifier::BOLD)
                } else if selected {
                    Style::default()
                        .fg(Theme::current().background)
                        .bg(Theme::current().accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::current().text)
                };
                Line::from(vec![
                    Span::styled(indicator, style),
                    Span::styled(
                        format!("{}", kind.name()),
                        style,
                    ),
                    if active {
                        Span::styled("  (active)", Style::default().fg(Theme::current().muted))
                    } else {
                        Span::raw("")
                    },
                ])
            })
            .collect();

        frame.render_widget(
            Paragraph::new(items).block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default().bg(Theme::current().surface)),
            ),
            inner,
        );
    }

    fn render_about_panel(frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" About ")
            .title_style(Theme::block_title())
            .border_style(Style::default().fg(Theme::current().accent_soft))
            .style(Style::default().bg(Theme::current().surface));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let current = Theme::current();
        let lines = vec![
            Line::from(vec![
                Span::styled("  ", Style::default().fg(Theme::current().success)),
                Span::styled(
                    "Yapraq",
                    Style::default()
                        .fg(Theme::current().accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  System Monitor & Manager",
                Style::default()
                    .fg(Theme::current().text)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("  Version 0.1.0  ·  Theme: {}", current.name),
                Style::default().fg(Theme::current().muted),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  A terminal-based workspace for monitoring",
                Style::default().fg(Theme::current().text),
            )),
            Line::from(Span::styled(
                "  system health, managing files, cleaning junk,",
                Style::default().fg(Theme::current().text),
            )),
            Line::from(Span::styled(
                "  and inspecting installed applications.",
                Style::default().fg(Theme::current().text),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press ? from the main view for help.",
                Style::default().fg(Theme::current().muted),
            )),
        ];

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default().bg(Theme::current().surface)),
            ),
            inner,
        );
    }
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
