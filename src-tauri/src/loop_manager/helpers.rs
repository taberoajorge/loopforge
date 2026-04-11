use super::{LoopError, StartLoopArgs};
use crate::db::DbState;
use crate::projects::documents;
use crate::projects::ProjectError;
use ralph_core::config::RalphConfig;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Runtime};
use uuid::Uuid;

pub(super) fn build_ralph_config(
    artifact_dir: &Path,
    work_dir: &Path,
    args: &StartLoopArgs,
) -> RalphConfig {
    let mut config = RalphConfig::from_defaults(artifact_dir);
    config.paths.work_dir = work_dir.to_path_buf();
    config.paths.prd_file = artifact_dir.join("prd.json");
    config.paths.prd_backup = artifact_dir.join("prd.backup.json");
    config.paths.prompt_file = artifact_dir.join("prompt.md");
    config.paths.guardrails_file = artifact_dir.join("guardrails.md");
    config.paths.progress_file = artifact_dir.join("progress.txt");
    config.paths.error_log = artifact_dir.join("error.log");
    config.paths.activity_log = artifact_dir.join("activity.log");
    config.paths.failure_memory_file = artifact_dir.join("failure_memory.json");
    config.paths.state_file = artifact_dir.join(".ralph_state");
    config.paths.pause_file = artifact_dir.join(".ralph-pause");
    config.paths.done_file = artifact_dir.join(".ralph-done");
    config.paths.codex_output_log = artifact_dir.join("agent_output.log");
    if let Some(max) = args.max_iterations {
        config.tuning.max_iterations = max;
    }
    if let Some(gutter) = args.gutter_threshold {
        config.tuning.gutter_threshold = gutter;
    }
    if let Some(cooldown) = args.cooldown_seconds {
        config.tuning.cooldown_secs = cooldown as u64;
    }
    if let Some(ref test_cmd) = args.test_command {
        config.tuning.test_command = Some(test_cmd.clone());
    }
    if let Some(retries) = args.max_verification_retries {
        config.tuning.max_verification_retries = retries;
    }
    config
}

pub(super) fn ensure_execution_prompt(
    artifact_dir: &Path,
    project_name: &str,
) -> Result<(), LoopError> {
    let prompt_path = artifact_dir.join("prompt.md");
    let needs_default = std::fs::read_to_string(&prompt_path)
        .map(|content| content.trim().is_empty())
        .unwrap_or(true);

    if needs_default {
        std::fs::write(
            &prompt_path,
            default_execution_prompt(project_name, artifact_dir),
        )?;
    }

    Ok(())
}

fn default_execution_prompt(project_name: &str, artifact_dir: &Path) -> String {
    let prd_path = artifact_dir.join("prd.json");
    format!(
        "# {project_name} — Execution Prompt\n\n\
You are implementing one LoopForge story at a time.\n\n\
Use `{}` as the only source of truth for story progress.\n\
Do not read from or modify any `prd.json` under the working tree unless it is this exact file.\n\
When a story passes verification, update only this artifact PRD and preserve existing `passes` and `blocked` values for every other story.\n",
        prd_path.display()
    )
}

pub(super) fn create_session(db: &DbState, project_id: &str) -> Result<String, LoopError> {
    let session_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    documents::insert_session(db, project_id, &session_id, &now).map_err(project_error)?;
    Ok(session_id)
}

pub(super) fn close_session(db: &DbState, session_id: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    let _ = documents::close_session(db, session_id, &now).map_err(project_error);
}

pub(super) fn artifact_dir<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
) -> Result<PathBuf, LoopError> {
    crate::storage::artifacts::project_artifact_dir(app, project_id).map_err(LoopError::Path)
}

fn project_error(error: ProjectError) -> LoopError {
    match error {
        ProjectError::Db(message) => LoopError::Db(message),
        ProjectError::Io(source) => LoopError::Io(source),
        ProjectError::Json(source) => LoopError::Internal(source.to_string()),
        ProjectError::NotFound(project_id) => LoopError::Path(project_id),
        ProjectError::Path(message) => LoopError::Path(message),
    }
}
