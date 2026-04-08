use crate::db::DbState;
use crate::projects::artifacts::{
    artifact_dir, non_empty_file_content, project_working_directory,
};
use crate::projects::ProjectError;
use ralph_core::prd::Prd;
use std::path::Path;
use tauri::{AppHandle, State};

pub async fn load_existing_plan(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<Option<String>, ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    let plan_path = artifacts.join("plan.md");
    if let Some(content) = non_empty_file_content(&plan_path)? {
        return Ok(Some(content));
    }

    let working_directory = project_working_directory(&db, &project_id)?;
    let legacy_path = Path::new(&working_directory).join("plan.md");
    if let Some(content) = non_empty_file_content(&legacy_path)? {
        std::fs::create_dir_all(&artifacts)?;
        let _ = std::fs::write(&plan_path, &content);
        return Ok(Some(content));
    }

    Ok(None)
}

pub async fn save_plan(
    app: AppHandle,
    project_id: String,
    content: String,
) -> Result<(), ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    std::fs::create_dir_all(&artifacts)?;
    std::fs::write(artifacts.join("plan.md"), &content)?;
    Ok(())
}

pub async fn save_prd(
    app: AppHandle,
    project_id: String,
    prd_json: String,
) -> Result<(), ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    std::fs::create_dir_all(&artifacts)?;
    serde_json::from_str::<Prd>(&prd_json)?;
    std::fs::write(artifacts.join("prd.json"), &prd_json)?;
    Ok(())
}

pub async fn save_config(
    app: AppHandle,
    project_id: String,
    config_json: String,
) -> Result<(), ProjectError> {
    let parsed_config = serde_json::from_str::<crate::projects::ProjectConfig>(&config_json)?;
    crate::projects::runtime_config::save_project_config(&app, &project_id, &parsed_config)?;
    Ok(())
}

pub async fn load_config(
    app: AppHandle,
    project_id: String,
) -> Result<Option<String>, ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    let config_path = artifacts.join("config.json");
    non_empty_file_content(&config_path)
}

pub async fn load_existing_prd(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<Option<Prd>, ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    let prd_path = artifacts.join("prd.json");
    if let Ok(prd) = Prd::load(&prd_path) {
        if !prd.stories.is_empty() {
            return Ok(Some(prd));
        }
    }

    let working_directory = project_working_directory(&db, &project_id)?;
    let legacy_path = Path::new(&working_directory).join("prd.json");
    if let Ok(prd) = Prd::load(&legacy_path) {
        if !prd.stories.is_empty() {
            std::fs::create_dir_all(&artifacts)?;
            let _ = prd.save(&prd_path);
            return Ok(Some(prd));
        }
    }

    Ok(None)
}

pub async fn get_guardrails(
    app: AppHandle,
    project_id: String,
) -> Result<String, ProjectError> {
    let dir = artifact_dir(&app, &project_id)?;
    let guardrails_path = dir.join("guardrails.md");

    if guardrails_path.exists() {
        std::fs::read_to_string(&guardrails_path).map_err(ProjectError::Io)
    } else {
        Ok(String::new())
    }
}

pub async fn load_output_log(
    app: AppHandle,
    project_id: String,
) -> Result<String, ProjectError> {
    let dir = artifact_dir(&app, &project_id)?;
    let output_path = dir.join("agent_output.log");
    if !output_path.exists() {
        return Ok(String::new());
    }

    let content = std::fs::read_to_string(output_path)?;
    Ok(tail_lines(&content, 400))
}

fn tail_lines(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= max_lines {
        return content.to_string();
    }

    lines[lines.len() - max_lines..].join("\n")
}
