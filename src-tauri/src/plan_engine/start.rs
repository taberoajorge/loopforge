use crate::activity::{ActivityClassifier, PlanEventKind};
use crate::plan_engine::args::{
    agent_env_vars, build_null_stdin_command, build_plan_args, needs_null_stdin,
};
use crate::plan_engine::helpers::{artifact_dir, build_plan_prompt, resolve_agent_binary};
use crate::plan_engine::sessions::{PlanSessionEntry, PlanSessionStatus};
use crate::plan_engine::{PlanEngineError, PlanSessionsState, StartPlanArgs};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

const HEARTBEAT_INTERVAL_SECS: u64 = 15;
const DEFAULT_STALL_THRESHOLD_SECS: u64 = 180;
const CURSOR_INITIAL_GRACE_SECS: u64 = 20;
const BATCH_FLUSH_INTERVAL_MS: u64 = 150;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanActivityPayload {
    pub project_id: String,
    pub kind: PlanEventKind,
    pub content: String,
    pub timestamp: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanActivityBatchPayload {
    pub project_id: String,
    pub events: Vec<PlanActivityPayload>,
    pub plan_content_delta: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanTerminalPayload {
    pub project_id: String,
    pub detail: String,
}

fn buffer_text_segments(
    text: &str,
    classifier: &Arc<Mutex<ActivityClassifier>>,
    buffer: &Arc<Mutex<Vec<PlanActivityPayload>>>,
    project_id: &str,
    last_activity: &Arc<Mutex<Instant>>,
) {
    for segment in text.lines() {
        let trimmed = segment.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(mut activity) = last_activity.lock() {
            *activity = Instant::now();
        }
        if let Ok(mut guard) = classifier.lock() {
            let plan_event = guard.classify(trimmed);
            if let Ok(mut buf) = buffer.lock() {
                buf.push(PlanActivityPayload {
                    project_id: project_id.to_string(),
                    kind: plan_event.kind,
                    content: plan_event.content,
                    timestamp: plan_event.timestamp,
                });
            }
        }
    }
}

fn flush_event_buffer(
    buffer: &Arc<Mutex<Vec<PlanActivityPayload>>>,
    app: &AppHandle,
    project_id: &str,
) {
    let events: Vec<PlanActivityPayload> = {
        let mut buf = match buffer.lock() {
            Ok(buf) => buf,
            Err(_) => return,
        };
        buf.drain(..).collect()
    };
    if events.is_empty() {
        return;
    }
    let plan_content_delta: String = events
        .iter()
        .filter(|evt| evt.kind == PlanEventKind::PlanContent)
        .map(|evt| evt.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let _ = app.emit(
        crate::events::EVENT_PLAN_ACTIVITY_BATCH,
        PlanActivityBatchPayload {
            project_id: project_id.to_string(),
            events,
            plan_content_delta,
        },
    );
}

pub async fn start_plan(
    app: AppHandle,
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

    let agent_binary = resolve_agent_binary(&app, &args.agent).await?;
    {
        let sessions = state.0.lock().map_err(|_| PlanEngineError::LockPoisoned)?;
        if sessions.sessions.contains_key(&args.project_id) {
            return Err(PlanEngineError::AlreadyRunning(args.project_id));
        }
    }

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

    let (mut event_rx, child) = if use_null_stdin {
        let wrapped = build_null_stdin_command(&agent_binary, &agent_args);
        app.shell()
            .command("/bin/zsh")
            .args(["-lc", &wrapped])
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

    {
        let mut sessions = state.0.lock().map_err(|_| PlanEngineError::LockPoisoned)?;
        sessions.sessions.insert(
            args.project_id.clone(),
            PlanSessionEntry {
                child,
                status: PlanSessionStatus::Running,
                agent_name: args.agent.clone(),
                started_at: now,
                last_activity_at: Arc::clone(&last_activity),
            },
        );
    }

    let project_id = args.project_id.clone();
    let artifact_path = artifacts.clone();
    let agent_name = args.agent.clone();
    let sessions_arc = Arc::clone(&state.0);
    let app_clone = app.clone();
    let last_activity_clone = Arc::clone(&last_activity);

    tokio::spawn(async move {
        let classifier = Arc::new(Mutex::new(ActivityClassifier::new(&agent_name)));
        let event_buffer: Arc<Mutex<Vec<PlanActivityPayload>>> =
            Arc::new(Mutex::new(Vec::new()));
        let plan_path = artifact_path.join("plan.md");

        let flush_classifier = Arc::clone(&classifier);
        let flush_path = plan_path.clone();
        let plan_flush_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            interval.tick().await;
            loop {
                interval.tick().await;
                if let Ok(guard) = flush_classifier.lock() {
                    let content = guard.accumulated_plan();
                    if !content.is_empty() {
                        let _ = std::fs::write(&flush_path, &content);
                    }
                }
            }
        });

        let batch_buffer = Arc::clone(&event_buffer);
        let batch_app = app_clone.clone();
        let batch_project = project_id.clone();
        let batch_flush_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(BATCH_FLUSH_INTERVAL_MS));
            interval.tick().await;
            loop {
                interval.tick().await;
                flush_event_buffer(&batch_buffer, &batch_app, &batch_project);
            }
        });

        let hb_app = app_clone.clone();
        let hb_activity = Arc::clone(&last_activity_clone);
        let hb_sessions = Arc::clone(&sessions_arc);
        let hb_project = project_id.clone();
        let hb_agent = agent_name.clone();
        let hb_start = now;
        let heartbeat_handle = tokio::spawn(async move {
            let stall_threshold = Duration::from_secs(DEFAULT_STALL_THRESHOLD_SECS);
            let initial_grace = if hb_agent == "cursor" {
                Duration::from_secs(CURSOR_INITIAL_GRACE_SECS)
            } else {
                Duration::ZERO
            };

            let mut interval =
                tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
            interval.tick().await;

            loop {
                interval.tick().await;
                let _ = hb_app.emit(
                    crate::events::EVENT_PLAN_HEARTBEAT,
                    PlanTerminalPayload {
                        project_id: hb_project.clone(),
                        detail: String::new(),
                    },
                );

                let since_last = hb_activity
                    .lock()
                    .map(|instant| instant.elapsed())
                    .unwrap_or_default();

                let effective_threshold = if hb_start.elapsed() < initial_grace {
                    stall_threshold + initial_grace
                } else {
                    stall_threshold
                };

                if since_last > effective_threshold {
                    let _ = hb_app.emit(
                        crate::events::EVENT_PLAN_ERROR,
                        PlanTerminalPayload {
                            project_id: hb_project.clone(),
                            detail: "stalled".to_string(),
                        },
                    );
                    if let Ok(mut sessions) = hb_sessions.lock() {
                        if let Some(entry) = sessions.sessions.get_mut(&hb_project) {
                            entry.status = PlanSessionStatus::Stalled;
                        }
                    }
                }
            }
        });

        while let Some(event) = event_rx.recv().await {
            match event {
                CommandEvent::Stdout(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    buffer_text_segments(
                        &text,
                        &classifier,
                        &event_buffer,
                        &project_id,
                        &last_activity_clone,
                    );
                }
                CommandEvent::Stderr(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    buffer_text_segments(
                        &text,
                        &classifier,
                        &event_buffer,
                        &project_id,
                        &last_activity_clone,
                    );
                }
                CommandEvent::Terminated(payload) => {
                    let exit_code = payload.code.unwrap_or(1);

                    flush_event_buffer(&event_buffer, &app_clone, &project_id);

                    if let Ok(guard) = classifier.lock() {
                        let plan_content = guard.accumulated_plan();
                        if !plan_content.is_empty() {
                            let _ = std::fs::write(&plan_path, &plan_content);
                        }
                    }

                    let has_plan = classifier
                        .lock()
                        .map(|guard| !guard.accumulated_plan().is_empty())
                        .unwrap_or(false);

                    if exit_code != 0 {
                        let _ = app_clone.emit(
                            crate::events::EVENT_PLAN_ERROR,
                            PlanTerminalPayload {
                                project_id: project_id.clone(),
                                detail: format!("exit_code={exit_code}"),
                            },
                        );
                    } else if !has_plan {
                        let _ = app_clone.emit(
                            crate::events::EVENT_PLAN_ERROR,
                            PlanTerminalPayload {
                                project_id: project_id.clone(),
                                detail: "empty_output".to_string(),
                            },
                        );
                    } else {
                        let _ = app_clone.emit(
                            crate::events::EVENT_PLAN_COMPLETE,
                            PlanTerminalPayload {
                                project_id: project_id.clone(),
                                detail: String::new(),
                            },
                        );
                    }
                    break;
                }
                _ => {}
            }
        }

        batch_flush_handle.abort();
        plan_flush_handle.abort();
        heartbeat_handle.abort();

        if let Ok(mut sessions) = sessions_arc.lock() {
            sessions.sessions.remove(&project_id);
        }
    });

    Ok(())
}
