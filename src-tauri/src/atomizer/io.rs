use crate::atomizer::AtomizerError;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Runtime};

pub(super) fn artifact_dir<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
) -> Result<PathBuf, AtomizerError> {
    crate::storage::artifacts::project_artifact_dir(app, project_id)
        .map_err(AtomizerError::Template)
}

pub(super) fn load_plan_content(
    artifact_path: &Path,
    project_dir: &Path,
) -> Result<String, AtomizerError> {
    let artifact_plan = artifact_path.join("plan.md");
    if artifact_plan.exists() {
        let content = std::fs::read_to_string(&artifact_plan)?;
        if !content.trim().is_empty() {
            return Ok(content);
        }
    }

    let legacy_plan = project_dir.join("plan.md");
    if legacy_plan.exists() {
        let content = std::fs::read_to_string(&legacy_plan)?;
        if !content.trim().is_empty() {
            let _ = std::fs::write(&artifact_plan, &content);
            return Ok(content);
        }
    }

    Ok(String::new())
}
