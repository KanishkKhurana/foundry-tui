use chrono::Local;
use foundry_tui_foundry::{StreamKind, ToolEvent, ToolKind, ToolRequest};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    model::{AnvilInstanceStatus, JobRecord, JobStatus, LogLine, LogStream},
    parsing::contains_placeholders,
    project_inventory::scan_project_inventory,
};

use super::AppController;

impl AppController {
    pub fn handle_tool_event(&mut self, event: ToolEvent) {
        match event {
            ToolEvent::Started {
                job_id,
                commandline,
            } => {
                let entry = LogLine {
                    ts: Local::now(),
                    job_id: Some(job_id),
                    stream: LogStream::System,
                    message: format!("starting `{}`", commandline),
                };
                self.push_log(entry.clone());
                self.push_anvil_log(job_id, entry);
                self.set_anvil_instance_status(job_id, AnvilInstanceStatus::Running);

                if let Some(job) = self.model.jobs.get_mut(&job_id) {
                    job.commandline = commandline;
                }
            }
            ToolEvent::Output {
                job_id,
                stream,
                line,
            } => {
                let stream = match stream {
                    StreamKind::Stdout => LogStream::Stdout,
                    StreamKind::Stderr => LogStream::Stderr,
                };

                let entry = LogLine {
                    ts: Local::now(),
                    job_id: Some(job_id),
                    stream,
                    message: line,
                };
                self.push_log(entry.clone());
                self.push_anvil_log(job_id, entry);
            }
            ToolEvent::Finished { job_id, result } => {
                let status = if result.cancelled {
                    JobStatus::Cancelled
                } else if result.status_code == Some(0) {
                    JobStatus::Success
                } else {
                    JobStatus::Failed
                };

                if let Some(job) = self.model.jobs.get_mut(&job_id) {
                    job.status = status;
                    job.finished_at = Some(Local::now());
                    job.duration_ms = Some(result.duration_ms);
                    job.status_code = result.status_code;
                }

                if let Some(job) = self.model.jobs.get(&job_id).cloned() {
                    self.model.history.insert(0, job);
                    self.model.history.truncate(self.config.jobs.keep_history);
                }

                self.job_manager.mark_finished(job_id);
                let anvil_status = match status {
                    JobStatus::Failed => AnvilInstanceStatus::Failed,
                    JobStatus::Running => AnvilInstanceStatus::Running,
                    JobStatus::Success | JobStatus::Cancelled => AnvilInstanceStatus::Stopped,
                };
                self.set_anvil_instance_status(job_id, anvil_status);

                self.model.notification = Some(format!(
                    "job #{job_id} {} in {}ms",
                    status.label(),
                    result.duration_ms
                ));

                let entry = LogLine {
                    ts: Local::now(),
                    job_id: Some(job_id),
                    stream: LogStream::System,
                    message: format!(
                        "completed status={} code={:?} stdout={} stderr={}",
                        status.label(),
                        result.status_code,
                        result.stdout_lines.len(),
                        result.stderr_lines.len()
                    ),
                };
                self.push_log(entry.clone());
                self.push_anvil_log(job_id, entry);
            }
            ToolEvent::Failed { job_id, error } => {
                if let Some(job) = self.model.jobs.get_mut(&job_id) {
                    job.status = JobStatus::Failed;
                    job.finished_at = Some(Local::now());
                }

                self.job_manager.mark_finished(job_id);
                self.set_anvil_instance_status(job_id, AnvilInstanceStatus::Failed);

                self.model.notification = Some(format!("job #{job_id} failed to start"));
                let entry = LogLine {
                    ts: Local::now(),
                    job_id: Some(job_id),
                    stream: LogStream::System,
                    message: format!("launch error: {error}"),
                };
                self.push_log(entry.clone());
                self.push_anvil_log(job_id, entry);
            }
        }
    }

