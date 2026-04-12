use crate::agents::AgentRegistryState;
use crate::commands::validation::required_trimmed;
use crate::projects::stories_crud::StoriesResponse;
use crate::projects::{
    AdvanceWizardResult, ConfigDefaultsResponse, DescribeInput, LaunchReadiness, ProjectConfig,
    ProjectError, ValidationErrors,
};
use crate::storage::db::DbState;
use ralph_core::prd::Prd;
use serde::Serialize;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn advance_wizard_step(
    target_step: u32,
) -> Result<AdvanceWizardResult, ProjectError> {
    crate::projects::wizard_state::advance_wizard_step(None, target_step)
        .map_err(ProjectError::Path)
}

#[tauri::command]
pub async fn get_default_config() -> Result<ConfigDefaultsResponse, ProjectError> {
    Ok(crate::projects::validation::get_config_defaults())
}

#[tauri::command]
pub async fn validate_project_config(
    config_json: String,
) -> Result<ValidationErrors, ProjectError> {
    let config: ProjectConfig = serde_json::from_str(&config_json)?;
    Ok(crate::projects::validation::validate_config(&config))
}

#[tauri::command]
pub async fn validate_describe_input(
    input_json: String,
    state: State<'_, AgentRegistryState>,
) -> Result<ValidationErrors, ProjectError> {
    let input: DescribeInput = serde_json::from_str(&input_json)?;
    let available_agents = {
        let registry = state
            .0
            .lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        registry
            .agents
            .iter()
            .filter(|agent| agent.available)
            .map(|agent| agent.name.clone())
            .collect::<Vec<_>>()
    };
    Ok(crate::projects::validation::validate_describe(
        &input,
        &available_agents,
    ))
}

#[tauri::command]
pub async fn validate_launch_readiness(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<LaunchReadiness, ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;

    let (project_name, working_directory) = {
        let conn = db
            .0
            .lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        conn.query_row(
            "SELECT name, working_directory FROM projects WHERE id = ?1",
            rusqlite::params![project_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|_| ProjectError::NotFound(project_id.clone()))?
    };

    let dir = crate::projects::artifacts::artifact_dir(&app, &project_id)?;
    let prd_path = dir.join("prd.json");
    let stories_count = if prd_path.exists() {
        Prd::load(&prd_path)
            .map(|prd| prd.stories.len())
            .unwrap_or(0)
    } else {
        0
    };

    let config_path = dir.join("config.json");
    let execute_agent = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        serde_json::from_str::<ProjectConfig>(&content)
            .map(|cfg| cfg.execute_agent)
            .unwrap_or_default()
    } else {
        String::new()
    };

    Ok(crate::projects::validation::validate_launch_readiness(
        &project_name,
        &working_directory,
        stories_count,
        &execute_agent,
    ))
}

#[tauri::command]
pub async fn add_story(
    app: AppHandle,
    project_id: String,
) -> Result<StoriesResponse, ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::stories_crud::add_story(&app, &project_id)
}

#[tauri::command]
pub async fn update_story(
    app: AppHandle,
    project_id: String,
    story_id: String,
    patch_json: String,
) -> Result<StoriesResponse, ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    let story_id = required_trimmed(story_id, "story_id").map_err(ProjectError::Path)?;
    crate::projects::stories_crud::update_story(&app, &project_id, &story_id, &patch_json)
}

#[tauri::command]
pub async fn remove_story(
    app: AppHandle,
    project_id: String,
    story_id: String,
) -> Result<StoriesResponse, ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    let story_id = required_trimmed(story_id, "story_id").map_err(ProjectError::Path)?;
    crate::projects::stories_crud::remove_story(&app, &project_id, &story_id)
}

#[tauri::command]
pub async fn reorder_stories(
    app: AppHandle,
    project_id: String,
    from_index: usize,
    to_index: usize,
) -> Result<StoriesResponse, ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::stories_crud::reorder_stories(&app, &project_id, from_index, to_index)
}

#[tauri::command]
pub async fn get_stories(
    app: AppHandle,
    project_id: String,
) -> Result<StoriesResponse, ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::stories_crud::get_stories(&app, &project_id)
}

#[tauri::command]
pub async fn save_wizard_draft(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
    step_name: String,
) -> Result<(), ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    let step_name = required_trimmed(step_name, "step_name").map_err(ProjectError::Path)?;

    let (name, description, working_directory) = {
        let conn = db
            .0
            .lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        conn.query_row(
            "SELECT name, description, working_directory FROM projects WHERE id = ?1",
            rusqlite::params![project_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .map_err(|_| ProjectError::NotFound(project_id.clone()))?
    };

    let dir = crate::projects::artifacts::artifact_dir(&app, &project_id)?;
    let plan_path = dir.join("plan.md");
    let plan_completed = plan_path.exists()
        && std::fs::read_to_string(&plan_path)
            .map(|content| !content.trim().is_empty())
            .unwrap_or(false);

    let prd_path = dir.join("prd.json");
    let stories_count = if prd_path.exists() {
        Prd::load(&prd_path)
            .map(|prd| prd.stories.len())
            .unwrap_or(0)
    } else {
        0
    };

    let config_path = dir.join("config.json");
    let config: Option<ProjectConfig> = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
    } else {
        None
    };

    let draft = serde_json::json!({
        "version": 1,
        "projectId": project_id,
        "currentStep": step_name,
        "describe": {
            "name": name,
            "description": description,
            "workingDirectory": working_directory,
            "planAgent": "claude",
            "planModel": null,
            "planEffort": null
        },
        "plan": { "completed": plan_completed },
        "atomize": { "storiesCount": stories_count },
        "configure": config.unwrap_or_default()
    });

    let draft_json = serde_json::to_string_pretty(&draft)?;
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("draft.json"), &draft_json)?;

    let now = chrono::Utc::now().to_rfc3339();
    let conn = db
        .0
        .lock()
        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let _ = conn.execute(
        "UPDATE projects SET wizard_step = ?1, wizard_state_json = NULL, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![step_name, now, project_id],
    );

    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaleResult {
    pub stale_from_step: u32,
}

#[tauri::command]
pub async fn mark_wizard_stale(
    db: State<'_, DbState>,
    project_id: String,
    from_step: u32,
) -> Result<StaleResult, ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;

    let now = chrono::Utc::now().to_rfc3339();
    let conn = db
        .0
        .lock()
        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let _ = conn.execute(
        "UPDATE projects SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, project_id],
    );

    Ok(StaleResult {
        stale_from_step: from_step,
    })
}
