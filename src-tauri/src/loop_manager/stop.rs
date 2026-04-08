use super::helpers::artifact_dir;
use super::{LoopError, LoopManagerState};
use crate::db::DbState;
use tauri::{AppHandle, Manager, State};

pub async fn stop_loop(
    app: AppHandle,
    loop_state: State<'_, LoopManagerState>,
    project_id: String,
) -> Result<(), LoopError> {
    {
        let handles = loop_state.0.lock().map_err(|_| LoopError::LockPoisoned)?;
        if let Some(handle) = handles.get(&project_id) {
            handle
                .shutdown_flag
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    if let Ok(project_artifacts) = artifact_dir(&app, &project_id) {
        let pause_path = project_artifacts.join(".ralph-pause");
        if pause_path.exists() {
            let _ = std::fs::remove_file(&pause_path);
        }
    }

    let db = app.state::<DbState>();
    let conn = db.0.lock().map_err(|_| LoopError::LockPoisoned)?;
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "UPDATE projects SET status = 'archived', updated_at = ?1 WHERE id = ?2 AND status IN ('active', 'blocked', 'paused')",
        rusqlite::params![now, project_id],
    );

    Ok(())
}
