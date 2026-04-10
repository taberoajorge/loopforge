use super::agents::fixture_agent_names;
use super::runtime::{resolve_test_mode, FixtureSet, TestConfigError, TestMode};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

static ENV_LOCK: Mutex<()> = Mutex::new(());

const TEST_ENV_KEYS: [&str; 4] = [
    "LOOPFORGE_TEST_MODE",
    "LOOPFORGE_TEST_FIXTURE_SET",
    "LOOPFORGE_TEST_DATA_DIR",
    "LOOPFORGE_TEST_DIALOG_DIR",
];

struct EnvGuard {
    _lock: MutexGuard<'static, ()>,
    values: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn new() -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let values = TEST_ENV_KEYS
            .into_iter()
            .map(|key| (key, std::env::var(key).ok()))
            .collect();
        Self {
            _lock: lock,
            values,
        }
    }

    fn clear(&self) {
        for key in TEST_ENV_KEYS {
            std::env::remove_var(key);
        }
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
    }
}

#[test]
fn disabled_when_mode_is_unset() {
    let guard = EnvGuard::new();
    guard.clear();

    let mode = resolve_test_mode().expect("resolve disabled runtime");

    assert!(matches!(mode, TestMode::Disabled));
    assert!(mode.runtime().is_none());
}

#[test]
fn enabled_runtime_reads_fixture_and_paths() {
    let guard = EnvGuard::new();
    guard.clear();
    std::env::set_var("LOOPFORGE_TEST_MODE", "true");
    std::env::set_var("LOOPFORGE_TEST_FIXTURE_SET", "atomize-error");
    std::env::set_var("LOOPFORGE_TEST_DATA_DIR", "/tmp/loopforge-data");
    std::env::set_var("LOOPFORGE_TEST_DIALOG_DIR", "/tmp/loopforge-dialogs");

    let runtime = resolve_test_mode()
        .expect("resolve enabled runtime")
        .runtime()
        .cloned()
        .expect("enabled runtime");

    assert_eq!(runtime.fixture_set(), FixtureSet::AtomizeError);
    assert_eq!(runtime.data_dir(), PathBuf::from("/tmp/loopforge-data"));
    assert_eq!(
        runtime.dialog_dir(),
        PathBuf::from("/tmp/loopforge-dialogs")
    );
}

#[test]
fn enabled_runtime_reads_no_agents_fixture() {
    let guard = EnvGuard::new();
    guard.clear();
    std::env::set_var("LOOPFORGE_TEST_MODE", "1");
    std::env::set_var("LOOPFORGE_TEST_FIXTURE_SET", "no-agents");

    let runtime = resolve_test_mode()
        .expect("resolve enabled runtime")
        .runtime()
        .cloned()
        .expect("enabled runtime");

    assert_eq!(runtime.fixture_set(), FixtureSet::NoAgents);
}

#[test]
fn rejects_unknown_fixture_set() {
    let guard = EnvGuard::new();
    guard.clear();
    std::env::set_var("LOOPFORGE_TEST_MODE", "1");
    std::env::set_var("LOOPFORGE_TEST_FIXTURE_SET", "bogus");

    let error = resolve_test_mode().expect_err("unknown fixture set must fail");

    assert_eq!(
        error,
        TestConfigError::UnknownFixtureSet("bogus".to_string())
    );
}

#[test]
fn rejects_invalid_test_mode() {
    let guard = EnvGuard::new();
    guard.clear();
    std::env::set_var("LOOPFORGE_TEST_MODE", "maybe");

    let error = resolve_test_mode().expect_err("invalid mode must fail");

    assert_eq!(error, TestConfigError::InvalidTestMode("maybe".to_string()));
}

#[tokio::test(flavor = "current_thread")]
async fn detect_agents_returns_empty_inventory_for_no_agents_fixture() {
    let guard = EnvGuard::new();
    guard.clear();
    std::env::set_var("LOOPFORGE_TEST_MODE", "1");
    std::env::set_var("LOOPFORGE_TEST_FIXTURE_SET", "no-agents");

    let runtime = resolve_test_mode()
        .expect("resolve enabled runtime")
        .runtime()
        .cloned()
        .expect("enabled runtime");
    let detected = super::agents::fixture_agents(&runtime);

    assert!(detected.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn detect_agents_returns_fixture_inventory_in_test_mode() {
    let guard = EnvGuard::new();
    guard.clear();
    std::env::set_var("LOOPFORGE_TEST_MODE", "1");
    std::env::set_var("LOOPFORGE_TEST_FIXTURE_SET", "loop-failure");

    let runtime = resolve_test_mode()
        .expect("resolve enabled runtime")
        .runtime()
        .cloned()
        .expect("enabled runtime");
    let detected = super::agents::fixture_agents(&runtime);

    let names: Vec<&str> = detected.iter().map(|agent| agent.name.as_str()).collect();
    assert_eq!(names, fixture_agent_names(FixtureSet::LoopFailure));
    assert!(detected.iter().all(|agent| agent.available));
    assert_eq!(
        detected[0].version.as_deref(),
        Some("loop-failure-claude-1.0.0")
    );
    assert_eq!(
        detected[1].version.as_deref(),
        Some("loop-failure-codex-1.0.0")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn get_agent_capabilities_returns_fixture_profile_in_test_mode() {
    let guard = EnvGuard::new();
    guard.clear();
    std::env::set_var("LOOPFORGE_TEST_MODE", "true");
    std::env::set_var("LOOPFORGE_TEST_FIXTURE_SET", "plan-error");

    let capabilities = crate::agents::get_agent_capabilities("codex".to_string())
        .await
        .expect("fixture capabilities");

    assert_eq!(capabilities.agent, "codex");
    assert_eq!(capabilities.source, "fixture:plan-error");
    assert!(capabilities.supports_model);
    assert!(capabilities.supports_effort);
    assert_eq!(
        capabilities.default_model.as_deref(),
        Some("fixture-gpt-5.4")
    );
    assert_eq!(capabilities.default_effort.as_deref(), Some("medium"));
    assert_eq!(capabilities.models.len(), 2);
    assert_eq!(capabilities.models[0].id, "fixture-gpt-5.4");
    assert_eq!(capabilities.efforts[2].id, "high");
}
