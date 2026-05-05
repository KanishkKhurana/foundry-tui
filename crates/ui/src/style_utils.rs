use ratatui::{
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Padding},
};

use crate::theme::UiTheme;

#[derive(Debug, Clone, Copy)]
pub(crate) enum BoxTone {
    AppFrame,
    Normal,
    Focused,
    Warning,
    Error,
    Modal,
}

pub(crate) fn themed_block(theme: UiTheme, tone: BoxTone, background: Color) -> Block<'static> {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(match tone {
            BoxTone::Focused => BorderType::Double,
            _ => BorderType::Plain,
        })
        .border_style(Style::default().fg(match tone {
            BoxTone::AppFrame => theme.app_frame_border,
            BoxTone::Normal => theme.panel_border,
            BoxTone::Focused => theme.focus_border,
            BoxTone::Warning => theme.warning,
            BoxTone::Error => theme.danger,
            BoxTone::Modal => theme.modal_border,
        }))
        .style(Style::default().bg(background).fg(theme.foreground));

    if !matches!(tone, BoxTone::AppFrame) {
        block = block.padding(Padding::new(1, 1, 0, 0));
    }

    block
}
