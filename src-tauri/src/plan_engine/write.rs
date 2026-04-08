use crate::plan_engine::{PlanEngineError, PlanSessionsState};
use tauri::State;

pub async fn write_to_plan(
    state: State<'_, PlanSessionsState>,
    project_id: String,
    input: String,
) -> Result<(), PlanEngineError> {
    let mut sessions = state.0.lock().map_err(|_| PlanEngineError::LockPoisoned)?;
    let entry = sessions
        .sessions
        .get_mut(&project_id)
        .ok_or_else(|| PlanEngineError::NoSession(project_id.clone()))?;

    let mut payload = input.into_bytes();
    payload.push(b'\n');
    entry
        .child
        .write(&payload)
        .map_err(|err| PlanEngineError::Shell(err.to_string()))?;

    Ok(())
}

pub async fn stop_plan(
    state: State<'_, PlanSessionsState>,
    project_id: String,
) -> Result<(), PlanEngineError> {
    let mut sessions = state.0.lock().map_err(|_| PlanEngineError::LockPoisoned)?;
    if let Some(entry) = sessions.sessions.remove(&project_id) {
        entry
            .child
            .kill()
            .map_err(|err| PlanEngineError::Shell(err.to_string()))?;
    }
    Ok(())
}
