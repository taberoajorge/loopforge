use super::{AgentResult, Provider};
use crate::detection::loop_detector::LoopDetector;
use crate::detection::stall::{self, StallVerdict};
use crate::logger;
use anyhow::Result;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::process::Command;

const OPENCODE_PERMISSION_ALLOW_ALL: &str =
    "{\"*\":\"allow\",\"external_directory\":\"allow\",\"doom_loop\":\"allow\"}";

pub struct CliProvider {
    agent_name: String,
    binary: String,
    model: String,
    build_args: fn(&CliProvider, &str, &Path) -> Vec<String>,
    use_current_dir: bool,
}

impl CliProvider {
    pub fn claude(model: String) -> Self {
        Self {
            agent_name: "claude".into(),
            binary: "claude".into(),
            model,
            build_args: |provider, prompt, _| {
                vec![
                    "-p".into(),
                    prompt.into(),
                    "--model".into(),
                    provider.model.clone(),
                    "--output-format".into(),
                    "text".into(),
                    "--dangerously-skip-permissions".into(),
                ]
            },
            use_current_dir: true,
        }
    }

    pub fn codex(model: String) -> Self {
        Self {
            agent_name: "codex".into(),
            binary: "codex".into(),
            model,
            build_args: |provider, prompt, work_dir| {
                vec![
                    "exec".into(),
                    "--skip-git-repo-check".into(),
                    "--dangerously-bypass-approvals-and-sandbox".into(),
                    "--model".into(),
                    provider.model.clone(),
                    "-C".into(),
                    work_dir.to_string_lossy().into(),
                    prompt.into(),
                ]
            },
            use_current_dir: false,
        }
    }

    pub fn gemini(model: String) -> Self {
        Self {
            agent_name: "gemini".into(),
            binary: "gemini".into(),
            model,
            build_args: |provider, prompt, _| {
                vec![
                    "-p".into(),
                    prompt.into(),
                    "--model".into(),
                    provider.model.clone(),
                    "--yolo".into(),
                ]
            },
            use_current_dir: true,
        }
    }

    pub fn opencode(model: String) -> Self {
        Self {
            agent_name: "opencode".into(),
            binary: "opencode".into(),
            model,
            build_args: |_provider, prompt, _| {
                vec!["run".into(), "--print-logs".into(), prompt.into()]
            },
            use_current_dir: true,
        }
    }

    pub fn cursor(model: String) -> Self {
        Self {
            agent_name: "cursor".into(),
            binary: "cursor".into(),
            model,
            build_args: |provider, prompt, work_dir| {
                vec![
                    "agent".into(),
                    "--print".into(),
                    "--force".into(),
                    "--model".into(),
                    provider.model.clone(),
                    "--output-format".into(),
                    "text".into(),
                    "--workspace".into(),
                    work_dir.to_string_lossy().into(),
                    "--sandbox".into(),
                    "disabled".into(),
                    "--approve-mcps".into(),
                    "--trust".into(),
                    prompt.into(),
                ]
            },
            use_current_dir: true,
        }
    }

    pub fn openrouter(model: String) -> Self {
        Self {
            agent_name: "openrouter".into(),
            binary: "opencode".into(),
            model,
            build_args: |provider, prompt, _| {
                vec![
                    "run".into(),
                    "--model".into(),
                    format!("openrouter:{}", provider.model),
                    "--print-logs".into(),
                    prompt.into(),
                ]
            },
            use_current_dir: true,
        }
    }

    pub fn from_name(name: &str, model: String) -> Option<Self> {
        match name {
            "claude" => Some(Self::claude(model)),
            "codex" => Some(Self::codex(model)),
            "gemini" => Some(Self::gemini(model)),
            "opencode" => Some(Self::opencode(model)),
            "openrouter" => Some(Self::openrouter(model)),
            "cursor" => Some(Self::cursor(model)),
            _ => None,
        }
    }
}

impl Provider for CliProvider {
    fn name(&self) -> &str {
        &self.agent_name
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn run_agent(
        &self,
        prompt: &str,
        story_id: &str,
        work_dir: &Path,
        stall_timeout_secs: u64,
        shutdown_flag: Arc<AtomicBool>,
        output_log: &Path,
    ) -> Result<AgentResult> {
        let args = (self.build_args)(self, prompt, work_dir);

        logger::log_info(&format!(
            "[{}] {} ({}) starting...",
            chrono::Local::now().format("%H:%M:%S"),
            self.agent_name,
            self.model,
        ));

        let mut cmd = Command::new(&self.binary);
        cmd.args(&args)
            .env("TERM", "xterm-256color")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if self.agent_name == "opencode" {
            cmd.env("OPENCODE_PERMISSION", OPENCODE_PERMISSION_ALLOW_ALL);
        }

        if self.use_current_dir {
            cmd.current_dir(work_dir);
        }

        let mut child = cmd.spawn()?;

        let (output_tx, mut output_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        let log_path = output_log.to_path_buf();
        let story_id_owned = story_id.to_string();
        let agent_label = self.agent_name.clone();
        let model_label = self.model.clone();

        let log_writer = tokio::spawn(async move {
            let mut collected_lines = Vec::new();
            let mut loop_detector = LoopDetector::new();
            let mut log_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .ok();

            if let Some(ref mut file) = log_file {
                use std::io::Write;
                let header = format!(
                    "\n=== [{}] Story: {} | Agent: {} | Model: {} ===\n",
                    chrono::Local::now().format("%H:%M:%S"),
                    story_id_owned,
                    agent_label,
                    model_label,
                );
                let _ = file.write_all(header.as_bytes());
            }

            while let Some(line) = output_rx.recv().await {
                tracing::trace!(target: "agent_output", "{line}");
                if let Some(ref mut file) = log_file {
                    use std::io::Write;
                    let _ = writeln!(file, "{line}");
                }
                loop_detector.feed_line(&line);
                if let Some(pattern) = loop_detector.detect_repetition() {
                    logger::log_warning(&format!("Loop detected: {pattern}"));
                }
                collected_lines.push(line);
            }
            collected_lines
        });

        let stall_timeout = std::time::Duration::from_secs(stall_timeout_secs);
        let verdict = stall::monitor_child_with_stall_detection(
            &mut child,
            stall_timeout,
            &shutdown_flag,
            Some(output_tx),
        )
        .await;

        let collected = log_writer.await.unwrap_or_default();

        let exit_code = child
            .try_wait()
            .ok()
            .flatten()
            .map_or(1, |status| status.code().unwrap_or(1));

        logger::log_info(&format!(
            "[{}] {} finished (exit: {exit_code})",
            chrono::Local::now().format("%H:%M:%S"),
            self.agent_name,
        ));

        let (rate_limited, retry_after_message) = AgentResult::detect_rate_limit(&collected);

        Ok(AgentResult {
            exit_code,
            stall_killed: verdict == StallVerdict::StalledAndKilled,
            output_lines: collected,
            rate_limited,
            retry_after_message,
        })
    }
}
