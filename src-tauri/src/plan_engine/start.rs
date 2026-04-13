use crate::activity::ActivityClassifier;
use crate::app_core_plan::{self, PlanCleanupReason};
use crate::plan_engine::args::{agent_env_vars, build_plan_args, needs_null_stdin};
use crate::plan_engine::fixture;
use crate::plan_engine::helpers::{artifact_dir, build_plan_prompt, resolve_agent_binary};
use crate::plan_engine::payloads::{PlanActivityPayload, PlanTerminalPayload};
use crate::plan_engine::sessions::{PlanSessionEntry, PlanSessionHandle, PlanSessionStatus};
use crate::plan_engine::trace::{SessionTracer, TraceEvent};
use crate::plan_engine::{monitor, output, PlanEngineError, PlanSessionsState, StartPlanArgs};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter, Runtime, State};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
use tokio::task::AbortHandle;

fn flush_partial_plan<R: Runtime>(
    buffer: &Arc<Mutex<Vec<PlanActivityPayload>>>,
    classifier: &Arc<Mutex<ActivityClassifier>>,
    plan_path: &PathBuf,
    app: &AppHandle<R>,
    project_id: &str,
) -> (usize, String) {
    output::flush_event_buffer(buffer, classifier, app, project_id);
    let plan_content = classifier
        .lock()
        .map(|guard| guard.accumulated_plan())
        .unwrap_or_default();
    if plan_content.is_empty() {
        return (0, String::new());
    }
    let plan_bytes = plan_content.len();
    let _ = std::fs::write(plan_path, &plan_content);
    (plan_bytes, plan_content)
}

fn emit_terminal<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
    exit_code: i32,
    plan_bytes: usize,
    plan_content: Option<String>,
    tracer: &Option<SessionTracer>,
) {
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
                final_content: None,
            },
        );
        return;
    }
    if plan_bytes == 0 {
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
                final_content: None,
            },
        );
        return;
    }
    if let Some(tracer) = tracer {
        tracer.log(TraceEvent::PlanComplete { plan_bytes });
    }
    let _ = app.emit(
        crate::events::EVENT_PLAN_COMPLETE,
        PlanTerminalPayload {
            project_id: project_id.to_string(),
            detail: String::new(),
            final_content: plan_content,
        },
    );
}

fn cleanup_hook<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
    buffer: Arc<Mutex<Vec<PlanActivityPayload>>>,
    classifier: Arc<Mutex<ActivityClassifier>>,
    plan_path: PathBuf,
    tracer: Option<SessionTracer>,
    plan_abort: AbortHandle,
    batch_abort: AbortHandle,
    heartbeat_abort: AbortHandle,
) -> Arc<dyn Fn(PlanCleanupReason) + Send + Sync> {
    Arc::new(move |reason| {
        batch_abort.abort();
        plan_abort.abort();
        heartbeat_abort.abort();
        let (plan_bytes, plan_content) =
            flush_partial_plan(&buffer, &classifier, &plan_path, &app, &project_id);
        if let PlanCleanupReason::ProcessExit { exit_code } = reason {
            if let Some(ref tracer) = tracer {
                tracer.log(TraceEvent::ProcessTerminated {
                    exit_code,
                    has_plan_content: plan_bytes != 0,
                });
            }
            let content_opt = if plan_content.is_empty() {
                None
            } else {
                Some(plan_content)
            };
            emit_terminal(
                &app,
                &project_id,
                exit_code,
                plan_bytes,
                content_opt,
                &tracer,
            );
        }
    })
}

fn record_output(
    stream: &'static str,
    bytes: &[u8],
    tracer: &Option<SessionTracer>,
    classifier: &Arc<Mutex<ActivityClassifier>>,
    event_buffer: &Arc<Mutex<Vec<PlanActivityPayload>>>,
    project_id: &str,
    last_activity: &Arc<Mutex<Instant>>,
) {
    let text = String::from_utf8_lossy(bytes);
    if let Some(ref tracer) = tracer {
        let lines: Vec<&str> = text.lines().collect();
        tracer.log(TraceEvent::Output {
            stream,
            byte_len: bytes.len(),
            line_count: lines.len(),
            first_line_preview: lines
                .first()
                .map(|line| line.chars().take(120).collect())
                .unwrap_or_default(),
        });
    }
    output::buffer_text_segments(&text, classifier, event_buffer, project_id, last_activity);
}

