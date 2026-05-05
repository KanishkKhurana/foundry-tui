use std::collections::BTreeMap;

use foundry_tui_foundry::{run_request, ToolEvent, ToolRequest};
use tokio::sync::{mpsc::UnboundedSender, watch};

#[derive(Debug)]
struct RunningJob {
    cancel_tx: watch::Sender<bool>,
}

#[derive(Debug)]
pub(crate) struct JobManager {
    next_job_id: u64,
    max_concurrent: usize,
    running: BTreeMap<u64, RunningJob>,
}

impl JobManager {
    pub(crate) fn new(max_concurrent: usize) -> Self {
        Self {
            next_job_id: 1,
            max_concurrent,
            running: BTreeMap::new(),
        }
    }

    pub(crate) fn spawn(
        &mut self,
        request: ToolRequest,
        tool_events: UnboundedSender<ToolEvent>,
    ) -> Result<u64, String> {
        if self.running.len() >= self.max_concurrent {
            return Err(format!(
                "concurrency limit reached ({})",
                self.max_concurrent
            ));
        }

        let job_id = self.next_job_id;
        self.next_job_id += 1;

        let (cancel_tx, cancel_rx) = watch::channel(false);
        self.running.insert(job_id, RunningJob { cancel_tx });

        tokio::spawn(async move {
            if let Err(error) = run_request(job_id, request, tool_events.clone(), cancel_rx).await {
                let _ = tool_events.send(ToolEvent::Failed {
                    job_id,
                    error: error.to_string(),
                });
            }
        });

        Ok(job_id)
    }

    pub(crate) fn cancel(&mut self, job_id: u64) -> bool {
        if let Some(job) = self.running.get(&job_id) {
            return job.cancel_tx.send(true).is_ok();
        }

        false
    }

    pub(crate) fn mark_finished(&mut self, job_id: u64) {
        self.running.remove(&job_id);
    }
}
