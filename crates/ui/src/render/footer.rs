use foundry_tui_app::AppModel;
use foundry_tui_config::ActionId;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::theme::UiTheme;

use super::common::key_for;

pub(super) fn render_footer(frame: &mut Frame<'_>, area: Rect, model: &AppModel, theme: UiTheme) {
    let spans = vec![
        Span::styled(" Palette ", Style::default().fg(theme.muted)),
        Span::styled(
            key_for(model, ActionId::OpenPalette),
            Style::default().fg(theme.accent),
        ),
        Span::styled("  Builder ", Style::default().fg(theme.muted)),
        Span::styled(
            key_for(model, ActionId::RunCustomCommand),
            Style::default().fg(theme.accent),
        ),
        Span::styled("  Tabs ", Style::default().fg(theme.muted)),
        Span::styled(
            format!(
                "{} / {}",
                key_for(model, ActionId::NextTab),
                key_for(model, ActionId::PrevTab)
            ),
            Style::default().fg(theme.accent),
        ),
        Span::styled("  Quit ", Style::default().fg(theme.muted)),
        Span::styled(
            key_for(model, ActionId::Quit),
            Style::default().fg(theme.accent),
        ),
        Span::styled("  Focus ", Style::default().fg(theme.muted)),
        Span::styled(
            format!(
                "{} / {}",
                key_for(model, ActionId::FocusPrevSection),
                key_for(model, ActionId::FocusNextSection)
            ),
            Style::default().fg(theme.accent),
        ),
        Span::styled("  Scroll ", Style::default().fg(theme.muted)),
        Span::styled(
            format!(
                "{} / {}",
                key_for(model, ActionId::ScrollLogsUp),
                key_for(model, ActionId::ScrollLogsDown)
            ),
            Style::default().fg(theme.accent),
        ),
        Span::styled("  Mouse ", Style::default().fg(theme.muted)),
        Span::styled(
            "F2 toggle/select",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    let help = Line::from(spans);

    let footer = Paragraph::new(help)
        .alignment(Alignment::Left)
        .style(Style::default().bg(theme.background).fg(theme.foreground));

    frame.render_widget(footer, area);
}
