use crate::commands::validation::required_trimmed;
#[cfg(not(test))]
use crate::commands::validation::optional_trimmed;
use crate::projects::{
    IterationStory, NotificationPrefs, ProjectConfig, ProjectError,
    ProjectsByStatus,
};
#[cfg(not(test))]
use crate::projects::{Project, ProjectDetail};
use crate::storage::db::DbState;
use tauri::{AppHandle, State};

#[cfg(not(test))]
#[tauri::command]
pub async fn create_project(
    app: AppHandle,
    db: State<'_, DbState>,
    name: String,
    description: String,
    working_directory: String,
    wizard_step: Option<String>,
) -> Result<Project, ProjectError> {
    let normalized_name = required_trimmed(name, "name").map_err(ProjectError::Path)?;
    let normalized_description =
        required_trimmed(description, "description").map_err(ProjectError::Path)?;
    let normalized_directory =
        required_trimmed(working_directory, "working_directory").map_err(ProjectError::Path)?;
    let normalized_wizard_step = optional_trimmed(wizard_step);
    crate::projects::catalog::create_project(
        app,
        db,
        normalized_name,
        normalized_description,
        normalized_directory,
        normalized_wizard_step,
    )
    .await
}

#[tauri::command]
pub async fn list_projects(db: State<'_, DbState>) -> Result<ProjectsByStatus, ProjectError> {
    crate::projects::catalog::list_projects(db).await
}

#[tauri::command]
pub async fn pause_project(app: AppHandle, project_id: String) -> Result<(), ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::control::pause_project(app, normalized_project_id).await
}

#[tauri::command]
pub async fn resume_project(app: AppHandle, project_id: String) -> Result<(), ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::control::resume_project(app, normalized_project_id).await
}

#[tauri::command]
pub async fn archive_project(
    db: State<'_, DbState>,
    project_id: String,
) -> Result<(), ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::catalog::archive_project(db, normalized_project_id).await
}

#[cfg(not(test))]
#[tauri::command]
pub async fn get_project_detail(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<ProjectDetail, ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::catalog::get_project_detail(app, db, normalized_project_id).await
}

#[tauri::command]
pub async fn get_project_stories(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<Vec<IterationStory>, ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::stories::get_project_stories(app, db, normalized_project_id).await
}

#[tauri::command]
pub async fn get_guardrails(app: AppHandle, project_id: String) -> Result<String, ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::documents::get_guardrails(app, normalized_project_id).await
}

#[tauri::command]
pub async fn get_project_config(
    app: AppHandle,
    project_id: String,
) -> Result<Option<ProjectConfig>, ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::runtime_config::get_project_config(app, normalized_project_id).await
}

#[tauri::command]
pub async fn get_notification_prefs(
    app: AppHandle,
    project_id: String,
) -> Result<NotificationPrefs, ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::notifications::get_notification_prefs(app, normalized_project_id).await
}

#[tauri::command]
pub async fn save_notification_prefs(
    app: AppHandle,
    project_id: String,
    prefs: NotificationPrefs,
) -> Result<(), ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::notifications::save_notification_prefs(app, normalized_project_id, prefs).await
}
