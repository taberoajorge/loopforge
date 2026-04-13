use super::payloads::PlanTerminalPayload;
use super::tests_fixture_support::{test_app, wait_for, EnvGuard};
use super::{query_plan_status, start_plan, PlanSessionsState, StartPlanArgs};
use crate::test_support::planning::plan_error_detail;
use std::sync::{Arc, Mutex};
use tauri::{Listener, Manager};

#[tokio::test(flavor = "current_thread")]
async fn plan_error_fixture_emits_deterministic_error_and_clears_session() {
    let guard = EnvGuard::new("plan-error");
    let app = test_app();
    let errors = Arc::new(Mutex::new(Vec::<PlanTerminalPayload>::new()));
    let completes = Arc::new(Mutex::new(Vec::<PlanTerminalPayload>::new()));

    app.listen_any(crate::events::EVENT_PLAN_ERROR, {
        let errors = Arc::clone(&errors);
        move |event: tauri::Event| {
            errors
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

    start_plan(
        app.handle().clone(),
        app.state::<PlanSessionsState>(),
        StartPlanArgs {
            project_id: "project-002".to_string(),
            project_dir: guard.work_dir(),
            agent: "missing-agent".to_string(),
            model: None,
            effort: None,
            initial_prompt: "Trigger fixture failure".to_string(),
        },
    )
    .await
    .expect("fixture start");

    wait_for(|| !errors.lock().unwrap().is_empty()).await;

    assert!(completes.lock().unwrap().is_empty());
    assert_eq!(errors.lock().unwrap()[0].detail, plan_error_detail());
    assert!(
        query_plan_status(app.state::<PlanSessionsState>(), "project-002".to_string())
            .await
            .unwrap()
            .is_none()
    );
}
