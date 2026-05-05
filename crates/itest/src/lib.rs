#[cfg(test)]
mod tests {
    use std::{path::PathBuf, process::Command};

    use anyhow::Result;
    use foundry_tui_config::AppConfig;
    use foundry_tui_foundry::{run_request, ToolEvent, ToolKind, ToolRequest};
    use tokio::sync::{mpsc, watch};

    #[test]
    fn config_defaults_include_rpc_presets() {
        let config = AppConfig::default();
        assert!(config.rpc_presets.contains_key("local"));
        assert!(config.jobs.max_concurrent > 0);
    }

    #[tokio::test]
    async fn cast_help_runs_when_binary_exists() -> Result<()> {
        if !has_binary("cast") {
            return Ok(());
        }

        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let (_cancel_tx, cancel_rx) = watch::channel(false);

        let request = ToolRequest::new(
            ToolKind::Cast,
            vec!["--help".to_string()],
            PathBuf::from("."),
        );

        run_request(1, request, event_tx, cancel_rx).await?;

        let mut started = false;
        let mut finished = false;

        while let Ok(event) = event_rx.try_recv() {
            match event {
                ToolEvent::Started { .. } => started = true,
                ToolEvent::Finished { .. } => finished = true,
                ToolEvent::Output { .. } | ToolEvent::Failed { .. } => {}
            }
        }

        assert!(started);
        assert!(finished);
        Ok(())
    }

    fn has_binary(binary: &str) -> bool {
        Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {binary}"))
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}
