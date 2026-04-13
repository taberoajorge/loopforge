use crate::db::DbState;
use crate::projects::artifacts::{artifact_dir, init_artifacts};
use crate::projects::repository::{group_projects_by_status, row_to_project, PROJECT_COLUMNS};
use crate::projects::{Project, ProjectDetail, ProjectError, ProjectsByStatus};
use ralph_core::prd::Prd;
use tauri::{AppHandle, State};
use uuid::Uuid;

pub async fn create_project(
    app: AppHandle,
    db: State<'_, DbState>,
    name: String,
    description: String,
    working_directory: String,
    wizard_step: Option<String>,
) -> Result<Project, ProjectError> {
    let project_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let dir = artifact_dir(&app, &project_id)?;
    init_artifacts(&dir, &name)?;

    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    conn.execute(
        "INSERT INTO projects (id, name, description, status, working_directory, created_at, updated_at, wizard_step)
         VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, ?7)",
        rusqlite::params![project_id, name, description, working_directory, now, now, wizard_step],
    )?;

    Ok(Project {
        id: project_id,
        name,
        description,
        status: "draft".to_string(),
        working_directory,
        created_at: now.clone(),
        updated_at: now,
        wizard_step,
    })
}

pub async fn list_projects(db: State<'_, DbState>) -> Result<ProjectsByStatus, ProjectError> {
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;

    let query = format!("SELECT {PROJECT_COLUMNS} FROM projects ORDER BY updated_at DESC");
    let mut stmt = conn.prepare(&query)?;

    let projects: Vec<Project> = stmt
        .query_map([], row_to_project)?
        .filter_map(Result::ok)
        .collect();

    Ok(group_projects_by_status(projects))
}

pub async fn archive_project(
    db: State<'_, DbState>,
    project_id: String,
) -> Result<(), ProjectError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let updated = conn.execute(
        "UPDATE projects SET status = 'archived', updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, project_id],
    )?;
    if updated == 0 {
        return Err(ProjectError::NotFound(project_id));
    }
    Ok(())
}

pub async fn get_project_detail(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<ProjectDetail, ProjectError> {
    let project = {
        let conn =
            db.0.lock()
                .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        let query = format!("SELECT {PROJECT_COLUMNS} FROM projects WHERE id = ?1");
        let mut stmt = conn.prepare(&query)?;
        stmt.query_row(rusqlite::params![project_id], row_to_project)
            .map_err(|_| ProjectError::NotFound(project_id.clone()))?
    };

    let dir = artifact_dir(&app, &project_id)?;
    let prd_path = dir.join("prd.json");

    let stories = if prd_path.exists() {
        Prd::load(&prd_path)
            .map(|prd| prd.stories)
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let total_stories = stories.len();
    let passed_count = stories.iter().filter(|story| story.passes).count();
    let blocked_count = stories.iter().filter(|story| story.blocked).count();
    let pending_count = total_stories - passed_count - blocked_count;

    Ok(ProjectDetail {
        project,
        total_stories,
        passed_count,
        blocked_count,
        pending_count,
        stories,
    })
}
