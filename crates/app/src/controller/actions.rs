use chrono::Local;
use crossterm::event::KeyEvent;
use foundry_tui_config::ActionId;
use foundry_tui_foundry::{ToolEvent, ToolKind};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    model::{LogLine, LogStream, SectionFocus, Tab},
    parsing::{is_non_evm_preset, key_matches, rpc_chain_label},
};

use super::AppController;

impl AppController {
    pub(crate) fn resolve_action(&self, key: KeyEvent) -> Option<ActionId> {
        self.config
            .keys
            .bindings
            .iter()
            .find_map(|(action, binding)| key_matches(binding, key).then_some(*action))
    }

    pub(crate) fn execute_action(
        &mut self,
        action: ActionId,
        tool_events: &UnboundedSender<ToolEvent>,
    ) {
        match action {
            ActionId::Quit => self.model.should_quit = true,
            ActionId::NextTab => self.step_tab(1),
            ActionId::PrevTab => self.step_tab(-1),
            ActionId::FocusNextSection => self.cycle_section_focus(true),
            ActionId::FocusPrevSection => self.cycle_section_focus(false),
            ActionId::OpenPalette => {
                self.model.palette_open = !self.model.palette_open;
                self.model.palette_index = 0;
            }
            ActionId::ToggleBuildOnboarding => {
                self.model.show_build_onboarding = true;
                self.model.notification = Some("build onboarding is always visible".to_string());
            }
            ActionId::OpenThemePicker | ActionId::ThemeNext | ActionId::ThemePrev => {
                self.model.notification = Some(
                    "theme switching was removed; using fixed bold-contrast theme".to_string(),
                );
            }
            ActionId::RunCustomCommand => self.open_custom_runner(),
            ActionId::RunBuild => {
                let _ = self.start_tool_job(
                    "Forge Build",
                    ToolKind::Forge,
                    self.config.foundry.workflows.build.clone(),
                    tool_events,
                );
            }
            ActionId::RunTest => {
                let _ = self.start_tool_job(
                    "Forge Test",
                    ToolKind::Forge,
                    self.config.foundry.workflows.test.clone(),
                    tool_events,
                );
            }
            ActionId::RunScript => {
                let _ = self.start_tool_job(
                    "Forge Script",
                    ToolKind::Forge,
                    self.config.foundry.workflows.script.clone(),
                    tool_events,
                );
            }
            ActionId::RunCastBlockNumber => {
                self.run_cast_block_number(tool_events);
            }
            ActionId::RunVerifyCheck => {
                let _ = self.start_tool_job(
                    "Forge Verify Check",
                    ToolKind::Forge,
                    self.config.foundry.workflows.verify_check.clone(),
                    tool_events,
                );
            }
            ActionId::RunChiselList => {
                let _ = self.start_tool_job(
                    "Chisel List",
                    ToolKind::Chisel,
                    self.config.foundry.workflows.chisel_list.clone(),
                    tool_events,
                );
            }
            ActionId::RunFoundryupUpdate => {
                let _ = self.start_tool_job(
                    "Foundryup Update",
                    ToolKind::Foundryup,
                    self.config.foundry.workflows.foundryup_update.clone(),
                    tool_events,
                );
            }
            ActionId::StartAnvil => self.start_anvil(),
            ActionId::StopAnvil => self.stop_anvil(),
            ActionId::ScrollLogsUp => self.scroll_focused_section(false),
            ActionId::ScrollLogsDown => self.scroll_focused_section(true),
        }
    }

    fn step_tab(&mut self, offset: isize) {
        let len = self.model.tabs.len() as isize;
        let current = self.model.active_tab_index() as isize;
        let next = (current + offset).rem_euclid(len) as usize;
        self.model.active_tab = self.model.tabs[next];
        self.normalize_focus_for_tab();
    }

