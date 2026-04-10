use crate::activity::ActivityClassifier;
use crate::plan_engine::output::flush_event_buffer;
use crate::plan_engine::payloads::{
    PlanActivityPayload, PlanTerminalPayload, BATCH_FLUSH_INTERVAL_MS, CURSOR_INITIAL_GRACE_SECS,
    DEFAULT_STALL_THRESHOLD_SECS, HEARTBEAT_INTERVAL_SECS,
};
use crate::plan_engine::sessions::{PlanSessionStatus, PlanSessions};
use crate::plan_engine::trace::{SessionTracer, TraceEvent};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Runtime};
use tokio::task::JoinHandle;

pub(super) fn spawn_plan_flush_task(
    classifier: Arc<Mutex<ActivityClassifier>>,
    plan_path: PathBuf,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Ok(guard) = classifier.lock() {
                let content = guard.accumulated_plan();
                if !content.is_empty() {
                    let _ = std::fs::write(&plan_path, &content);
                }
            }
        }
    })
}

pub(super) fn spawn_batch_flush_task<R: Runtime>(
    buffer: Arc<Mutex<Vec<PlanActivityPayload>>>,
    app: AppHandle<R>,
    project_id: String,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(BATCH_FLUSH_INTERVAL_MS));
        interval.tick().await;
        loop {
            interval.tick().await;
            flush_event_buffer(&buffer, &app, &project_id);
        }
    })
}

pub(super) fn spawn_heartbeat_task<R: Runtime>(
    app: AppHandle<R>,
    last_activity: Arc<Mutex<Instant>>,
    sessions: Arc<Mutex<PlanSessions>>,
    project_id: String,
    agent_name: String,
    started_at: Instant,
    tracer: Option<SessionTracer>,
) -> JoinHandle<()> {
    let stall_threshold = Duration::from_secs(DEFAULT_STALL_THRESHOLD_SECS);
    let initial_grace = if agent_name == "cursor" {
        Duration::from_secs(CURSOR_INITIAL_GRACE_SECS)
    } else {
        Duration::ZERO
    };

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
        interval.tick().await;

        loop {
            interval.tick().await;
            let _ = app.emit(
                crate::events::EVENT_PLAN_HEARTBEAT,
                PlanTerminalPayload {
                    project_id: project_id.clone(),
                    detail: String::new(),
                },
            );

            let since_last = last_activity
                .lock()
                .map(|instant| instant.elapsed())
                .unwrap_or_default();

            let effective_threshold = if started_at.elapsed() < initial_grace {
                stall_threshold + initial_grace
            } else {
                stall_threshold
            };

            let triggered = since_last > effective_threshold;

            if let Some(ref tracer) = tracer {
                tracer.log(TraceEvent::HeartbeatCheck {
                    since_last_activity_ms: since_last.as_millis() as u64,
                    effective_threshold_ms: effective_threshold.as_millis() as u64,
                    triggered_stall: triggered,
                });
            }

            if triggered {
                if let Some(ref tracer) = tracer {
                    tracer.log(TraceEvent::StallDetected {
                        since_last_activity_ms: since_last.as_millis() as u64,
                    });
                }
                let _ = app.emit(
                    crate::events::EVENT_PLAN_ERROR,
                    PlanTerminalPayload {
                        project_id: project_id.clone(),
                        detail: "stalled".to_string(),
                    },
                );
                if let Ok(mut guard) = sessions.lock() {
                    if let Some(entry) = guard.sessions.get_mut(&project_id) {
                        entry.status = PlanSessionStatus::Stalled;
                    }
                }
            }
        }
    })
}

pub(super) fn handle_termination<R: Runtime>(
    event_buffer: &Arc<Mutex<Vec<PlanActivityPayload>>>,
    classifier: &Arc<Mutex<ActivityClassifier>>,
    plan_path: &PathBuf,
    app: &AppHandle<R>,
    project_id: &str,
    exit_code: i32,
    tracer: &Option<SessionTracer>,
) {
    flush_event_buffer(event_buffer, app, project_id);

    if let Ok(guard) = classifier.lock() {
        let plan_content = guard.accumulated_plan();
        if !plan_content.is_empty() {
            let _ = std::fs::write(plan_path, &plan_content);
        }
    }

    let has_plan = classifier
        .lock()
        .map(|guard| !guard.accumulated_plan().is_empty())
        .unwrap_or(false);

    if exit_code != 0 {
        if let Some(tracer) = tracer {
            tracer.log(TraceEvent::ErrorEmitted {
                detail: format!("exit_code={exit_code}"),
            });
        }
        let _ = app.emit(
            crate::events::EVENT_PLAN_ERROR,
            PlanTerminalPayload {
                project_id: project_id.to_string(),
                detail: format!("exit_code={exit_code}"),
            },
        );
    } else if !has_plan {
        if let Some(tracer) = tracer {
            tracer.log(TraceEvent::ErrorEmitted {
                detail: "empty_output".to_string(),
            });
        }
        let _ = app.emit(
            crate::events::EVENT_PLAN_ERROR,
            PlanTerminalPayload {
                project_id: project_id.to_string(),
                detail: "empty_output".to_string(),
            },
        );
    } else {
        let plan_bytes = classifier
            .lock()
            .map(|g| g.accumulated_plan().len())
            .unwrap_or(0);
        if let Some(tracer) = tracer {
            tracer.log(TraceEvent::PlanComplete { plan_bytes });
        }
        let _ = app.emit(
            crate::events::EVENT_PLAN_COMPLETE,
            PlanTerminalPayload {
                project_id: project_id.to_string(),
                detail: String::new(),
            },
        );
    }
}
