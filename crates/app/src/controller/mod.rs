use std::path::PathBuf;

use chrono::Local;
use foundry_tui_config::{
    default_custom_templates, ActionId, AppConfig, TemplateLoadState, TemplateTool,
};

use crate::{
    job_manager::JobManager,
    model::{AppModel, LogLine, LogStream, SectionFocus, Tab},
    parsing::rpc_chain_label,
    project_inventory::scan_project_inventory,
};

mod actions;
mod anvil;
mod custom;
mod input;
mod jobs;

pub struct AppController {
    pub model: AppModel,
    config: AppConfig,
    job_manager: JobManager,
    ticks_since_index: u64,
}

impl AppController {
    pub fn new(config: AppConfig, project_root: PathBuf, config_path: PathBuf) -> Self {
        Self::new_with_templates(
            config,
            project_root,
            config_path,
            TemplateLoadState {
                templates: Vec::new(),
                global_path: PathBuf::new(),
                project_path: PathBuf::new(),
            },
        )
    }

    pub fn new_with_templates(
        config: AppConfig,
        project_root: PathBuf,
        config_path: PathBuf,
        templates: TemplateLoadState,
    ) -> Self {
        let mut custom_templates = templates
            .templates
            .into_iter()
            .filter(|template| matches!(template.tool, TemplateTool::Forge))
            .collect::<Vec<_>>();
        if custom_templates.is_empty() {
            custom_templates = default_custom_templates()
                .into_iter()
                .filter(|template| matches!(template.tool, TemplateTool::Forge))
                .collect();
        }

        let inventory = scan_project_inventory(&project_root);
        let (active_rpc_preset, active_rpc_url) = resolve_active_rpc_target(&config);
        let active_rpc_chain = active_rpc_preset
            .as_ref()
            .map(|preset| rpc_chain_label(preset));

        let mut model = AppModel {
            active_tab: Tab::Build,
            tabs: Tab::ALL.to_vec(),
            jobs: std::collections::BTreeMap::new(),
            history: Vec::new(),
            logs: Vec::new(),
            focused_section: SectionFocus::MainPanel,
            palette_open: false,
            palette_index: 0,
            palette_actions: ActionId::palette_defaults(),
            show_build_onboarding: true,
            mouse_mode_enabled: false,
            main_scroll: 0,
            jobs_scroll: 0,
            logs_scroll: 0,
            anvil_logs_scroll: 0,
            notification: None,
            key_hints: config.keys.bindings.clone(),
            project_root,
            config_path,
            project_sol_files: inventory.sol_files,
            project_has_foundry_toml: inventory.has_foundry_toml,
            project_has_remappings: inventory.has_remappings,
            project_indexed_at: Local::now(),
            active_rpc_preset,
            active_rpc_chain,
            active_rpc_url,
            custom_templates,
            custom_template_index: 0,
            custom_templates_global_path: templates.global_path,
            custom_templates_project_path: templates.project_path,
            custom_modal: None,
            anvil_instances: Vec::new(),
            selected_anvil_index: 0,
            anvil_prompt: None,
            should_quit: false,
            launched_at: Local::now(),
        };

        model.logs.push(LogLine {
            ts: Local::now(),
            job_id: None,
            stream: LogStream::System,
            message: "Foundry TUI ready. Press Ctrl+P for command palette.".to_string(),
        });

        Self {
            model,
            job_manager: JobManager::new(config.jobs.max_concurrent),
            config,
            ticks_since_index: 0,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.model.should_quit
    }

    pub fn on_tick(&mut self) {
        if self.config.jobs.auto_scroll_logs
            && self.model.focused_section != SectionFocus::LogsPanel
        {
            self.model.logs_scroll = 0;
        }
        if self.config.jobs.auto_scroll_logs
            && self.model.focused_section != SectionFocus::AnvilInstanceLogsPanel
        {
            self.model.anvil_logs_scroll = 0;
        }

        self.ticks_since_index = self.ticks_since_index.saturating_add(1);
        if self.ticks_since_index >= 20 {
            self.refresh_project_inventory();
            self.ticks_since_index = 0;
        }
    }

    pub fn set_focused_section(&mut self, section: SectionFocus) {
        if self.available_sections().contains(&section) {
            self.model.focused_section = section;
        }
    }
}

fn resolve_active_rpc_target(config: &AppConfig) -> (Option<String>, Option<String>) {
    if let Some(preset) = &config.foundry.default_rpc_preset {
        if let Some(url) = config.rpc_presets.get(preset) {
            return (Some(preset.clone()), Some(url.clone()));
        }
    }

    if let Some((preset, url)) = config.rpc_presets.iter().next() {
        return (Some(preset.clone()), Some(url.clone()));
    }

    (None, None)
}
