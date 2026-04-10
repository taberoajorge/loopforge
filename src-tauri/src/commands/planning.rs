use crate::commands::validation::required_trimmed;
#[cfg(not(test))]
use crate::commands::validation::optional_trimmed;
use crate::plan_engine::{PlanEngineError, PlanSessionsState};
#[cfg(not(test))]
use crate::plan_engine::{PlanSessionInfo, StartPlanArgs};
#[cfg(not(test))]
use tauri::AppHandle;
use tauri::State;

#[cfg(not(test))]
#[tauri::command]
pub async fn start_plan(
    app: AppHandle,
    state: State<'_, PlanSessionsState>,
    args: StartPlanArgs,
) -> Result<(), PlanEngineError> {
    let normalized_args = StartPlanArgs {
        project_id: required_trimmed(args.project_id, "project_id")
            .map_err(PlanEngineError::Path)?,
        project_dir: args.project_dir,
        agent: required_trimmed(args.agent, "agent").map_err(PlanEngineError::Path)?,
        model: optional_trimmed(args.model),
        effort: optional_trimmed(args.effort),
        initial_prompt: required_trimmed(args.initial_prompt, "initial_prompt")
            .map_err(PlanEngineError::Path)?,
    };
    crate::plan_engine::start_plan(app, state, normalized_args).await
}

#[tauri::command]
pub async fn write_to_plan(
    state: State<'_, PlanSessionsState>,
    project_id: String,
    input: String,
) -> Result<(), PlanEngineError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(PlanEngineError::Path)?;
    crate::plan_engine::write_to_plan(state, normalized_project_id, input).await
}

#[tauri::command]
pub async fn stop_plan(
    state: State<'_, PlanSessionsState>,
    project_id: String,
) -> Result<(), PlanEngineError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(PlanEngineError::Path)?;
    crate::plan_engine::stop_plan(state, normalized_project_id).await
}

#[cfg(not(test))]
#[tauri::command]
pub async fn query_plan_status(
    state: State<'_, PlanSessionsState>,
    project_id: String,
) -> Result<Option<PlanSessionInfo>, PlanEngineError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(PlanEngineError::Path)?;
    crate::plan_engine::query_plan_status(state, normalized_project_id).await
}
