use crate::activity::ActivityClassifier;
use crate::plan_engine::args::{
    agent_env_vars, build_null_stdin_command, build_plan_args, needs_null_stdin,
};
use crate::plan_engine::helpers::{artifact_dir, build_plan_prompt, resolve_agent_binary};
use crate::plan_engine::payloads::PlanActivityPayload;
use crate::plan_engine::sessions::{PlanSessionEntry, PlanSessionStatus};
use crate::plan_engine::{monitor, output, PlanEngineError, PlanSessionsState, StartPlanArgs};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, State};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

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

        let plan_flush = monitor::spawn_plan_flush_task(
            Arc::clone(&classifier),
            plan_path.clone(),
        );
        let batch_flush = monitor::spawn_batch_flush_task(
            Arc::clone(&event_buffer),
            app_clone.clone(),
            project_id.clone(),
        );
        let heartbeat = monitor::spawn_heartbeat_task(
            app_clone.clone(),
            Arc::clone(&last_activity_clone),
            Arc::clone(&sessions_arc),
            project_id.clone(),
            agent_name.clone(),
            now,
        );

        while let Some(event) = event_rx.recv().await {
            match event {
                CommandEvent::Stdout(bytes) | CommandEvent::Stderr(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    output::buffer_text_segments(
                        &text,
                        &classifier,
                        &event_buffer,
                        &project_id,
                        &last_activity_clone,
                    );
                }
                CommandEvent::Terminated(payload) => {
                    monitor::handle_termination(
                        &event_buffer,
                        &classifier,
                        &plan_path,
                        &app_clone,
                        &project_id,
                        payload.code.unwrap_or(1),
                    );
                    break;
                }
                _ => {}
            }
        }

        batch_flush.abort();
        plan_flush.abort();
        heartbeat.abort();

        if let Ok(mut sessions) = sessions_arc.lock() {
            sessions.sessions.remove(&project_id);
        }
    });

    Ok(())
}
