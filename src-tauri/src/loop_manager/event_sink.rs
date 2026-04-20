use crate::projects::notification_filter::should_emit_verification_failed;
use crate::projects::notifications::{create_notification_and_emit, NotificationCreateInput};
use ralph_core::events::{LoopEvent, LoopEventSink};
use tauri::{AppHandle, Emitter, Runtime};

pub struct TauriEventSink<R: Runtime> {
    app: AppHandle<R>,
    project_id: String,
    session_id: String,
}

impl<R: Runtime> TauriEventSink<R> {
    pub fn new(app: AppHandle<R>, project_id: String, session_id: String) -> Self {
        Self {
            app,
            project_id,
            session_id,
        }
    }
}

impl<R: Runtime> LoopEventSink for TauriEventSink<R> {
    fn emit(&self, event: LoopEvent) {
        let (event_name, payload) = match &event {
            LoopEvent::Heartbeat {
                elapsed_secs,
                total_secs,
                context,
            } => (
                crate::events::EVENT_HEARTBEAT,
                serde_json::json!({
                    "projectId": self.project_id,
                    "sessionId": self.session_id,
                    "elapsedSecs": elapsed_secs,
                    "totalSecs": total_secs,
                    "context": context,
                }),
            ),
            LoopEvent::VerificationStarted {
                story_id,
                attempt,
                max_attempts,
            } => (
                crate::events::EVENT_VERIFICATION_STARTED,
                serde_json::json!({
                    "projectId": self.project_id,
                    "sessionId": self.session_id,
                    "storyId": story_id,
                    "attempt": attempt,
                    "maxAttempts": max_attempts,
                }),
            ),
            LoopEvent::VerificationFailed {
                story_id,
                attempt,
                error_count,
                circuit_breaker,
            } => (
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
            LoopEvent::PromptBuilt {
                story_id,
                size_bytes,
                hash,
                truncated,
            } => (
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
        match event {
            LoopEvent::VerificationFailed {
                story_id,
                attempt,
                error_count,
                circuit_breaker,
            } => {
                if should_emit_verification_failed(attempt, circuit_breaker) {
                    let title = if circuit_breaker {
                        "Circuit breaker triggered".to_string()
                    } else {
                        "Verification failed".to_string()
                    };
                    let message = if circuit_breaker {
                        format!("{story_id}: same error repeated")
                    } else {
                        format!("{story_id}: attempt {attempt} failed ({error_count} errors)")
                    };
                    let _ = create_notification_and_emit(
                        &self.app,
                        NotificationCreateInput {
                            project_id: self.project_id.clone(),
                            notification_type: "loop_error".to_string(),
                            title,
                            message,
                        },
                    );
                }
            }
            LoopEvent::StorySkipped { story_id, reason } => {
                let _ = create_notification_and_emit(
                    &self.app,
                    NotificationCreateInput {
                        project_id: self.project_id.clone(),
                        notification_type: "story_blocked".to_string(),
                        title: "Story skipped".to_string(),
                        message: format!("{story_id}: {reason}"),
                    },
                );
            }
            _ => {}
        }
    }
}
