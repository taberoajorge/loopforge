use app_services::session::{IterationCounts, IterationSummary, LatestSessionRecord, RuntimeSessionService, SessionRepository, SessionService, StoryCounts};
use app_services::{ServiceError, SessionStats};
use ralph_core::prd::Prd;
use rusqlite::{Connection, OptionalExtension};
use tauri::Manager;

use crate::commands::execution::IterationRow;
use crate::loop_manager::LoopError;

pub struct TauriSessionAdapter<'a, R: tauri::Runtime> {
    window: &'a tauri::Window<R>,
}

impl<'a, R: tauri::Runtime> TauriSessionAdapter<'a, R> {
    fn new(window: &'a tauri::Window<R>) -> Self {
        Self { window }
    }

    fn with_connection<T>(
        &self,
        run: impl FnOnce(&Connection) -> Result<T, ServiceError>,
    ) -> Result<T, ServiceError> {
        let db = self.window.state::<crate::db::DbState>();
        let conn =
            db.0.lock()
                .map_err(|_| ServiceError::Internal("database lock poisoned".into()))?;
        run(&conn)
    }
}

impl<R: tauri::Runtime> SessionRepository for TauriSessionAdapter<'_, R> {
    fn is_running(&self, project_id: &str) -> Result<bool, ServiceError> {
        let loop_state = self.window.state::<crate::loop_manager::LoopManagerState>();
        let handles = loop_state
            .0
            .lock()
            .map_err(|_| ServiceError::Internal("loop state lock poisoned".into()))?;
        Ok(handles.contains_key(project_id))
    }

    fn latest_session_record(&self, project_id: &str) -> Result<Option<LatestSessionRecord>, ServiceError> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT id, started_at FROM sessions WHERE project_id = ?1 ORDER BY started_at DESC LIMIT 1",
                rusqlite::params![project_id],
                |row| Ok(LatestSessionRecord { id: row.get(0)?, started_at: row.get(1)? }),
            )
            .optional()
            .map_err(|error| ServiceError::Internal(error.to_string()))
        })
    }

    fn iteration_counts(&self, session_id: &str) -> Result<IterationCounts, ServiceError> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT COUNT(*), SUM(CASE WHEN result = 'success' THEN 1 ELSE 0 END), SUM(CASE WHEN result = 'rate_limited' THEN 1 ELSE 0 END) FROM iterations WHERE session_id = ?1",
                rusqlite::params![session_id],
                |row| {
                    Ok(IterationCounts {
                        total: row.get(0)?,
                        success: row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                        rate_limited: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                    })
                },
            )
            .map_err(|error| ServiceError::Internal(error.to_string()))
        })
    }

    fn latest_agent(&self, session_id: &str) -> Result<Option<String>, ServiceError> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT agent_used FROM iterations WHERE session_id = ?1 ORDER BY started_at DESC LIMIT 1",
                rusqlite::params![session_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| ServiceError::Internal(error.to_string()))
        })
    }

    fn story_counts(&self, project_id: &str) -> Result<StoryCounts, ServiceError> {
        let artifact_dir =
            crate::storage::artifacts::project_artifact_dir(&self.window.app_handle(), project_id)
                .map_err(ServiceError::Internal)?;
        let prd_path = artifact_dir.join("prd.json");
        if !prd_path.exists() {
            return Ok(StoryCounts::default());
        }
        Prd::load(&prd_path)
            .map(|prd| StoryCounts {
                total: prd.total_stories(),
                passed: prd.passed_count(),
                blocked: prd.blocked_count(),
                pending: prd.pending_count(),
            })
            .map_err(|error| ServiceError::Internal(error.to_string()))
    }

    fn stories_per_hour(&self, session: Option<&LatestSessionRecord>, passed_stories: usize) -> Result<f64, ServiceError> {
        let Some(started_at) = session.and_then(|record| record.started_at.as_deref()) else {
            return Ok(0.0);
        };
        let Ok(started_at) = chrono::DateTime::parse_from_rfc3339(started_at) else {
            return Ok(0.0);
        };
        let elapsed_hours =
            (chrono::Utc::now() - started_at.with_timezone(&chrono::Utc)).num_minutes() as f64
                / 60.0;
        if elapsed_hours > 0.0 {
            Ok(passed_stories as f64 / elapsed_hours)
        } else {
            Ok(0.0)
        }
    }

    fn iteration_history(&self, project_id: &str, limit: usize) -> Result<Vec<IterationSummary>, ServiceError> {
        self.with_connection(|conn| {
            let mut statement = conn
                .prepare(
                "SELECT i.story_id, i.started_at, i.duration_secs, i.result, i.agent_used
                 FROM iterations i
                 JOIN sessions s ON i.session_id = s.id
                 WHERE s.project_id = ?1
                 ORDER BY i.started_at DESC
                 LIMIT ?2",
                )
                .map_err(|error| ServiceError::Internal(error.to_string()))?;
            let rows = statement
                .query_map(rusqlite::params![project_id, limit as i64], |row| {
                    Ok(IterationSummary {
                        story_id: row.get(0)?,
                        started_at: row.get(1)?,
                        duration_secs: row.get(2)?,
                        result: row.get(3)?,
                        agent_used: row.get(4)?,
                    })
                })
                .map_err(|error| ServiceError::Internal(error.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|error| ServiceError::Internal(error.to_string()))
        })
    }
}

#[tauri::command]
pub async fn session_stats_command<R: tauri::Runtime>(window: tauri::Window<R>, project_id: String) -> Result<SessionStats, LoopError> {
    RuntimeSessionService::new(TauriSessionAdapter::new(&window))
        .session_stats(&required(project_id, "project_id").map_err(LoopError::Path)?)
        .map_err(loop_error)
}

#[tauri::command]
pub async fn get_iteration_history_command<R: tauri::Runtime>(
    window: tauri::Window<R>,
    project_id: String,
) -> Result<Vec<IterationRow>, LoopError> {
    RuntimeSessionService::new(TauriSessionAdapter::new(&window))
        .iteration_history(&required(project_id, "project_id").map_err(LoopError::Path)?, 200)
        .map(|rows| {
            rows.into_iter()
                .map(|row| IterationRow {
                    story_id: row.story_id,
                    started_at: row.started_at,
                    duration_secs: row.duration_secs,
                    result: row.result,
                    agent_used: row.agent_used,
                })
                .collect()
        })
        .map_err(loop_error)
}

fn required(value: String, name: &str) -> Result<String, String> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Err(format!("{name} is required"));
    }
    Ok(trimmed)
}

fn loop_error(error: ServiceError) -> LoopError {
    match error {
        ServiceError::Invalid(message) => LoopError::Path(message),
        ServiceError::NotFound(message)
        | ServiceError::Conflict(message)
        | ServiceError::Internal(message) => LoopError::Internal(message),
    }
}
