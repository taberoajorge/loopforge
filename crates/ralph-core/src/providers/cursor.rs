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

pub struct CursorProvider {
    pub model: String,
}

impl CursorProvider {
    pub fn new(model: String) -> Self {
        Self { model }
    }
}

impl Provider for CursorProvider {
    fn name(&self) -> &str {
        "cursor"
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
        logger::log_info(&format!(
            "[{}] agent -p ({}) autonomous mode...",
            chrono::Local::now().format("%H:%M:%S"),
            self.model,
        ));

        let mut child = Command::new("agent")
            .args([
                "-p",
                "--force",
                "--model",
                &self.model,
                "--output-format",
                "stream-json",
                "--workspace",
                &work_dir.to_string_lossy(),
                "--sandbox",
                "disabled",
                "--approve-mcps",
                prompt,
            ])
            .current_dir(work_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let (output_tx, mut output_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        let log_path = output_log.to_path_buf();
        let story_id_owned = story_id.to_string();
        let model_name = self.model.clone();
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
                    "\n=== [{}] Story: {} | Provider: cursor | Model: {} ===\n",
                    chrono::Local::now().format("%H:%M:%S"),
                    story_id_owned,
                    model_name,
                );
                let _ = file.write_all(header.as_bytes());
            }

            while let Some(line) = output_rx.recv().await {
                println!("   {line}");
                if let Some(ref mut file) = log_file {
                    use std::io::Write;
                    let _ = writeln!(file, "{line}");
                }
                loop_detector.feed_line(&line);
                if let Some(pattern) = loop_detector.detect_repetition() {
                    logger::log_warning(&format!("Loop detected in output: {pattern}"));
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
            .map(|status| status.code().unwrap_or(1))
            .unwrap_or(1);

        logger::log_info(&format!(
            "[{}] cursor finished (exit: {exit_code})",
            chrono::Local::now().format("%H:%M:%S"),
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
