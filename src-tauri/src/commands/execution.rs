use crate::commands::validation::required_trimmed;
#[cfg(not(test))]
use crate::commands::validation::{non_empty_trimmed_list, optional_trimmed};
use crate::loop_manager::{LoopError, LoopManagerState, SessionStats};
#[cfg(not(test))]
use crate::loop_manager::StartLoopArgs;
use crate::storage::db::DbState;
use serde::Serialize;
use tauri::{AppHandle, State};

#[cfg(not(test))]
#[tauri::command]
pub async fn start_loop(app: AppHandle, args: StartLoopArgs) -> Result<String, LoopError> {
    let normalized_args = StartLoopArgs {
        project_id: required_trimmed(args.project_id, "project_id").map_err(LoopError::Path)?,
        project_name: args
            .project_name
            .map(|value| required_trimmed(value, "project_name"))
            .transpose()
            .map_err(LoopError::Path)?,
        working_directory: args
            .working_directory
            .map(|value| required_trimmed(value, "working_directory"))
            .transpose()
            .map_err(LoopError::Path)?,
        agent: args
            .agent
            .map(|value| required_trimmed(value, "agent"))
            .transpose()
            .map_err(LoopError::Path)?,
        model: optional_trimmed(args.model),
        effort: optional_trimmed(args.effort),
        fallback_agents: non_empty_trimmed_list(args.fallback_agents),
        max_iterations: args.max_iterations,
        gutter_threshold: args.gutter_threshold,
        cooldown_seconds: args.cooldown_seconds,
        test_command: optional_trimmed(args.test_command),
        max_verification_retries: args.max_verification_retries,
    };
    crate::loop_manager::start_loop(app, normalized_args).await
}

#[tauri::command]
pub async fn stop_loop(
    app: AppHandle,
    loop_state: State<'_, LoopManagerState>,
    project_id: String,
) -> Result<(), LoopError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(LoopError::Path)?;
    crate::loop_manager::stop_loop(app, loop_state, normalized_project_id).await
}

#[tauri::command]
pub async fn session_stats(
    app: AppHandle,
    db: State<'_, DbState>,
    loop_state: State<'_, LoopManagerState>,
    project_id: String,
) -> Result<SessionStats, LoopError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(LoopError::Path)?;
    crate::loop_manager::session_stats(app, db, loop_state, normalized_project_id).await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IterationRow {
    pub story_id: String,
    pub started_at: String,
    pub duration_secs: i64,
    pub result: String,
    pub agent_used: String,
}

#[tauri::command]
pub async fn get_iteration_history(
    db: State<'_, DbState>,
    project_id: String,
) -> Result<Vec<IterationRow>, LoopError> {
    let pid = required_trimmed(project_id, "project_id").map_err(LoopError::Path)?;
    let conn =
        db.0.lock()
            .map_err(|_| LoopError::Internal("database lock poisoned".into()))?;
    let mut stmt = conn
        .prepare(
            "SELECT i.story_id, i.started_at, i.duration_secs, i.result, i.agent_used \
             FROM iterations i \
             JOIN sessions s ON i.session_id = s.id \
             WHERE s.project_id = ?1 \
             ORDER BY i.started_at DESC \
             LIMIT 200",
        )
        .map_err(|err| LoopError::Internal(err.to_string()))?;
    let rows = stmt
        .query_map(rusqlite::params![pid], |row| {
            Ok(IterationRow {
                story_id: row.get(0)?,
                started_at: row.get(1)?,
                duration_secs: row.get(2)?,
                result: row.get(3)?,
                agent_used: row.get(4)?,
            })
        })
        .map_err(|err| LoopError::Internal(err.to_string()))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|err| LoopError::Internal(err.to_string()))?);
    }
    Ok(result)
}
