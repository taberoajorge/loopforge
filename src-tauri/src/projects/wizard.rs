use crate::db::DbState;
use crate::projects::artifacts::{artifact_dir, non_empty_file_content};
use crate::projects::config_types::ProjectConfig;
use crate::projects::repository::{row_to_project, PROJECT_COLUMNS};
use crate::projects::{ProjectError, WizardHydrationResult, WizardProjectData, WizardResumeState};
use ralph_core::prd::Prd;
use serde_json::{Map, Value};
use std::path::Path;
use tauri::{AppHandle, Manager, Runtime, State};

pub async fn finalize_draft<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<(), ProjectError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    conn.execute(
        "UPDATE projects SET wizard_step = NULL, wizard_state_json = NULL, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, project_id],
    )?;
    drop(conn);
    let artifacts = artifact_dir(&app, &project_id)?;
    let draft_path = artifacts.join("draft.json");
    if draft_path.exists() {
        let _ = std::fs::remove_file(draft_path);
    }
    Ok(())
}

pub async fn discard_draft<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<(), ProjectError> {
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let deleted = conn.execute(
        "DELETE FROM projects WHERE id = ?1 AND status = 'draft'",
        rusqlite::params![project_id],
    )?;
    drop(conn);

    if deleted == 0 {
        return Err(ProjectError::NotFound(project_id));
    }

    let dir = artifact_dir(&app, &project_id)?;
    if dir.exists() {
        let _ = std::fs::remove_dir_all(&dir);
    }

    Ok(())
}

pub async fn save_wizard_state(
    db: State<'_, DbState>,
    project_id: String,
    wizard_step: String,
    wizard_state_json: String,
) -> Result<(), ProjectError> {
    let (_, canonical_json) =
        canonical_wizard_payload(&wizard_state_json, Some(wizard_step.as_str()))?;
    let now = chrono::Utc::now().to_rfc3339();
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let updated = conn.execute(
        "UPDATE projects SET wizard_step = ?1, wizard_state_json = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![wizard_step, canonical_json, now, project_id],
    )?;
    if updated == 0 {
        return Err(ProjectError::NotFound(project_id));
    }
    Ok(())
}

pub async fn save_draft<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
    draft_json: String,
) -> Result<(), ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    std::fs::create_dir_all(&artifacts)?;
    let (draft_step, canonical_json) = canonical_wizard_payload(&draft_json, None)?;
    std::fs::write(artifacts.join("draft.json"), &canonical_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    let db = app.state::<DbState>();
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let _ = conn.execute(
        "UPDATE projects SET wizard_step = ?1, wizard_state_json = NULL, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![draft_step, now, project_id],
    );
    Ok(())
}

fn canonical_wizard_payload(
    payload_json: &str,
    fallback_step: Option<&str>,
) -> Result<(String, String), ProjectError> {
    let mut payload = serde_json::from_str::<Map<String, Value>>(payload_json)?;
    let step = normalized_wizard_step(fallback_step, &payload);
    payload.insert("currentStep".to_string(), Value::String(step.clone()));
    Ok((step, serde_json::to_string(&Value::Object(payload))?))
}

fn normalized_wizard_step(fallback_step: Option<&str>, payload: &Map<String, Value>) -> String {
    fallback_step
        .map(str::trim)
        .filter(|step| !step.is_empty())
        .map(str::to_string)
        .or_else(|| {
            payload
                .get("currentStep")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|step| !step.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "describe".to_string())
}

pub async fn load_draft<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
) -> Result<Option<String>, ProjectError> {
    let artifacts = artifact_dir(&app, &project_id)?;
    non_empty_file_content(&artifacts.join("draft.json"))
}

