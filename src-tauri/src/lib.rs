mod activity;
mod agent_profiles;
mod agent_runtime;
mod agent_runtime_env;
mod agents;
mod ask_engine;
mod atomizer;
mod commands;
mod db;
mod events;
mod loop_manager;
mod models;
mod plan_engine;
mod projects;
mod storage;
mod test_support;
mod tray;

mod connections;
mod ephemeral_query;

#[cfg(feature = "frozen")]
#[allow(dead_code)]
mod diagnostic_parser;
#[cfg(feature = "frozen")]
#[allow(dead_code)]
mod notifications;
#[cfg(feature = "frozen")]
#[allow(dead_code)]
mod plugin_registry;
#[cfg(feature = "frozen")]
#[allow(dead_code)]
mod scm_watcher;
#[cfg(feature = "frozen")]
#[allow(dead_code)]
mod summary_generator;
#[cfg(feature = "frozen")]
#[allow(dead_code)]
mod worktree_manager;

#[cfg(test)]
mod contract_tests;
#[cfg(test)]
mod tests;

use db::DbState;
use tauri::{Manager, RunEvent, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    agent_runtime_env::ensure_full_path_env();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(agents::AgentRegistryState::default())
        .manage(plan_engine::PlanSessionsState::default())
        .manage(loop_manager::LoopManagerState::default())
        .manage(ask_engine::AskSessionsState::default())
        .invoke_handler(tauri::generate_handler![
            agents::detect_agents,
            agents::refresh_agents,
            agents::get_agent_capabilities,
            commands::planning::start_plan,
            commands::planning::write_to_plan,
            commands::planning::stop_plan,
            commands::planning::query_plan_status,
            commands::projects_artifacts::load_existing_plan,
            commands::projects_artifacts::load_existing_prd,
            commands::projects_artifacts::load_output_log,
            commands::projects_artifacts::save_plan,
            commands::projects_artifacts::save_prd,
            commands::projects_artifacts::save_config,
            commands::projects_artifacts::load_config,
            commands::atomization::run_atomizer,
            commands::projects_lifecycle::create_project,
            commands::projects_wizard::finalize_draft,
            commands::projects_wizard::discard_draft,
            commands::projects_wizard::save_wizard_state,
            commands::projects_wizard::resume_wizard,
            commands::projects_wizard::save_draft,
            commands::projects_wizard::load_draft,
            commands::projects_lifecycle::list_projects,
            commands::projects_lifecycle::pause_project,
            commands::projects_lifecycle::resume_project,
            commands::projects_lifecycle::archive_project,
            commands::projects_lifecycle::get_project_detail,
            commands::projects_lifecycle::get_project_stories,
            commands::projects_lifecycle::get_guardrails,
            commands::projects_lifecycle::get_project_config,
            commands::projects::get_project_snapshot,
            commands::projects_listing::list_projects_enriched,
            commands::projects_lifecycle::get_notification_prefs,
            commands::projects_lifecycle::save_notification_prefs,
            commands::execution::start_loop,
            commands::execution::stop_loop,
            commands::execution::session_stats,
            commands::execution::get_iteration_history,
            commands::ask::ask_question,
            commands::ask::ask_history,
            commands::ask::stop_ask,
            commands::ask::copy_ask_message,
            commands::ask::truncate_ask_from,
            commands::ask::retry_ask,
            ephemeral_query::ephemeral_query,
            connections::list_connections,
            connections::build_connection_workspace,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let plan_state = window.state::<plan_engine::PlanSessionsState>();
                if let Ok(mut sessions) = plan_state.0.lock() {
                    let ids: Vec<String> = sessions.sessions.keys().cloned().collect();
                    for plan_id in ids {
                        if let Some(entry) = sessions.sessions.remove(&plan_id) {
                            let _ = entry.handle.kill();
                        }
                    }
                }

                let ask_state = window.state::<ask_engine::AskSessionsState>();
                ask_state.kill_all();

                if !cfg!(debug_assertions) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            let db_state = DbState::open(app.handle())?;
            app.manage(db_state);

            tray::setup_tray(app)?;

            let restore_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let db = restore_handle.state::<DbState>();

                if let Ok(conn) = db.0.lock() {
                    let now = chrono::Utc::now().to_rfc3339();
                    let _ = conn.execute(
                        "UPDATE sessions SET ended_at = ?1 WHERE ended_at IS NULL",
                        rusqlite::params![now],
                    );
                    let _ = conn.execute(
                        "UPDATE projects SET status = 'paused' WHERE status = 'active'",
                        [],
                    );
                }

                let saved = db.load_loop_states();
                db.clear_loop_states();
                for (_, args_json) in saved {
                    if let Ok(args) =
                        serde_json::from_str::<loop_manager::StartLoopArgs>(&args_json)
                    {
                        let _ = loop_manager::start_loop(restore_handle.clone(), args).await;
                    }
                }
            });

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(move |app_handle, event| {
        if let RunEvent::ExitRequested { .. } = event {
            let db = app_handle.state::<DbState>();
            let loop_state = app_handle.state::<loop_manager::LoopManagerState>();
            if let Ok(handles) = loop_state.0.lock() {
                for handle in handles.values() {
                    let args_json = serde_json::to_string(&handle.args).unwrap_or_default();
                    let _ = db.save_loop_state(&handle.args.project_id, &args_json);
                }
            }
            loop_state.shutdown_all();
        }
    });
}
