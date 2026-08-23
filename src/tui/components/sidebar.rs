use crate::app::Tab;
use crate::tui::theme::Theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

pub struct Sidebar;

impl Sidebar {
    pub fn render(frame: &mut Frame, area: Rect, current_tab: &Tab) {
        let items: Vec<ListItem> = Tab::ALL
            .into_iter()
            .map(|tab| {
                let (icon, description) = match tab {
                    Tab::Monitor => ("◉", "Live system health"),
                    Tab::Apps => ("▣", "Installed applications"),
                    Tab::Packages => ("◇", "Package inventory"),
                    Tab::Files => ("▤", "Browse and manage files"),
                    Tab::Storage => ("◫", "Disk usage overview"),
                    Tab::Junk => ("⌁", "Find reclaimable files"),
                    Tab::Devices => ("♧", "Connected USB devices"),
                    Tab::Info => ("ⓘ", "Machine details"),
                };
                let active = tab == *current_tab;
                let style = if active {
                    Theme::active_tab()
                } else {
                    Style::default().fg(Theme::current().text)
                };
                let description_style = if active {
                    Style::default()
                        .fg(Theme::current().accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Theme::current().muted)
                };
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(format!(" {icon} "), style.add_modifier(Modifier::BOLD)),
                        Span::styled(tab.label(), style.add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(Span::styled(
                        format!("    {description}"),
                        description_style,
                    )),
                ])
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Navigation ")
            .title_style(Theme::block_title())
            .border_style(Theme::border())
            .style(Style::default().bg(Theme::current().surface));

        let list = List::new(items)
            .block(block)
            .highlight_style(Theme::active_tab())
            .highlight_symbol("▌");

        frame.render_widget(list, area);
    }
}
