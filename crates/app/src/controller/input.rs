use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind};
use foundry_tui_foundry::ToolEvent;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    model::{CustomCommandModal, CustomModalStep},
    parsing::{anvil_prompt_field_mut, custom_form_placeholders, initial_placeholder_value},
};

use super::AppController;

impl AppController {
    pub fn handle_key(&mut self, key: KeyEvent, tool_events: &UnboundedSender<ToolEvent>) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        if self.model.custom_modal.is_some() {
            self.handle_custom_modal_key(key, tool_events);
            return;
        }

        if self.model.anvil_prompt.is_some() {
            self.handle_anvil_prompt_key(key, tool_events);
            return;
        }

        if self.model.palette_open {
            self.handle_palette_key(key, tool_events);
            return;
        }

        if let Some(action) = self.resolve_action(key) {
            self.execute_action(action, tool_events);
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        if self.model.custom_modal.is_some() || self.model.anvil_prompt.is_some() {
            return;
        }

        match mouse.kind {
            MouseEventKind::ScrollDown => {
                if self.model.palette_open {
                    if !self.model.palette_actions.is_empty() {
                        self.model.palette_index =
                            (self.model.palette_index + 1) % self.model.palette_actions.len();
                    }
                    return;
                }
                self.scroll_focused_section(true);
            }
            MouseEventKind::ScrollUp => {
                if self.model.palette_open {
                    if self.model.palette_index == 0 {
                        self.model.palette_index =
                            self.model.palette_actions.len().saturating_sub(1);
                    } else {
                        self.model.palette_index -= 1;
                    }
                    return;
                }
                self.scroll_focused_section(false);
            }
            _ => {}
        }
    }

