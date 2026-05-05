use std::{
    collections::BTreeMap,
    path::PathBuf,
    process::Stdio,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::{mpsc::UnboundedSender, watch},
};

const REDACTED: &str = "******";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolKind {
    Forge,
    Cast,
    Anvil,
    Chisel,
    Foundryup,
}

impl ToolKind {
    pub fn binary(self) -> &'static str {
        match self {
            ToolKind::Forge => "forge",
            ToolKind::Cast => "cast",
            ToolKind::Anvil => "anvil",
            ToolKind::Chisel => "chisel",
            ToolKind::Foundryup => "foundryup",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub tool: ToolKind,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
    pub profile: Option<String>,
    pub rpc_target: Option<String>,
    pub display_commandline: Option<String>,
}

impl ToolRequest {
    pub fn new(tool: ToolKind, args: Vec<String>, cwd: PathBuf) -> Self {
        Self {
            tool,
            args,
            cwd,
            env: BTreeMap::new(),
            profile: None,
            rpc_target: None,
            display_commandline: None,
        }
    }

    pub fn commandline(&self) -> String {
        redacted_commandline(self.tool, &self.args)
    }

    pub fn display_commandline(&self) -> String {
        self.display_commandline
            .clone()
            .unwrap_or_else(|| self.commandline())
    }
}

pub fn redact_cli_args(args: &[String]) -> Vec<String> {
    let mut redacted = Vec::with_capacity(args.len());
    let mut index = 0usize;

    while index < args.len() {
        let token = &args[index];

        if token.starts_with('-') {
            if let Some((flag, value)) = token.split_once('=') {
                if is_sensitive_flag(flag) || looks_like_secret_hex(value) {
                    redacted.push(format!("{flag}={REDACTED}"));
                } else {
                    redacted.push(token.clone());
                }
                index += 1;
                continue;
            }

            if is_sensitive_flag(token) {
                redacted.push(token.clone());
                if let Some(next) = args.get(index + 1) {
                    if !next.starts_with('-') {
                        redacted.push(REDACTED.to_string());
                        index += 2;
                        continue;
                    }
                }
                index += 1;
                continue;
            }
        }

        if looks_like_secret_hex(token) {
            redacted.push(REDACTED.to_string());
        } else {
            redacted.push(token.clone());
        }
        index += 1;
    }

    redacted
}

fn redacted_commandline(tool: ToolKind, args: &[String]) -> String {
    format!("{} {}", tool.binary(), redact_cli_args(args).join(" "))
        .trim()
        .to_string()
}

fn is_sensitive_flag(flag: &str) -> bool {
    let normalized = flag
        .trim_start_matches('-')
        .trim()
        .to_ascii_lowercase()
        .replace('_', "-");

    matches!(
        normalized.as_str(),
        "private-key"
            | "etherscan-api-key"
            | "api-key"
            | "mnemonic"
            | "mnemonic-passphrase"
            | "password"
            | "passphrase"
            | "token"
    ) || normalized.contains("private-key")
        || normalized.contains("api-key")
        || normalized.contains("secret")
        || normalized.contains("password")
        || normalized.contains("passphrase")
        || normalized.ends_with("-token")
}

fn looks_like_secret_hex(value: &str) -> bool {
    let trimmed = value.trim().trim_matches('"').trim_matches('\'');
    trimmed.len() == 66
        && trimmed.starts_with("0x")
        && trimmed[2..].chars().all(|ch| ch.is_ascii_hexdigit())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub commandline: String,
    pub status_code: Option<i32>,
    pub duration_ms: u128,
    pub cancelled: bool,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ToolEvent {
    Started {
        job_id: u64,
        commandline: String,
    },
    Output {
        job_id: u64,
        stream: StreamKind,
        line: String,
    },
    Finished {
        job_id: u64,
        result: ToolResult,
    },
    Failed {
        job_id: u64,
        error: String,
    },
}

pub async fn run_request(
    job_id: u64,
    request: ToolRequest,
    events: UnboundedSender<ToolEvent>,
    mut cancel_rx: watch::Receiver<bool>,
) -> Result<()> {
    let started = Instant::now();
    let commandline = request.display_commandline();

    let mut command = Command::new(request.tool.binary());
    command
        .args(&request.args)
        .current_dir(&request.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(profile) = &request.profile {
        if matches!(request.tool, ToolKind::Forge) {
            command.env("FOUNDRY_PROFILE", profile);
        }
    }

    if let Some(rpc_target) = &request.rpc_target {
        command.env("ETH_RPC_URL", rpc_target);
    }

    for (key, value) in &request.env {
        command.env(key, value);
    }

    let _ = events.send(ToolEvent::Started {
        job_id,
        commandline: commandline.clone(),
    });

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn `{}`", commandline))?;

    let stdout = child
        .stdout
        .take()
        .context("child process missing stdout pipe")?;
    let stderr = child
        .stderr
        .take()
        .context("child process missing stderr pipe")?;

    let stdout_task = tokio::spawn(read_stream(
        stdout,
        job_id,
        StreamKind::Stdout,
        events.clone(),
    ));
    let stderr_task = tokio::spawn(read_stream(
        stderr,
        job_id,
        StreamKind::Stderr,
        events.clone(),
    ));

    let mut cancelled = false;
    let exit_status = loop {
        if *cancel_rx.borrow() {
            cancelled = true;
            let _ = child.start_kill();
            break child
                .wait()
                .await
                .context("failed while waiting after kill")?;
        }

        if let Some(status) = child.try_wait().context("failed to poll child status")? {
            break status;
        }

        tokio::select! {
            changed = cancel_rx.changed() => {
                if changed.is_err() {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    };

    let stdout_lines = stdout_task.await.context("stdout task join failed")?;
    let stderr_lines = stderr_task.await.context("stderr task join failed")?;

    let result = ToolResult {
        commandline,
        status_code: exit_status.code(),
        duration_ms: started.elapsed().as_millis(),
        cancelled,
        stdout_lines,
        stderr_lines,
    };

    let _ = events.send(ToolEvent::Finished { job_id, result });
    Ok(())
}

async fn read_stream<R>(
    reader: R,
    job_id: u64,
    stream: StreamKind,
    events: UnboundedSender<ToolEvent>,
) -> Vec<String>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    let mut captured = Vec::new();

    while let Ok(Some(line)) = lines.next_line().await {
        captured.push(line.clone());
        let _ = events.send(ToolEvent::Output {
            job_id,
            stream,
            line,
        });
    }

    captured
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commandline_is_stable() {
        let request = ToolRequest::new(
            ToolKind::Cast,
            vec!["block-number".to_string()],
            PathBuf::from("."),
        );

        assert_eq!(request.commandline(), "cast block-number");
    }

    #[test]
    fn commandline_redacts_secret_flag_value() {
        let request = ToolRequest::new(
            ToolKind::Forge,
            vec![
                "verify-contract".to_string(),
                "--etherscan-api-key".to_string(),
                "super-secret-value".to_string(),
                "--chain-id".to_string(),
                "1".to_string(),
            ],
            PathBuf::from("."),
        );

        assert_eq!(
            request.commandline(),
            "forge verify-contract --etherscan-api-key ****** --chain-id 1"
        );
    }

    #[test]
    fn commandline_redacts_equals_syntax() {
        let request = ToolRequest::new(
            ToolKind::Forge,
            vec![
                "script".to_string(),
                "--private-key=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                    .to_string(),
            ],
            PathBuf::from("."),
        );

        assert_eq!(request.commandline(), "forge script --private-key=******");
    }

    #[test]
    fn commandline_redacts_private_key_like_positional_hex() {
        let request = ToolRequest::new(
            ToolKind::Forge,
            vec![
                "script".to_string(),
                "script/Deploy.s.sol:DeployScript".to_string(),
                "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
            ],
            PathBuf::from("."),
        );

        assert_eq!(
            request.commandline(),
            "forge script script/Deploy.s.sol:DeployScript ******"
        );
    }
}
