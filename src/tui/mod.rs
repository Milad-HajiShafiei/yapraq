pub mod components;
pub mod theme;

use self::components::{
    footer::Footer, header::Header, help::Help, settings::Settings, sidebar::Sidebar,
};
use crate::app::{App, FileDialog, Tab};
use crate::tui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

pub struct UI;

impl UI {
    pub fn draw(frame: &mut Frame, app: &App) {
        frame.render_widget(
            Block::default().style(Style::default().bg(Theme::current().background)),
            frame.area(),
        );

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(frame.area());

        Header::render(frame, main_chunks[0], app);
        Footer::render(frame, main_chunks[2], app);

        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(main_chunks[1]);

        Sidebar::render(frame, body_chunks[0], &app.current_tab);

        match app.current_tab {
            Tab::Monitor => crate::features::monitor::ui::render(
                frame,
                body_chunks[1],
                &app.monitor,
                app.scroll_offset,
            ),
            Tab::Apps => crate::features::apps::ui::render(frame, body_chunks[1], &app.apps),
            Tab::Packages => {
                crate::features::packages::ui::render(frame, body_chunks[1], &app.packages)
            }
            Tab::Files => crate::features::files::ui::render(frame, body_chunks[1], &app.files),
            Tab::Storage => {
                crate::features::storage::ui::render(frame, body_chunks[1], &app.storage)
            }
            Tab::Junk => crate::features::junk::ui::render(frame, body_chunks[1], &app.junk),
            Tab::Devices => {
                crate::features::devices::ui::render(frame, body_chunks[1], &app.devices)
            }
            Tab::Info => crate::features::info::ui::render(
                frame,
                body_chunks[1],
                &app.info,
                app.scroll_offset,
            ),
        }

        if app.show_settings {
            render_backdrop(frame, frame.area());
            Settings::render(frame, frame.area(), app);
        } else if app.show_help {
            render_backdrop(frame, frame.area());
            Help::render(frame, frame.area(), app);
        } else if let Some(dialog) = app.file_dialog {
            render_backdrop(frame, frame.area());
            render_input_dialog(frame, frame.area(), app, dialog);
        } else if let Some(target) = &app.delete_confirmation {
            render_backdrop(frame, frame.area());
            render_delete_dialog(frame, frame.area(), target);
        }
    }
}

fn render_backdrop(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Block::default().style(Style::default().bg(Theme::current().background)),
        area,
    );
}

fn render_input_dialog(frame: &mut Frame, area: Rect, app: &App, dialog: FileDialog) {
    let popup = centered_rect(62, 30, area);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(format!(" {} ", dialog.title()))
        .title_style(Theme::block_title())
        .border_style(Style::default().fg(Theme::current().accent))
        .style(Style::default().bg(Theme::current().surface));
    let inner = block.inner(popup);
    let content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .margin(1)
        .split(inner);
    frame.render_widget(block, popup);
    frame.render_widget(
        Paragraph::new(dialog.prompt()).style(Style::default().fg(Theme::current().muted)),
        content[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("> ", Style::default().fg(Theme::current().accent)),
            Span::styled(
                &app.dialog_input,
                Style::default().fg(Theme::current().text),
            ),
            Span::styled("_", Style::default().fg(Theme::current().accent)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Theme::border()),
        ),
        content[1],
    );
    frame.render_widget(
        Paragraph::new("Enter confirm  ·  Esc cancel")
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Theme::current().muted)
                    .add_modifier(Modifier::ITALIC),
            ),
        content[2],
    );
}

fn render_delete_dialog(frame: &mut Frame, area: Rect, target: &std::path::Path) {
    let popup = centered_rect(68, 28, area);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Confirm deletion ")
        .title_style(
            Style::default()
                .fg(Theme::current().secondary)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(Theme::current().secondary))
        .style(Style::default().bg(Theme::current().surface));
    let text = vec![
        Line::from(Span::styled(
            "Delete this item permanently?",
            Style::default()
                .fg(Theme::current().text)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            target.display().to_string(),
            Style::default().fg(Theme::current().muted),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " y/Enter ",
                Style::default()
                    .fg(Theme::current().background)
                    .bg(Theme::current().secondary),
            ),
            Span::styled(" delete    ", Style::default().fg(Theme::current().text)),
            Span::styled(
                " n/Esc ",
                Style::default()
                    .fg(Theme::current().background)
                    .bg(Theme::current().accent),
            ),
            Span::styled(" cancel", Style::default().fg(Theme::current().text)),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true }),
        popup,
    );
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
