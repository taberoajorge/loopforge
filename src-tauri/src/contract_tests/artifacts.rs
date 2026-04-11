#[path = "wizard_persistence.rs"]
mod wizard_persistence;
#[path = "merge_coordinator.rs"]
mod merge_coordinator;

use super::harness::TestHarness;
use super::support::{
    infer_status, load_config, project_detail, seed_project, session_ended_at, start_ask,
};
use crate::commands;
use crate::db::DbState;
use crate::models::ProjectStatus;
use ralph_core::prd::Prd;
use tauri::Manager;

#[tokio::test(flavor = "current_thread")]
async fn happy_path_persists_artifacts_and_runtime_histories() {
    let harness = TestHarness::new();
    let project = seed_project(&harness).await;
    let artifact_dir = harness.artifact_dir(&project.id);
    let detail = project_detail(&harness, &project.id).await;
    let has_config = load_config(&harness, &project.id).await.is_some();

    assert!(matches!(
        infer_status(
            &detail.project.status,
            detail.total_stories,
            has_config,
            false
        ),
        ProjectStatus::Ready
    ));
    assert_eq!(detail.total_stories, 1);
    assert_eq!(
        artifact_dir.to_string_lossy(),
        harness.artifact_dir(&project.id).to_string_lossy()
    );
    for path in crate::storage::artifacts::file_paths(&artifact_dir) {
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(path.exists(), "{name} must exist");
    }

    let _session_id = harness.start_loop(&project.id).await;
    harness.wait_for_completion(&project.id).await;
    start_ask(&harness, &project.id, "What changed?").await;
    let ask_history = harness.wait_for_ask_messages(&project.id, 2).await;

    let detail = project_detail(&harness, &project.id).await;
    let has_config = load_config(&harness, &project.id).await.is_some();
    assert!(matches!(
        infer_status(
            &detail.project.status,
            detail.total_stories,
            has_config,
            false
        ),
        ProjectStatus::Completed
    ));
    assert_eq!(detail.passed_count, 1);
    assert!(session_ended_at(&harness, &project.id).is_some());

    let iterations = commands::execution::get_iteration_history(
        harness.app.state::<DbState>(),
        project.id.clone(),
    )
    .await
    .expect("iteration history");
    assert_eq!(iterations.len(), 1);
    assert_eq!(iterations[0].story_id, "FIX-001");
    assert_eq!(iterations[0].result, "success");
    assert_eq!(iterations[0].agent_used, harness.agent_name);

    assert_eq!(ask_history.len(), 2);
    assert!(ask_history.iter().any(|item| item.role == "user"));
    assert!(ask_history.iter().any(|item| item.role == "assistant"));

    let output = crate::projects::documents::load_output_log(
        harness.app.handle().clone(),
        project.id.clone(),
    )
    .await
    .expect("output log");
    assert!(output.contains("fixture agent completed"));

    let draft: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(artifact_dir.join("draft.json")).unwrap())
            .unwrap();
    assert_eq!(draft["currentStep"], "describe");
    let plan = std::fs::read_to_string(artifact_dir.join("plan.md")).unwrap();
    assert!(plan.contains("# Contract Plan"));
    let prd = Prd::load(&artifact_dir.join("prd.json")).unwrap();
    assert!(prd.stories[0].passes);
    let config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(artifact_dir.join("config.json")).unwrap())
            .unwrap();
    assert_eq!(config["executeAgent"], "fixture-agent");

    assert!(harness.db_path().exists(), "SQLite file must exist on disk");
    let conn = rusqlite::Connection::open(harness.db_path()).unwrap();
    let iteration_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM iterations", [], |row| row.get(0))
        .unwrap();
    let ask_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM ask_messages", [], |row| row.get(0))
        .unwrap();
    assert_eq!(iteration_count, 1);
    assert_eq!(ask_count, 2);
}

#[tokio::test(flavor = "current_thread")]
async fn malformed_artifacts_are_detected_deterministically() {
    let harness = TestHarness::new();
    let project = crate::projects::catalog::create_project(
        harness.app.handle().clone(),
        harness.app.state::<DbState>(),
        "Malformed Contract".to_string(),
        "Exercise malformed artifacts".to_string(),
        harness.work_dir.to_string_lossy().to_string(),
        Some("describe".to_string()),
    )
    .await
    .expect("project");
    let artifact_dir = harness.artifact_dir(&project.id);

    std::fs::write(artifact_dir.join("prd.json"), "{ broken").unwrap();
    std::fs::write(artifact_dir.join("config.json"), "{ also broken").unwrap();

    assert!(Prd::load(&artifact_dir.join("prd.json")).is_err());
    let raw_config = load_config(&harness, &project.id)
        .await
        .expect("config payload");
    assert!(serde_json::from_str::<crate::projects::ProjectConfig>(&raw_config).is_err());
}
