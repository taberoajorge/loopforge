use super::helpers::{artifact_dir, close_session};
use super::state::LoopManagerState;
use crate::db::DbState;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

pub(super) async fn finalize_loop_run(
    app: &AppHandle,
    project_id: &str,
    project_name: &str,
    session_id: &str,
    working_directory: &str,
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

    match outcome {
        "completed" => crate::notifications::notify_loop_completed(app, project_name),
        "failed" => crate::notifications::notify_loop_error(app, project_name),
        _ => {}
    }

    let _ = app.emit(
        crate::events::EVENT_SESSION_ENDED,
        serde_json::json!({
            "projectId": project_id,
            "sessionId": session_id,
            "outcome": outcome,
        }),
    );
}