    fn handle_palette_key(&mut self, key: KeyEvent, tool_events: &UnboundedSender<ToolEvent>) {
        match key.code {
            KeyCode::Esc => {
                self.model.palette_open = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.model.palette_index == 0 {
                    self.model.palette_index = self.model.palette_actions.len().saturating_sub(1);
                } else {
                    self.model.palette_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.model.palette_index =
                    (self.model.palette_index + 1) % self.model.palette_actions.len();
            }
            KeyCode::Enter => {
                if let Some(action) = self
                    .model
                    .palette_actions
                    .get(self.model.palette_index)
                    .copied()
                {
                    self.execute_action(action, tool_events);
                }
                self.model.palette_open = false;
            }
            _ => {}
        }
    }

    fn handle_custom_modal_key(&mut self, key: KeyEvent, tool_events: &UnboundedSender<ToolEvent>) {
        let Some(mut modal) = self.model.custom_modal.take() else {
            return;
        };

        let close_modal = match modal.step {
            CustomModalStep::TemplatePicker => self.handle_custom_picker_key(&mut modal, key),
            CustomModalStep::Editor => self.handle_custom_editor_key(&mut modal, key),
            CustomModalStep::Preview => {
                self.handle_custom_preview_key(&mut modal, key, tool_events)
            }
        };

        if close_modal {
            self.model.custom_modal = None;
            return;
        }

        self.model.custom_modal = Some(modal);
    }

    fn handle_custom_picker_key(&mut self, modal: &mut CustomCommandModal, key: KeyEvent) -> bool {
        if modal.paste_mode {
            match key.code {
                KeyCode::Esc => {
                    modal.paste_mode = false;
                    modal.error = None;
                }
                KeyCode::Backspace => {
                    modal.paste_input.pop();
                    modal.error = None;
                }
                KeyCode::Enter => match self.parse_pasted_template(&modal.paste_input) {
                    Ok(template) => {
                        modal.draft = Some(self.new_custom_draft(template));
                        modal.step = CustomModalStep::Editor;
                        modal.editor_index = 0;
                        modal.paste_mode = false;
                        modal.error = None;
                        self.model.notification =
                            Some("pasted command parsed. adjust options and continue".to_string());
                    }
                    Err(error) => modal.error = Some(error),
                },
                KeyCode::Char(ch) => {
                    modal.paste_input.push(ch);
                    modal.error = None;
                }
                _ => {}
            }
            return false;
        }

        let total_rows = self.model.custom_templates.len() + 1;
        match key.code {
            KeyCode::Esc => return true,
            KeyCode::Up | KeyCode::Char('k') => {
                if modal.picker_index == 0 {
                    modal.picker_index = total_rows.saturating_sub(1);
                } else {
                    modal.picker_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if total_rows > 0 {
                    modal.picker_index = (modal.picker_index + 1) % total_rows;
                }
            }
            KeyCode::Enter => {
                if modal.picker_index == 0 {
                    modal.paste_mode = true;
                    modal.paste_input.clear();
                    modal.error = None;
                    self.model.notification =
                        Some("paste full foundry command and press Enter to parse it".to_string());
                } else if let Some(template) = self
                    .model
                    .custom_templates
                    .get(modal.picker_index - 1)
                    .cloned()
                {
                    modal.draft = Some(self.new_custom_draft(template));
                    modal.step = CustomModalStep::Editor;
                    modal.editor_index = 0;
                    modal.error = None;
                }
            }
            KeyCode::Char('p') => {
                modal.paste_mode = true;
                modal.paste_input.clear();
                modal.error = None;
            }
            _ => {}
        }
        false
    }

    fn handle_custom_editor_key(&mut self, modal: &mut CustomCommandModal, key: KeyEvent) -> bool {
        let Some(draft) = modal.draft.as_mut() else {
            modal.step = CustomModalStep::TemplatePicker;
            return false;
        };

        let placeholders = custom_form_placeholders(draft);
        let field_count = 2 + placeholders.len();
        if field_count == 0 || modal.editor_index >= field_count {
            modal.editor_index = 0;
        }

        match key.code {
            KeyCode::Esc => {
                modal.step = CustomModalStep::TemplatePicker;
                modal.draft = None;
                modal.error = None;
            }
            KeyCode::Tab | KeyCode::Down => {
                if field_count > 0 {
                    modal.editor_index = (modal.editor_index + 1) % field_count;
                }
                modal.error = None;
            }
            KeyCode::BackTab | KeyCode::Up => {
                if field_count > 0 {
                    modal.editor_index = if modal.editor_index == 0 {
                        field_count - 1
                    } else {
                        modal.editor_index - 1
                    };
                }
                modal.error = None;
            }
            KeyCode::Left => {
                if modal.editor_index == 0 {
                    let previous_rpc_url = self.rpc_url_for_preset(&draft.rpc_preset);
                    self.cycle_rpc_preset(&mut draft.rpc_preset, false);
                    self.sync_draft_rpc_url_with_preset(draft, previous_rpc_url);
                }
                modal.error = None;
            }
            KeyCode::Right => {
                if modal.editor_index == 0 {
                    let previous_rpc_url = self.rpc_url_for_preset(&draft.rpc_preset);
                    self.cycle_rpc_preset(&mut draft.rpc_preset, true);
                    self.sync_draft_rpc_url_with_preset(draft, previous_rpc_url);
                }
                modal.error = None;
            }
            KeyCode::Backspace => {
                if modal.editor_index == 1 {
                    draft.raw_args.pop();
                } else if modal.editor_index >= 2 {
                    let placeholder = placeholders[modal.editor_index - 2].clone();
                    let initial = initial_placeholder_value(
                        &draft.template,
                        &draft.param_values,
                        &placeholder,
                        self.rpc_url_for_preset(&draft.rpc_preset),
                    );
                    let entry = draft.param_values.entry(placeholder).or_insert(initial);
                    entry.pop();
                }
                modal.error = None;
            }
            KeyCode::Char(ch) => {
                if modal.editor_index == 1 {
                    draft.raw_args.push(ch);
                } else if modal.editor_index >= 2 {
                    let placeholder = placeholders[modal.editor_index - 2].clone();
                    let initial = initial_placeholder_value(
                        &draft.template,
                        &draft.param_values,
                        &placeholder,
                        self.rpc_url_for_preset(&draft.rpc_preset),
                    );
                    let entry = draft.param_values.entry(placeholder).or_insert(initial);
                    entry.push(ch);
                }
                modal.error = None;
            }
            KeyCode::Enter => {
                if let Err(error) = self.prepare_custom_preview(modal) {
                    modal.error = Some(error);
                } else {
                    modal.error = None;
                }
            }
            _ => {}
        }

        false
    }

    fn handle_custom_preview_key(
        &mut self,
        modal: &mut CustomCommandModal,
        key: KeyEvent,
        tool_events: &UnboundedSender<ToolEvent>,
    ) -> bool {
        match key.code {
            KeyCode::Esc => {
                modal.step = CustomModalStep::Editor;
                modal.error = None;
                false
            }
            KeyCode::Enter => {
                let Some(draft) = modal.draft.as_ref() else {
                    modal.error = Some("missing draft state".to_string());
                    return false;
                };

                match self.run_custom_draft(draft, tool_events) {
                    Ok(_) => true,
                    Err(error) => {
                        modal.error = Some(error);
                        false
                    }
                }
            }
            _ => false,
        }
    }

    fn handle_anvil_prompt_key(&mut self, key: KeyEvent, tool_events: &UnboundedSender<ToolEvent>) {
        match key.code {
            KeyCode::Enter => {
                self.submit_anvil_prompt(tool_events);
                return;
            }
            KeyCode::Esc => {
                self.model.anvil_prompt = None;
                self.model.notification = Some("anvil launch prompt dismissed".to_string());
                return;
            }
            _ => {}
        }

        let Some(prompt) = self.model.anvil_prompt.as_mut() else {
            return;
        };

        match key.code {
            KeyCode::Tab | KeyCode::Down => {
                prompt.focus = prompt.focus.next();
                prompt.error = None;
            }
            KeyCode::BackTab | KeyCode::Up => {
                prompt.focus = prompt.focus.prev();
                prompt.error = None;
            }
            KeyCode::Backspace => {
                anvil_prompt_field_mut(prompt).pop();
                prompt.error = None;
            }
            KeyCode::Char(ch) => {
                if matches!(prompt.focus, crate::model::AnvilPromptField::Port)
                    && !ch.is_ascii_digit()
                {
                    return;
                }

                anvil_prompt_field_mut(prompt).push(ch);
                prompt.error = None;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crossterm::event::{KeyModifiers, MouseEvent, MouseEventKind};
    use foundry_tui_config::AppConfig;

    use crate::model::SectionFocus;

    use super::AppController;

    #[test]
    fn mouse_wheel_scrolls_focused_logs_panel() {
        let config = AppConfig::default();
        let mut controller = AppController::new(config, PathBuf::from("."), PathBuf::from("cfg"));
        controller.model.focused_section = SectionFocus::LogsPanel;
        controller.model.logs_scroll = 2;

        controller.handle_mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(controller.model.logs_scroll, 1);

        controller.handle_mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(controller.model.logs_scroll, 2);
    }

    #[test]
    fn mouse_wheel_navigates_palette() {
        let config = AppConfig::default();
        let mut controller = AppController::new(config, PathBuf::from("."), PathBuf::from("cfg"));
        controller.model.palette_open = true;
        controller.model.palette_index = 0;

        controller.handle_mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(controller.model.palette_index, 1);

        controller.handle_mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(controller.model.palette_index, 0);
    }
}
