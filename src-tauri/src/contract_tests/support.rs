use super::harness::TestHarness;
use crate::ask_engine::types::StartAskArgs;
use crate::db::DbState;
use crate::models::ProjectStatus;
use rusqlite::OptionalExtension;
use tauri::Manager;
use uuid::Uuid;

pub async fn seed_project(harness: &TestHarness) -> crate::projects::Project {
    let project = crate::projects::catalog::create_project(
        harness.app.handle().clone(),
        harness.app.state::<DbState>(),
        "Artifact Contract".to_string(),
        "Exercise artifact persistence".to_string(),
        harness.work_dir.to_string_lossy().to_string(),
        Some("describe".to_string()),
    )
    .await
    .expect("project");
    crate::projects::wizard::save_draft(
        harness.app.handle().clone(),
        project.id.clone(),
        serde_json::json!({"currentStep": "describe", "description": "Contract draft"}).to_string(),
    )
    .await
    .expect("save draft");
    crate::projects::documents::save_plan(
        harness.app.handle().clone(),
        project.id.clone(),
        "# Contract Plan\n\n- Persist artifacts\n- Verify histories\n".to_string(),
    )
    .await
    .expect("save plan");
    crate::projects::documents::save_prd(
        harness.app.handle().clone(),
        project.id.clone(),
        std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../crates/ralph-core/tests/fixtures/happy_path_prd.json"),
        )
        .unwrap(),
    )
    .await
    .expect("save prd");
    crate::projects::documents::save_config(
        harness.app.handle().clone(),
        project.id.clone(),
        serde_json::json!({
            "schemaVersion": 1,
            "executeAgent": harness.agent_name,
            "executeModel": null,
            "executeEffort": null,
            "fallbackChain": [],
            "gutterThreshold": 3,
            "maxIterations": 5,
            "cooldownSeconds": 0,
            "testCommand": "",
            "maxVerificationRetries": 1,
            "scmProvider": "auto",
            "reviewPollingInterval": 60,
            "reviewTimeout": 600
        })
        .to_string(),
    )
    .await
    .expect("save config");
    project
}

pub async fn project_detail(
    harness: &TestHarness,
    project_id: &str,
) -> crate::projects::ProjectDetail {
    crate::projects::catalog::get_project_detail(
        harness.app.handle().clone(),
        harness.app.state::<DbState>(),
        project_id.to_string(),
    )
    .await
    .expect("project detail")
}

pub async fn load_config(harness: &TestHarness, project_id: &str) -> Option<String> {
    crate::projects::documents::load_config(harness.app.handle().clone(), project_id.to_string())
        .await
        .expect("load config")
}

pub fn infer_status(
    raw_status: &str,
    total_stories: usize,
    has_config: bool,
    has_loop_handle: bool,
) -> ProjectStatus {
    let mut status = ProjectStatus::from_db_status(raw_status);
    if has_loop_handle {
        status = ProjectStatus::Running;
    } else if matches!(status, ProjectStatus::Running) {
        status = ProjectStatus::Paused;
    }
    if matches!(status, ProjectStatus::Draft) && total_stories > 0 && has_config {
        status = ProjectStatus::Ready;
    }
    status
}

pub fn session_ended_at(harness: &TestHarness, project_id: &str) -> Option<String> {
    let db = harness.app.state::<DbState>();
    let conn = db.0.lock().unwrap();
    conn.query_row(
        "SELECT ended_at FROM sessions WHERE project_id = ?1 ORDER BY started_at DESC LIMIT 1",
        rusqlite::params![project_id],
        |row| row.get(0),
    )
    .optional()
    .unwrap()
    .flatten()
}

pub async fn start_ask(harness: &TestHarness, project_id: &str, question: &str) {
    let project_dir = {
        let db = harness.app.state::<DbState>();
        let conn = db.0.lock().unwrap();
        let conversation =
            crate::ask_engine::storage::get_or_create_conversation(&conn, project_id).unwrap();
        crate::ask_engine::storage::insert_message(
            &conn,
            &conversation.id,
            "user",
            question,
            None,
            None,
        )
        .unwrap();
        let dir: String = conn
            .query_row(
                "SELECT working_directory FROM projects WHERE id = ?1",
                rusqlite::params![project_id],
                |row| row.get(0),
            )
            .unwrap();
        std::path::PathBuf::from(dir)
    };
    crate::ask_engine::stream::spawn_ask(
        harness.app.handle().clone(),
        harness
            .app
            .state::<crate::ask_engine::session::AskSessionsState>()
            .inner()
            .clone(),
        StartAskArgs {
            project_id: project_id.to_string(),
            question: question.to_string(),
            agent: harness.agent_name.clone(),
            model: None,
        },
        Uuid::new_v4().to_string(),
        project_dir,
    )
    .await
    .expect("ask question");
}
