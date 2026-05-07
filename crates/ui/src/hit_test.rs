use foundry_tui_app::{AppModel, SectionFocus, Tab};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::layout_utils::{inset_rect, rect_contains};

pub fn section_at_position(
    model: &AppModel,
    screen: Rect,
    column: u16,
    row: u16,
) -> Option<SectionFocus> {
    if model.palette_open || model.anvil_prompt.is_some() || model.custom_modal.is_some() {
        return None;
    }

    let app = inset_rect(screen, 1, 1);
    if app.width == 0 || app.height == 0 {
        return None;
    }

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(12),
            Constraint::Length(10),
            Constraint::Length(2),
        ])
        .split(app);

    if rect_contains(outer[2], column, row) {
        return Some(SectionFocus::LogsPanel);
    }

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(76), Constraint::Percentage(24)])
        .split(outer[1]);

    if rect_contains(middle[1], column, row) {
        return Some(SectionFocus::JobsPanel);
    }

    if !rect_contains(middle[0], column, row) {
        return None;
    }

    if model.active_tab != Tab::Anvil {
        return Some(SectionFocus::MainPanel);
    }

    let anvil_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(middle[0]);

    if rect_contains(anvil_split[0], column, row) {
        Some(SectionFocus::AnvilInstancesPanel)
    } else if rect_contains(anvil_split[1], column, row) {
        Some(SectionFocus::AnvilInstanceLogsPanel)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use foundry_tui_app::AppController;
    use foundry_tui_config::AppConfig;
    use ratatui::layout::Rect;

    use super::{section_at_position, SectionFocus, Tab};

    fn model_for(tab: Tab) -> foundry_tui_app::AppModel {
        let mut controller = AppController::new(
            AppConfig::default(),
            PathBuf::from("."),
            PathBuf::from("config.toml"),
        );
        controller.model.active_tab = tab;
        controller.model
    }

    #[test]
    fn hover_focus_targets_primary_panes() {
        let model = model_for(Tab::Build);
        let screen = Rect::new(0, 0, 120, 40);

        assert_eq!(
            section_at_position(&model, screen, 8, 10),
            Some(SectionFocus::MainPanel)
        );
        assert_eq!(
            section_at_position(&model, screen, 100, 10),
            Some(SectionFocus::JobsPanel)
        );
        assert_eq!(
            section_at_position(&model, screen, 40, 30),
            Some(SectionFocus::LogsPanel)
        );
    }

    #[test]
    fn hover_focus_targets_anvil_subpanes() {
        let model = model_for(Tab::Anvil);
        let screen = Rect::new(0, 0, 120, 40);

        assert_eq!(
            section_at_position(&model, screen, 8, 10),
            Some(SectionFocus::AnvilInstancesPanel)
        );
        assert_eq!(
            section_at_position(&model, screen, 60, 10),
            Some(SectionFocus::AnvilInstanceLogsPanel)
        );
    }

    #[test]
    fn hover_focus_ignores_background_when_modal_open() {
        let mut model = model_for(Tab::Build);
        model.palette_open = true;
        let screen = Rect::new(0, 0, 120, 40);

        assert_eq!(section_at_position(&model, screen, 8, 10), None);
    }

    #[test]
    fn hover_focus_ignores_background_when_custom_modal_open() {
        let mut model = model_for(Tab::Custom);
        model.custom_modal = Some(foundry_tui_app::CustomCommandModal {
            step: foundry_tui_app::CustomModalStep::TemplatePicker,
            picker_index: 0,
            paste_mode: false,
            paste_input: String::new(),
            editor_index: 0,
            draft: None,
            error: None,
        });
        let screen = Rect::new(0, 0, 120, 40);

        assert_eq!(section_at_position(&model, screen, 8, 10), None);
    }
}
