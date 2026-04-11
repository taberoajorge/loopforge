use app_services::monitor::{MonitorEvent, MonitorRepository, MonitorService, RuntimeMonitorService};
use app_services::{MonitorSnapshot, MonitorStream, OutputEntry, ServiceError, SessionInfo};
use rusqlite::OptionalExtension;
use tauri::Manager;

pub struct TauriMonitorAdapter<'a, R: tauri::Runtime> {
    window: &'a tauri::Window<R>,
}

impl<'a, R: tauri::Runtime> TauriMonitorAdapter<'a, R> {
    pub fn new(window: &'a tauri::Window<R>) -> Self {
        Self { window }
    }

    pub fn snapshot(&self, project_id: &str) -> Result<MonitorSnapshot, ServiceError> {
        RuntimeMonitorService::new(Self::new(self.window)).snapshot(project_id)
    }
}

impl<R: tauri::Runtime> MonitorRepository for TauriMonitorAdapter<'_, R> {
    fn latest_session(&self, project_id: &str) -> Result<Option<SessionInfo>, ServiceError> {
        let db = self.window.state::<crate::db::DbState>();
        let conn = db
            .0
            .lock()
            .map_err(|_| ServiceError::Internal("database lock poisoned".into()))?;

        conn.query_row(
            "SELECT id, started_at, ended_at FROM sessions WHERE project_id = ?1 ORDER BY started_at DESC LIMIT 1",
            rusqlite::params![project_id],
            |row| {
                Ok(SessionInfo {
                    id: row.get(0)?,
                    started_at: row.get(1).ok(),
                    ended_at: row.get(2).ok(),
                })
            },
        )
        .optional()
        .map_err(|error| ServiceError::Internal(error.to_string()))
    }

    fn recent_output(&self, project_id: &str, limit: usize) -> Result<Vec<OutputEntry>, ServiceError> {
        let artifact_dir = crate::storage::artifacts::project_artifact_dir(
            &self.window.app_handle(),
            project_id,
        )
        .map_err(ServiceError::Internal)?;
        let output_path = artifact_dir.join("agent_output.log");
        let content = std::fs::read_to_string(output_path).unwrap_or_default();

        Ok(content
            .lines()
            .rev()
            .filter(|line| !line.trim().is_empty())
            .take(limit)
            .map(|line| OutputEntry {
                project_id: project_id.to_string(),
                session_id: None,
                stream: MonitorStream::Stdout,
                content: line.to_string(),
                emitted_at: None,
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect())
    }

    fn recent_events(
        &self,
        project_id: &str,
        limit: usize,
    ) -> Result<Vec<MonitorEvent>, ServiceError> {
        let db = self.window.state::<crate::db::DbState>();
        let conn = db
            .0
            .lock()
            .map_err(|_| ServiceError::Internal("database lock poisoned".into()))?;
        let mut statement = conn
            .prepare(
                "SELECT s.id, i.story_id, i.agent_used, i.duration_secs, i.result
                 FROM iterations i
                 JOIN sessions s ON i.session_id = s.id
                 WHERE s.project_id = ?1
                 ORDER BY i.started_at DESC
                 LIMIT ?2",
            )
            .map_err(|error| ServiceError::Internal(error.to_string()))?;
        let rows = statement
            .query_map(rusqlite::params![project_id, limit as i64], |row| {
                Ok(MonitorEvent::IterationCompleted {
                    project_id: project_id.to_string(),
                    session_id: row.get(0)?,
                    story_id: row.get(1)?,
                    agent: row.get(2)?,
                    duration_secs: row.get::<_, i64>(3)?.max(0) as u64,
                    result: row.get(4)?,
                })
            })
            .map_err(|error| ServiceError::Internal(error.to_string()))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| ServiceError::Internal(error.to_string()))
    }
}