    pub(crate) fn start_tool_job(
        &mut self,
        name: &str,
        tool: ToolKind,
        args: Vec<String>,
        tool_events: &UnboundedSender<ToolEvent>,
    ) -> Option<u64> {
        if contains_placeholders(&args) {
            self.model.notification = Some(format!(
                "{name} has placeholders. Edit config at {}",
                self.model.config_path.display()
            ));
            return None;
        }

        let mut request = ToolRequest::new(tool, args, self.model.project_root.clone());

        if matches!(tool, ToolKind::Forge) {
            request.profile = Some(self.config.foundry.profile.clone());
        }

        if matches!(tool, ToolKind::Cast | ToolKind::Forge) {
            request.rpc_target = self.default_rpc_target();
        }

        match self.start_tool_request_job(name, request, tool_events) {
            Ok(job_id) => {
                self.model.notification = Some(format!("queued {name} as job #{job_id}"));
                Some(job_id)
            }
            Err(_) => None,
        }
    }

    pub(crate) fn start_tool_request_job(
        &mut self,
        name: &str,
        request: ToolRequest,
        tool_events: &UnboundedSender<ToolEvent>,
    ) -> std::result::Result<u64, String> {
        let commandline = request.display_commandline();
        let started_at = Local::now();

        let job_id = match self.job_manager.spawn(request, tool_events.clone()) {
            Ok(job_id) => job_id,
            Err(error) => {
                self.model.notification = Some(error.clone());
                self.push_log(LogLine {
                    ts: Local::now(),
                    job_id: None,
                    stream: LogStream::System,
                    message: format!("unable to queue `{name}`: {error}"),
                });
                return Err(error);
            }
        };

        self.model.jobs.insert(
            job_id,
            JobRecord {
                id: job_id,
                name: name.to_string(),
                commandline,
                status: JobStatus::Running,
                started_at,
                finished_at: None,
                duration_ms: None,
                status_code: None,
            },
        );

        Ok(job_id)
    }

    pub(crate) fn default_rpc_target(&self) -> Option<String> {
        self.active_rpc_target().map(|(_, url)| url)
    }

    pub(crate) fn active_rpc_target(&self) -> Option<(String, String)> {
        if let Some(preset) = &self.config.foundry.default_rpc_preset {
            if let Some(url) = self.config.rpc_presets.get(preset) {
                return Some((preset.clone(), url.clone()));
            }
        }

        self.config
            .rpc_presets
            .iter()
            .next()
            .map(|(name, url)| (name.clone(), url.clone()))
    }

    pub(crate) fn refresh_project_inventory(&mut self) {
        let inventory = scan_project_inventory(&self.model.project_root);
        self.model.project_sol_files = inventory.sol_files;
        self.model.project_has_foundry_toml = inventory.has_foundry_toml;
        self.model.project_has_remappings = inventory.has_remappings;
        self.model.project_indexed_at = Local::now();
    }

    pub(crate) fn push_log(&mut self, entry: LogLine) {
        self.model.logs.push(entry);
        if self.model.logs.len() > self.config.jobs.max_log_lines {
            let overflow = self.model.logs.len() - self.config.jobs.max_log_lines;
            self.model.logs.drain(0..overflow);
        }
    }

    pub(crate) fn push_anvil_log(&mut self, job_id: u64, entry: LogLine) {
        let Some(instance) = self
            .model
            .anvil_instances
            .iter_mut()
            .find(|instance| instance.job_id == job_id)
        else {
            return;
        };

        instance.logs.push(entry);
        if instance.logs.len() > self.config.jobs.max_log_lines {
            let overflow = instance.logs.len() - self.config.jobs.max_log_lines;
            instance.logs.drain(0..overflow);
        }
    }

    pub(crate) fn set_anvil_instance_status(&mut self, job_id: u64, status: AnvilInstanceStatus) {
        if let Some(instance) = self
            .model
            .anvil_instances
            .iter_mut()
            .find(|instance| instance.job_id == job_id)
        {
            instance.status = status;
        }
    }
}
