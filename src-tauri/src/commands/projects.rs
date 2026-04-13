use crate::commands::validation::required_trimmed;
use crate::loop_manager::LoopManagerState;
use crate::models::{
    ArtifactPaths, ProgressInfo, ProjectConfig as SnapshotConfig, ProjectSnapshot, ProjectStatus,
    SessionInfo,
};
use crate::projects::{ProjectConfig, ProjectError};
use crate::storage;
use crate::storage::db::DbState;
use rusqlite::OptionalExtension;
use tauri::{AppHandle, Manager, Runtime, State};

fn to_snapshot_config(config: ProjectConfig) -> SnapshotConfig {
    SnapshotConfig {
        schema_version: config.schema_version,
        execute_agent: config.execute_agent,
        execute_model: config.execute_model,
        execute_effort: config.execute_effort,
        fallback_chain: config.fallback_chain,
        gutter_threshold: config.gutter_threshold,
        max_iterations: config.max_iterations,
        cooldown_seconds: config.cooldown_seconds,
        test_command: config.test_command,
        max_verification_retries: config.max_verification_retries,
        scm_provider: config.scm_provider,
        review_polling_interval: config.review_polling_interval,
        review_timeout: config.review_timeout,
    }
}

fn build_duration_label(started_at: Option<&str>, ended_at: Option<&str>) -> String {
    let Some(start_value) = started_at else {
        return String::new();
    };
    let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(start_value) else {
        return String::new();
    };
    let end_time = ended_at
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .unwrap_or_else(|| chrono::Utc::now().fixed_offset());
    let elapsed_secs = (end_time - start_time).num_seconds().max(0);
    if elapsed_secs < 60 {
        return format!("{elapsed_secs}s");
    }
    let elapsed_minutes = elapsed_secs / 60;
    if elapsed_minutes < 60 {
        return format!("{elapsed_minutes}m");
    }
    let elapsed_hours = elapsed_minutes / 60;
    let remaining_minutes = elapsed_minutes % 60;
    format!("{elapsed_hours}h {remaining_minutes}m")
}

#[tauri::command]
pub async fn get_project_snapshot<R: Runtime>(
    app: AppHandle<R>,
    db: State<'_, DbState>,
    loop_state: State<'_, LoopManagerState>,
    project_id: String,
) -> Result<ProjectSnapshot, ProjectError> {
    let normalized_project_id =
        required_trimmed(project_id, "project_id").map_err(ProjectError::Path)?;
    let has_loop_handle = loop_state
        .0
        .lock()
        .map(|handles| handles.contains_key(&normalized_project_id))
        .unwrap_or(false);

    let detail = crate::projects::catalog::get_project_detail(
        app.clone(),
        db,
        normalized_project_id.clone(),
    )
    .await?;
    let config = crate::projects::runtime_config::get_project_config(
        app.clone(),
        normalized_project_id.clone(),
    )
    .await
    .unwrap_or(None);

    let db_state = app.state::<DbState>();
    let conn = db_state
        .0
        .lock()
        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;

    let active_session: Option<SessionInfo> = conn
        .query_row(
            "SELECT id, started_at, ended_at FROM sessions WHERE project_id = ?1 ORDER BY started_at DESC LIMIT 1",
            rusqlite::params![normalized_project_id],
            |row| {
                Ok(SessionInfo {
                    id: row.get(0)?,
                    started_at: row.get(1).ok(),
                    ended_at: row.get(2).ok(),
                })
            },
        )
        .optional()
        .map_err(|err| ProjectError::Db(err.to_string()))?;

    let current_story = if has_loop_handle {
        if let Some(session) = active_session.as_ref() {
            conn.query_row(
                "SELECT story_id FROM iterations WHERE session_id = ?1 ORDER BY started_at DESC LIMIT 1",
                rusqlite::params![session.id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| ProjectError::Db(err.to_string()))?
            .or_else(|| {
                detail
                    .stories
                    .iter()
                    .find(|story| !story.passes && !story.blocked)
                    .map(|story| story.id.clone())
            })
        } else {
            None
        }
    } else {
        None
    };

    let mut status = ProjectStatus::from_db_status(&detail.project.status);
    if has_loop_handle {
        status = ProjectStatus::Running;
    } else if matches!(status, ProjectStatus::Running) {
        status = ProjectStatus::Paused;
    }
    if matches!(status, ProjectStatus::Draft) && detail.total_stories > 0 && config.is_some() {
        status = ProjectStatus::Ready;
    }

    let artifact_root = storage::artifacts::project_artifact_dir(&app, &detail.project.id)
        .map_err(ProjectError::Path)?;
    let paths = storage::artifacts::file_paths(&artifact_root);
    let artifact_paths = ArtifactPaths {
        root: artifact_root.to_string_lossy().to_string(),
        draft: paths[0].to_string_lossy().to_string(),
        plan: paths[1].to_string_lossy().to_string(),
        prd: paths[2].to_string_lossy().to_string(),
        config: paths[3].to_string_lossy().to_string(),
        prompt: paths[4].to_string_lossy().to_string(),
        guardrails: paths[5].to_string_lossy().to_string(),
    };
    let progress_percent = if detail.total_stories == 0 {
        0
    } else {
        ((detail.passed_count as f64 / detail.total_stories as f64) * 100.0).round() as u32
    };
    let uptime_label = build_duration_label(
        active_session
            .as_ref()
            .and_then(|session| session.started_at.as_deref()),
        active_session
            .as_ref()
            .and_then(|session| session.ended_at.as_deref()),
    );

    Ok(ProjectSnapshot {
        project: detail.project,
        status,
        active_session,
        progress: ProgressInfo {
            stories_total: detail.total_stories,
            stories_done: detail.passed_count,
            current_story,
        },
        config: config.map(to_snapshot_config),
        artifact_paths,
        progress_percent,
        uptime_label,
    })
}