pub async fn resume_wizard<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<WizardResumeState, ProjectError> {
    let (project, wizard_state_json) = {
        let conn =
            db.0.lock()
                .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        let query =
            format!("SELECT {PROJECT_COLUMNS}, wizard_state_json FROM projects WHERE id = ?1");
        let mut stmt = conn.prepare(&query)?;
        stmt.query_row(rusqlite::params![project_id], |row| {
            Ok((row_to_project(row)?, row.get::<_, Option<String>>(8)?))
        })
        .map_err(|_| ProjectError::NotFound(project_id.clone()))?
    };

    let step = match project.wizard_step.clone() {
        Some(value) => value,
        None if project.status == "draft" => "describe".to_string(),
        None => {
            return Err(ProjectError::NotFound(format!(
                "No wizard state for {project_id}"
            )))
        }
    };

    let dir = artifact_dir(&app, &project_id)?;
    let artifact_plan = non_empty_file_content(&dir.join("plan.md"))?;
    let legacy_plan =
        non_empty_file_content(&Path::new(&project.working_directory).join("plan.md"))?;
    let has_plan = artifact_plan.is_some() || legacy_plan.is_some();

    let artifact_prd = Prd::load(&dir.join("prd.json"))
        .ok()
        .filter(|prd| !prd.stories.is_empty());
    let legacy_prd = Prd::load(&Path::new(&project.working_directory).join("prd.json"))
        .ok()
        .filter(|prd| !prd.stories.is_empty());
    let has_prd = artifact_prd.is_some() || legacy_prd.is_some();

    Ok(WizardResumeState {
        project,
        wizard_step: step,
        wizard_state_json,
        has_plan,
        has_prd,
    })
}

pub async fn hydrate_wizard<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<WizardHydrationResult, ProjectError> {
    let resume = resume_wizard(app.clone(), db, project_id.clone()).await?;
    let dir = artifact_dir(&app, &project_id)?;
    let current_step_number =
        crate::projects::wizard_state::step_name_to_number(&resume.wizard_step).unwrap_or(1);

    let mut project_data = WizardProjectData {
        name: resume.project.name.clone(),
        description: resume.project.description.clone(),
        working_directory: resume.project.working_directory.clone(),
        plan_agent: "claude".to_string(),
        plan_model: None,
        plan_effort: None,
    };

    let mut plan_complete = resume.has_plan;
    let mut config: Option<ProjectConfig> = None;
    let mut highest_step = current_step_number;

    let draft_content = non_empty_file_content(&dir.join("draft.json"))?;
    let state_fallback = draft_content
        .is_none()
        .then(|| resume.wizard_state_json.clone())
        .flatten();
    let hydration_source = draft_content.or(state_fallback);
    if let Some(raw) = hydration_source {
        if let Ok(draft) = serde_json::from_str::<Value>(&raw) {
            if let Some(describe) = draft.get("describe") {
                if let Some(val) = describe.get("name").and_then(Value::as_str) {
                    if !val.is_empty() {
                        project_data.name = val.to_string();
                    }
                }
                if let Some(val) = describe.get("description").and_then(Value::as_str) {
                    if !val.is_empty() {
                        project_data.description = val.to_string();
                    }
                }
                if let Some(val) = describe.get("workingDirectory").and_then(Value::as_str) {
                    if !val.is_empty() {
                        project_data.working_directory = val.to_string();
                    }
                }
                if let Some(val) = describe.get("planAgent").and_then(Value::as_str) {
                    if !val.is_empty() {
                        project_data.plan_agent = val.to_string();
                    }
                }
                project_data.plan_model = describe
                    .get("planModel")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                project_data.plan_effort = describe
                    .get("planEffort")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
            if let Some(plan) = draft.get("plan") {
                if plan
                    .get("completed")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    plan_complete = true;
                }
            }
            if let Some(value) = draft.get("highestStep").and_then(Value::as_u64) {
                let parsed = value as u32;
                if parsed > highest_step {
                    highest_step = parsed;
                }
            }
            if let Some(configure) = draft.get("configure") {
                config = serde_json::from_value(configure.clone()).ok();
            }
        }
    }

    let stories = if resume.has_prd {
        Prd::load(&dir.join("prd.json"))
            .map(|prd| prd.stories)
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    Ok(WizardHydrationResult {
        project: resume.project,
        wizard_step: resume.wizard_step,
        highest_step,
        project_data,
        plan_complete,
        stories,
        config,
    })
}
