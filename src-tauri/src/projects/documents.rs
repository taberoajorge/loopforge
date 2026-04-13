use crate::db::DbState;
use crate::projects::artifacts::{artifact_dir, non_empty_file_content, project_working_directory};
use crate::projects::ProjectError;
use ralph_core::prd::Prd;
#[cfg(test)]
use ralph_core::{CompletionScheduler, MergeAction, WorktreeCompletion};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Runtime, State};

static MERGE_GATE: OnceLock<Mutex<()>> = OnceLock::new();

pub(crate) fn load_text_artifact_with_legacy_fallback(
    artifacts: &Path,
    working_directory: &Path,
    file_name: &str,
) -> Result<Option<String>, ProjectError> {
    let artifact_path = artifacts.join(file_name);
    if let Some(content) = non_empty_file_content(&artifact_path)? {
        return Ok(Some(content));
    }

    let legacy_path = working_directory.join(file_name);
    let legacy_content = non_empty_file_content(&legacy_path)?;
    if let Some(content) = legacy_content {
        let _ = std::fs::create_dir_all(artifacts);
        let _ = std::fs::write(&artifact_path, &content);
        return Ok(Some(content));
    }

    Ok(None)
}

pub async fn load_existing_plan<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<Option<String>, ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    let working_directory = project_working_directory(&db, &project_id)?;
    load_text_artifact_with_legacy_fallback(&artifacts, Path::new(&working_directory), "plan.md")
}
pub async fn save_plan<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
    content: String,
) -> Result<(), ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    std::fs::create_dir_all(&artifacts)?;
    std::fs::write(artifacts.join("plan.md"), &content)?;
    Ok(())
}
pub async fn save_prd<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
    prd_json: String,
) -> Result<(), ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    std::fs::create_dir_all(&artifacts)?;
    serde_json::from_str::<Prd>(&prd_json)?;
    with_merge_gate(|| {
        std::fs::write(artifacts.join("prd.json"), &prd_json)?;
        Ok(())
    })?;
    Ok(())
}
pub async fn save_config<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
    config_json: String,
) -> Result<(), ProjectError> {
    let parsed_config = serde_json::from_str::<crate::projects::ProjectConfig>(&config_json)?;
    crate::projects::runtime_config::save_project_config(&app, &project_id, &parsed_config)?;
    Ok(())
}
pub async fn load_config<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
) -> Result<Option<String>, ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    let config_path = artifacts.join("config.json");
    non_empty_file_content(&config_path)
}
pub async fn load_existing_prd<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<Option<Prd>, ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    let prd_path = artifacts.join("prd.json");
    if let Ok(prd) = Prd::load(&prd_path) {
        if !prd.stories.is_empty() {
            return Ok(Some(prd));
        }
    }

    let working_directory = project_working_directory(&db, &project_id)?;
    let legacy_path = Path::new(&working_directory).join("prd.json");
    if let Ok(prd) = Prd::load(&legacy_path) {
        if !prd.stories.is_empty() {
            std::fs::create_dir_all(&artifacts)?;
            let _ = prd.save(&prd_path);
            return Ok(Some(prd));
        }
    }

    Ok(None)
}
pub async fn get_guardrails<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
) -> Result<String, ProjectError> {
    let dir = artifact_dir(&app, &project_id)?;
    let guardrails_path = dir.join("guardrails.md");
    if guardrails_path.exists() {
        std::fs::read_to_string(&guardrails_path).map_err(ProjectError::Io)
    } else {
        Ok(String::new())
    }
}
pub async fn load_output_log<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
) -> Result<String, ProjectError> {
    let dir = artifact_dir(&app, &project_id)?;
    let output_path = dir.join("agent_output.log");
    if !output_path.exists() {
        return Ok(String::new());
    }
    let content = std::fs::read_to_string(output_path)?;
    Ok(tail_lines(&content, 400))
}
pub(crate) fn insert_session(
    db: &DbState,
    project_id: &str,
    session_id: &str,
    started_at: &str,
) -> Result<(), ProjectError> {
    with_merge_gate(|| {
        let conn =
            db.0.lock()
                .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        conn.execute(
            "INSERT INTO sessions (id, project_id, started_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![session_id, project_id, started_at],
        )?;
        Ok(())
    })
}
pub(crate) fn close_session(
    db: &DbState,
    session_id: &str,
    ended_at: &str,
) -> Result<(), ProjectError> {
    with_merge_gate(|| {
        let conn =
            db.0.lock()
                .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        conn.execute(
            "UPDATE sessions SET ended_at = ?1 WHERE id = ?2",
            rusqlite::params![ended_at, session_id],
        )?;
        Ok(())
    })
}
#[cfg(test)]
pub(crate) fn ordered_merge_actions(completions: Vec<WorktreeCompletion>) -> Vec<MergeAction> {
    let mut scheduler = CompletionScheduler::new();
    scheduler.submit_batch(completions);
    scheduler.drain_ordered()
}
#[cfg(test)]
pub(crate) fn merge_action_target(project_dir: &Path, action: &MergeAction) -> String {
    match action {
        MergeAction::UpdateStoryStatus { .. } => project_dir.join("prd.json").display().to_string(),
        MergeAction::AppendGuardrail { .. } => {
            project_dir.join("guardrails.md").display().to_string()
        }
        MergeAction::UpdateSessionHead { worktree_id, .. } => format!("session:{worktree_id}"),
    }
}
#[cfg(test)]
pub(crate) fn apply_merge_action(
    project_dir: &Path,
    action: &MergeAction,
) -> Result<(), ProjectError> {
    match action {
        MergeAction::UpdateStoryStatus {
            story_id,
            passed,
            blocked,
        } => with_merge_gate(|| update_story_status(project_dir, story_id, *passed, *blocked)),
        MergeAction::AppendGuardrail { content, .. } => append_guardrails(project_dir, content),
        MergeAction::UpdateSessionHead { .. } => Ok(()),
    }
}
fn with_merge_gate<T>(write: impl FnOnce() -> Result<T, ProjectError>) -> Result<T, ProjectError> {
    let _guard = MERGE_GATE
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    write()
}
#[cfg(test)]
fn update_story_status(
    project_dir: &Path,
    story_id: &str,
    passed: bool,
    blocked: bool,
) -> Result<(), ProjectError> {
    let prd_path = project_dir.join("prd.json");
    let mut prd = serde_json::from_str::<Prd>(&std::fs::read_to_string(&prd_path)?)?;
    if let Some(story) = prd.stories.iter_mut().find(|story| story.id == story_id) {
        story.passes = passed;
        story.blocked = blocked;
    }
    std::fs::write(prd_path, serde_json::to_string_pretty(&prd)?)?;
    Ok(())
}
#[cfg(test)]
fn append_guardrails(project_dir: &Path, content: &str) -> Result<(), ProjectError> {
    with_merge_gate(|| {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(project_dir.join("guardrails.md"))?;
        let payload = if content.ends_with('\n') {
            content.to_string()
        } else {
            format!("{content}\n")
        };
        use std::io::Write;
        file.write_all(payload.as_bytes())?;
        Ok(())
    })
}
fn tail_lines(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= max_lines {
        return content.to_string();
    }
    lines[lines.len() - max_lines..].join("\n")
}
