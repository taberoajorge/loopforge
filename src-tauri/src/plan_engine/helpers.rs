use crate::plan_engine::args::is_safe_binary_name;
use crate::plan_engine::PlanEngineError;
use std::path::PathBuf;
use tauri::{AppHandle, Runtime};
use tauri_plugin_shell::ShellExt;

pub(super) async fn resolve_agent_binary<R: Runtime>(
    app: &AppHandle<R>,
    agent: &str,
) -> Result<String, PlanEngineError> {
    let binary = crate::agent_runtime::cli_binary_name(agent);
    if !is_safe_binary_name(binary) {
        return Err(PlanEngineError::Shell(format!(
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
        .map_err(|err| PlanEngineError::Shell(err.to_string()))?;

    if !output.status.success() {
        return Err(PlanEngineError::Shell(format!(
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
        return Err(PlanEngineError::Shell(format!(
            "Agent '{agent}' could not be resolved from shell PATH."
        )));
    }

    Ok(resolved)
}

pub(super) fn artifact_dir<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
) -> Result<PathBuf, PlanEngineError> {
    crate::storage::artifacts::project_artifact_dir(app, project_id).map_err(PlanEngineError::Path)
}

pub(super) fn build_plan_prompt(user_description: &str) -> String {
    format!(
        "You are a senior software architect. Research the codebase and create a detailed \
implementation plan for the following feature request.\n\n\
Think through the problem carefully. Identify affected files, dependencies, and edge cases.\n\
Output a structured plan in markdown.\n\n\
Feature request:\n{user_description}"
    )
}
