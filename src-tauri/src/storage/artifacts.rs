use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub fn project_artifact_dir(app: &AppHandle, project_id: &str) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|data_dir| data_dir.join("projects").join(project_id))
        .map_err(|err| err.to_string())
}

pub fn read_optional_non_empty(path: &Path) -> std::io::Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path)?;
    if content.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(content))
    }
}

pub fn file_paths(root: &Path) -> [PathBuf; 6] {
    [
        root.join("draft.json"),
        root.join("plan.md"),
        root.join("prd.json"),
        root.join("config.json"),
        root.join("prompt.md"),
        root.join("guardrails.md"),
    ]
}
