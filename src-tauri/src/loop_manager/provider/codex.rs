use super::command_args::resolve_agent_binary_path;
use super::{AgentOutputLine, ShellProvider};
use ralph_core::providers::AgentResult;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Runtime};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;

const STDIN_WAIT_LINE: &str = "Reading additional input from stdin...";
const STDIN_WAIT_GRACE_SECS: u64 = 2;

pub(super) async fn run_codex_process<R: Runtime>(
    provider: &ShellProvider<R>,
    args: &[String],
    env_vars: &[(String, String)],
    work_dir: &Path,
    stall_timeout_secs: u64,
    shutdown_flag: Arc<AtomicBool>,
    output_log: &Path,
) -> anyhow::Result<AgentResult> {
    let agent_binary = resolve_agent_binary_path("codex")
        .ok_or_else(|| anyhow::anyhow!("Spawn failed: codex binary not found in PATH"))?;
    let mut child = TokioCommand::new(agent_binary);
    child
        .args(args)
        .envs(env_vars.iter().cloned())
        .current_dir(work_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = child
        .spawn()
        .map_err(|err| anyhow::anyhow!("Spawn failed: {err}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Spawn failed: missing stdout pipe"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("Spawn failed: missing stderr pipe"))?;
    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stderr_lines = BufReader::new(stderr).lines();
    let mut output_lines = Vec::new();
    let mut log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_log)
        .ok();
    let stall_threshold = std::time::Duration::from_secs(stall_timeout_secs);
    let wait_grace = std::time::Duration::from_secs(STDIN_WAIT_GRACE_SECS);
    let mut last_output = std::time::Instant::now();
    let mut stdin_wait_started: Option<std::time::Instant> = None;
    let mut exit_code = 0;
    let mut stdout_done = false;
    let mut stderr_done = false;
    let mut stall_killed = false;

    loop {
        if shutdown_flag.load(Ordering::SeqCst) {
            let _ = child.kill().await;
            stall_killed = true;
            break;
        }
        if matches!(stdin_wait_started, Some(started) if started.elapsed() >= wait_grace) {
            let _ = child.kill().await;
            exit_code = 0;
            break;
        }
        if last_output.elapsed() > stall_threshold {
            let _ = child.kill().await;
            stall_killed = true;
            break;
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|err| anyhow::anyhow!("Wait failed: {err}"))?
        {
            exit_code = status.code().unwrap_or(1);
            if stdout_done && stderr_done {
                break;
            }
        }

        let sleep = tokio::time::sleep(std::time::Duration::from_millis(250));
        tokio::pin!(sleep);
        tokio::select! {
            line = stdout_lines.next_line(), if !stdout_done => match line {
                Ok(Some(line)) => capture_line(provider, &line, "stdout", &mut last_output, &mut stdin_wait_started, &mut output_lines, &mut log_file),
                Ok(None) => stdout_done = true,
                Err(err) => return Err(anyhow::anyhow!("Stdout read failed: {err}")),
            },
            line = stderr_lines.next_line(), if !stderr_done => match line {
                Ok(Some(line)) => capture_line(provider, &line, "stderr", &mut last_output, &mut stdin_wait_started, &mut output_lines, &mut log_file),
                Ok(None) => stderr_done = true,
                Err(err) => return Err(anyhow::anyhow!("Stderr read failed: {err}")),
            },
            () = &mut sleep => {}
        }

        if stdout_done && stderr_done && child.try_wait().ok().flatten().is_some() {
            break;
        }
    }

    let (rate_limited, retry_after_message) = AgentResult::detect_rate_limit(&output_lines);
    Ok(AgentResult {
        exit_code,
        stall_killed,
        output_lines,
        rate_limited,
        retry_after_message,
    })
}

pub(super) fn is_stdin_wait_line(line: &str) -> bool {
    line.trim() == STDIN_WAIT_LINE
}

pub(super) fn has_substantive_output(output_lines: &[String]) -> bool {
    output_lines.iter().any(|line| !is_stdin_wait_line(line))
}

fn capture_line<R: Runtime>(
    provider: &ShellProvider<R>,
    line: &str,
    stream: &str,
    last_output: &mut std::time::Instant,
    stdin_wait_started: &mut Option<std::time::Instant>,
    output_lines: &mut Vec<String>,
    log_file: &mut Option<std::fs::File>,
) {
    if line.is_empty() {
        return;
    }
    if is_stdin_wait_line(line) {
        if has_substantive_output(output_lines) && stdin_wait_started.is_none() {
            *stdin_wait_started = Some(std::time::Instant::now());
        }
        return;
    }

    *stdin_wait_started = None;
    *last_output = std::time::Instant::now();
    if let Some(file) = log_file {
        let _ = writeln!(file, "{line}");
    }
    let _ = provider.app.emit(
        crate::events::EVENT_AGENT_OUTPUT_STREAM,
        AgentOutputLine {
            project_id: provider.project_id.clone(),
            session_id: provider.session_id.clone(),
            line: line.to_string(),
            stream: stream.to_string(),
        },
    );
    output_lines.push(line.to_string());
}
