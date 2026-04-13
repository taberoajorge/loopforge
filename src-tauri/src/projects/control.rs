use crate::projects::artifacts::artifact_dir;
use crate::projects::ProjectError;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};

const TRANSITION_WAIT_STEPS: usize = 24;
const TRANSITION_WAIT_MS: u64 = 250;

fn loop_handle_state(app: &AppHandle, project_id: &str) -> (bool, bool) {
    let loop_state = app.state::<crate::loop_manager::LoopManagerState>();
    let Ok(handles) = loop_state.0.lock() else {
        return (false, false);
    };
    if let Some(handle) = handles.get(project_id) {
        let shutdown_requested = handle.shutdown_flag.load(Ordering::SeqCst);
        return (true, shutdown_requested);
    }
    (false, false)
}

async fn emit_project_state_changed(app: &AppHandle, project_id: &str) {
    let snapshot = crate::commands::projects::get_project_snapshot(
        app.clone(),
        app.state::<crate::db::DbState>(),
        app.state::<crate::loop_manager::LoopManagerState>(),
        project_id.to_string(),
    )
    .await
    .ok();
    let payload = if let Some(snapshot) = snapshot {
        serde_json::json!({
            "projectId": project_id,
            "snapshot": snapshot,
        })
    } else {
        serde_json::json!({ "projectId": project_id })
    };
    let _ = app.emit(crate::events::EVENT_PROJECT_STATE_CHANGED, payload);
}

async fn wait_for_shutdown_transition(app: &AppHandle, project_id: &str) {
    for _ in 0..TRANSITION_WAIT_STEPS {
        let (has_handle, shutdown_requested) = loop_handle_state(app, project_id);
        if !has_handle || !shutdown_requested {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(TRANSITION_WAIT_MS)).await;
    }
}

pub async fn pause_project(app: AppHandle, project_id: String) -> Result<(), ProjectError> {
    {
        let loop_state = app.state::<crate::loop_manager::LoopManagerState>();
        let lock_result = loop_state.0.lock();
        if let Ok(handles) = lock_result {
            if let Some(handle) = handles.get(project_id.as_str()) {
                handle.shutdown_flag.store(true, Ordering::SeqCst);
            }
        }
    }
    wait_for_shutdown_transition(&app, &project_id).await;

    let dir = artifact_dir(&app, &project_id)?;
    let _ = std::fs::write(dir.join(".ralph-pause"), "paused");

    let db = app.state::<crate::db::DbState>();
    let now = chrono::Utc::now().to_rfc3339();
    let updated = {
        let conn =
            db.0.lock()
                .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        conn.execute(
            "UPDATE projects SET status = 'paused', updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, &project_id],
        )?
    };
    if updated == 0 {
        return Err(ProjectError::NotFound(project_id));
    }
    emit_project_state_changed(&app, &project_id).await;
    Ok(())
}

pub async fn resume_project(app: AppHandle, project_id: String) -> Result<(), ProjectError> {
    {
        let db = app.state::<crate::db::DbState>();
        let conn =
            db.0.lock()
                .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
        let exists = conn
            .query_row(
                "SELECT id FROM projects WHERE id = ?1",
                rusqlite::params![&project_id],
                |row| row.get::<_, String>(0),
            )
            .is_ok();
        if !exists {
            return Err(ProjectError::NotFound(project_id.clone()));
        }
    }
    let dir = artifact_dir(&app, &project_id)?;
    let pause_file = dir.join(".ralph-pause");
    if pause_file.exists() {
        let _ = std::fs::remove_file(&pause_file);
    }

    let (has_loop_handle, shutdown_requested) = loop_handle_state(&app, &project_id);
    if shutdown_requested {
        wait_for_shutdown_transition(&app, &project_id).await;
    }
    let (still_has_loop_handle, _) = loop_handle_state(&app, &project_id);
    if has_loop_handle && still_has_loop_handle {
        let now = chrono::Utc::now().to_rfc3339();
        let db = app.state::<crate::db::DbState>();
        {
            let conn =
                db.0.lock()
                    .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
            let _ = conn.execute(
                "UPDATE projects SET status = 'active', updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, &project_id],
            );
        }
        emit_project_state_changed(&app, &project_id).await;
        return Ok(());
    }
    let restart_args = crate::loop_manager::StartLoopArgs {
        project_id: project_id.clone(),
        project_name: None,
        working_directory: None,
        agent: None,
        model: None,
        effort: None,
        fallback_agents: vec![],
        max_iterations: None,
        gutter_threshold: None,
        cooldown_seconds: None,
        test_command: None,
        max_verification_retries: None,
        scm_provider: None,
        review_polling_interval: None,
        review_timeout: None,
    };
    crate::loop_manager::start_loop(app, restart_args)
        .await
        .map_err(|err| ProjectError::Db(err.to_string()))?;
    Ok(())
}
