use super::helpers::close_session;
use super::state::LoopManagerState;
use crate::db::DbState;
use crate::projects::notifications::{create_notification_and_emit, NotificationCreateInput};
use tauri::{AppHandle, Emitter, Manager};

pub(super) async fn finalize_loop_run(
    app: &AppHandle,
    project_id: &str,
    _project_name: &str,
    session_id: &str,
    _working_directory: &str,
    outcome: &str,
) {
    let db_state = app.state::<DbState>();
    close_session(&db_state, session_id);

    let _ = app.state::<LoopManagerState>().0.lock().map(|mut handles| {
        handles.remove(project_id);
    });

    let active = app.state::<LoopManagerState>().active_count();
    crate::tray::update_tooltip(app, active);

    if let Ok(conn) = db_state.0.lock() {
        let now = chrono::Utc::now().to_rfc3339();
        let _ = conn.execute(
            "UPDATE projects SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status != 'archived'",
            rusqlite::params![outcome, now, project_id],
        );
    }

    #[cfg(feature = "frozen")]
    if let Ok(project_artifacts) = artifact_dir(app, project_id) {
        let prd_file = project_artifacts.join("prd.json");
        if prd_file.exists() {
            if let Ok(prd) = ralph_core::prd::Prd::load(&prd_file) {
                if let Ok(conn) = db_state.0.lock() {
                    let work_dir = PathBuf::from(working_directory);
                    let mut summary = crate::summary_generator::generate_summary(
                        &conn,
                        project_id,
                        project_name,
                        session_id,
                        &prd,
                        &work_dir,
                    );
                    if summary.narrative.is_empty() {
                        summary.narrative = format!(
                            "Loop for {} completed: {}/{} passed.",
                            project_name, summary.passed, summary.total_stories
                        );
                    }
                    let summary_file = project_artifacts.join("summary.json");
                    let _ = serde_json::to_string_pretty(&summary)
                        .map(|json| std::fs::write(&summary_file, json));
                }
            }
        }
    }

    #[cfg(feature = "frozen")]
    match outcome {
        "completed" => crate::notifications::notify_loop_completed(app, project_name),
        "failed" => crate::notifications::notify_loop_error(app, project_name),
        _ => {}
    }
    let _ = create_notification_and_emit(
        app,
        NotificationCreateInput {
            project_id: project_id.to_string(),
            notification_type: "loop_completed".to_string(),
            title: "Loop finished".to_string(),
            message: "Session completed.".to_string(),
        },
    );

    let _ = app.emit(
        crate::events::EVENT_SESSION_ENDED,
        serde_json::json!({
            "projectId": project_id,
            "sessionId": session_id,
            "outcome": outcome,
        }),
    );
    let _ = app.emit(
        crate::events::EVENT_STORIES_UPDATED,
        serde_json::json!({ "projectId": project_id }),
    );
    let snapshot = crate::commands::projects::get_project_snapshot(
        app.clone(),
        app.state::<DbState>(),
        app.state::<LoopManagerState>(),
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
