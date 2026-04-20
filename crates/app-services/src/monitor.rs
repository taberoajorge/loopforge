use crate::session::SessionInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct OutputEntry {
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub stream: MonitorStream,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub emitted_at: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MonitorStream {
    #[default]
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MonitorSnapshot {
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub session: Option<SessionInfo>,
    #[serde(default)]
    pub recent_output: Vec<OutputEntry>,
    #[serde(default)]
    pub events: Vec<MonitorEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MonitorEvent {
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
    SessionEnded {
        project_id: String,
        session_id: String,
        outcome: String,
    },
    Output {
        entry: OutputEntry,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MonitorMessage {
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub snapshot: Option<MonitorSnapshot>,
    #[serde(default)]
    pub event: Option<MonitorEvent>,
}

pub trait MonitorRepository {
    fn latest_session(
        &self,
        project_id: &str,
    ) -> crate::session::ServiceResult<Option<SessionInfo>>;
    fn recent_output(
        &self,
        project_id: &str,
        limit: usize,
    ) -> crate::session::ServiceResult<Vec<OutputEntry>>;
    fn recent_events(
        &self,
        project_id: &str,
        limit: usize,
    ) -> crate::session::ServiceResult<Vec<MonitorEvent>>;
}

pub trait MonitorService {
    fn snapshot(&self, project_id: &str) -> crate::session::ServiceResult<MonitorSnapshot>;
    fn recent_output(
        &self,
        project_id: &str,
        limit: usize,
    ) -> crate::session::ServiceResult<Vec<OutputEntry>>;
    fn recent_events(
        &self,
        project_id: &str,
        limit: usize,
    ) -> crate::session::ServiceResult<Vec<MonitorEvent>>;
}

pub struct RuntimeMonitorService<R> {
    repository: R,
}

impl<R> RuntimeMonitorService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: MonitorRepository> MonitorService for RuntimeMonitorService<R> {
    fn snapshot(&self, project_id: &str) -> crate::session::ServiceResult<MonitorSnapshot> {
        Ok(MonitorSnapshot {
            project_id: project_id.to_string(),
            session: self.repository.latest_session(project_id)?,
            recent_output: self.repository.recent_output(project_id, 200)?,
            events: self.repository.recent_events(project_id, 100)?,
        })
    }

    fn recent_output(
        &self,
        project_id: &str,
        limit: usize,
    ) -> crate::session::ServiceResult<Vec<OutputEntry>> {
        self.repository.recent_output(project_id, limit)
    }

    fn recent_events(
        &self,
        project_id: &str,
        limit: usize,
    ) -> crate::session::ServiceResult<Vec<MonitorEvent>> {
        self.repository.recent_events(project_id, limit)
    }
}
