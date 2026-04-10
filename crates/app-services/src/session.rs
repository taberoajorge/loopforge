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

pub trait SessionService {
    fn latest_session(&self, project_id: &str) -> ServiceResult<Option<SessionInfo>>;
    fn session_stats(&self, project_id: &str) -> ServiceResult<SessionStats>;
    fn iteration_history(
        &self,
        project_id: &str,
        limit: usize,
    ) -> ServiceResult<Vec<IterationSummary>>;
}

#[cfg(test)]
mod tests {
    use super::{IterationSummary, SessionInfo, SessionStats};

    #[test]
    fn serializes_session_dtos_with_camel_case() {
        let payload = serde_json::to_value(SessionStats {
            project_id: "project-1".into(),
            session_id: Some("session-1".into()),
            total_iterations: 4,
            success_count: 3,
            failure_count: 1,
            rate_limited_count: 0,
            success_rate: 0.75,
            stories_per_hour: 1.5,
            total_stories: 5,
            passed_stories: 3,
            blocked_stories: 0,
            pending_stories: 2,
            current_agent: Some("codex".into()),
            is_running: true,
        })
        .expect("session stats serialize");

        assert_eq!(payload["projectId"], "project-1");
        assert_eq!(payload["sessionId"], "session-1");
        assert_eq!(payload["successRate"], 0.75);
    }

    #[test]
    fn defaults_allow_backward_compatible_decoding() {
        let decoded: SessionInfo =
            serde_json::from_str(r#"{"id":"session-1"}"#).expect("session info decodes");
        let iteration: IterationSummary = serde_json::from_str(
            r#"{"storyId":"S-1","startedAt":"2026-04-10T00:00:00Z","durationSecs":3,"result":"success","agentUsed":"codex"}"#,
        )
        .expect("iteration summary decodes");

        assert_eq!(decoded.id, "session-1");
        assert_eq!(iteration.story_id, "S-1");
    }
}
