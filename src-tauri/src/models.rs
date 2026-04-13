use crate::projects::Project;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Draft,
    Ready,
    Running,
    Paused,
    Blocked,
    Failed,
    Completed,
    Archived,
}

impl ProjectStatus {
    pub fn from_db_status(status: &str) -> Self {
        match status {
            "active" => Self::Running,
            "running" => Self::Running,
            "paused" => Self::Paused,
            "blocked" => Self::Blocked,
            "failed" => Self::Failed,
            "completed" => Self::Completed,
            "archived" => Self::Archived,
            "ready" => Self::Ready,
            _ => Self::Draft,
        }
    }

    pub fn resolve_canonical(
        db_status: &str,
        has_prd: bool,
        has_config: bool,
        has_active_session: bool,
    ) -> Self {
        let base_status = Self::from_db_status(db_status);

        if has_active_session {
            return Self::Running;
        }

        if matches!(base_status, Self::Running | Self::Paused) {
            return Self::Paused;
        }

        if matches!(
            base_status,
            Self::Blocked | Self::Failed | Self::Completed | Self::Archived
        ) {
            return base_status;
        }

        if !has_prd {
            return Self::Draft;
        }

        if has_config {
            Self::Ready
        } else {
            Self::Draft
        }
    }

    pub fn as_project_status(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Ready => "ready",
            Self::Running => "active",
            Self::Paused => "paused",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
            Self::Completed => "completed",
            Self::Archived => "archived",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactPaths {
    #[serde(default)]
    pub root: String,
    #[serde(default)]
    pub draft: String,
    #[serde(default)]
    pub plan: String,
    #[serde(default)]
    pub prd: String,
    #[serde(default)]
    pub config: String,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub guardrails: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub execute_agent: String,
    #[serde(default)]
    pub execute_model: Option<String>,
    #[serde(default)]
    pub execute_effort: Option<String>,
    #[serde(default)]
    pub fallback_chain: Vec<String>,
    #[serde(default)]
    pub gutter_threshold: u32,
    #[serde(default)]
    pub max_iterations: u32,
    #[serde(default)]
    pub cooldown_seconds: u32,
    #[serde(default)]
    pub test_command: String,
    #[serde(default)]
    pub max_verification_retries: u32,
    #[serde(default)]
    pub scm_provider: String,
    #[serde(default)]
    pub review_polling_interval: u64,
    #[serde(default)]
    pub review_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProgressInfo {
    #[serde(default)]
    pub stories_total: usize,
    #[serde(default)]
    pub stories_done: usize,
    #[serde(default)]
    pub current_story: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshot {
    pub project: Project,
    pub status: ProjectStatus,
    #[serde(default)]
    pub active_session: Option<SessionInfo>,
    #[serde(default)]
    pub progress: ProgressInfo,
    #[serde(default)]
    pub config: Option<ProjectConfig>,
    #[serde(default)]
    pub artifact_paths: ArtifactPaths,
    #[serde(default)]
    pub progress_percent: u32,
    #[serde(default)]
    pub uptime_label: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LoopEvent {
    SessionStarted {
        project_id: String,
        session_id: String,
        agent: Option<String>,
    },
    IterationStarted {
        project_id: String,
        session_id: String,
        story_id: String,
        agent: String,
        iteration: u32,
    },
    IterationCompleted {
        project_id: String,
        session_id: String,
        story_id: String,
        agent: String,
        duration_secs: u64,
        result: String,
    },
    RateLimitDetected {
        project_id: String,
        session_id: String,
        agent: String,
        retry_after: Option<String>,
    },
    AgentSwitched {
        project_id: String,
        session_id: String,
        from_agent: String,
        to_agent: String,
    },
    StoryBlocked {
        project_id: String,
        story_id: String,
        reason: Option<String>,
    },
    SessionEnded {
        project_id: String,
        session_id: String,
        outcome: String,
    },
}

#[cfg(test)]
mod tests {
    use super::ProjectStatus;

    #[test]
    fn canonical_project_status_resolution() {
        assert_eq!(
            ProjectStatus::resolve_canonical("ready", false, true, false),
            ProjectStatus::Draft
        );
        assert_eq!(
            ProjectStatus::resolve_canonical("paused", true, true, false),
            ProjectStatus::Paused
        );
        assert_eq!(
            ProjectStatus::resolve_canonical("draft", true, true, false),
            ProjectStatus::Ready
        );
        assert_eq!(
            ProjectStatus::resolve_canonical("active", true, true, false),
            ProjectStatus::Paused
        );
        assert_eq!(
            ProjectStatus::resolve_canonical("paused", true, false, true),
            ProjectStatus::Running
        );
    }
}
