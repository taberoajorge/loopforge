use crate::loop_manager::LoopManagerState;
use crate::models::ProjectStatus;
use crate::projects::artifacts::artifact_dir;
use crate::projects::ProjectError;
use crate::storage::db::DbState;
use ralph_core::prd::Prd;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedProject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub working_directory: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wizard_step: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stories_completed: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_stories: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_ended_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_label: Option<String>,
}

fn frontend_status(status: &ProjectStatus) -> &'static str {
    match status {
        ProjectStatus::Draft | ProjectStatus::Ready => "draft",
        ProjectStatus::Running => "active",
        ProjectStatus::Paused => "paused",
        ProjectStatus::Blocked => "blocked",
        ProjectStatus::Failed => "failed",
        ProjectStatus::Completed => "completed",
        ProjectStatus::Archived => "archived",
    }
}

fn build_duration_label(started_at: Option<&str>, ended_at: Option<&str>) -> Option<String> {
    let started = started_at?;
    let start_time = chrono::DateTime::parse_from_rfc3339(started).ok()?;
    let end_time = ended_at
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .unwrap_or_else(|| chrono::Utc::now().fixed_offset());
    let elapsed_secs = (end_time - start_time).num_seconds().max(0);
    if elapsed_secs < 60 {
        return Some(format!("{elapsed_secs}s"));
    }
    let elapsed_minutes = elapsed_secs / 60;
    if elapsed_minutes < 60 {
        return Some(format!("{elapsed_minutes}m"));
    }
    let elapsed_hours = elapsed_minutes / 60;
    let remaining_minutes = elapsed_minutes % 60;
    Some(format!("{elapsed_hours}h {remaining_minutes}m"))
}

#[tauri::command]
pub async fn list_projects_enriched(
    app: AppHandle,
    db: State<'_, DbState>,
    loop_state: State<'_, LoopManagerState>,
) -> Result<Vec<EnrichedProject>, ProjectError> {
    let running_ids: Vec<String> = loop_state
        .0
        .lock()
        .map(|handles| handles.keys().cloned().collect())
        .unwrap_or_default();

    let mut projects = {
        let conn =
            db.0.lock()
                .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;

        let query = "\
            SELECT p.id, p.name, p.description, p.status, p.working_directory, \
                   p.created_at, p.updated_at, p.wizard_step, \
                   (SELECT started_at FROM sessions WHERE project_id = p.id ORDER BY started_at DESC LIMIT 1), \
                   (SELECT ended_at FROM sessions WHERE project_id = p.id ORDER BY started_at DESC LIMIT 1) \
            FROM projects p ORDER BY p.updated_at DESC";

        let mut stmt = conn.prepare(query)?;
        let rows: Vec<EnrichedProject> = stmt
            .query_map([], |row| {
                Ok(EnrichedProject {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    status: row.get(3)?,
                    working_directory: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                    wizard_step: row.get(7).unwrap_or(None),
                    session_started_at: row.get(8).unwrap_or(None),
                    session_ended_at: row.get(9).unwrap_or(None),
                    duration_label: None,
                    stories_completed: None,
                    total_stories: None,
                    current_agent: None,
                })
            })?
            .filter_map(Result::ok)
            .collect();
        rows
    };

    for project in &mut projects {
        let is_running = running_ids.contains(&project.id);
        let mut status = ProjectStatus::from_db_status(&project.status);

        if is_running {
            status = ProjectStatus::Running;
        } else if matches!(status, ProjectStatus::Running) {
            status = ProjectStatus::Paused;
        }

        if project.status != "draft" {
            if let Ok(dir) = artifact_dir(&app, &project.id) {
                let prd_path = dir.join("prd.json");
                if prd_path.exists() {
                    if let Ok(prd) = Prd::load(&prd_path) {
                        let total = prd.stories.len();
                        let done = prd.stories.iter().filter(|story| story.passes).count();
                        project.total_stories = Some(total);
                        project.stories_completed = Some(done);

                        if matches!(status, ProjectStatus::Draft) && total > 0 {
                            let has_config = dir.join("config.json").exists();
                            if has_config {
                                status = ProjectStatus::Ready;
                            }
                        }
                    }
                }

                let config_path = dir.join("config.json");
                if config_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&config_path) {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                            project.current_agent = parsed
                                .get("executeAgent")
                                .and_then(|val| val.as_str())
                                .map(String::from);
                        }
                    }
                }
            }
        }

        project.status = frontend_status(&status).to_string();
        project.duration_label = build_duration_label(
            project.session_started_at.as_deref(),
            project.session_ended_at.as_deref(),
        );
    }

    Ok(projects)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupedProjects {
    pub active: Vec<EnrichedProject>,
    pub drafts: Vec<EnrichedProject>,
    pub finished: Vec<EnrichedProject>,
    pub archived: Vec<EnrichedProject>,
}

#[tauri::command]
pub async fn list_projects_grouped(
    app: AppHandle,
    db: State<'_, DbState>,
    loop_state: State<'_, LoopManagerState>,
) -> Result<GroupedProjects, ProjectError> {
    let all = list_projects_enriched(app, db, loop_state).await?;
    let mut active = Vec::new();
    let mut drafts = Vec::new();
    let mut finished = Vec::new();
    let mut archived = Vec::new();

    for project in all {
        match project.status.as_str() {
            "draft" => drafts.push(project),
            "active" | "paused" | "blocked" => active.push(project),
            "completed" | "failed" => finished.push(project),
            "archived" => archived.push(project),
            _ => active.push(project),
        }
    }

    Ok(GroupedProjects {
        active,
        drafts,
        finished,
        archived,
    })
}
