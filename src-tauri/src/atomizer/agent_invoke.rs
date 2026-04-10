use crate::atomizer::agent_args::{
    agent_env_vars, build_agent_args, build_null_stdin_shell_command, is_safe_binary_name,
};
use crate::atomizer::progress::emit_progress;
use crate::atomizer::AtomizerError;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Runtime};
use tauri_plugin_shell::ShellExt;

async fn resolve_agent_binary<R: Runtime>(
    app: &AppHandle<R>,
    agent: &str,
) -> Result<String, AtomizerError> {
    let binary = crate::agent_runtime::cli_binary_name(agent);
    if !is_safe_binary_name(binary) {
        return Err(AtomizerError::AgentFailed(format!(
            "Invalid agent command name: {agent}"
        )));
    }

    let lookup = format!("command -v {binary}");
    let output = app
        .shell()
        .command("/bin/zsh")
        .args(["-lc", &lookup])
        .output()
        .await
        .map_err(|err| AtomizerError::AgentFailed(err.to_string()))?;

    if !output.status.success() {
        return Err(AtomizerError::AgentFailed(format!(
            "Agent '{agent}' not found. Install it or choose another plan agent."
        )));
    }

    let resolved = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    if resolved.is_empty() {
        return Err(AtomizerError::AgentFailed(format!(
            "Agent '{agent}' could not be resolved from shell PATH."
        )));
    }

    Ok(resolved)
}

fn spawn_heartbeat<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
    stage: u8,
    stage_name: String,
) -> Arc<AtomicBool> {
    let done = Arc::new(AtomicBool::new(false));
    let done_flag = done.clone();
    tokio::spawn(async move {
        let mut tick: u64 = 0;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(8)).await;
            if done_flag.load(Ordering::Relaxed) {
                break;
            }
            tick += 1;
            let elapsed = tick * 8;
            emit_progress(
                &app,
                &project_id,
                stage,
                &stage_name,
                &format!("Working... {elapsed}s"),
            );
        }
    });
    done
}

pub(super) async fn invoke_agent<R: Runtime>(
    app: &AppHandle<R>,
    agent: &str,
    model: Option<&str>,
    effort: Option<&str>,
    prompt: &str,
    project_dir: &Path,
) -> Result<String, AtomizerError> {
    invoke_agent_with_heartbeat(app, agent, model, effort, prompt, project_dir, None).await
}

pub(super) async fn invoke_agent_with_heartbeat<R: Runtime>(
    app: &AppHandle<R>,
    agent: &str,
    model: Option<&str>,
    effort: Option<&str>,
    prompt: &str,
    project_dir: &Path,
    heartbeat: Option<(String, u8, String)>,
) -> Result<String, AtomizerError> {
    let agent_binary = resolve_agent_binary(app, agent).await?;
    let args = build_agent_args(agent, prompt, project_dir, model, effort);
    let env_vars = agent_env_vars(agent);

    let hb_done =
        heartbeat.map(|(pid, stage, name)| spawn_heartbeat(app.clone(), pid, stage, name));

    let result = invoke_inner(app, agent, &agent_binary, &args, &env_vars, project_dir).await;

    if let Some(flag) = hb_done {
        flag.store(true, Ordering::Relaxed);
    }

    result
}

fn needs_null_stdin(agent: &str) -> bool {
    matches!(agent, "codex" | "gemini" | "opencode")
}

async fn invoke_inner<R: Runtime>(
    app: &AppHandle<R>,
    agent: &str,
    agent_binary: &str,
    args: &[String],
    env_vars: &[(String, String)],
    project_dir: &Path,
) -> Result<String, AtomizerError> {
    let output = if needs_null_stdin(agent) {
        let shell_command = build_null_stdin_shell_command(agent_binary, args);
        app.shell()
            .command("/bin/zsh")
            .args(["-lc", &shell_command])
            .envs(env_vars.to_vec())
            .current_dir(project_dir)
            .output()
            .await
            .map_err(|err| AtomizerError::AgentFailed(err.to_string()))?
    } else {
        app.shell()
            .command(agent_binary)
            .args(args)
            .envs(env_vars.to_vec())
            .current_dir(project_dir)
            .output()
            .await
            .map_err(|err| AtomizerError::AgentFailed(err.to_string()))?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AtomizerError::AgentFailed(stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
