use crate::atomizer::{AtomizeArgs, AtomizerError};
use crate::db::DbState;
use crate::loop_manager::{LoopError, StartLoopArgs};
use crate::plan_engine::{PlanEngineError, PlanSessionsState, StartPlanArgs};
use crate::projects::{Project, ProjectDetail, ProjectError};
use crate::services;
use ralph_core::prd::Prd;
use tauri::{AppHandle, Runtime, State};

#[tauri::command]
pub async fn create_project<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    name: String,
    description: String,
    working_directory: String,
    wizard_step: Option<String>,
) -> Result<Project, ProjectError> {
    let request = services::build_create_project_request(
        required(name, "name").map_err(ProjectError::Path)?,
        required(description, "description").map_err(ProjectError::Path)?,
        required(working_directory, "working_directory").map_err(ProjectError::Path)?,
        optional(wizard_step),
    );
    crate::projects::catalog::create_project(
        app,
        db,
        request.name,
        request.description,
        request.working_directory,
        request.wizard_step,
    )
    .await
}

#[tauri::command]
pub async fn save_draft<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
    draft_json: String,
) -> Result<(), ProjectError> {
    let project_id = required(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::wizard::save_draft(app, project_id, draft_json).await
}

#[tauri::command]
pub async fn load_draft<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
) -> Result<Option<String>, ProjectError> {
    let project_id = required(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::wizard::load_draft(app, project_id).await
}

#[tauri::command]
pub async fn finalize_draft<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<(), ProjectError> {
    let project_id = required(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::wizard::finalize_draft(app, db, project_id).await
}

#[tauri::command]
pub async fn start_plan<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, PlanSessionsState>,
    args: StartPlanArgs,
) -> Result<(), PlanEngineError> {
    let args = StartPlanArgs {
        project_id: required(args.project_id, "project_id").map_err(PlanEngineError::Path)?,
        project_dir: args.project_dir,
        agent: required(args.agent, "agent").map_err(PlanEngineError::Path)?,
        model: optional(args.model),
        effort: optional(args.effort),
        initial_prompt: required(args.initial_prompt, "initial_prompt")
            .map_err(PlanEngineError::Path)?,
    };
    crate::plan_engine::start_plan(app, state, args).await
}

#[tauri::command]
pub async fn query_plan_status(
    state: State<'_, PlanSessionsState>,
    project_id: String,
) -> Result<Option<serde_json::Value>, PlanEngineError> {
    let project_id = required(project_id, "project_id").map_err(PlanEngineError::Path)?;
    crate::plan_engine::query_plan_status(state, project_id)
        .await
        .map(|value| value.map(|info| serde_json::to_value(info).expect("plan session info")))
}

#[tauri::command]
pub async fn load_existing_plan<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<Option<String>, ProjectError> {
    let project_id = required(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::documents::load_existing_plan(app, db, project_id).await
}

#[tauri::command]
pub async fn save_plan<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
    content: String,
) -> Result<(), ProjectError> {
    let project_id = required(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::documents::save_plan(app, project_id, content).await
}

#[tauri::command]
pub async fn save_config<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
    config_json: String,
) -> Result<(), ProjectError> {
    let project_id = required(project_id, "project_id").map_err(ProjectError::Path)?;
    crate::projects::documents::save_config(app, project_id, config_json).await
}

#[tauri::command]
pub async fn run_atomizer<R: Runtime>(
    app: AppHandle<R>,
    args: AtomizeArgs,
) -> Result<Prd, AtomizerError> {
    let request = services::build_atomizer_request(
        required(args.project_id, "project_id").map_err(AtomizerError::Path)?,
    );
    let args = AtomizeArgs {
        project_id: request.project_id,
        project_name: required(args.project_name, "project_name").map_err(AtomizerError::Path)?,
        project_dir: args.project_dir,
        agent: required(args.agent, "agent").map_err(AtomizerError::Path)?,
        model: optional(args.model),
        effort: optional(args.effort),
    };
    crate::atomizer::run_atomizer(app, args).await
}

#[tauri::command]
pub async fn start_loop<R: Runtime>(
    app: AppHandle<R>,
    args: StartLoopArgs,
) -> Result<String, LoopError> {
    let loop_command = services::build_start_loop_command(
        required(args.project_id, "project_id").map_err(LoopError::Path)?,
    );
    let project_id = match loop_command {
        services::app_core_loop_session::LoopCommand::Start { project_id }
        | services::app_core_loop_session::LoopCommand::Stop { project_id } => project_id,
    };
    let args = StartLoopArgs {
        project_id,
        project_name: args.project_name,
        working_directory: args.working_directory,
        agent: args.agent,
        model: args.model,
        effort: args.effort,
        fallback_agents: args.fallback_agents,
        max_iterations: args.max_iterations,
        gutter_threshold: args.gutter_threshold,
        cooldown_seconds: args.cooldown_seconds,
        test_command: optional(args.test_command),
        max_verification_retries: args.max_verification_retries,
        scm_provider: args.scm_provider,
        review_polling_interval: args.review_polling_interval,
        review_timeout: args.review_timeout,
    };
    crate::loop_manager::start_loop(app, args).await
}

#[tauri::command]
pub async fn get_project_detail<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<ProjectDetail, ProjectError> {
    let project_id = required(project_id, "project_id").map_err(ProjectError::Path)?;
    let detail = crate::projects::catalog::get_project_detail(app, db, project_id).await?;
    Ok(services::from_shared_project_detail(
        services::to_shared_project_detail(detail),
    ))
}

fn required(value: String, name: &str) -> Result<String, String> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Err(format!("{name} is required"));
    }
    Ok(trimmed)
}

fn optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
