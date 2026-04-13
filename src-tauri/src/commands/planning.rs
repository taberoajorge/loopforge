#[cfg(not(test))]
use crate::commands::validation::optional_trimmed;
use crate::commands::validation::required_trimmed;
use crate::plan_engine::{PlanEngineError, PlanSessionsState};
#[cfg(not(test))]
use crate::plan_engine::{PlanSessionInfo, StartPlanArgs};
use serde::Deserialize;
use serde::Serialize;
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum PlanStepState {
    Running,
    HasExistingPlan { content: String },
    NeedsFreshPlan,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvePlanActionResult {
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_content: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlanUserActionKind {
    Feedback,
    Replan,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanUserActionResult {
    pub mode: String,
}

#[cfg(not(test))]
#[tauri::command]
pub async fn resolve_plan_state(
    app: AppHandle,
    state: State<'_, PlanSessionsState>,
    project_id: String,
) -> Result<PlanStepState, PlanEngineError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(PlanEngineError::Path)?;

    let status = crate::plan_engine::query_plan_status(state, project_id.clone()).await?;
    if let Some(info) = status {
        if info.status == crate::plan_engine::PlanSessionStatus::Running {
            return Ok(PlanStepState::Running);
        }
    }

    let dir = crate::projects::artifacts::artifact_dir(&app, &project_id)
        .map_err(|err| PlanEngineError::Path(err.to_string()))?;
    let plan_path = dir.join("plan.md");
    if plan_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&plan_path) {
            if !content.trim().is_empty() {
                return Ok(PlanStepState::HasExistingPlan { content });
            }
        }
    }

    Ok(PlanStepState::NeedsFreshPlan)
}

#[cfg(not(test))]
#[tauri::command]
pub async fn resolve_plan_action(
    app: AppHandle,
    state: State<'_, PlanSessionsState>,
    project_id: String,
) -> Result<ResolvePlanActionResult, PlanEngineError> {
    let resolved = resolve_plan_state(app, state, project_id).await?;
    match resolved {
        PlanStepState::Running => Ok(ResolvePlanActionResult {
            action: "resume".to_string(),
            plan_content: None,
        }),
        PlanStepState::HasExistingPlan { content } => Ok(ResolvePlanActionResult {
            action: "prompt_existing".to_string(),
            plan_content: Some(content),
        }),
        PlanStepState::NeedsFreshPlan => Ok(ResolvePlanActionResult {
            action: "start".to_string(),
            plan_content: None,
        }),
    }
}

#[cfg(not(test))]
#[tauri::command]
pub async fn plan_user_action(
    app: AppHandle,
    state: State<'_, PlanSessionsState>,
    db: State<'_, crate::storage::db::DbState>,
    project_id: String,
    input: String,
    action: PlanUserActionKind,
) -> Result<PlanUserActionResult, PlanEngineError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(PlanEngineError::Path)?;
    let normalized_input = required_trimmed(input, "input").map_err(PlanEngineError::Path)?;
    let plan_status =
        crate::plan_engine::query_plan_status(state.clone(), project_id.clone()).await?;
    match action {
        PlanUserActionKind::Feedback => {
            let is_running = plan_status
                .as_ref()
                .is_some_and(|info| info.status == crate::plan_engine::PlanSessionStatus::Running);
            if is_running {
                crate::plan_engine::write_to_plan(state, project_id, normalized_input).await?;
                return Ok(PlanUserActionResult {
                    mode: "sent".to_string(),
                });
            }
            replan(app, state, db, project_id, normalized_input).await?;
            Ok(PlanUserActionResult {
                mode: "replanned".to_string(),
            })
        }
        PlanUserActionKind::Replan => {
            let _ = crate::plan_engine::stop_plan(state.clone(), project_id.clone()).await;
            replan(app, state, db, project_id, normalized_input).await?;
            Ok(PlanUserActionResult {
                mode: "replanned".to_string(),
            })
        }
    }
}

#[cfg(not(test))]
#[tauri::command]
pub async fn replan(
    app: AppHandle,
    state: State<'_, PlanSessionsState>,
    db: State<'_, crate::storage::db::DbState>,
    project_id: String,
    feedback: String,
) -> Result<(), PlanEngineError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(PlanEngineError::Path)?;
    let feedback = required_trimmed(feedback, "feedback").map_err(PlanEngineError::Path)?;

    let (description, working_directory) = {
        let conn = db.0.lock().map_err(|_| PlanEngineError::LockPoisoned)?;
        conn.query_row(
            "SELECT description, working_directory FROM projects WHERE id = ?1",
            rusqlite::params![project_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|_| PlanEngineError::Path(format!("Project not found: {project_id}")))?
    };

    let artifacts = crate::projects::artifacts::artifact_dir(&app, &project_id)
        .map_err(|err| PlanEngineError::Path(err.to_string()))?;
    let plan_path = artifacts.join("plan.md");
    let existing_plan = if plan_path.exists() {
        std::fs::read_to_string(&plan_path).unwrap_or_default()
    } else {
        String::new()
    };

    let draft_path = artifacts.join("draft.json");
    let (agent, model, effort) = if draft_path.exists() {
        let draft_content = std::fs::read_to_string(&draft_path)
            .map_err(|err| PlanEngineError::Path(err.to_string()))?;
        let draft: serde_json::Value = serde_json::from_str(&draft_content)
            .map_err(|err| PlanEngineError::Path(err.to_string()))?;
        let describe = &draft["describe"];
        (
            describe["planAgent"]
                .as_str()
                .unwrap_or("claude")
                .to_string(),
            describe["planModel"].as_str().map(String::from),
            describe["planEffort"].as_str().map(String::from),
        )
    } else {
        ("claude".to_string(), None, None)
    };

    let has_plan_structure = existing_plan
        .lines()
        .any(|line| line.trim_start().starts_with('#'));

    let composed_prompt = if existing_plan.is_empty() || !has_plan_structure {
        format!("{description}\n\nUser clarification: {feedback}")
    } else {
        format!(
            "{description}\n\n\
             Previous plan:\n{existing_plan}\n\n\
             Feedback: {feedback}"
        )
    };

    {
        let mut sessions = state.0.lock().map_err(|_| PlanEngineError::LockPoisoned)?;
        if let Some(entry) = sessions.sessions.remove(&project_id) {
            let _ = entry.handle.kill();
        }
    }

    let replan_args = StartPlanArgs {
        project_id: project_id.clone(),
        project_dir: std::path::PathBuf::from(&working_directory),
        agent,
        model: optional_trimmed(model),
        effort: optional_trimmed(effort),
        initial_prompt: composed_prompt,
    };
    crate::plan_engine::start_plan(app, state, replan_args).await
}
