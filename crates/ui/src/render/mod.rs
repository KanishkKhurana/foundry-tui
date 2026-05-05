mod common;
mod footer;
mod header;
mod modals;
mod palette;
mod panels;
mod tabs;

use foundry_tui_app::AppModel;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::{
    style_utils::{themed_block, BoxTone},
    theme::UiTheme,
};

pub fn draw(frame: &mut Frame<'_>, model: &AppModel, theme: UiTheme) {
    let screen = frame.size();
    let shell = themed_block(theme, BoxTone::AppFrame, theme.background);
    let app = shell.inner(screen);
    frame.render_widget(shell, screen);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(12),
            Constraint::Length(10),
            Constraint::Length(2),
        ])
        .split(app);

    header::render_header(frame, outer[0], model, theme);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(outer[1]);

    tabs::render_active_tab(frame, middle[0], model, theme);
    panels::render_jobs(frame, middle[1], model, theme);
    panels::render_logs(frame, outer[2], model, theme);
    footer::render_footer(frame, outer[3], model, theme);

    if model.palette_open {
        palette::render_palette(frame, model, theme);
    }

    if model.anvil_prompt.is_some() {
        modals::render_anvil_prompt(frame, model, theme);
    }

    if model.custom_modal.is_some() {
        modals::render_custom_modal(frame, model, theme);
    }
}
