use foundry_tui_app::AppModel;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding},
    Frame,
};

use crate::{layout_utils::centered_rect, theme::UiTheme};

use super::common::key_for;

pub(super) fn render_palette(frame: &mut Frame<'_>, model: &AppModel, theme: UiTheme) {
    let screen = frame.size();
    let overlay = Block::default().style(Style::default().bg(theme.muted));
    frame.render_widget(overlay, screen);

    let area = centered_rect(60, 60, frame.size());
    frame.render_widget(Clear, area);

    let items = model
        .palette_actions
        .iter()
        .enumerate()
        .map(|(index, action)| {
            let marker = if index == model.palette_index {
                ">"
            } else {
                " "
            };
            let key = key_for(model, *action);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{marker} "),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(action.label(), Style::default().fg(theme.foreground)),
                Span::styled(format!("  ({key})"), Style::default().fg(theme.muted)),
            ]))
        })
        .collect::<Vec<_>>();

    let palette = List::new(items).block(
        Block::default()
            .title("Command Palette")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .style(
                Style::default()
                    .bg(theme.modal_background)
                    .fg(theme.foreground),
            )
            .padding(Padding::new(1, 1, 0, 0)),
    );

    frame.render_widget(palette, area);
}
