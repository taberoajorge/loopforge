use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Error)]
#[serde(rename_all = "camelCase")]
pub enum ServiceError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("invalid request: {0}")]
    Invalid(String),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct IterationSummary {
    #[serde(default)]
    pub story_id: String,
    #[serde(default)]
    pub started_at: String,
    #[serde(default)]
    pub duration_secs: i64,
    #[serde(default)]
    pub result: String,
    #[serde(default)]
    pub agent_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub total_iterations: i64,
    #[serde(default)]
    pub success_count: i64,
    #[serde(default)]
    pub failure_count: i64,
    #[serde(default)]
    pub rate_limited_count: i64,
    #[serde(default)]
    pub success_rate: f64,
    #[serde(default)]
    pub stories_per_hour: f64,
    #[serde(default)]
    pub total_stories: usize,
    #[serde(default)]
    pub passed_stories: usize,
    #[serde(default)]
    pub blocked_stories: usize,
    #[serde(default)]
    pub pending_stories: usize,
    #[serde(default)]
    pub current_agent: Option<String>,
    #[serde(default)]
    pub is_running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LatestSessionRecord {
    pub id: String,
    pub started_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IterationCounts {
    pub total: i64,
    pub success: i64,
    pub rate_limited: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StoryCounts {
    pub total: usize,
    pub passed: usize,
    pub blocked: usize,
    pub pending: usize,
}

pub trait SessionRepository {
    fn is_running(&self, project_id: &str) -> ServiceResult<bool>;
    fn latest_session_record(&self, project_id: &str)
        -> ServiceResult<Option<LatestSessionRecord>>;
    fn iteration_counts(&self, session_id: &str) -> ServiceResult<IterationCounts>;
    fn latest_agent(&self, session_id: &str) -> ServiceResult<Option<String>>;
    fn story_counts(&self, project_id: &str) -> ServiceResult<StoryCounts>;
    fn stories_per_hour(
        &self,
        session: Option<&LatestSessionRecord>,
        passed_stories: usize,
    ) -> ServiceResult<f64>;
    fn iteration_history(
        &self,
        project_id: &str,
        limit: usize,
    ) -> ServiceResult<Vec<IterationSummary>>;
}

pub trait SessionService {
    fn latest_session(&self, project_id: &str) -> ServiceResult<Option<SessionInfo>>;
    fn session_stats(&self, project_id: &str) -> ServiceResult<SessionStats>;
    fn iteration_history(
        &self,
        project_id: &str,
        limit: usize,
    ) -> ServiceResult<Vec<IterationSummary>>;
}

pub struct RuntimeSessionService<R> {
    repository: R,
}

impl<R> RuntimeSessionService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: SessionRepository> SessionService for RuntimeSessionService<R> {
    fn latest_session(&self, project_id: &str) -> ServiceResult<Option<SessionInfo>> {
        self.repository
            .latest_session_record(project_id)
            .map(|session| {
                session.map(|record| SessionInfo {
                    id: record.id,
                    started_at: record.started_at,
                    ended_at: None,
                })
            })
    }

    fn session_stats(&self, project_id: &str) -> ServiceResult<SessionStats> {
        let is_running = self.repository.is_running(project_id)?;
        let session = self.repository.latest_session_record(project_id)?;
        let counts = session
            .as_ref()
            .map(|record| self.repository.iteration_counts(&record.id))
            .transpose()?
            .unwrap_or_default();
        let current_agent = session
            .as_ref()
            .map(|record| self.repository.latest_agent(&record.id))
            .transpose()?
            .flatten();
        let stories = self.repository.story_counts(project_id)?;
        let failure_count = counts.total - counts.success - counts.rate_limited;
        let success_rate = if counts.total > 0 {
            counts.success as f64 / counts.total as f64
        } else {
            0.0
        };
        let stories_per_hour = self
            .repository
            .stories_per_hour(session.as_ref(), stories.passed)?;

        Ok(SessionStats {
            project_id: project_id.to_string(),
            session_id: self.latest_session(project_id)?.map(|info| info.id),
            total_iterations: counts.total,
            success_count: counts.success,
            failure_count,
            rate_limited_count: counts.rate_limited,
            success_rate,
            stories_per_hour,
            total_stories: stories.total,
            passed_stories: stories.passed,
            blocked_stories: stories.blocked,
            pending_stories: stories.pending,
            current_agent,
            is_running,
        })
    }

    fn iteration_history(
        &self,
        project_id: &str,
        limit: usize,
    ) -> ServiceResult<Vec<IterationSummary>> {
        self.repository.iteration_history(project_id, limit)
    }
}
