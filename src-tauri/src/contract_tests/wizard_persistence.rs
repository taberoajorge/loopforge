use super::super::harness::TestHarness;
use crate::db::DbState;
use serde_json::json;
use tauri::Manager;

#[tokio::test(flavor = "current_thread")]
async fn db_snapshot_matches_canonical_draft_json() {
    let harness = TestHarness::new();
    let project = crate::projects::catalog::create_project(
        harness.app.handle().clone(),
        harness.app.state::<DbState>(),
        "Wizard Persistence".to_string(),
        "Normalize wizard payloads".to_string(),
        harness.work_dir.to_string_lossy().to_string(),
        Some("describe".to_string()),
    )
    .await
    .expect("project");
    let canonical = json!({
        "version": 1,
        "projectId": project.id,
        "currentStep": "configure",
        "describe": {
            "name": "Wizard Persistence",
            "description": "Normalize wizard payloads",
            "workingDirectory": harness.work_dir.to_string_lossy()
        },
        "plan": { "completed": true },
        "atomize": { "storiesCount": 2 },
        "configure": { "executeAgent": "codex", "fallbackChain": ["claude"] }
    });
    let db_payload = json!({
        "version": 1,
        "projectId": project.id,
        "describe": {
            "name": "Wizard Persistence",
            "description": "Normalize wizard payloads",
            "workingDirectory": harness.work_dir.to_string_lossy()
        },
        "plan": { "completed": true },
        "atomize": { "storiesCount": 2 },
        "configure": { "executeAgent": "codex", "fallbackChain": ["claude"] }
    });

    crate::projects::wizard::save_wizard_state(
        harness.app.state::<DbState>(),
        project.id.clone(),
        "configure".to_string(),
        db_payload.to_string(),
    )
    .await
    .expect("save wizard state");

    let db_snapshot: String = {
        let db = harness.app.state::<DbState>();
        let conn = db.0.lock().expect("db lock");
        conn.query_row(
            "SELECT wizard_state_json FROM projects WHERE id = ?1",
            rusqlite::params![project.id.clone()],
            |row: &rusqlite::Row| row.get(0),
        )
        .expect("wizard state json")
    };

    crate::projects::wizard::save_draft(
        harness.app.handle().clone(),
        project.id.clone(),
        canonical.to_string(),
    )
    .await
    .expect("save draft");

    let draft_json = std::fs::read_to_string(harness.artifact_dir(&project.id).join("draft.json"))
        .expect("draft json");

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&db_snapshot).expect("db value"),
        serde_json::from_str::<serde_json::Value>(&draft_json).expect("draft value")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn resume_wizard_succeeds_with_canonical_draft_payload() {
    let harness = TestHarness::new();
    let working_directory = harness.work_dir.to_string_lossy().to_string();
    let project = crate::projects::catalog::create_project(
        harness.app.handle().clone(),
        harness.app.state::<DbState>(),
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
        "highestStep": 4,
        "staleFromStep": serde_json::Value::Null,
        "describe": {
            "name": "Roundtrip Project",
            "description": "Persist wizard draft values",
            "workingDirectory": working_directory,
            "planAgent": "codex",
            "planModel": serde_json::Value::Null,
            "planEffort": "high"
        },
        "plan": { "completed": true },
        "atomize": { "stories": [], "storiesCount": 3 },
        "configure": {
            "executeAgent": "codex",
            "executeModel": serde_json::Value::Null,
            "executeEffort": "medium",
            "fallbackChain": ["claude", "gemini"],
            "gutterThreshold": 4,
            "maxIterations": 25,
            "cooldownSeconds": 15,
            "testCommand": "cargo test wizard_draft_roundtrip",
            "maxVerificationRetries": 2,
            "schemaVersion": 1,
            "scmProvider": "auto",
            "reviewPollingInterval": 90,
            "reviewTimeout": 900
        }
    });

    crate::projects::wizard::save_draft(
        harness.app.handle().clone(),
        project.id.clone(),
        draft.to_string(),
    )
    .await
    .expect("save draft");

    let resume_state = crate::projects::wizard::resume_wizard(
        harness.app.handle().clone(),
        harness.app.state::<DbState>(),
        project.id.clone(),
    )
    .await
    .expect("resume wizard");
    let loaded_draft =
        crate::projects::wizard::load_draft(harness.app.handle().clone(), project.id)
            .await
            .expect("load draft")
            .expect("draft content");

    assert_eq!(resume_state.wizard_step, "configure");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&loaded_draft).expect("draft json"),
        draft
    );
}
