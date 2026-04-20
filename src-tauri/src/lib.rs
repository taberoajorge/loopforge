mod activity;
mod adapters;
mod agent_profiles;
mod agent_runtime;
mod agent_runtime_env;
mod agents;
mod ask_engine;
mod atomizer;
mod commands;
mod db;
mod events;
mod invoke;
#[cfg(test)]
mod invoke_contract;
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

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(agents::AgentRegistryState::default())
        .manage(plan_engine::PlanSessionsState::default())
        .manage(loop_manager::LoopManagerState::default())
        .manage(ask_engine::AskSessionsState::default())
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
        });

    let app = invoke::attach_app(builder)
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
