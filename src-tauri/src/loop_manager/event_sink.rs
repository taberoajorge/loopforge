use ralph_core::events::{LoopEvent, LoopEventSink};
use tauri::{AppHandle, Emitter};

pub struct TauriEventSink {
    app: AppHandle,
    project_id: String,
    session_id: String,
}

impl TauriEventSink {
    pub fn new(app: AppHandle, project_id: String, session_id: String) -> Self {
        Self {
            app,
            project_id,
            session_id,
        }
    }
}

impl LoopEventSink for TauriEventSink {
    fn emit(&self, event: LoopEvent) {
        let (event_name, payload) = match &event {
            LoopEvent::Heartbeat { elapsed_secs, total_secs, context } => (
                crate::events::EVENT_HEARTBEAT,
                serde_json::json!({
                    "projectId": self.project_id,
                    "sessionId": self.session_id,
                    "elapsedSecs": elapsed_secs,
                    "totalSecs": total_secs,
                    "context": context,
                }),
            ),
            LoopEvent::VerificationStarted { story_id, attempt, max_attempts } => (
                crate::events::EVENT_VERIFICATION_STARTED,
                serde_json::json!({
                    "projectId": self.project_id,
                    "sessionId": self.session_id,
                    "storyId": story_id,
                    "attempt": attempt,
                    "maxAttempts": max_attempts,
                }),
            ),
            LoopEvent::VerificationFailed { story_id, attempt, error_count, circuit_breaker } => (
                crate::events::EVENT_VERIFICATION_FAILED,
                serde_json::json!({
                    "projectId": self.project_id,
                    "sessionId": self.session_id,
                    "storyId": story_id,
                    "attempt": attempt,
                    "errorCount": error_count,
                    "circuitBreaker": circuit_breaker,
                }),
            ),
            LoopEvent::VerificationPassed { story_id, attempt } => (
                crate::events::EVENT_VERIFICATION_PASSED,
                serde_json::json!({
                    "projectId": self.project_id,
                    "sessionId": self.session_id,
                    "storyId": story_id,
                    "attempt": attempt,
                }),
            ),
            LoopEvent::PromptBuilt { story_id, size_bytes, hash, truncated } => (
                crate::events::EVENT_PROMPT_BUILT,
                serde_json::json!({
                    "projectId": self.project_id,
                    "sessionId": self.session_id,
                    "storyId": story_id,
                    "sizeBytes": size_bytes,
                    "hash": hash,
                    "truncated": truncated,
                }),
            ),
            LoopEvent::StorySkipped { story_id, reason } => (
                crate::events::EVENT_STORY_SKIPPED,
                serde_json::json!({
                    "projectId": self.project_id,
                    "sessionId": self.session_id,
                    "storyId": story_id,
                    "reason": reason,
                }),
            ),
            LoopEvent::HealthCheckWaiting { service } => (
                crate::events::EVENT_HEARTBEAT,
                serde_json::json!({
                    "projectId": self.project_id,
                    "sessionId": self.session_id,
                    "context": "health_check_wait",
                    "service": service,
                }),
            ),
        };

        let _ = self.app.emit(event_name, payload);
    }
}
