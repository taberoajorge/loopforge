use crate::events::{EVENT_PLAN_ACTIVITY_BATCH, EVENT_PLAN_COMPLETE, EVENT_PLAN_ERROR};
use crate::plan_engine::helpers::artifact_dir;
use crate::plan_engine::payloads::PlanTerminalPayload;
use crate::plan_engine::sessions::{
    FixturePlanSession, PlanSessionEntry, PlanSessionHandle, PlanSessionStatus,
};
use crate::plan_engine::trace::{SessionTracer, TraceEvent};
use crate::plan_engine::{PlanEngineError, PlanSessionsState, StartPlanArgs};
use crate::test_support::planning::{fixture_plan_run, FixturePlanTerminal};
use crate::test_support::runtime::{resolve_test_mode, TestRuntime};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Runtime};

pub(super) fn resolve_test_runtime() -> Result<Option<TestRuntime>, PlanEngineError> {
    resolve_test_mode()
        .map_err(|err| PlanEngineError::Config(err.to_string()))
        .map(|mode| mode.runtime().cloned())
}

pub(super) async fn start_fixture_plan<R: Runtime>(
    app: AppHandle<R>,
    state: PlanSessionsState,
    args: StartPlanArgs,
    runtime: TestRuntime,
) -> Result<(), PlanEngineError> {
    let artifacts = artifact_dir(&app, &args.project_id)?;
    std::fs::create_dir_all(&artifacts)?;

    let tracer = SessionTracer::new(&artifacts, &args.project_id);
    if let Some(ref tracer) = tracer {
        tracer.log(TraceEvent::SessionStart {
            agent: args.agent.clone(),
            binary: format!("fixture:{}", runtime.fixture_set().as_str()),
            args: Vec::new(),
            use_null_stdin: false,
            stall_threshold_secs: crate::plan_engine::payloads::DEFAULT_STALL_THRESHOLD_SECS,
        });
    }

    let now = Instant::now();
    let last_activity = Arc::new(Mutex::new(now));
    let sessions = state.clone();
    let project_id = args.project_id.clone();
    let agent_name = args.agent.clone();
    let plan_path = artifacts.join("plan.md");
    let run = fixture_plan_run(&runtime, &project_id);
    let app_handle = app.clone();
    let tracer_for_task = tracer.clone();
    let last_activity_for_task = Arc::clone(&last_activity);

    let task = tokio::spawn(async move {
        for batch in run.batches {
            tokio::time::sleep(Duration::from_millis(batch.delay_ms)).await;
            if let Ok(mut activity) = last_activity_for_task.lock() {
                *activity = Instant::now();
            }
            let _ = app_handle.emit(EVENT_PLAN_ACTIVITY_BATCH, batch.payload);
        }

        match run.terminal {
            FixturePlanTerminal::Complete { plan_markdown } => {
                let _ = std::fs::write(&plan_path, &plan_markdown);
                if let Some(ref tracer) = tracer_for_task {
                    tracer.log(TraceEvent::PlanComplete {
                        plan_bytes: plan_markdown.len(),
                    });
                }
                let _ = app_handle.emit(
                    EVENT_PLAN_COMPLETE,
                    PlanTerminalPayload {
                        project_id: project_id.clone(),
                        detail: String::new(),
                    },
                );
            }
            FixturePlanTerminal::Error { detail } => {
                if let Some(ref tracer) = tracer_for_task {
                    tracer.log(TraceEvent::ErrorEmitted {
                        detail: detail.clone(),
                    });
                }
                let _ = app_handle.emit(
                    EVENT_PLAN_ERROR,
                    PlanTerminalPayload {
                        project_id: project_id.clone(),
                        detail,
                    },
                );
            }
        }

        if let Ok(mut guard) = sessions.0.lock() {
            guard.sessions.remove(&project_id);
        }
    });

    let mut guard = state.0.lock().map_err(|_| PlanEngineError::LockPoisoned)?;
    guard.sessions.insert(
        args.project_id,
        PlanSessionEntry {
            handle: PlanSessionHandle::Fixture(FixturePlanSession::new(task.abort_handle())),
            status: PlanSessionStatus::Running,
            agent_name,
            started_at: now,
            last_activity_at: last_activity,
        },
    );

    Ok(())
}
