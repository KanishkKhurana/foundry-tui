use foundry_tui_app::AppModel;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Tabs},
    Frame,
};

use crate::{
    style_utils::{themed_block, BoxTone},
    theme::UiTheme,
};

pub(super) fn render_header(frame: &mut Frame<'_>, area: Rect, model: &AppModel, theme: UiTheme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(2)])
        .split(area);

    let meta = format!(
        "root={} | launched={}",
        model.project_root.display(),
        model.launched_at.format("%H:%M:%S")
    );

    let status = model
        .notification
        .clone()
        .unwrap_or_else(|| "Ready".to_string());
    let focus = format!("focus={}", model.focused_section.label());

    let compact = area.width < 95;
    if compact {
        let title_widget = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "FOUNDRY TUI",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(status.clone(), Style::default().fg(theme.accent)),
            ]),
            Line::from(vec![
                Span::styled(meta.clone(), Style::default().fg(theme.foreground)),
                Span::raw("  "),
                Span::styled(focus.clone(), Style::default().fg(theme.muted)),
            ]),
            Line::from(""),
        ]);
        frame.render_widget(title_widget, chunks[0]);
    } else {
        let header_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(10), Constraint::Min(10)])
            .split(chunks[0]);

        let logo_style = Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD);
        let logo_widget = Paragraph::new(vec![
            Line::from(Span::styled("  .-^-.", logo_style)),
            Line::from(Span::styled("  /|X|\\", logo_style)),
            Line::from(Span::styled("   /_\\ ", logo_style)),
        ]);
        frame.render_widget(logo_widget, header_split[0]);

        let info_widget = Paragraph::new(vec![
            Line::from(Span::styled("FOUNDRY TUI", logo_style)),
            Line::from(Span::styled(meta, Style::default().fg(theme.foreground))),
            Line::from(vec![
                Span::styled(status, Style::default().fg(theme.accent)),
                Span::raw("  "),
                Span::styled(focus, Style::default().fg(theme.muted)),
            ]),
        ]);
        frame.render_widget(info_widget, header_split[1]);
    }

    let tab_titles = model
        .tabs
        .iter()
        .map(|tab| Line::from(Span::raw(format!(" {} ", tab.title()))))
        .collect::<Vec<_>>();

    let tabs = Tabs::new(tab_titles)
        .block(themed_block(theme, BoxTone::Normal, theme.panel).title("Workflows"))
        .style(Style::default().fg(theme.muted))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .select(model.active_tab_index());

    frame.render_widget(tabs, chunks[1]);
}
