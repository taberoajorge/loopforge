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

    let (shell_program, shell_args) = crate::shell_resolve::resolve_binary_via_shell(binary);
    let output = app
        .shell()
        .command(&shell_program)
        .args(shell_args)
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
        "You are a senior software architect creating an implementation plan. \
Follow this workflow strictly.\n\n\
## Phase 1: Understand the request\n\
Read the feature request below. Identify the core objective, implicit requirements, \
and constraints. Do not ask clarifying questions. Infer reasonable defaults and \
document every assumption you make.\n\n\
## Phase 2: Explore the codebase\n\
Before proposing any changes, explore the relevant parts of the codebase. \
Identify existing patterns, conventions, and dependencies. Never propose changes \
to code you have not read. List the files and modules you examined.\n\n\
## Phase 3: Design the plan\n\
Write a structured implementation plan in markdown. Use # headers to separate \
each area of work. For every section include:\n\
- Affected files (full relative paths)\n\
- New files to create (if any)\n\
- Dependencies on other sections\n\
- Edge cases and error scenarios\n\
- Verification criteria (how to confirm the section works)\n\n\
## Output rules\n\
- Use clear markdown headings (# for top-level, ## for subsections)\n\
- Reference file paths explicitly, never use vague references like \"the config file\"\n\
- Order sections from foundational (data layer, types) to dependent (UI, integration)\n\
- End with a summary listing all files to modify, all files to create, and \
the recommended implementation order\n\
- Do not include code snippets unless they clarify a non-obvious approach\n\
- Keep the plan actionable: every section should map to one or more implementable units\n\n\
---\n\n\
Feature request:\n{user_description}"
    )
}
