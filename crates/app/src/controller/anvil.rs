use chrono::Local;
use foundry_tui_foundry::{ToolEvent, ToolKind};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    model::{
        AnvilInstance, AnvilInstanceStatus, AnvilLaunchPrompt, AnvilPromptField, LogLine,
        LogStream, Tab,
    },
    parsing::{parse_cli_args, parse_flag_value, remove_flag_and_value},
};

use super::AppController;

impl AppController {
    pub(crate) fn start_anvil(&mut self) {
        self.open_anvil_prompt();
    }

    pub(crate) fn stop_anvil(&mut self) {
        let selected_job = self
            .model
            .anvil_instances
            .get(self.model.selected_anvil_index)
            .filter(|instance| {
                matches!(
                    instance.status,
                    AnvilInstanceStatus::Starting | AnvilInstanceStatus::Running
                )
            })
            .map(|instance| instance.job_id);

        let fallback_job = self
            .model
            .anvil_instances
            .iter()
            .rev()
            .find(|instance| {
                matches!(
                    instance.status,
                    AnvilInstanceStatus::Starting | AnvilInstanceStatus::Running
                )
            })
            .map(|instance| instance.job_id);

        let Some(job_id) = selected_job.or(fallback_job) else {
            self.model.notification = Some("no running anvil instances".to_string());
            return;
        };

        if self.job_manager.cancel(job_id) {
            self.model.notification = Some(format!("stopping anvil job #{job_id}"));
            let entry = LogLine {
                ts: Local::now(),
                job_id: Some(job_id),
                stream: LogStream::System,
                message: "anvil cancellation requested".to_string(),
            };
            self.push_log(entry.clone());
            self.push_anvil_log(job_id, entry);
        } else {
            self.model.notification = Some("failed to signal anvil shutdown".to_string());
        }
    }

    fn start_anvil_instance(
        &mut self,
        name: String,
        port: u16,
        fork_url: String,
        extra_flags: Vec<String>,
        tool_events: &UnboundedSender<ToolEvent>,
    ) {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            self.model.notification = Some("anvil instance name cannot be empty".to_string());
            return;
        }

        if self.port_in_use(port) {
            self.model.notification = Some(format!("anvil port {port} is already in use"));
            return;
        }

        let args = self.build_anvil_args(port, &fork_url, &extra_flags);
        let label = format!("Anvil Node ({trimmed_name})");
        let Some(job_id) = self.start_tool_job(&label, ToolKind::Anvil, args, tool_events) else {
            return;
        };

        let fork_url = fork_url.trim();
        let fork_url = (!fork_url.is_empty()).then(|| fork_url.to_string());

        self.model.anvil_instances.push(AnvilInstance {
            job_id,
            name: trimmed_name.to_string(),
            port,
            fork_url,
            status: AnvilInstanceStatus::Starting,
            logs: Vec::new(),
        });
        self.model.selected_anvil_index = self.model.anvil_instances.len().saturating_sub(1);
        self.model.active_tab = Tab::Anvil;
        self.normalize_focus_for_tab();

