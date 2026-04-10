use crate::commands::validation::required_trimmed;
use crate::projects::{ProjectError, WizardResumeState};
use crate::storage::db::DbState;
use tauri::{AppHandle, State};

#[cfg(not(test))]
#[tauri::command]
pub async fn finalize_draft(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<(), ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::wizard::finalize_draft(app, db, normalized_project_id).await
}

#[tauri::command]
pub async fn discard_draft(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<(), ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::wizard::discard_draft(app, db, normalized_project_id).await
}

#[tauri::command]
pub async fn save_wizard_state(
    db: State<'_, DbState>,
    project_id: String,
    wizard_step: String,
    wizard_state_json: String,
) -> Result<(), ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    let normalized_step =
        required_trimmed(wizard_step, "wizard_step").map_err(ProjectError::Path)?;
    crate::projects::wizard::save_wizard_state(
        db,
        normalized_project_id,
        normalized_step,
        wizard_state_json,
    )
    .await
}

#[cfg(not(test))]
#[tauri::command]
pub async fn save_draft(
    app: AppHandle,
    project_id: String,
    draft_json: String,
) -> Result<(), ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::wizard::save_draft(app, normalized_project_id, draft_json).await
}

#[cfg(not(test))]
#[tauri::command]
pub async fn load_draft(
    app: AppHandle,
    project_id: String,
) -> Result<Option<String>, ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::wizard::load_draft(app, normalized_project_id).await
}

#[tauri::command]
pub async fn resume_wizard(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<WizardResumeState, ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::wizard::resume_wizard(app, db, normalized_project_id).await
}
