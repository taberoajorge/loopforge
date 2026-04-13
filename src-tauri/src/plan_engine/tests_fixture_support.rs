use super::PlanSessionsState;
use std::path::PathBuf;
use std::sync::MutexGuard;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App;

pub(super) struct EnvGuard {
    _lock: MutexGuard<'static, ()>,
    root_dir: PathBuf,
    values: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    pub(super) fn new(fixture_set: &str) -> Self {
        let lock = crate::test_env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let root_dir =
            std::env::temp_dir().join(format!("loopforge-plan-fixture-{}", uuid::Uuid::new_v4()));
        let home_dir = root_dir.join("home");
        let config_dir = home_dir.join(".config");
        let data_dir = home_dir.join(".local").join("share");
        std::fs::create_dir_all(root_dir.join("work")).unwrap();
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        let values = [
            "HOME",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "LOOPFORGE_TEST_MODE",
            "LOOPFORGE_TEST_FIXTURE_SET",
        ]
        .into_iter()
        .map(|key| (key, std::env::var(key).ok()))
        .collect();
        std::env::set_var("HOME", &home_dir);
        std::env::set_var("XDG_CONFIG_HOME", &config_dir);
        std::env::set_var("XDG_DATA_HOME", &data_dir);
        std::env::set_var("LOOPFORGE_TEST_MODE", "1");
        std::env::set_var("LOOPFORGE_TEST_FIXTURE_SET", fixture_set);

        Self {
            _lock: lock,
            root_dir,
            values,
        }
    }

    pub(super) fn work_dir(&self) -> PathBuf {
        self.root_dir.join("work")
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in self.values.drain(..) {
            match value {
                Some(previous) => std::env::set_var(key, previous),
                None => std::env::remove_var(key),
            }
        }
        let _ = std::fs::remove_dir_all(&self.root_dir);
    }
}

pub(super) fn test_app() -> App<MockRuntime> {
    mock_builder()
        .manage(PlanSessionsState::default())
        .build(mock_context(noop_assets()))
        .expect("test app")
}

pub(super) async fn wait_for(predicate: impl Fn() -> bool) {
    for _ in 0..50 {
        if predicate() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("condition not met");
}
