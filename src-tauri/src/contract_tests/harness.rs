use crate::ask_engine::types::AskMessage;
use crate::commands;
use crate::db::DbState;
use crate::loop_manager::{LoopManagerState, StartLoopArgs};
use std::path::{Path, PathBuf};
use std::sync::MutexGuard;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::{App, Manager};

pub struct TestHarness {
    _lock: MutexGuard<'static, ()>,
    env_guard: EnvGuard,
    root_dir: PathBuf,
    pub app: App<MockRuntime>,
    pub work_dir: PathBuf,
    pub agent_name: String,
}

struct EnvGuard {
    keys: Vec<(&'static str, Option<String>)>,
}

impl TestHarness {
    pub fn new() -> Self {
        let lock = crate::test_env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let root_dir =
            std::env::temp_dir().join(format!("loopforge-contract-{}", uuid::Uuid::new_v4()));
        let home_dir = root_dir.join("home");
        let bin_dir = root_dir.join("bin");
        let work_dir = root_dir.join("work");
        std::fs::create_dir_all(&home_dir).expect("home dir");
        std::fs::create_dir_all(&bin_dir).expect("bin dir");
        std::fs::create_dir_all(&work_dir).expect("work dir");

        let agent_name = "fixture-agent".to_string();
        install_fixture_agent(&bin_dir, &agent_name);
        let env_guard = EnvGuard::set(&home_dir, &bin_dir);

        let app = mock_builder()
            .plugin(tauri_plugin_shell::init())
            .manage(LoopManagerState::default())
            .manage(crate::ask_engine::session::AskSessionsState::default())
            .build(mock_context(noop_assets()))
            .expect("test app");
        app.manage(DbState::open(app.handle()).expect("db state"));

        Self {
            _lock: lock,
            env_guard,
            root_dir,
            app,
            work_dir,
            agent_name,
        }
    }

    pub fn artifact_dir(&self, project_id: &str) -> PathBuf {
        crate::storage::artifacts::project_artifact_dir(self.app.handle(), project_id)
            .expect("artifact dir")
    }

    pub fn db_path(&self) -> PathBuf {
        self.app
            .path()
            .app_data_dir()
            .expect("app data dir")
            .join("loopforge.db")
    }

    pub async fn wait_for_completion(&self, project_id: &str) {
        for _ in 0..100 {
            let status = {
                let db = self.app.state::<DbState>();
                let conn = db.0.lock().expect("db lock");
                conn.query_row(
                    "SELECT status FROM projects WHERE id = ?1",
                    rusqlite::params![project_id],
                    |row: &rusqlite::Row| row.get::<_, String>(0),
                )
                .expect("project status")
            };
            if status == "completed" {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        panic!("loop did not complete for {project_id}");
    }

    pub async fn wait_for_ask_messages(
        &self,
        project_id: &str,
        expected: usize,
    ) -> Vec<AskMessage> {
        for _ in 0..100 {
            let messages =
                commands::ask::ask_history(self.app.state::<DbState>(), project_id.to_string())
                    .await
                    .expect("ask history");
            if messages.len() >= expected {
                return messages;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        panic!("ask history did not reach {expected} messages");
    }

    pub async fn start_loop(&self, project_id: &str) -> String {
        crate::loop_manager::start_loop(
            self.app.handle().clone(),
            StartLoopArgs {
                project_id: project_id.to_string(),
                project_name: None,
                working_directory: None,
                agent: None,
                model: None,
                effort: None,
                fallback_agents: vec![],
                max_iterations: None,
                gutter_threshold: None,
                cooldown_seconds: None,
                test_command: None,
                max_verification_retries: None,
                scm_provider: None,
                review_polling_interval: None,
                review_timeout: None,
            },
        )
        .await
        .expect("start loop")
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root_dir);
        let _ = &self.env_guard;
    }
}

impl EnvGuard {
    fn set(home_dir: &Path, bin_dir: &Path) -> Self {
        let mut path_items = vec![bin_dir.to_path_buf()];
        if let Some(existing_path) = std::env::var_os("PATH") {
            path_items.extend(std::env::split_paths(&existing_path));
        }
        let path = std::env::join_paths(path_items)
            .ok()
            .and_then(|joined| joined.into_string().ok())
            .unwrap_or_else(|| bin_dir.display().to_string());
        let keys = vec![
            ("HOME", std::env::var("HOME").ok()),
            ("XDG_DATA_HOME", std::env::var("XDG_DATA_HOME").ok()),
            ("XDG_CONFIG_HOME", std::env::var("XDG_CONFIG_HOME").ok()),
            ("PATH", std::env::var("PATH").ok()),
        ];
        std::env::set_var("HOME", home_dir);
        std::env::set_var("XDG_DATA_HOME", home_dir.join(".local").join("share"));
        std::env::set_var("XDG_CONFIG_HOME", home_dir.join(".config"));
        std::env::set_var("PATH", path);
        Self { keys }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, previous) in self.keys.drain(..) {
            match previous {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }
}

fn install_fixture_agent(bin_dir: &Path, agent_name: &str) {
    #[cfg(windows)]
    let script = bin_dir.join(format!("{agent_name}.cmd"));
    #[cfg(not(windows))]
    let script = bin_dir.join(agent_name);
    #[cfg(windows)]
    let content = "@echo off\r\necho fixture agent completed\r\n";
    #[cfg(not(windows))]
    let content = "#!/bin/sh\nprintf 'fixture agent completed\\n'\n";
    std::fs::write(&script, content).expect("fixture agent");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script)
            .expect("fixture metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script, perms).expect("fixture perms");
    }
}
