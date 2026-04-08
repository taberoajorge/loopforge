use super::helpers::artifact_dir;
use super::{LoopError, LoopManagerState};
use crate::db::DbState;
use ralph_core::prd::Prd;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    pub project_id: String,
    pub session_id: Option<String>,
    pub total_iterations: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub rate_limited_count: i64,
    pub success_rate: f64,
    pub stories_per_hour: f64,
    pub total_stories: usize,
    pub passed_stories: usize,
    pub blocked_stories: usize,
    pub pending_stories: usize,
    pub current_agent: Option<String>,
    pub is_running: bool,
}

pub async fn session_stats(
    app: AppHandle,
    db: State<'_, DbState>,
    loop_state: State<'_, LoopManagerState>,
    project_id: String,
) -> Result<SessionStats, LoopError> {
    let is_running = {
        let handles = loop_state.0.lock().map_err(|_| LoopError::LockPoisoned)?;
        handles.contains_key(&project_id)
    };

    let conn = db.0.lock().map_err(|_| LoopError::LockPoisoned)?;
    let session_row: Option<(String, Option<String>)> = conn
        .query_row(
            "SELECT id, started_at FROM sessions WHERE project_id = ?1 ORDER BY started_at DESC LIMIT 1",
            rusqlite::params![project_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .optional()?;

    let (session_id, session_started_at) = match session_row {
        Some((sid, started)) => (Some(sid), started),
        None => (None, None),
    };

    let (total_iterations, success_count, failure_count, rate_limited_count) =
        if let Some(ref sid) = session_id {
            let total: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM iterations WHERE session_id = ?1",
                    rusqlite::params![sid],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            let success: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM iterations WHERE session_id = ?1 AND result = 'success'",
                    rusqlite::params![sid],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            let rate_limited: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM iterations WHERE session_id = ?1 AND result = 'rate_limited'",
                    rusqlite::params![sid],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            (total, success, total - success - rate_limited, rate_limited)
        } else {
            (0, 0, 0, 0)
        };

    let current_agent: Option<String> = if let Some(ref sid) = session_id {
        conn.query_row(
            "SELECT agent_used FROM iterations WHERE session_id = ?1 ORDER BY started_at DESC LIMIT 1",
            rusqlite::params![sid],
            |row| row.get(0),
        )
        .optional()
        .unwrap_or(None)
    } else {
        None
    };
    drop(conn);

    let artifacts = artifact_dir(&app, &project_id)?;
    let prd_path = artifacts.join("prd.json");
    let (total_stories, passed_stories, blocked_stories, pending_stories) = if prd_path.exists() {
        Prd::load(&prd_path)
            .map(|prd| {
                let total = prd.total_stories();
                let passed = prd.passed_count();
                let blocked = prd.blocked_count();
                (total, passed, blocked, prd.pending_count())
            })
            .unwrap_or((0, 0, 0, 0))
    } else {
        (0, 0, 0, 0)
    };

    let success_rate = if total_iterations > 0 {
        success_count as f64 / total_iterations as f64
    } else {
        0.0
    };

    let stories_per_hour = session_started_at
        .as_deref()
        .and_then(|started| chrono::DateTime::parse_from_rfc3339(started).ok())
        .map(|started| {
            let elapsed_hours =
                (chrono::Utc::now() - started.with_timezone(&chrono::Utc)).num_minutes() as f64
                    / 60.0;
            if elapsed_hours > 0.0 {
                passed_stories as f64 / elapsed_hours
            } else {
                0.0
            }
        })
        .unwrap_or(0.0);

    Ok(SessionStats {
        project_id,
        session_id,
        total_iterations,
        success_count,
        failure_count,
        rate_limited_count,
        success_rate,
        stories_per_hour,
        total_stories,
        passed_stories,
        blocked_stories,
        pending_stories,
        current_agent,
        is_running,
    })
}
