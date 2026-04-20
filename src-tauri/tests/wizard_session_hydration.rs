include!("../src/lib.rs");

use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::MutexGuard;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App;

struct TestHarness {
    _lock: MutexGuard<'static, ()>,
    _env_guard: EnvGuard,
    root_dir: PathBuf,
    app: App<MockRuntime>,
    work_dir: PathBuf,
}

struct EnvGuard {
    keys: Vec<(&'static str, Option<String>)>,
}

impl TestHarness {
    fn new() -> Self {
        let lock = test_env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let root_dir =
            std::env::temp_dir().join(format!("loopforge-hydrate-{}", uuid::Uuid::new_v4()));
        let home_dir = root_dir.join("home");
        let work_dir = root_dir.join("work");
        std::fs::create_dir_all(&home_dir).expect("home dir");
        std::fs::create_dir_all(&work_dir).expect("work dir");

        let app = mock_builder()
            .build(mock_context(noop_assets()))
            .expect("test app");
        let env_guard = EnvGuard::set(&home_dir);
        app.manage(db::DbState::open(app.handle()).expect("db state"));

        Self {
            _lock: lock,
            _env_guard: env_guard,
            root_dir,
            app,
            work_dir,
        }
    }

    fn artifact_dir(&self, project_id: &str) -> PathBuf {
        storage::artifacts::project_artifact_dir(self.app.handle(), project_id).expect("artifacts")
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root_dir);
    }
}

impl EnvGuard {
    fn set(home_dir: &Path) -> Self {
        let keys = vec![
            ("HOME", std::env::var("HOME").ok()),
            ("XDG_DATA_HOME", std::env::var("XDG_DATA_HOME").ok()),
            ("XDG_CONFIG_HOME", std::env::var("XDG_CONFIG_HOME").ok()),
        ];
        std::env::set_var("HOME", home_dir);
        std::env::set_var("XDG_DATA_HOME", home_dir.join(".local").join("share"));
        std::env::set_var("XDG_CONFIG_HOME", home_dir.join(".config"));
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

#[tokio::test(flavor = "current_thread")]
async fn wizard_session_hydration_marks_only_impacted_steps_invalid() {
    let harness = TestHarness::new();
    let working_directory = harness.work_dir.to_string_lossy().to_string();
    let project = projects::catalog::create_project(
        harness.app.handle().clone(),
        harness.app.state::<db::DbState>(),
        "Hydrate Project".to_string(),
        "Session hydrate".to_string(),
        working_directory.clone(),
        Some("launch".to_string()),
    )
    .await
    .expect("project");
    let artifacts = harness.artifact_dir(&project.id);

    std::fs::write(
        artifacts.join("draft.json"),
        json!({
            "currentStep": "launch",
            "highestStep": 5,
            "name": "Legacy Hydrate",
            "description": "Hydrates old draft payloads",
            "workingDirectory": working_directory,
            "planAgent": "codex"
        })
        .to_string(),
    )
    .expect("draft");
    let _ = std::fs::remove_file(artifacts.join("plan.md"));
    std::fs::write(artifacts.join("prd.json"), "{ broken").expect("prd");
    std::fs::write(artifacts.join("config.json"), "{ broken").expect("config");
    let _ = std::fs::remove_file(artifacts.join("prompt.md"));
    std::fs::write(artifacts.join("guardrails.md"), [0_u8, 159, 146, 150]).expect("guardrails");

    let session = projects::runtime_config::wizard_session_adapter::hydrate_canonical_session(
        harness.app.handle(),
        &project,
        None,
    )
    .expect("hydrate");

    assert_eq!(session.project_data.name, "Legacy Hydrate");
    assert_eq!(session.project_data.plan_agent, "codex");
    assert_eq!(session.highest_step, 5);
    assert!(session.validation.describe.valid);
    assert!(!session.validation.plan.valid);
    assert!(!session.validation.atomize.valid);
    assert!(!session.validation.configure.valid);
    assert!(!session.validation.launch.valid);
    assert_eq!(session.validation.plan.issues, vec!["plan.md is missing"]);
    assert_eq!(session.stories.len(), 0);
    assert!(session.config.is_none());
    assert!(session.prompt.is_none());
    assert!(session.guardrails.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn wizard_session_hydration_reads_legacy_working_directory_artifacts() {
    let harness = TestHarness::new();
    let working_directory = harness.work_dir.to_string_lossy().to_string();
    let project = projects::catalog::create_project(
        harness.app.handle().clone(),
        harness.app.state::<db::DbState>(),
        "Legacy Workspace".to_string(),
        "Session hydrate".to_string(),
        working_directory.clone(),
        Some("launch".to_string()),
    )
    .await
    .expect("project");
    let artifacts = harness.artifact_dir(&project.id);

    std::fs::write(
        artifacts.join("draft.json"),
        json!({ "currentStep": "launch", "highestStep": 5 }).to_string(),
    )
    .expect("draft");
    let _ = std::fs::remove_file(artifacts.join("plan.md"));
    let _ = std::fs::remove_file(artifacts.join("prd.json"));
    let _ = std::fs::remove_file(artifacts.join("config.json"));
    let _ = std::fs::remove_file(artifacts.join("prompt.md"));
    let _ = std::fs::remove_file(artifacts.join("guardrails.md"));

    std::fs::write(harness.work_dir.join("plan.md"), "# Legacy plan").expect("legacy plan");
    std::fs::write(
        harness.work_dir.join("prd.json"),
        json!({
            "projectName": "Legacy Workspace",
            "workingDirectory": working_directory,
            "stories": [{ "id": "S-001", "title": "Hydrate", "acceptanceCriteria": ["Works"] }]
        })
        .to_string(),
    )
    .expect("legacy prd");
    std::fs::write(
        harness.work_dir.join("config.json"),
        json!({ "executeAgent": "codex", "maxIterations": 25 }).to_string(),
    )
    .expect("legacy config");
    std::fs::write(harness.work_dir.join("prompt.md"), "Legacy prompt").expect("legacy prompt");
    std::fs::write(harness.work_dir.join("guardrails.md"), "Legacy guardrails")
        .expect("legacy guardrails");

    let session = projects::runtime_config::wizard_session_adapter::hydrate_canonical_session(
        harness.app.handle(),
        &project,
        None,
    )
    .expect("hydrate");

    assert!(session.validation.plan.valid);
    assert!(session.validation.atomize.valid);
    assert!(session.validation.configure.valid);
    assert!(session.validation.launch.valid);
    assert_eq!(session.plan.as_deref(), Some("# Legacy plan"));
    assert_eq!(session.stories.len(), 1);
    assert_eq!(
        session.config.as_ref().map(|config| &config.execute_agent),
        Some(&"codex".to_string())
    );
    assert_eq!(session.prompt.as_deref(), Some("Legacy prompt"));
    assert_eq!(session.guardrails.as_deref(), Some("Legacy guardrails"));
}
