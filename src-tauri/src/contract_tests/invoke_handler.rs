use super::invoke_harness::InvokeHarness;
use crate::projects::{Project, ProjectConfig, ProjectDetail};
use ralph_core::prd::Prd;
use serde_json::json;
use std::path::Path;

#[tokio::test(flavor = "current_thread")]
async fn invoke_handler_covers_lifecycle_and_runtime_commands() {
    let harness = InvokeHarness::new(Some("happy-path"));
    let project = create_project(&harness, "Invoke Contract");

    harness.invoke_ok::<()>(
        "save_draft",
        json!({ "projectId": project.id, "draftJson": json!({ "currentStep": "describe" }).to_string() }),
    );
    harness.invoke_ok::<()>("finalize_draft", json!({ "projectId": project.id }));
    let draft: Option<String> = harness.invoke_ok("load_draft", json!({ "projectId": project.id }));
    assert!(draft.is_none());

    harness.invoke_ok::<()>(
        "start_plan",
        json!({ "args": {
            "projectId": project.id,
            "projectDir": harness.work_dir,
            "agent": "codex",
            "initialPrompt": "Build invoke handler contract coverage"
        }}),
    );
    let status: Option<serde_json::Value> =
        harness.invoke_ok("query_plan_status", json!({ "projectId": project.id }));
    assert!(status.is_some());
    harness.wait_for_plan_idle(&project.id).await;
    let plan: Option<String> = harness.invoke_ok("load_existing_plan", json!({ "projectId": project.id }));
    assert!(plan.is_some());

    let prd: Prd = harness.invoke_ok(
        "run_atomizer",
        json!({ "args": {
            "projectId": project.id,
            "projectName": project.name,
            "projectDir": harness.work_dir,
            "agent": harness.atomizer_agent
        }}),
    );
    assert!(!prd.stories.is_empty());
    let loop_prd = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../crates/ralph-core/tests/fixtures/happy_path_prd.json"),
    )
    .unwrap();
    crate::projects::documents::save_prd(
        harness.app.handle().clone(),
        project.id.clone(),
        loop_prd,
    )
    .await
    .expect("save prd");

    let config = ProjectConfig {
        execute_agent: harness.loop_agent.clone(),
        ..ProjectConfig::default()
    };
    harness.invoke_ok::<()>(
        "save_config",
        json!({ "projectId": project.id, "configJson": serde_json::to_string(&config).unwrap() }),
    );

    let session_id: String =
        harness.invoke_ok("start_loop", json!({ "args": { "projectId": project.id } }));
    assert!(!session_id.is_empty());
    harness.wait_for_completion(&project.id).await;

    let detail: ProjectDetail =
        harness.invoke_ok("get_project_detail", json!({ "projectId": project.id }));
    assert_eq!(detail.project.status, "completed");
    assert!(detail.passed_count >= 1);
    let iterations: Vec<serde_json::Value> =
        harness.invoke_ok("get_iteration_history", json!({ "projectId": project.id }));
    assert!(!iterations.is_empty());

    harness.invoke_ok::<()>("archive_project", json!({ "projectId": project.id }));
    let archived: ProjectDetail =
        harness.invoke_ok("get_project_detail", json!({ "projectId": project.id }));
    assert_eq!(archived.project.status, "archived");
}

#[tokio::test(flavor = "current_thread")]
async fn invoke_handler_surfaces_deterministic_invoke_errors() {
    let planning = InvokeHarness::new(Some("bogus"));
    let project = create_project(&planning, "Planning Error");
    let plan_error = planning.invoke_err(
        "start_plan",
        json!({ "args": {
            "projectId": project.id,
            "projectDir": planning.work_dir,
            "agent": "codex",
            "initialPrompt": "Trigger planning failure"
        }}),
    );
    assert!(plan_error.contains("Config error"));
    assert!(plan_error.contains("unknown fixture set 'bogus'"));
    drop(planning);

    let atomizer = InvokeHarness::new(None);
    let project = create_project(&atomizer, "Atomizer Error");
    atomizer.invoke_ok::<()>(
        "save_plan",
        json!({ "projectId": project.id, "content": "# Broken\n\nTrigger chunk parse failure.\n" }),
    );
    let atomizer_error = atomizer.invoke_err(
        "run_atomizer",
        json!({ "args": {
            "projectId": project.id,
            "projectName": project.name,
            "projectDir": atomizer.work_dir,
            "agent": atomizer.broken_atomizer_agent
        }}),
    );
    assert!(atomizer_error.contains("JSON parse error at stage chunk"));
}

fn create_project(harness: &InvokeHarness, name: &str) -> Project {
    harness.invoke_ok(
        "create_project",
        json!({
            "name": name,
            "description": "Exercise invoke handler contracts",
            "workingDirectory": harness.work_dir,
            "wizardStep": "describe"
        }),
    )
}
