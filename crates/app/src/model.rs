use std::{collections::BTreeMap, path::PathBuf};

use chrono::{DateTime, Local};
use foundry_tui_config::{ActionId, CustomTemplate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Build,
    Test,
    Script,
    Anvil,
    Cast,
    Verify,
    Custom,
    Logs,
    History,
}

impl Tab {
    pub const ALL: [Tab; 9] = [
        Tab::Build,
        Tab::Test,
        Tab::Script,
        Tab::Anvil,
        Tab::Cast,
        Tab::Verify,
        Tab::Custom,
        Tab::Logs,
        Tab::History,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Build => "Build",
            Tab::Test => "Test",
            Tab::Script => "Script",
            Tab::Anvil => "Anvil",
            Tab::Cast => "Cast",
            Tab::Verify => "Verify",
            Tab::Custom => "Builder",
            Tab::Logs => "Logs",
            Tab::History => "History",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionFocus {
    MainPanel,
    JobsPanel,
    LogsPanel,
    AnvilInstancesPanel,
    AnvilInstanceLogsPanel,
}

impl SectionFocus {
    pub fn label(self) -> &'static str {
        match self {
            SectionFocus::MainPanel => "Main",
            SectionFocus::JobsPanel => "Jobs",
            SectionFocus::LogsPanel => "Logs",
            SectionFocus::AnvilInstancesPanel => "Anvil Instances",
            SectionFocus::AnvilInstanceLogsPanel => "Anvil Logs",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Success,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn label(self) -> &'static str {
        match self {
            JobStatus::Running => "running",
            JobStatus::Success => "success",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone)]
pub struct JobRecord {
    pub id: u64,
    pub name: String,
    pub commandline: String,
    pub status: JobStatus,
    pub started_at: DateTime<Local>,
    pub finished_at: Option<DateTime<Local>>,
    pub duration_ms: Option<u128>,
    pub status_code: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogStream {
    System,
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub struct LogLine {
    pub ts: DateTime<Local>,
    pub job_id: Option<u64>,
    pub stream: LogStream,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnvilInstanceStatus {
    Starting,
    Running,
    Failed,
    Stopped,
}

impl AnvilInstanceStatus {
    pub fn label(self) -> &'static str {
        match self {
            AnvilInstanceStatus::Starting => "starting",
            AnvilInstanceStatus::Running => "running",
            AnvilInstanceStatus::Failed => "failed",
            AnvilInstanceStatus::Stopped => "stopped",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnvilInstance {
    pub job_id: u64,
    pub name: String,
    pub port: u16,
    pub fork_url: Option<String>,
    pub status: AnvilInstanceStatus,
    pub logs: Vec<LogLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnvilPromptField {
    Name,
    Port,
    ForkUrl,
    ExtraFlags,
}

impl AnvilPromptField {
    pub(crate) fn next(self) -> Self {
        match self {
            AnvilPromptField::Name => AnvilPromptField::Port,
            AnvilPromptField::Port => AnvilPromptField::ForkUrl,
            AnvilPromptField::ForkUrl => AnvilPromptField::ExtraFlags,
            AnvilPromptField::ExtraFlags => AnvilPromptField::Name,
        }
    }

    pub(crate) fn prev(self) -> Self {
        match self {
            AnvilPromptField::Name => AnvilPromptField::ExtraFlags,
            AnvilPromptField::Port => AnvilPromptField::Name,
            AnvilPromptField::ForkUrl => AnvilPromptField::Port,
            AnvilPromptField::ExtraFlags => AnvilPromptField::ForkUrl,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnvilLaunchPrompt {
    pub name: String,
    pub port: String,
    pub fork_url: String,
    pub extra_flags: String,
    pub focus: AnvilPromptField,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomModalStep {
    TemplatePicker,
    Editor,
    Preview,
}

#[derive(Debug, Clone)]
pub struct CustomCommandDraft {
    pub template: CustomTemplate,
    pub rpc_preset: String,
    pub raw_args: String,
    pub param_values: BTreeMap<String, String>,
    pub merged_args: Vec<String>,
    pub resolved_args: Vec<String>,
    pub display_args: Vec<String>,
    pub display_commandline: String,
    pub rpc_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CustomCommandModal {
    pub step: CustomModalStep,
    pub picker_index: usize,
    pub paste_mode: bool,
    pub paste_input: String,
    pub editor_index: usize,
    pub draft: Option<CustomCommandDraft>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppModel {
    pub active_tab: Tab,
    pub tabs: Vec<Tab>,
    pub jobs: BTreeMap<u64, JobRecord>,
    pub history: Vec<JobRecord>,
    pub logs: Vec<LogLine>,
    pub focused_section: SectionFocus,
    pub palette_open: bool,
    pub palette_index: usize,
    pub palette_actions: Vec<ActionId>,
    pub show_build_onboarding: bool,
    pub mouse_mode_enabled: bool,
    pub main_scroll: usize,
    pub jobs_scroll: usize,
    pub logs_scroll: usize,
    pub anvil_logs_scroll: usize,
    pub notification: Option<String>,
    pub key_hints: BTreeMap<ActionId, String>,
    pub project_root: PathBuf,
    pub config_path: PathBuf,
    pub project_sol_files: Vec<String>,
    pub project_has_foundry_toml: bool,
    pub project_has_remappings: bool,
    pub project_indexed_at: DateTime<Local>,
    pub active_rpc_preset: Option<String>,
    pub active_rpc_chain: Option<String>,
    pub active_rpc_url: Option<String>,
    pub custom_templates: Vec<CustomTemplate>,
    pub custom_template_index: usize,
    pub custom_templates_global_path: PathBuf,
    pub custom_templates_project_path: PathBuf,
    pub custom_modal: Option<CustomCommandModal>,
    pub anvil_instances: Vec<AnvilInstance>,
    pub selected_anvil_index: usize,
    pub anvil_prompt: Option<AnvilLaunchPrompt>,
    pub should_quit: bool,
    pub launched_at: DateTime<Local>,
}

impl AppModel {
    pub fn active_tab_index(&self) -> usize {
        self.tabs
            .iter()
            .position(|tab| *tab == self.active_tab)
            .unwrap_or(0)
    }
}
