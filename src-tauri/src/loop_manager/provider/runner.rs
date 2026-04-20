use super::codex::run_codex_process;
use super::command_args::{agent_cli_args, agent_env_vars};
use super::{AgentOutputLine, ShellProvider};
use ralph_core::providers::AgentResult;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Runtime};
use tauri_plugin_shell::process::{CommandChild, CommandEvent as ShellCommandEvent};
use tauri_plugin_shell::ShellExt;

impl<R: Runtime> ShellProvider<R> {
    pub(super) async fn run_with_agent(
        &self,
        agent: &str,
        prompt: &str,
        work_dir: &Path,
        stall_timeout_secs: u64,
        shutdown_flag: Arc<AtomicBool>,
        output_log: &Path,
    ) -> anyhow::Result<AgentResult> {
        let selected_model = if agent == self.primary_agent {
            self.selected_model.as_deref()
        } else {
            None
        };
        let selected_effort = if agent == self.primary_agent {
            self.selected_effort.as_deref()
        } else {
            None
        };
        let args = agent_cli_args(agent, prompt, work_dir, selected_model, selected_effort);
        let env_vars = agent_env_vars(agent);
        if agent == "codex" {
            return self
                .run_codex_with_null_stdin(
                    &args,
                    &env_vars,
                    work_dir,
                    stall_timeout_secs,
                    shutdown_flag,
                    output_log,
                )
                .await;
        }

        let (mut rx, proc): (tokio::sync::mpsc::Receiver<ShellCommandEvent>, CommandChild) = self
            .app
            .shell()
            .command(agent)
            .args(&args)
            .envs(env_vars)
            .current_dir(work_dir)
            .spawn()
            .map_err(|err| anyhow::anyhow!("Spawn failed: {err}"))?;

        let mut output_lines: Vec<String> = Vec::new();
        let stall_threshold = std::time::Duration::from_secs(stall_timeout_secs);
        let mut last_output = std::time::Instant::now();
        let mut exit_code: i32 = 0;
        let mut stall_killed = false;

        let mut log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(output_log)
            .ok();

        loop {
            if shutdown_flag.load(Ordering::SeqCst) {
                let _ = proc.kill();
                stall_killed = true;
                break;
            }

            let recv = tokio::time::timeout(std::time::Duration::from_secs(5), rx.recv()).await;

            match recv {
                Ok(Some(ShellCommandEvent::Stdout(bytes))) => {
                    let line = String::from_utf8_lossy(&bytes).trim_end().to_string();
                    if !line.is_empty() {
                        last_output = std::time::Instant::now();
                        if let Some(ref mut file) = log_file {
                            let _ = writeln!(file, "{line}");
                        }
                        let _ = self.app.emit(
                            crate::events::EVENT_AGENT_OUTPUT_STREAM,
                            AgentOutputLine {
                                project_id: self.project_id.clone(),
                                session_id: self.session_id.clone(),
                                line: line.clone(),
                                stream: "stdout".to_string(),
                            },
                        );
                        output_lines.push(line);
                    }
                }
                Ok(Some(ShellCommandEvent::Stderr(bytes))) => {
                    let line = String::from_utf8_lossy(&bytes).trim_end().to_string();
                    if !line.is_empty() {
                        last_output = std::time::Instant::now();
                        if let Some(ref mut file) = log_file {
                            let _ = writeln!(file, "{line}");
                        }
                        let _ = self.app.emit(
                            crate::events::EVENT_AGENT_OUTPUT_STREAM,
                            AgentOutputLine {
                                project_id: self.project_id.clone(),
                                session_id: self.session_id.clone(),
                                line: line.clone(),
                                stream: "stderr".to_string(),
                            },
                        );
                        output_lines.push(line);
                    }
                }
                Ok(Some(ShellCommandEvent::Terminated(payload))) => {
                    exit_code = payload.code.unwrap_or(1);
                    break;
                }
                Ok(Some(_)) => {}
                Ok(None) => break,
                Err(_timeout) => {
                    if last_output.elapsed() > stall_threshold {
                        let _ = proc.kill();
                        stall_killed = true;
                        break;
                    }
                }
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

    async fn run_codex_with_null_stdin(
        &self,
        args: &[String],
        env_vars: &[(String, String)],
        work_dir: &Path,
        stall_timeout_secs: u64,
        shutdown_flag: Arc<AtomicBool>,
        output_log: &Path,
    ) -> anyhow::Result<AgentResult> {
        run_codex_process(
            self,
            args,
            env_vars,
            work_dir,
            stall_timeout_secs,
            shutdown_flag,
            output_log,
        )
        .await
    }
}
