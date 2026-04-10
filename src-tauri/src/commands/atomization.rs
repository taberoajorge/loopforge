#[cfg(not(test))]
use crate::atomizer::{AtomizeArgs, AtomizerError};
#[cfg(not(test))]
use crate::commands::validation::{optional_trimmed, required_trimmed};
#[cfg(not(test))]
use ralph_core::prd::Prd;
#[cfg(not(test))]
use tauri::AppHandle;

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