        let entry = LogLine {
            ts: Local::now(),
            job_id: Some(job_id),
            stream: LogStream::System,
            message: format!("queued anvil instance `{trimmed_name}` on port {port}"),
        };
        self.push_log(entry.clone());
        self.push_anvil_log(job_id, entry);
    }

    fn open_anvil_prompt(&mut self) {
        let running_instances = self.has_running_anvil_instances();
        let prompt = AnvilLaunchPrompt {
            name: self.next_default_anvil_name(),
            port: self.next_available_anvil_port().to_string(),
            fork_url: parse_flag_value(&self.config.foundry.workflows.anvil_start, "--fork-url")
                .or_else(|| self.default_rpc_target())
                .unwrap_or_default(),
            extra_flags: self.default_anvil_extra_flags(),
            focus: AnvilPromptField::Name,
            error: None,
        };

        self.model.anvil_prompt = Some(prompt);
        self.model.active_tab = Tab::Anvil;
        self.normalize_focus_for_tab();
        self.model.notification = Some(if running_instances {
            "anvil instance already running: configure the new instance and press Enter".to_string()
        } else {
            "configure anvil flags and press Enter to launch".to_string()
        });
    }

    pub(crate) fn submit_anvil_prompt(&mut self, tool_events: &UnboundedSender<ToolEvent>) {
        let Some(mut prompt) = self.model.anvil_prompt.take() else {
            return;
        };

        let name = prompt.name.trim().to_string();
        if name.is_empty() {
            prompt.error = Some("name is required".to_string());
            self.model.anvil_prompt = Some(prompt);
            return;
        }

        let Ok(port) = prompt.port.trim().parse::<u16>() else {
            prompt.error = Some("port must be a valid number".to_string());
            self.model.anvil_prompt = Some(prompt);
            return;
        };

        if port == 0 {
            prompt.error = Some("port must be greater than 0".to_string());
            self.model.anvil_prompt = Some(prompt);
            return;
        }

        if self.port_in_use(port) {
            prompt.error = Some(format!(
                "port {port} already in use by another running anvil"
            ));
            self.model.anvil_prompt = Some(prompt);
            return;
        }

        let fork_url = prompt.fork_url.trim().to_string();
        let extra_flags = match parse_cli_args(&prompt.extra_flags) {
            Ok(flags) => flags,
            Err(error) => {
                prompt.error = Some(error);
                self.model.anvil_prompt = Some(prompt);
                return;
            }
        };

        self.start_anvil_instance(name, port, fork_url, extra_flags, tool_events);
    }

    fn has_running_anvil_instances(&self) -> bool {
        self.model.anvil_instances.iter().any(|instance| {
            matches!(
                instance.status,
                AnvilInstanceStatus::Starting | AnvilInstanceStatus::Running
            )
        })
    }

    fn next_default_anvil_name(&self) -> String {
        let mut index = 1usize;
        loop {
            let candidate = format!("anvil-{index}");
            if self
                .model
                .anvil_instances
                .iter()
                .all(|instance| instance.name != candidate)
            {
                return candidate;
            }
            index = index.saturating_add(1);
        }
    }

    fn next_available_anvil_port(&self) -> u16 {
        let base_port = parse_flag_value(&self.config.foundry.workflows.anvil_start, "--port")
            .and_then(|value| value.parse::<u16>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(8545);

        let mut port = base_port;
        while self.port_in_use(port) {
            port = port.saturating_add(1);
            if port == 0 {
                return base_port;
            }
        }

        port
    }

    fn port_in_use(&self, port: u16) -> bool {
        self.model.anvil_instances.iter().any(|instance| {
            instance.port == port
                && matches!(
                    instance.status,
                    AnvilInstanceStatus::Starting | AnvilInstanceStatus::Running
                )
        })
    }

    fn default_anvil_extra_flags(&self) -> String {
        let mut args = self.config.foundry.workflows.anvil_start.clone();
        remove_flag_and_value(&mut args, "--port");
        remove_flag_and_value(&mut args, "--fork-url");
        args.join(" ")
    }

    fn build_anvil_args(&self, port: u16, fork_url: &str, extra_flags: &[String]) -> Vec<String> {
        let mut args = extra_flags.to_vec();
        args.push("--port".to_string());
        args.push(port.to_string());

        let trimmed_fork_url = fork_url.trim();
        if !trimmed_fork_url.is_empty() {
            args.push("--fork-url".to_string());
            args.push(trimmed_fork_url.to_string());
        }

        args
    }

    pub(crate) fn select_prev_anvil_instance(&mut self) {
        let len = self.model.anvil_instances.len();
        if len == 0 {
            return;
        }

        if self.model.selected_anvil_index == 0 {
            self.model.selected_anvil_index = len - 1;
        } else {
            self.model.selected_anvil_index -= 1;
        }
    }

    pub(crate) fn select_next_anvil_instance(&mut self) {
        let len = self.model.anvil_instances.len();
        if len == 0 {
            return;
        }

        self.model.selected_anvil_index = (self.model.selected_anvil_index + 1) % len;
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use foundry_tui_config::AppConfig;

    use crate::model::{AnvilInstance, AnvilInstanceStatus};

    use super::AppController;

    #[test]
    fn start_anvil_uses_prompt_when_one_is_running() {
        let config = AppConfig::default();
        let mut controller = AppController::new(config, PathBuf::from("."), PathBuf::from("cfg"));
        controller.model.anvil_instances.push(AnvilInstance {
            job_id: 1,
            name: "anvil-1".to_string(),
            port: 8545,
            fork_url: None,
            status: AnvilInstanceStatus::Running,
            logs: Vec::new(),
        });

        controller.start_anvil();

        assert!(controller.model.anvil_prompt.is_some());
    }
}
