use crate::plan_engine::sessions::PlanSessionInfo;
use crate::plan_engine::{PlanEngineError, PlanSessionsState};
use tauri::State;

pub async fn query_plan_status(
    state: State<'_, PlanSessionsState>,
    project_id: String,
) -> Result<Option<PlanSessionInfo>, PlanEngineError> {
    let sessions = state.0.lock().map_err(|_| PlanEngineError::LockPoisoned)?;
    let entry = match sessions.sessions.get(&project_id) {
        Some(found) => found,
        None => return Ok(None),
    };

    let elapsed_secs = entry.started_at.elapsed().as_secs();
    let last_activity_secs_ago = entry
        .last_activity_at
        .lock()
        .map(|instant| instant.elapsed().as_secs())
        .unwrap_or(0);

    Ok(Some(PlanSessionInfo {
        status: entry.status.clone(),
        agent_name: entry.agent_name.clone(),
        elapsed_secs,
        last_activity_secs_ago,
    }))
}