pub async fn start_plan<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, PlanSessionsState>,
    args: StartPlanArgs,
) -> Result<(), PlanEngineError> {
    if !args.project_dir.exists() {
        return Err(PlanEngineError::Path(format!(
            "Working directory not found: {}",
            args.project_dir.display()
        )));
    }
    if !args.project_dir.is_dir() {
        return Err(PlanEngineError::Path(format!(
            "Working directory is not a directory: {}",
            args.project_dir.display()
        )));
    }
    {
        let sessions = state.0.lock().map_err(|_| PlanEngineError::LockPoisoned)?;
        if sessions.sessions.contains_key(&args.project_id) {
            return Err(PlanEngineError::AlreadyRunning(args.project_id));
        }
    }
    if let Some(runtime) = fixture::resolve_test_runtime()? {
        return fixture::start_fixture_plan(app, state.inner().clone(), args, runtime).await;
    }

    let agent_binary = resolve_agent_binary(&app, &args.agent).await?;
    let plan_prompt = build_plan_prompt(&args.initial_prompt);
    let agent_args = build_plan_args(
        &args.agent,
        &plan_prompt,
        args.project_dir.as_path(),
        args.model.as_deref(),
        args.effort.as_deref(),
    );
    let env_vars = agent_env_vars(&args.agent);
    let artifacts = artifact_dir(&app, &args.project_id)?;
    std::fs::create_dir_all(&artifacts)?;
    let use_null_stdin = needs_null_stdin(&args.agent);
    let tracer = SessionTracer::new(&artifacts, &args.project_id);
    if let Some(ref tracer) = tracer {
        tracer.log(TraceEvent::SessionStart {
            agent: args.agent.clone(),
            binary: agent_binary.clone(),
            args: agent_args.clone(),
            use_null_stdin,
            stall_threshold_secs: crate::plan_engine::payloads::DEFAULT_STALL_THRESHOLD_SECS,
        });
    }
    let (mut event_rx, child) = if use_null_stdin {
        let (shell_program, shell_args) =
            crate::shell_resolve::build_null_stdin_command(&agent_binary, &agent_args);
        app.shell()
            .command(&shell_program)
            .args(shell_args)
            .envs(env_vars)
            .current_dir(&args.project_dir)
            .spawn()
            .map_err(|err| PlanEngineError::Shell(err.to_string()))?
    } else {
        app.shell()
            .command(&agent_binary)
            .args(agent_args)
            .envs(env_vars)
            .current_dir(&args.project_dir)
            .spawn()
            .map_err(|err| PlanEngineError::Shell(err.to_string()))?
    };

    let now = Instant::now();
    let last_activity = Arc::new(Mutex::new(now));
    let classifier = Arc::new(Mutex::new(ActivityClassifier::new(&args.agent)));
    let event_buffer = Arc::new(Mutex::new(Vec::new()));
    let plan_path = artifacts.join("plan.md");
    let plan_abort =
        monitor::spawn_plan_flush_task(Arc::clone(&classifier), plan_path.clone()).abort_handle();
    let batch_abort = monitor::spawn_batch_flush_task(
        Arc::clone(&event_buffer),
        Arc::clone(&classifier),
        app.clone(),
        args.project_id.clone(),
    )
    .abort_handle();
    let heartbeat_abort = monitor::spawn_heartbeat_task(
        app.clone(),
        Arc::clone(&last_activity),
        Arc::clone(&state.0),
        args.project_id.clone(),
        args.agent.clone(),
        now,
        tracer.clone(),
    )
    .abort_handle();
    app_core_plan::register_cleanup(
        &args.project_id,
        cleanup_hook(
            app.clone(),
            args.project_id.clone(),
            Arc::clone(&event_buffer),
            Arc::clone(&classifier),
            plan_path,
            tracer.clone(),
            plan_abort,
            batch_abort,
            heartbeat_abort,
        ),
    );

    state
        .0
        .lock()
        .map_err(|_| PlanEngineError::LockPoisoned)?
        .sessions
        .insert(
            args.project_id.clone(),
            PlanSessionEntry {
                handle: PlanSessionHandle::Shell(child),
                status: PlanSessionStatus::Running,
                agent_name: args.agent.clone(),
                started_at: now,
                last_activity_at: Arc::clone(&last_activity),
            },
        );

    let project_id = args.project_id.clone();
    let sessions = Arc::clone(&state.0);
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                CommandEvent::Stdout(ref bytes) => record_output(
                    "stdout",
                    bytes,
                    &tracer,
                    &classifier,
                    &event_buffer,
                    &project_id,
                    &last_activity,
                ),
                CommandEvent::Stderr(ref bytes) => record_output(
                    "stderr",
                    bytes,
                    &tracer,
                    &classifier,
                    &event_buffer,
                    &project_id,
                    &last_activity,
                ),
                CommandEvent::Terminated(payload) => {
                    let _: Result<bool, PlanEngineError> = app_core_plan::cleanup_session(
                        &project_id,
                        PlanCleanupReason::ProcessExit {
                            exit_code: payload.code.unwrap_or(1),
                        },
                        |project_id| {
                            let mut guard =
                                sessions.lock().map_err(|_| PlanEngineError::LockPoisoned)?;
                            Ok(guard.sessions.remove(project_id))
                        },
                        |_| Ok(()),
                    );
                    break;
                }
                _ => {}
            }
        }
    });
    Ok(())
}
