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
            std::env::temp_dir().join(format!("loopforge-wizard-{}", uuid::Uuid::new_v4()));
        let home_dir = root_dir.join("home");
        let work_dir = root_dir.join("work");
        std::fs::create_dir_all(&home_dir).expect("home dir");
        std::fs::create_dir_all(&work_dir).expect("work dir");

        let env_guard = EnvGuard::set(&home_dir);
        let app = mock_builder()
            .build(mock_context(noop_assets()))
            .expect("test app");
        app.manage(db::DbState::open(app.handle()).expect("db state"));

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
async fn wizard_draft_roundtrip_preserves_nested_and_optional_fields() {
    let harness = TestHarness::new();
    let working_directory = harness.work_dir.to_string_lossy().to_string();
    let project = projects::catalog::create_project(
        harness.app.handle().clone(),
        harness.app.state::<db::DbState>(),
        "Roundtrip Project".to_string(),
        "Persist wizard draft values".to_string(),
        working_directory.clone(),
        Some("describe".to_string()),
    )
    .await
    .expect("project");

    let draft = json!({
        "version": 1,
        "projectId": project.id,
        "currentStep": "configure",
        "describe": {
            "name": "Roundtrip Project",
            "description": "Persist wizard draft values",
            "workingDirectory": working_directory,
            "planAgent": "codex",
            "planModel": serde_json::Value::Null,
            "planEffort": "high"
        },
        "plan": {
            "completed": true
        },
        "highestStep": 4,
        "staleFromStep": serde_json::Value::Null,
        "atomize": {
            "stories": [],
            "storiesCount": 3
        },
        "configure": {
            "schemaVersion": 1,
            "executeAgent": "codex",
            "executeModel": serde_json::Value::Null,
            "executeEffort": "medium",
            "fallbackChain": ["claude", "gemini"],
            "gutterThreshold": 4,
            "maxIterations": 25,
            "cooldownSeconds": 15,
            "testCommand": "cargo test wizard_draft_roundtrip",
            "maxVerificationRetries": 2,
            "scmProvider": "auto",
            "reviewPollingInterval": 90,
            "reviewTimeout": 900
        }
    });

    projects::wizard::save_draft(
        harness.app.handle().clone(),
        project.id.clone(),
        draft.to_string(),
    )
    .await
    .expect("save draft");

    let resume_state = projects::wizard::resume_wizard(
        harness.app.handle().clone(),
        harness.app.state::<db::DbState>(),
        project.id.clone(),
    )
    .await
    .expect("resume wizard");
    let loaded_draft = projects::wizard::load_draft(harness.app.handle().clone(), project.id)
        .await
        .expect("load draft")
        .expect("draft content");

    assert_eq!(resume_state.project.name, "Roundtrip Project");
    assert_eq!(
        resume_state.project.description,
        "Persist wizard draft values"
    );
    assert_eq!(resume_state.project.working_directory, working_directory);
    assert_eq!(resume_state.wizard_step, "configure");
    assert!(resume_state.wizard_state_json.is_some());
    assert!(resume_state.has_plan);
    assert!(!resume_state.has_prd);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&loaded_draft).expect("draft json"),
        draft
    );
}