    fn cycle_section_focus(&mut self, forward: bool) {
        let sections = self.available_sections();
        if sections.is_empty() {
            return;
        }

        let current_index = sections
            .iter()
            .position(|section| *section == self.model.focused_section)
            .unwrap_or(0);

        let next_index = if forward {
            (current_index + 1) % sections.len()
        } else if current_index == 0 {
            sections.len() - 1
        } else {
            current_index - 1
        };

        self.model.focused_section = sections[next_index];
        self.model.notification = Some(format!(
            "focused section: {}",
            self.model.focused_section.label()
        ));
    }

    pub(crate) fn scroll_focused_section(&mut self, down: bool) {
        match self.model.focused_section {
            SectionFocus::MainPanel => {
                if self.model.active_tab == Tab::Custom {
                    if down {
                        self.select_next_custom_template();
                    } else {
                        self.select_prev_custom_template();
                    }
                    return;
                }
                if down {
                    self.model.main_scroll = self.model.main_scroll.saturating_add(1);
                } else {
                    self.model.main_scroll = self.model.main_scroll.saturating_sub(1);
                }
            }
            SectionFocus::JobsPanel => {
                if down {
                    self.model.jobs_scroll = self.model.jobs_scroll.saturating_add(1);
                } else {
                    self.model.jobs_scroll = self.model.jobs_scroll.saturating_sub(1);
                }
            }
            SectionFocus::LogsPanel => {
                if down {
                    self.model.logs_scroll = self.model.logs_scroll.saturating_sub(1);
                } else {
                    self.model.logs_scroll = self.model.logs_scroll.saturating_add(1);
                }
            }
            SectionFocus::AnvilInstancesPanel => {
                if down {
                    self.select_next_anvil_instance();
                } else {
                    self.select_prev_anvil_instance();
                }
            }
            SectionFocus::AnvilInstanceLogsPanel => {
                if down {
                    self.model.anvil_logs_scroll = self.model.anvil_logs_scroll.saturating_sub(1);
                } else {
                    self.model.anvil_logs_scroll = self.model.anvil_logs_scroll.saturating_add(1);
                }
            }
        }
    }

    pub(crate) fn available_sections(&self) -> Vec<SectionFocus> {
        match self.model.active_tab {
            Tab::Anvil => vec![
                SectionFocus::AnvilInstancesPanel,
                SectionFocus::AnvilInstanceLogsPanel,
                SectionFocus::JobsPanel,
                SectionFocus::LogsPanel,
            ],
            _ => vec![
                SectionFocus::MainPanel,
                SectionFocus::JobsPanel,
                SectionFocus::LogsPanel,
            ],
        }
    }

    pub(crate) fn normalize_focus_for_tab(&mut self) {
        let sections = self.available_sections();
        if !sections.contains(&self.model.focused_section) {
            self.model.focused_section = sections[0];
        }
    }

    fn run_cast_block_number(&mut self, tool_events: &UnboundedSender<ToolEvent>) {
        let Some((preset, url)) = self.active_rpc_target() else {
            self.model.notification = Some("no RPC preset configured for cast".to_string());
            return;
        };

        let chain_label = rpc_chain_label(&preset);
        self.model.active_rpc_preset = Some(preset.clone());
        self.model.active_rpc_chain = Some(chain_label.clone());
        self.model.active_rpc_url = Some(url.clone());

        if is_non_evm_preset(&preset) {
            self.model.notification = Some(format!(
                "preset `{}` is non-EVM ({}). select an EVM preset for cast",
                preset, chain_label
            ));
            return;
        }

        let entry = LogLine {
            ts: Local::now(),
            job_id: None,
            stream: LogStream::System,
            message: format!(
                "cast block-number target => chain={} preset={} rpc={}",
                chain_label, preset, url
            ),
        };
        self.push_log(entry);

        self.model.notification = Some(format!("querying {} via `{}`", chain_label, preset));

        let label = format!("Cast Block Number ({chain_label})");
        let _ = self.start_tool_job(
            &label,
            ToolKind::Cast,
            self.config.foundry.workflows.cast_block_number.clone(),
            tool_events,
        );
    }
}
