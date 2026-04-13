use crate::db::DbState;
use crate::projects::ProjectError;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Runtime, State};

pub fn artifact_dir<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
) -> Result<PathBuf, ProjectError> {
    crate::storage::artifacts::project_artifact_dir(app, project_id).map_err(ProjectError::Path)
}

pub fn non_empty_file_content(path: &Path) -> Result<Option<String>, ProjectError> {
    crate::storage::artifacts::read_optional_non_empty(path).map_err(ProjectError::Io)
}

pub fn project_working_directory(
    db: &State<'_, DbState>,
    project_id: &str,
) -> Result<String, ProjectError> {
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    conn.query_row(
        "SELECT working_directory FROM projects WHERE id = ?1",
        rusqlite::params![project_id],
        |row| row.get(0),
    )
    .map_err(|_| ProjectError::NotFound(project_id.to_string()))
}

pub fn init_artifacts(dir: &Path, project_name: &str) -> Result<(), ProjectError> {
    std::fs::create_dir_all(dir)?;

    let plan_path = dir.join("plan.md");
    if !plan_path.exists() {
        std::fs::write(&plan_path, "")?;
    }

    let prd_path = dir.join("prd.json");
    if !prd_path.exists() {
        let empty = serde_json::json!({
            "projectName": project_name,
            "generatedAt": chrono::Utc::now().to_rfc3339(),
            "stories": []
        });
        std::fs::write(&prd_path, serde_json::to_string_pretty(&empty)?)?;
    }

    let prompt_path = dir.join("prompt.md");
    if !prompt_path.exists() {
        std::fs::write(&prompt_path, "")?;
    }

    let guardrails_path = dir.join("guardrails.md");
    if !guardrails_path.exists() {
        std::fs::write(&guardrails_path, "")?;
    }

    Ok(())
}
