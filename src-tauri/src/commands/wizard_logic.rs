use crate::agents::AgentRegistryState;
use crate::commands::validation::{non_empty_trimmed_list, optional_trimmed, required_trimmed};
use crate::loop_manager::StartLoopArgs;
use crate::plan_engine::PlanSessionsState;
use crate::projects::stories_crud::StoriesResponse;
use crate::projects::{
    AdvanceWizardResult, ConfigDefaultsResponse, DescribeInput, LaunchReadiness, ProjectConfig,
    ProjectError, ValidationErrors,
};
use crate::storage::db::DbState;
use ralph_core::prd::Prd;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn advance_wizard_step(target_step: u32) -> Result<AdvanceWizardResult, ProjectError> {
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
    name: String,
    description: String,
    working_directory: String,
    plan_agent: String,
    state: State<'_, AgentRegistryState>,
) -> Result<ValidationErrors, ProjectError> {
    let input = DescribeInput {
        name,
        description,
        working_directory,
        plan_agent,
    };
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
        let conn =
            db.0.lock()
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
    let (stories_count, estimated_minutes) = if prd_path.exists() {
        Prd::load(&prd_path)
            .map(|prd| {
                let minutes: Vec<u32> = prd
                    .stories
                    .iter()
                    .map(|story| story.estimated_minutes)
                    .collect();
                (prd.stories.len(), minutes)
            })
            .unwrap_or_default()
    } else {
        (0, Vec::new())
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
        &estimated_minutes,
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
        let conn =
            db.0.lock()
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
    let draft_path = dir.join("draft.json");
    let (plan_agent, plan_model, plan_effort, previous_highest_step) = if draft_path.exists() {
        std::fs::read_to_string(&draft_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .map_or(("claude".to_string(), None, None, 1), |draft| {
                let describe = draft
                    .get("describe")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                let step = draft
                    .get("highestStep")
                    .and_then(serde_json::Value::as_u64)
                    .map_or(1, |value| value as u32);
                (
                    describe
                        .get("planAgent")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("claude")
                        .to_string(),
                    describe
                        .get("planModel")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string),
                    describe
                        .get("planEffort")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string),
                    step,
                )
            })
    } else {
        ("claude".to_string(), None, None, 1)
    };
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
    let current_step_number =
        crate::projects::wizard_state::step_name_to_number(&step_name).unwrap_or(1);
    let highest_step = previous_highest_step.max(current_step_number);

    let draft = serde_json::json!({
        "version": 1,
        "projectId": project_id,
        "currentStep": step_name,
        "highestStep": highest_step,
        "describe": {
            "name": name,
            "description": description,
            "workingDirectory": working_directory,
            "planAgent": plan_agent,
            "planModel": plan_model,
            "planEffort": plan_effort
        },
        "plan": { "completed": plan_completed },
        "atomize": { "storiesCount": stories_count },
        "configure": config.unwrap_or_default()
    });

    let draft_json = serde_json::to_string_pretty(&draft)?;
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("draft.json"), &draft_json)?;

    let now = chrono::Utc::now().to_rfc3339();
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let _ = conn.execute(
        "UPDATE projects SET wizard_step = ?1, wizard_state_json = NULL, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![step_name, now, project_id],
    );

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteDescribeInput {
    #[serde(default)]
    pub project_id: Option<String>,
    pub name: String,
    pub description: String,
    pub working_directory: String,
    #[serde(default)]
    pub connection_id: Option<String>,
    pub plan_agent: String,
    #[serde(default)]
    pub plan_model: Option<String>,
    #[serde(default)]
    pub plan_effort: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardRouteResult {
    pub project_id: String,
    pub next_route: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
}

fn resolve_connection_workspace(
    app: &AppHandle,
    db: &DbState,
    connection_id: &str,
) -> Result<String, ProjectError> {
    let (repos, connection_name) = {
        let conn =
            db.0.lock()
                .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        let mut repo_stmt = conn
            .prepare(
                "SELECT repo_path, display_name FROM connection_repos WHERE connection_id = ?1",
            )
            .map_err(|err| ProjectError::Db(err.to_string()))?;
        let repos = repo_stmt
            .query_map(rusqlite::params![connection_id], |row| {
                Ok(crate::connections::ConnectionRepo {
                    repo_path: row.get::<_, String>(0)?,
                    display_name: row.get::<_, Option<String>>(1)?,
                })
            })
            .map_err(|err| ProjectError::Db(err.to_string()))?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        let name = conn
            .query_row(
                "SELECT name FROM connections WHERE id = ?1",
                rusqlite::params![connection_id],
                |row| row.get::<_, String>(0),
            )
            .map_err(|_| ProjectError::NotFound(connection_id.to_string()))?;
        (repos, name)
    };
    let workspace_dir = crate::connections::get_workspace_dir(app, connection_id);
    crate::connections::build_workspace(&workspace_dir, &repos)
        .map_err(|err| ProjectError::Db(err.to_string()))?;
    crate::connections::generate_workspace_manifest(&workspace_dir, &connection_name, &repos)
        .map_err(|err| ProjectError::Db(err.to_string()))?;
    Ok(workspace_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn complete_describe_step(
    app: AppHandle,
    db: State<'_, DbState>,
    input: CompleteDescribeInput,
) -> Result<WizardRouteResult, ProjectError> {
    let name = required_trimmed(input.name, "name").map_err(ProjectError::Path)?;
    let description =
        required_trimmed(input.description, "description").map_err(ProjectError::Path)?;
    let plan_agent =
        required_trimmed(input.plan_agent, "plan_agent").map_err(ProjectError::Path)?;
    let plan_model = optional_trimmed(input.plan_model);
    let plan_effort = optional_trimmed(input.plan_effort);
    let working_directory = if let Some(connection_id) = optional_trimmed(input.connection_id) {
        resolve_connection_workspace(&app, db.inner(), &connection_id)?
    } else {
        required_trimmed(input.working_directory, "working_directory")
            .map_err(ProjectError::Path)?
    };
    let now = chrono::Utc::now().to_rfc3339();
    let project_id = if let Some(existing_project_id) = optional_trimmed(input.project_id) {
        let conn =
            db.0.lock()
                .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        let updated = conn.execute(
            "UPDATE projects SET name = ?1, description = ?2, working_directory = ?3, wizard_step = 'plan', updated_at = ?4 WHERE id = ?5",
            rusqlite::params![name, description, working_directory, now, existing_project_id],
        )?;
        if updated == 0 {
            return Err(ProjectError::NotFound(existing_project_id));
        }
        existing_project_id
    } else {
        let created_project_id = uuid::Uuid::new_v4().to_string();
        let artifacts = crate::projects::artifacts::artifact_dir(&app, &created_project_id)?;
        crate::projects::artifacts::init_artifacts(&artifacts, &name)?;
        let conn =
            db.0.lock()
                .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        conn.execute(
            "INSERT INTO projects (id, name, description, status, working_directory, created_at, updated_at, wizard_step) VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, 'plan')",
            rusqlite::params![created_project_id, name, description, working_directory, now, now],
        )?;
        created_project_id
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
    let config: Option<ProjectConfig> = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
    } else {
        None
    };
    let previous_highest = std::fs::read_to_string(dir.join("draft.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|draft| draft.get("highestStep").and_then(serde_json::Value::as_u64))
        .map_or(1, |value| value as u32);
    let draft = serde_json::json!({
        "version": 1,
        "projectId": project_id,
        "currentStep": "plan",
        "highestStep": previous_highest.max(2),
        "describe": {
            "name": name,
            "description": description,
            "workingDirectory": working_directory,
            "planAgent": plan_agent,
            "planModel": plan_model,
            "planEffort": plan_effort
        },
        "plan": { "completed": false },
        "atomize": { "storiesCount": stories_count },
        "configure": config.unwrap_or_default()
    });
    std::fs::create_dir_all(&dir)?;
    std::fs::write(
        dir.join("draft.json"),
        serde_json::to_string_pretty(&draft)?,
    )?;
    Ok(WizardRouteResult {
        project_id: project_id.clone(),
        next_route: format!("/new/plan/{project_id}"),
        working_directory: Some(working_directory),
    })
}

#[tauri::command]
pub async fn complete_atomize_step(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<WizardRouteResult, ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    save_wizard_draft(app, db, project_id.clone(), "configure".to_string()).await?;
    let _ = advance_wizard_step(4).await;
    Ok(WizardRouteResult {
        project_id: project_id.clone(),
        next_route: format!("/new/configure/{project_id}"),
        working_directory: None,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteConfigureResult {
    pub config: ProjectConfig,
    pub errors: std::collections::HashMap<String, String>,
    pub next_route: String,
}

#[tauri::command]
pub async fn complete_configure_step(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
    raw: RawConfigInput,
) -> Result<CompleteConfigureResult, ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    let result = submit_project_config(app.clone(), project_id.clone(), raw).await?;
    if !result.errors.is_empty() {
        return Ok(CompleteConfigureResult {
            config: result.config,
            errors: result.errors,
            next_route: format!("/new/configure/{project_id}"),
        });
    }
    save_wizard_draft(app, db, project_id.clone(), "launch".to_string()).await?;
    let _ = advance_wizard_step(5).await;
    Ok(CompleteConfigureResult {
        config: result.config,
        errors: std::collections::HashMap::new(),
        next_route: format!("/new/launch/{project_id}"),
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchProjectResult {
    pub session_id: String,
    pub monitor_route: String,
}

#[tauri::command]
pub async fn launch_project(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<LaunchProjectResult, ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::wizard::finalize_draft(app.clone(), db, project_id.clone()).await?;
    let session_id = crate::loop_manager::start_loop(
        app,
        StartLoopArgs {
            project_id: project_id.clone(),
            project_name: None,
            working_directory: None,
            agent: None,
            model: None,
            effort: None,
            fallback_agents: Vec::new(),
            max_iterations: None,
            gutter_threshold: None,
            cooldown_seconds: None,
            test_command: None,
            max_verification_retries: None,
            scm_provider: None,
            review_polling_interval: None,
            review_timeout: None,
        },
    )
    .await
    .map_err(|err| ProjectError::Db(err.to_string()))?;
    Ok(LaunchProjectResult {
        session_id,
        monitor_route: format!("/monitor/{project_id}"),
    })
}

#[tauri::command]
pub async fn exit_wizard(
    app: AppHandle,
    db: State<'_, DbState>,
    plan_state: State<'_, PlanSessionsState>,
    project_id: String,
    current_step: u32,
) -> Result<(), ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    if current_step == 2 {
        let _ = crate::plan_engine::stop_plan(plan_state, project_id.clone()).await;
    }
    let step_name = crate::projects::wizard_state::step_number_to_name(current_step)
        .unwrap_or("describe")
        .to_string();
    save_wizard_draft(app, db, project_id, step_name).await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardStepMeta {
    pub number: u32,
    pub label: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorTabMeta {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub disable_for_inactive: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardDefaultsResponse {
    pub default_agent: String,
    pub placeholder_config: ProjectConfig,
    pub wizard_steps: Vec<WizardStepMeta>,
    pub monitor_tabs: Vec<MonitorTabMeta>,
}

#[tauri::command]
pub async fn get_wizard_defaults() -> Result<WizardDefaultsResponse, ProjectError> {
    Ok(WizardDefaultsResponse {
        default_agent: "claude".to_string(),
        placeholder_config: ProjectConfig::default(),
        wizard_steps: vec![
            WizardStepMeta {
                number: 1,
                label: "Describe".to_string(),
                slug: "describe".to_string(),
            },
            WizardStepMeta {
                number: 2,
                label: "Plan".to_string(),
                slug: "plan".to_string(),
            },
            WizardStepMeta {
                number: 3,
                label: "Atomize".to_string(),
                slug: "atomize".to_string(),
            },
            WizardStepMeta {
                number: 4,
                label: "Configure".to_string(),
                slug: "configure".to_string(),
            },
            WizardStepMeta {
                number: 5,
                label: "Launch".to_string(),
                slug: "launch".to_string(),
            },
        ],
        monitor_tabs: vec![
            MonitorTabMeta {
                id: "progress".to_string(),
                label: "Progress".to_string(),
                disable_for_inactive: false,
            },
            MonitorTabMeta {
                id: "activity".to_string(),
                label: "Activity".to_string(),
                disable_for_inactive: false,
            },
            MonitorTabMeta {
                id: "output".to_string(),
                label: "Output".to_string(),
                disable_for_inactive: false,
            },
            MonitorTabMeta {
                id: "ask".to_string(),
                label: "Ask".to_string(),
                disable_for_inactive: true,
            },
            MonitorTabMeta {
                id: "config".to_string(),
                label: "Config".to_string(),
                disable_for_inactive: false,
            },
        ],
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawConfigInput {
    #[serde(default)]
    pub execute_agent: String,
    #[serde(default)]
    pub execute_model: Option<String>,
    #[serde(default)]
    pub execute_effort: Option<String>,
    #[serde(default)]
    pub fallback_chain: Vec<String>,
    #[serde(default)]
    pub gutter_threshold: u32,
    #[serde(default)]
    pub max_iterations: u32,
    #[serde(default)]
    pub cooldown_seconds: u32,
    #[serde(default)]
    pub test_command: String,
    #[serde(default)]
    pub max_verification_retries: u32,
    #[serde(default)]
    pub scm_provider: String,
    #[serde(default)]
    pub review_polling_interval: u64,
    #[serde(default)]
    pub review_timeout: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitConfigResult {
    pub config: ProjectConfig,
    pub errors: std::collections::HashMap<String, String>,
}

#[tauri::command]
pub async fn submit_project_config(
    app: AppHandle,
    project_id: String,
    raw: RawConfigInput,
) -> Result<SubmitConfigResult, ProjectError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    let agent = required_trimmed(raw.execute_agent, "execute_agent").unwrap_or_default();

    let sanitized_chain = non_empty_trimmed_list(raw.fallback_chain)
        .into_iter()
        .filter(|entry| entry != &agent)
        .collect::<Vec<_>>();

    let config = ProjectConfig {
        schema_version: 1,
        execute_agent: agent,
        execute_model: raw.execute_model.filter(|val| !val.trim().is_empty()),
        execute_effort: raw.execute_effort.filter(|val| !val.trim().is_empty()),
        fallback_chain: sanitized_chain,
        gutter_threshold: raw.gutter_threshold,
        max_iterations: raw.max_iterations,
        cooldown_seconds: raw.cooldown_seconds,
        test_command: raw.test_command.trim().to_string(),
        max_verification_retries: raw.max_verification_retries,
        scm_provider: if raw.scm_provider.trim().is_empty() {
            "auto".to_string()
        } else {
            raw.scm_provider.trim().to_string()
        },
        review_polling_interval: raw.review_polling_interval,
        review_timeout: raw.review_timeout,
    };

    let validation = crate::projects::validation::validate_config(&config);
    if !validation.is_empty() {
        return Ok(SubmitConfigResult {
            config,
            errors: validation.errors,
        });
    }

    let config_json = serde_json::to_string_pretty(&config)?;
    let dir = crate::projects::artifacts::artifact_dir(&app, &project_id)?;
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("config.json"), &config_json)?;

    Ok(SubmitConfigResult {
        config,
        errors: std::collections::HashMap::new(),
    })
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
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let _ = conn.execute(
        "UPDATE projects SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, project_id],
    );

    Ok(StaleResult {
        stale_from_step: from_step,
    })
}
