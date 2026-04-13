use super::event_sink::TauriEventSink;
use super::helpers::{artifact_dir, build_ralph_config, create_session, ensure_execution_prompt};
use super::provider::ShellProvider;
use super::start_finalize::finalize_loop_run;
use super::start_resolve::{resolve_start_loop, ResolvedStartLoop};
use super::{LoopError, LoopHandle, LoopManagerState, StartLoopArgs};
use crate::db::DbState;
use ralph_core::loop_engine;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Runtime};

pub async fn start_loop<R: Runtime>(
    app: AppHandle<R>,
    args: StartLoopArgs,
) -> Result<String, LoopError> {
    let db = app.state::<DbState>();
    let loop_state = app.state::<LoopManagerState>();
    let resolved = resolve_start_loop(&app, &db, &args).await?;

    {
        let mut handles = loop_state.0.lock().map_err(|_| LoopError::LockPoisoned)?;
        if handles.contains_key(&resolved.project_id) {
            return Err(LoopError::AlreadyRunning(resolved.project_id));
        }
        handles.insert(
            resolved.project_id.clone(),
            LoopHandle::placeholder(args.clone()),
        );
    }

    let result = do_start_loop(&app, &db, &loop_state, resolved.clone()).await;
    match result {
        Ok((session_id, real_handle)) => {
            let active = {
                let mut handles = loop_state.0.lock().map_err(|_| LoopError::LockPoisoned)?;
                handles.insert(resolved.project_id.clone(), real_handle);
                handles.len()
            };
            crate::tray::update_tooltip(&app, active);
            Ok(session_id)
        }
        Err(err) => {
            if let Ok(mut handles) = loop_state.0.lock() {
                handles.remove(&resolved.project_id);
            }
            Err(err)
        }
    }
}

async fn do_start_loop<R: Runtime>(
    app: &AppHandle<R>,
    db: &DbState,
    _loop_state: &LoopManagerState,
    resolved: ResolvedStartLoop,
) -> Result<(String, LoopHandle), LoopError> {
    let stored_args = StartLoopArgs {
        project_id: resolved.project_id.clone(),
        project_name: Some(resolved.project_name.clone()),
        working_directory: Some(resolved.working_directory.clone()),
        agent: Some(resolved.agent.clone()),
        model: resolved.model.clone(),
        effort: resolved.effort.clone(),
        fallback_agents: resolved.fallback_agents.clone(),
        max_iterations: resolved.max_iterations,
        gutter_threshold: resolved.gutter_threshold,
        cooldown_seconds: resolved.cooldown_seconds,
        test_command: resolved.test_command.clone(),
        max_verification_retries: resolved.max_verification_retries,
        scm_provider: Some(resolved.scm_provider.clone()),
        review_polling_interval: Some(resolved.review_polling_interval),
        review_timeout: Some(resolved.review_timeout),
    };
    let artifacts = artifact_dir(app, &resolved.project_id)?;
    let gutter_threshold = resolved.gutter_threshold.unwrap_or(3);
    crate::projects::reconcile::reconcile_project_prd(app, &resolved.project_id, gutter_threshold)
        .map_err(|err| LoopError::Db(err.to_string()))?;
    ensure_execution_prompt(&artifacts, &resolved.project_name)?;
    let session_id = create_session(db, &resolved.project_id)?;

    if let Ok(json) = serde_json::to_string_pretty(&stored_args) {
        let _ = std::fs::write(artifacts.join("loop_args.json"), json);
    }

    let work_dir = PathBuf::from(&resolved.working_directory);
    let config = build_ralph_config(&artifacts, &work_dir, &stored_args);

    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown_flag.clone();
    let pause_file = config.paths.pause_file.clone();

    let all_agents: Vec<String> = std::iter::once(resolved.agent.clone())
        .chain(resolved.fallback_agents.clone().into_iter())
        .collect();

    let provider = ShellProvider {
        app: app.clone(),
        agent_name: Arc::new(std::sync::Mutex::new(resolved.agent.clone())),
        primary_agent: resolved.agent.clone(),
        selected_model: resolved.model.clone(),
        selected_effort: resolved.effort.clone(),
        fallback_agents: all_agents,
        fallback_index: Arc::new(std::sync::Mutex::new(0)),
        project_id: resolved.project_id.clone(),
        project_name: resolved.project_name.clone(),
        session_id: session_id.clone(),
        iteration_counter: Arc::new(std::sync::Mutex::new(0)),
    };

    let _ = app.emit(
        crate::events::EVENT_SESSION_STARTED,
        serde_json::json!({
            "projectId": resolved.project_id,
            "sessionId": session_id,
            "agent": resolved.agent,
            "model": resolved.model,
            "effort": resolved.effort,
        }),
    );

    let app_clone = app.clone();
    let project_id_clone = resolved.project_id.clone();
    let project_name_clone = resolved.project_name.clone();
    let session_id_clone = session_id.clone();
    let working_dir_clone = resolved.working_directory.clone();

    let event_sink =
        TauriEventSink::new(app.clone(), resolved.project_id.clone(), session_id.clone());

    let join_handle = tokio::spawn(async move {
        let run_result =
            loop_engine::run(&config, &provider, shutdown_clone.clone(), &event_sink).await;
        let outcome = if shutdown_clone.load(Ordering::SeqCst) {
            if pause_file.exists() {
                "paused"
            } else {
                "failed"
            }
        } else if run_result.is_ok() {
            "completed"
        } else {
            "failed"
        };
        finalize_loop_run(
            &app_clone,
            &project_id_clone,
            &project_name_clone,
            &session_id_clone,
            &working_dir_clone,
            outcome,
        )
        .await;
    });
    {
        let conn = db.0.lock().map_err(|_| LoopError::LockPoisoned)?;
        let now = chrono::Utc::now().to_rfc3339();
        let _ = conn.execute(
            "UPDATE projects SET status = 'active', updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, resolved.project_id],
        );
    }
    let snapshot = crate::commands::projects::get_project_snapshot(
        app.clone(),
        app.state::<DbState>(),
        app.state::<LoopManagerState>(),
        resolved.project_id.clone(),
    )
    .await
    .ok();
    let payload = if let Some(snapshot) = snapshot {
        serde_json::json!({
            "projectId": resolved.project_id,
            "snapshot": snapshot,
        })
    } else {
        serde_json::json!({ "projectId": resolved.project_id })
    };
    let _ = app.emit(crate::events::EVENT_PROJECT_STATE_CHANGED, payload);

    Ok((
        session_id,
        LoopHandle {
            shutdown_flag,
            args: stored_args,
            _join_handle: join_handle,
        },
    ))
}
