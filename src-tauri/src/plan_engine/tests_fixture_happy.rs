use super::payloads::{PlanActivityBatchPayload, PlanTerminalPayload};
use super::tests_fixture_support::{test_app, wait_for, EnvGuard};
use super::{
    query_plan_status, start_plan, stop_plan, write_to_plan, PlanSessionsState, StartPlanArgs,
};
use std::sync::{Arc, Mutex};
use tauri::{Listener, Manager};

#[tokio::test(flavor = "current_thread")]
async fn happy_path_fixture_emits_deterministic_plan_batches() {
    let guard = EnvGuard::new("happy-path");
    let app = test_app();
    let batches = Arc::new(Mutex::new(Vec::<PlanActivityBatchPayload>::new()));
    let completes = Arc::new(Mutex::new(Vec::<PlanTerminalPayload>::new()));
    let errors = Arc::new(Mutex::new(Vec::<PlanTerminalPayload>::new()));

    app.listen_any(crate::events::EVENT_PLAN_ACTIVITY_BATCH, {
        let batches = Arc::clone(&batches);
        move |event: tauri::Event| {
            batches
                .lock()
                .unwrap()
                .push(serde_json::from_str(event.payload()).unwrap())
        }
    });
    app.listen_any(crate::events::EVENT_PLAN_COMPLETE, {
        let completes = Arc::clone(&completes);
        move |event: tauri::Event| {
            completes
                .lock()
                .unwrap()
                .push(serde_json::from_str(event.payload()).unwrap())
        }
    });
    app.listen_any(crate::events::EVENT_PLAN_ERROR, {
        let errors = Arc::clone(&errors);
        move |event: tauri::Event| {
            errors
                .lock()
                .unwrap()
                .push(serde_json::from_str(event.payload()).unwrap())
        }
    });

    start_plan(
        app.handle().clone(),
        app.state::<PlanSessionsState>(),
        StartPlanArgs {
            project_id: "project-001".to_string(),
            project_dir: guard.work_dir(),
            agent: "missing-agent".to_string(),
            model: None,
            effort: None,
            initial_prompt: "Build a deterministic plan stream".to_string(),
        },
    )
    .await
    .expect("fixture start");

    let status = query_plan_status(app.state::<PlanSessionsState>(), "project-001".to_string())
        .await
        .unwrap()
        .expect("fixture session");
    assert_eq!(status.agent_name, "missing-agent");
    write_to_plan(
        app.state::<PlanSessionsState>(),
        "project-001".to_string(),
        "Refine the scope".to_string(),
    )
    .await
    .expect("fixture write");

    wait_for(|| !completes.lock().unwrap().is_empty()).await;

    assert!(errors.lock().unwrap().is_empty());
    assert_eq!(batches.lock().unwrap().len(), 3);
    let flattened: Vec<(String, String, String)> = batches
        .lock()
        .unwrap()
        .iter()
        .flat_map(|batch| batch.events.iter())
        .map(|event| {
            (
                format!("{:?}", event.kind),
                event.content.clone(),
                event.timestamp.clone(),
            )
        })
        .collect();
    assert_eq!(
        flattened[0],
        (
            "Thinking".to_string(),
            "Inspecting planning fixture inputs".to_string(),
            "2026-04-09T10:00:00.000Z".to_string()
        )
    );
    assert_eq!(
        flattened[6],
        (
            "PlanContent".to_string(),
            "3. Preserve write and stop session behavior.".to_string(),
            "2026-04-09T10:00:06.000Z".to_string()
        )
    );
    assert_eq!(batches.lock().unwrap()[2].plan_content_delta, "# Fixture Planning Summary\n\n1. Confirm deterministic test runtime seams.\n2. Emit stable planning activity batches.\n3. Preserve write and stop session behavior.");
    let plan_path = crate::storage::artifacts::project_artifact_dir(app.handle(), "project-001")
        .unwrap()
        .join("plan.md");
    assert_eq!(
        std::fs::read_to_string(plan_path).unwrap(),
        batches.lock().unwrap()[2].plan_content_delta
    );
    assert!(
        query_plan_status(app.state::<PlanSessionsState>(), "project-001".to_string())
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test(flavor = "current_thread")]
async fn stop_plan_cancels_fixture_session_before_terminal_event() {
    let guard = EnvGuard::new("happy-path");
    let app = test_app();
    let completes = Arc::new(Mutex::new(Vec::<PlanTerminalPayload>::new()));
    let errors = Arc::new(Mutex::new(Vec::<PlanTerminalPayload>::new()));

    app.listen_any(crate::events::EVENT_PLAN_COMPLETE, {
        let completes = Arc::clone(&completes);
        move |event: tauri::Event| {
            completes
                .lock()
                .unwrap()
                .push(serde_json::from_str(event.payload()).unwrap())
        }
    });
    app.listen_any(crate::events::EVENT_PLAN_ERROR, {
        let errors = Arc::clone(&errors);
        move |event: tauri::Event| {
            errors
                .lock()
                .unwrap()
                .push(serde_json::from_str(event.payload()).unwrap())
        }
    });

    start_plan(
        app.handle().clone(),
        app.state::<PlanSessionsState>(),
        StartPlanArgs {
            project_id: "project-003".to_string(),
            project_dir: guard.work_dir(),
            agent: "missing-agent".to_string(),
            model: None,
            effort: None,
            initial_prompt: "Stop early".to_string(),
        },
    )
    .await
    .expect("fixture start");
    stop_plan(app.state::<PlanSessionsState>(), "project-003".to_string())
        .await
        .expect("fixture stop");
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;

    assert!(completes.lock().unwrap().is_empty());
    assert!(errors.lock().unwrap().is_empty());
    assert!(
        query_plan_status(app.state::<PlanSessionsState>(), "project-003".to_string())
            .await
            .unwrap()
            .is_none()
    );
}
