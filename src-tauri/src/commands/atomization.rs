use crate::atomizer::{
    get_pipeline_snapshot, ActivityLogState, AtomizeActivity, PipelineRegistryState,
    PipelineSnapshot,
};
use tauri::{AppHandle, State};

#[cfg(not(test))]
use crate::atomizer::{AtomizeArgs, AtomizerError};
#[cfg(not(test))]
use crate::commands::validation::{optional_trimmed, required_trimmed};
#[cfg(not(test))]
use ralph_core::prd::Prd;

#[cfg(not(test))]
#[tauri::command]
pub async fn run_atomizer(app: AppHandle, args: AtomizeArgs) -> Result<Prd, AtomizerError> {
    let normalized_args = AtomizeArgs {
        project_id: required_trimmed(args.project_id, "project_id").map_err(AtomizerError::Path)?,
        project_name: required_trimmed(args.project_name, "project_name")
            .map_err(AtomizerError::Path)?,
        project_dir: args.project_dir,
        agent: required_trimmed(args.agent, "agent").map_err(AtomizerError::Path)?,
        model: optional_trimmed(args.model),
        effort: optional_trimmed(args.effort),
    };
    crate::atomizer::run_atomizer(app, normalized_args).await
}

#[tauri::command]
pub fn get_atomizer_activity_log(
    state: State<'_, ActivityLogState>,
    project_id: String,
) -> Vec<AtomizeActivity> {
    state
        .0
        .lock()
        .map(|log| log.get(&project_id))
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_atomizer_pipeline_state(
    app: AppHandle,
    _registry: State<'_, PipelineRegistryState>,
    project_id: String,
) -> Option<PipelineSnapshot> {
    get_pipeline_snapshot(&app, &project_id)
}
