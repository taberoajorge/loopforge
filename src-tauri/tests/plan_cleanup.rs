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
            std::env::temp_dir().join(format!("loopforge-plan-cleanup-{}", uuid::Uuid::new_v4()));
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

    async fn create_project(&self, name: &str, working_directory: &Path) -> projects::Project {
        std::fs::create_dir_all(working_directory).expect("working dir");
        projects::catalog::create_project(
            self.app.handle().clone(),
            self.app.state::<db::DbState>(),
            name.to_string(),
            format!("{name} description"),
            working_directory.to_string_lossy().to_string(),
            Some("plan".to_string()),
        )
        .await
        .expect("project")
    }

    fn artifact_dir(&self, project_id: &str) -> PathBuf {
        storage::artifacts::project_artifact_dir(self.app.handle(), project_id)
            .expect("artifact dir")
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

async fn seed_plan_artifacts(harness: &TestHarness, project: &projects::Project) {
    let artifact_dir = harness.artifact_dir(&project.id);
    projects::documents::save_plan(
        harness.app.handle().clone(),
        project.id.clone(),
        "# Generated plan\n\n- Capture cleanup contract\n".to_string(),
    )
    .await
    .expect("save plan");
    projects::documents::save_prd(
        harness.app.handle().clone(),
        project.id.clone(),
        json!({
            "projectName": project.name,
            "generatedAt": "2026-04-10T00:00:00Z",
            "stories": [{
                "id": "S-008",
                "title": "Cleanup contract",
                "acceptanceCriteria": ["Cleanup removes only intended plan artifacts"]
            }]
        })
        .to_string(),
    )
    .await
    .expect("save prd");
    projects::runtime_config::save_project_config(
        harness.app.handle(),
        &project.id,
        &projects::ProjectConfig::default(),
    )
    .expect("save config");
    std::fs::write(artifact_dir.join("prompt.md"), "Execution prompt").expect("prompt");
    std::fs::write(artifact_dir.join("guardrails.md"), "Execution guardrails").expect("guardrails");
}

#[tokio::test(flavor = "current_thread")]
async fn plan_cleanup_removes_generated_artifacts_for_target_project() {
    let harness = TestHarness::new();
    let project = harness
        .create_project("Cleanup Target", &harness.work_dir.join("target"))
        .await;
    let artifact_dir = harness.artifact_dir(&project.id);
    seed_plan_artifacts(&harness, &project).await;

    assert!(artifact_dir.join("plan.md").exists());
    assert!(artifact_dir.join("prd.json").exists());
    assert!(artifact_dir.join("config.json").exists());

    projects::wizard::discard_draft(
        harness.app.handle().clone(),
        harness.app.state::<db::DbState>(),
        project.id,
    )
    .await
    .expect("discard draft");

    assert!(!artifact_dir.exists());
}

#[tokio::test(flavor = "current_thread")]
async fn plan_cleanup_preserves_unrelated_project_files_and_artifacts() {
    let harness = TestHarness::new();
    let target_work_dir = harness.work_dir.join("target");
    let other_work_dir = harness.work_dir.join("other");
    let target = harness
        .create_project("Cleanup Target", &target_work_dir)
        .await;
    let other = harness
        .create_project("Cleanup Neighbor", &other_work_dir)
        .await;
    let target_artifact_dir = harness.artifact_dir(&target.id);
    let other_artifact_dir = harness.artifact_dir(&other.id);
    let working_tree_file = target_work_dir.join("README.md");
    seed_plan_artifacts(&harness, &target).await;
    seed_plan_artifacts(&harness, &other).await;
    std::fs::write(&working_tree_file, "keep me").expect("working tree file");

    projects::wizard::discard_draft(
        harness.app.handle().clone(),
        harness.app.state::<db::DbState>(),
        target.id,
    )
    .await
    .expect("discard draft");

    assert!(!target_artifact_dir.exists());
    assert!(working_tree_file.exists());
    assert_eq!(
        std::fs::read_to_string(&working_tree_file).expect("working tree content"),
        "keep me"
    );
    assert!(other_artifact_dir.join("plan.md").exists());
    assert!(other_artifact_dir.join("prd.json").exists());
    assert!(other_artifact_dir.join("config.json").exists());
}
