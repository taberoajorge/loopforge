use crate::commands::validation::required_trimmed;
use crate::projects::ProjectError;
use crate::storage::db::DbState;
use ralph_core::prd::Prd;
use tauri::{AppHandle, State};

#[cfg(not(test))]
#[tauri::command]
pub async fn load_existing_plan(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<Option<String>, ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::documents::load_existing_plan(app, db, normalized_project_id).await
}

#[tauri::command]
pub async fn load_existing_prd(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<Option<Prd>, ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::documents::load_existing_prd(app, db, normalized_project_id).await
}

#[tauri::command]
pub async fn load_output_log(app: AppHandle, project_id: String) -> Result<String, ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::documents::load_output_log(app, normalized_project_id).await
}

#[cfg(not(test))]
#[tauri::command]
pub async fn save_plan(
    app: AppHandle,
    project_id: String,
    content: String,
) -> Result<(), ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::documents::save_plan(app, normalized_project_id, content).await
}

#[tauri::command]
pub async fn save_prd(
    app: AppHandle,
    project_id: String,
    prd_json: String,
) -> Result<(), ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::documents::save_prd(app, normalized_project_id, prd_json).await
}

#[cfg(not(test))]
#[tauri::command]
pub async fn save_config(
    app: AppHandle,
    project_id: String,
    config_json: String,
) -> Result<(), ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::documents::save_config(app, normalized_project_id, config_json).await
}

#[tauri::command]
pub async fn load_config(
    app: AppHandle,
    project_id: String,
) -> Result<Option<String>, ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::documents::load_config(app, normalized_project_id).await
}
