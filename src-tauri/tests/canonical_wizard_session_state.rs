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
            std::env::temp_dir().join(format!("loopforge-canonical-{}", uuid::Uuid::new_v4()));
        let home_dir = root_dir.join("home");
        let work_dir = root_dir.join("work");
        std::fs::create_dir_all(&home_dir).expect("home dir");
        std::fs::create_dir_all(&work_dir).expect("work dir");
        let env_guard = EnvGuard::set(&home_dir);
        let app = mock_builder()
            .build(mock_context(noop_assets()))
            .expect("test app");
        app.manage(DbState::open(app.handle()).expect("db state"));
        Self {
            _lock: lock,
            _env_guard: env_guard,
            root_dir,
            app,
            work_dir,
        }
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
async fn canonical_wizard_session_state_defaults_legacy_draft_fields() {
    let harness = TestHarness::new();
    let project = projects::catalog::create_project(
        harness.app.handle().clone(),
        harness.app.state::<DbState>(),
        "Legacy Project".to_string(),
        "Legacy wizard payload".to_string(),
        harness.work_dir.to_string_lossy().to_string(),
        Some("describe".to_string()),
    )
    .await
    .expect("project");
    let legacy_draft = json!({
        "projectId": project.id,
        "describe": {
            "name": "Legacy Project",
            "description": "Legacy wizard payload",
            "workingDirectory": harness.work_dir.to_string_lossy()
        }
    });

    projects::wizard::save_draft(
        harness.app.handle().clone(),
        project.id.clone(),
        legacy_draft.to_string(),
    )
    .await
    .expect("save draft");

    let resume = projects::wizard::resume_wizard(
        harness.app.handle().clone(),
        harness.app.state::<DbState>(),
        project.id,
    )
    .await
    .expect("resume");
    let session = resume.wizard_session.expect("canonical session");

    assert_eq!(session.version, 1);
    assert_eq!(session.current_step, "describe");
    assert_eq!(session.highest_step, 1);
    assert_eq!(session.describe.plan_agent, "claude");
    assert!(session.describe.plan_model.is_none());
    assert!(session.describe.plan_effort.is_none());
    assert!(!session.plan.completed);
    assert!(session.plan.document.is_none());
    assert_eq!(session.atomize.stories_count, 0);
    assert!(session.atomize.stories.is_empty());
    assert!(session.configure.is_none());
    assert!(session.prompt.is_none());
    assert!(session.guardrails.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn canonical_wizard_session_state_hydrates_without_downstream_artifacts() {
    let harness = TestHarness::new();
    let working_directory = harness.work_dir.to_string_lossy().to_string();
    let project = projects::catalog::create_project(
        harness.app.handle().clone(),
        harness.app.state::<DbState>(),
        "Hydration Project".to_string(),
        "Hydrates optional state".to_string(),
        working_directory.clone(),
        Some("plan".to_string()),
    )
    .await
    .expect("project");
    let draft = json!({
        "projectId": project.id,
        "currentStep": "plan",
        "describe": {
            "name": "Hydration Project",
            "description": "Hydrates optional state",
            "workingDirectory": working_directory,
            "planAgent": "codex"
        }
    });

    projects::wizard::save_draft(
        harness.app.handle().clone(),
        project.id.clone(),
        draft.to_string(),
    )
    .await
    .expect("save draft");

    let hydration = projects::wizard::hydrate_wizard(
        harness.app.handle().clone(),
        harness.app.state::<DbState>(),
        project.id,
    )
    .await
    .expect("hydrate");

    assert_eq!(hydration.wizard_step, "plan");
    assert_eq!(hydration.highest_step, 2);
    assert_eq!(hydration.project_data.plan_agent, "codex");
    assert!(!hydration.plan_complete);
    assert!(hydration.stories.is_empty());
    assert!(hydration.config.is_none());
}
