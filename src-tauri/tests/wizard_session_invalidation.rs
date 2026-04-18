include!("../src/lib.rs");

use serde_json::json;
use std::path::Path;

mod shared_harness {
    include!("../src/contract_tests/harness.rs");
}

use shared_harness::TestHarness;

fn seed_descendants(artifact_dir: &Path, work_dir: &Path) {
    let prd = json!({
        "projectName": "Persisted",
        "generatedAt": "2026-04-18T12:00:00Z",
        "stories": [{ "id": "S-001", "title": "Persisted story", "acceptanceCriteria": ["Works"] }]
    });
    std::fs::write(artifact_dir.join("plan.md"), "# Persisted plan").expect("artifact plan");
    std::fs::write(artifact_dir.join("prd.json"), prd.to_string()).expect("artifact prd");
    std::fs::write(
        artifact_dir.join("config.json"),
        json!({ "executeAgent": "claude", "maxIterations": 25 }).to_string(),
    )
    .expect("artifact config");
    std::fs::write(artifact_dir.join("prompt.md"), "Persisted prompt").expect("artifact prompt");
    std::fs::write(artifact_dir.join("guardrails.md"), "Persisted guardrails")
        .expect("artifact guardrails");
    std::fs::write(work_dir.join("plan.md"), "# Legacy plan").expect("legacy plan");
    std::fs::write(work_dir.join("prd.json"), prd.to_string()).expect("legacy prd");
    std::fs::write(
        work_dir.join("config.json"),
        json!({ "executeAgent": "gemini", "maxIterations": 50 }).to_string(),
    )
    .expect("legacy config");
    std::fs::write(work_dir.join("prompt.md"), "Legacy prompt").expect("legacy prompt");
    std::fs::write(work_dir.join("guardrails.md"), "Legacy guardrails").expect("legacy guardrails");
}

async fn create_project(harness: &TestHarness, name: &str) -> projects::Project {
    projects::catalog::create_project(
        harness.app.handle().clone(),
        harness.app.state::<DbState>(),
        name.to_string(),
        format!("{name} description"),
        harness.work_dir.to_string_lossy().to_string(),
        Some("launch".to_string()),
    )
    .await
    .expect("project")
}

#[tokio::test(flavor = "current_thread")]
async fn wizard_session_invalidation_clears_all_descendants_after_description_change() {
    let harness = TestHarness::new();
    let project = create_project(&harness, "Description Reset").await;
    let artifact_dir = harness.artifact_dir(&project.id);
    seed_descendants(&artifact_dir, &harness.work_dir);
    std::fs::write(
        artifact_dir.join("draft.json"),
        json!({
            "projectId": project.id, "currentStep": "launch", "highestStep": 5,
            "describe": { "name": project.name, "description": "Old description", "workingDirectory": harness.work_dir, "planAgent": "claude" },
            "plan": { "completed": true, "document": "# Persisted plan" },
            "atomize": { "storiesCount": 1, "stories": [{ "id": "S-001", "title": "Persisted story", "acceptanceCriteria": ["Works"] }] },
            "configure": { "executeAgent": "claude", "maxIterations": 25 },
            "prompt": "Persisted prompt", "guardrails": "Persisted guardrails"
        })
        .to_string(),
    )
    .expect("previous draft");

    let saved = projects::runtime_config::wizard_session_adapter::save_canonical_session(
        harness.app.handle(),
        &project,
        &json!({
            "projectId": project.id, "currentStep": "launch", "highestStep": 5,
            "describe": { "name": project.name, "description": "New description", "workingDirectory": harness.work_dir, "planAgent": "claude" },
            "plan": { "completed": true, "document": "# Stale plan" },
            "atomize": { "storiesCount": 1, "stories": [{ "id": "S-001", "title": "Stale story", "acceptanceCriteria": ["Works"] }] },
            "configure": { "executeAgent": "codex", "maxIterations": 40 },
            "prompt": "Stale prompt", "guardrails": "Stale guardrails"
        })
        .to_string(),
        None,
    )
    .expect("save session");

    assert_eq!(saved.stale_from_step, Some(2));
    assert!(!saved.plan.completed);
    assert!(saved.atomize.stories.is_empty());
    assert!(saved.prompt.is_none());
    assert!(saved.guardrails.is_none());
    for file_name in ["plan.md", "prd.json", "config.json", "prompt.md", "guardrails.md"] {
        assert!(!artifact_dir.join(file_name).exists(), "{file_name} should be removed");
    }

    let session = projects::runtime_config::wizard_session_adapter::hydrate_canonical_session(
        harness.app.handle(),
        &project,
        None,
    )
    .expect("hydrate");

    assert!(artifact_dir.join("draft.json").exists());
    assert_eq!(session.project_data.description, "New description");
    assert!(session.plan.is_none());
    assert!(!session.plan_complete);
    assert!(session.stories.is_empty());
    assert!(session.prompt.is_none());
    assert!(session.guardrails.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn wizard_session_invalidation_preserves_plan_and_prd_after_config_change() {
    let harness = TestHarness::new();
    let project = create_project(&harness, "Config Reset").await;
    let artifact_dir = harness.artifact_dir(&project.id);
    seed_descendants(&artifact_dir, &harness.work_dir);
    std::fs::write(
        artifact_dir.join("draft.json"),
        json!({
            "projectId": project.id, "currentStep": "launch", "highestStep": 5,
            "describe": { "name": project.name, "description": project.description, "workingDirectory": harness.work_dir, "planAgent": "claude" },
            "plan": { "completed": true }, "atomize": { "storiesCount": 1 },
            "configure": { "executeAgent": "claude", "maxIterations": 25 }
        })
        .to_string(),
    )
    .expect("previous draft");

    let saved = projects::runtime_config::wizard_session_adapter::save_canonical_session(
        harness.app.handle(),
        &project,
        &json!({
            "projectId": project.id, "currentStep": "launch", "highestStep": 5,
            "describe": { "name": project.name, "description": project.description, "workingDirectory": harness.work_dir, "planAgent": "claude" },
            "plan": { "completed": true }, "atomize": { "storiesCount": 1 },
            "configure": { "executeAgent": "codex", "maxIterations": 40 },
            "prompt": "Stale prompt", "guardrails": "Stale guardrails"
        })
        .to_string(),
        None,
    )
    .expect("save session");

    assert_eq!(saved.stale_from_step, Some(4));
    assert!(artifact_dir.join("plan.md").exists());
    assert!(artifact_dir.join("prd.json").exists());
    for file_name in ["config.json", "prompt.md", "guardrails.md"] {
        assert!(!artifact_dir.join(file_name).exists(), "{file_name} should be removed");
    }

    let session = projects::runtime_config::wizard_session_adapter::hydrate_canonical_session(
        harness.app.handle(),
        &project,
        None,
    )
    .expect("hydrate");

    assert_eq!(session.plan.as_deref(), Some("# Persisted plan"));
    assert_eq!(session.stories.len(), 1);
    assert_eq!(
        session.config.as_ref().map(|config| config.execute_agent.as_str()),
        Some("codex")
    );
    assert!(session.prompt.is_none());
    assert!(session.guardrails.is_none());
}
