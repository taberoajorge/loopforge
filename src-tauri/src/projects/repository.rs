use crate::models::ProjectStatus;
use crate::projects::{Project, ProjectsByStatus};

pub const PROJECT_COLUMNS: &str =
    "id, name, description, status, working_directory, created_at, updated_at, wizard_step";

pub fn row_to_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    let raw_status: String = row.get(3)?;
    let status = ProjectStatus::resolve_canonical(&raw_status, false, false, false);

    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        status: status.as_project_status().to_string(),
        working_directory: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        wizard_step: row.get(7).unwrap_or(None),
    })
}

pub fn group_projects_by_status(projects: Vec<Project>) -> ProjectsByStatus {
    let mut grouped = ProjectsByStatus {
        active: Vec::new(),
        paused: Vec::new(),
        completed: Vec::new(),
        draft: Vec::new(),
        archived: Vec::new(),
        blocked: Vec::new(),
        failed: Vec::new(),
    };

    for project in projects {
        match project.status.as_str() {
            "active" => grouped.active.push(project),
            "paused" => grouped.paused.push(project),
            "completed" => grouped.completed.push(project),
            "archived" => grouped.archived.push(project),
            "blocked" => grouped.blocked.push(project),
            "failed" => grouped.failed.push(project),
            _ => grouped.draft.push(project),
        }
    }

    grouped
}
