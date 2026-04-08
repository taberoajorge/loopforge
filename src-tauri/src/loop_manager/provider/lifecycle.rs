use super::ShellProvider;
use ralph_core::providers::{AgentResult, Provider};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tauri::Emitter;

impl Provider for ShellProvider {
    fn name(&self) -> &str {
        "shell"
    }

    fn model(&self) -> &str {
        "default"
    }

    async fn run_agent(
        &self,
        prompt: &str,
        story_id: &str,
        work_dir: &Path,
        stall_timeout_secs: u64,
        shutdown_flag: Arc<AtomicBool>,
        output_log: &Path,
    ) -> anyhow::Result<AgentResult> {
        let iter_count = {
            let mut counter = self
                .iteration_counter
                .lock()
                .unwrap_or_else(|err| err.into_inner());
            *counter += 1;
            *counter
        };

        let agent = self.current_agent();

        let _ = self.app.emit(
            crate::events::EVENT_ITERATION_STARTED,
            serde_json::json!({
                "projectId": self.project_id,
                "sessionId": self.session_id,
                "storyId": story_id,
                "agent": agent,
                "iteration": iter_count,
            }),
        );

        let started = std::time::Instant::now();
        let result = self
            .run_with_agent(
                &agent,
                prompt,
                work_dir,
                stall_timeout_secs,
                shutdown_flag.clone(),
                output_log,
            )
            .await?;
        let duration_secs = started.elapsed().as_secs();

        if result.rate_limited {
            let _ = self.app.emit(
                crate::events::EVENT_RATE_LIMIT_DETECTED,
                serde_json::json!({
                    "projectId": self.project_id,
                    "sessionId": self.session_id,
                    "agent": agent,
                    "retryAfter": result.retry_after_message,
                }),
            );

            match self.try_advance_fallback() {
                None => {
                    #[cfg(feature = "frozen")]
                    crate::notifications::notify_all_rate_limited(&self.app, &self.project_name);
                }
                Some(fallback) => {
                    let _ = self.app.emit(
                        crate::events::EVENT_AGENT_SWITCHED,
                        serde_json::json!({
                            "projectId": self.project_id,
                            "sessionId": self.session_id,
                            "fromAgent": agent,
                            "toAgent": fallback,
                        }),
                    );

                    let fallback_result: AgentResult = self
                        .run_with_agent(
                            &fallback,
                            prompt,
                            work_dir,
                            stall_timeout_secs,
                            shutdown_flag,
                            output_log,
                        )
                        .await?;

                    let fb_duration = started.elapsed().as_secs();
                    let fb_outcome = if fallback_result.success() {
                        "success"
                    } else {
                        "failed"
                    };
                    self.insert_iteration(story_id, &fallback, fb_duration, fb_outcome);

                    let _ = self.app.emit(
                        crate::events::EVENT_ITERATION_COMPLETED,
                        serde_json::json!({
                            "projectId": self.project_id,
                            "sessionId": self.session_id,
                            "storyId": story_id,
                            "agent": fallback,
                            "durationSecs": fb_duration,
                            "result": fb_outcome,
                        }),
                    );

                    return Ok(fallback_result);
                }
            }
        }

        let outcome = if result.success() {
            "success"
        } else if result.rate_limited {
            "rate_limited"
        } else {
            "failed"
        };

        self.insert_iteration(story_id, &agent, duration_secs, outcome);
        if outcome == "success" {
            #[cfg(feature = "frozen")]
            crate::notifications::notify_story_completed(&self.app, story_id, &self.project_name);
        }

        let _ = self.app.emit(
            crate::events::EVENT_ITERATION_COMPLETED,
            serde_json::json!({
                "projectId": self.project_id,
                "sessionId": self.session_id,
                "storyId": story_id,
                "agent": agent,
                "durationSecs": duration_secs,
                "result": outcome,
            }),
        );

        Ok(result)
    }
}
