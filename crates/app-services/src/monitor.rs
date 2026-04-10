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

#[cfg(test)]
mod tests {
    use super::{MonitorEvent, MonitorMessage, MonitorStream, OutputEntry};

    #[test]
    fn serializes_monitor_event_payloads() {
        let event = MonitorEvent::Output {
            entry: OutputEntry {
                project_id: "project-1".into(),
                session_id: Some("session-1".into()),
                stream: MonitorStream::Stderr,
                content: "line".into(),
                emitted_at: Some("2026-04-10T00:00:00Z".into()),
            },
        };
        let message = serde_json::to_value(MonitorMessage {
            project_id: "project-1".into(),
            snapshot: None,
            event: Some(event),
        })
        .expect("monitor message serializes");

        assert_eq!(message["projectId"], "project-1");
        assert_eq!(message["event"]["type"], "output");
        assert_eq!(message["event"]["entry"]["stream"], "stderr");
    }
}
