use crate::db::DbState;
use crate::projects::artifacts::{artifact_dir, non_empty_file_content};
use crate::projects::repository::{row_to_project, PROJECT_COLUMNS};
use crate::projects::{ProjectError, WizardResumeState};
use ralph_core::prd::Prd;
use std::path::Path;
use tauri::{AppHandle, Manager, State};

pub async fn finalize_draft(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<(), ProjectError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = db
        .0
        .lock()
        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    conn.execute(
        "UPDATE projects SET wizard_step = NULL, wizard_state_json = NULL, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, project_id],
    )?;
    drop(conn);
    let artifacts = artifact_dir(&app, &project_id)?;
    let draft_path = artifacts.join("draft.json");
    if draft_path.exists() {
        let _ = std::fs::remove_file(draft_path);
    }
    Ok(())
}

pub async fn discard_draft(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<(), ProjectError> {
    let conn = db
        .0
        .lock()
        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let deleted = conn.execute(
        "DELETE FROM projects WHERE id = ?1 AND status = 'draft'",
        rusqlite::params![project_id],
    )?;
    drop(conn);

    if deleted == 0 {
        return Err(ProjectError::NotFound(project_id));
    }

    let dir = artifact_dir(&app, &project_id)?;
    if dir.exists() {
        let _ = std::fs::remove_dir_all(&dir);
    }

    Ok(())
}

pub async fn save_wizard_state(
    db: State<'_, DbState>,
    project_id: String,
    wizard_step: String,
    wizard_state_json: String,
) -> Result<(), ProjectError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = db
        .0
        .lock()
        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let updated = conn.execute(
        "UPDATE projects SET wizard_step = ?1, wizard_state_json = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![wizard_step, wizard_state_json, now, project_id],
    )?;
    if updated == 0 {
        return Err(ProjectError::NotFound(project_id));
    }
    Ok(())
}

pub async fn save_draft(
    app: AppHandle,
    project_id: String,
    draft_json: String,
) -> Result<(), ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    std::fs::create_dir_all(&artifacts)?;
    let draft_value = serde_json::from_str::<serde_json::Value>(&draft_json)?;
    std::fs::write(artifacts.join("draft.json"), &draft_json)?;
    let draft_step = draft_value
        .get("currentStep")
        .and_then(|value| value.as_str())
        .map(|step| step.trim().to_string())
        .filter(|step| !step.is_empty())
        .unwrap_or_else(|| "describe".to_string());
    let now = chrono::Utc::now().to_rfc3339();
    let db = app.state::<DbState>();
    let conn = db
        .0
        .lock()
        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let _ = conn.execute(
        "UPDATE projects SET wizard_step = ?1, wizard_state_json = NULL, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![draft_step, now, project_id],
    );
    Ok(())
}

pub async fn load_draft(
    app: AppHandle,
    project_id: String,
) -> Result<Option<String>, ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    non_empty_file_content(&artifacts.join("draft.json"))
}

pub async fn resume_wizard(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<WizardResumeState, ProjectError> {
    let (project, wizard_state_json) = {
        let conn = db
            .0
            .lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        let query = format!(
            "SELECT {PROJECT_COLUMNS}, wizard_state_json FROM projects WHERE id = ?1"
        );
        let mut stmt = conn.prepare(&query)?;
        stmt.query_row(rusqlite::params![project_id], |row| {
            Ok((row_to_project(row)?, row.get::<_, Option<String>>(8)?))
        })
        .map_err(|_| ProjectError::NotFound(project_id.clone()))?
    };

    let step = match project.wizard_step.clone() {
        Some(value) => value,
        None if project.status == "draft" => "describe".to_string(),
        None => {
            return Err(ProjectError::NotFound(format!(
                "No wizard state for {project_id}"
            )))
        }
    };

    let dir = artifact_dir(&app, &project_id)?;
    let artifact_plan = non_empty_file_content(&dir.join("plan.md"))?;
    let legacy_plan = non_empty_file_content(&Path::new(&project.working_directory).join("plan.md"))?;
    let has_plan = artifact_plan.is_some() || legacy_plan.is_some();

    let artifact_prd = Prd::load(&dir.join("prd.json"))
        .ok()
        .filter(|prd| !prd.stories.is_empty());
    let legacy_prd = Prd::load(&Path::new(&project.working_directory).join("prd.json"))
        .ok()
        .filter(|prd| !prd.stories.is_empty());
    let has_prd = artifact_prd.is_some() || legacy_prd.is_some();

    Ok(WizardResumeState {
        project,
        wizard_step: step,
        wizard_state_json,
        has_plan,
        has_prd,
    })
}
