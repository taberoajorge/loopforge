use super::invoke_fixtures::FixtureGuard;
use crate::db::DbState;
use crate::invoke;
use crate::loop_manager::LoopManagerState;
use crate::plan_engine::PlanSessionsState;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::path::PathBuf;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{
    get_ipc_response, mock_builder, mock_context, noop_assets, MockRuntime, INVOKE_KEY,
};
use tauri::{App, Manager, WebviewWindow, WebviewWindowBuilder};

pub struct InvokeHarness {
    _guard: FixtureGuard,
    pub app: App<MockRuntime>,
    webview: WebviewWindow<MockRuntime>,
    pub work_dir: PathBuf,
    pub loop_agent: String,
    pub atomizer_agent: String,
    pub broken_atomizer_agent: String,
}

impl InvokeHarness {
    pub fn new(fixture_set: Option<&str>) -> Self {
        let guard = FixtureGuard::new(fixture_set);
        let builder = mock_builder()
            .plugin(tauri_plugin_shell::init())
            .manage(crate::agents::AgentRegistryState::default())
            .manage(PlanSessionsState::default())
            .manage(LoopManagerState::default())
            .manage(crate::ask_engine::session::AskSessionsState::default());
        let app = invoke::attach_contract(builder)
            .build(mock_context(noop_assets()))
            .expect("test app");
        app.manage(DbState::open(app.handle()).expect("db state"));
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("webview");

        Self {
            work_dir: guard.work_dir.clone(),
            loop_agent: guard.loop_agent.clone(),
            atomizer_agent: guard.atomizer_agent.clone(),
            broken_atomizer_agent: guard.broken_atomizer_agent.clone(),
            _guard: guard,
            app,
            webview,
        }
    }

    pub fn invoke_ok<T: DeserializeOwned>(&self, cmd: &str, payload: Value) -> T {
        get_ipc_response(&self.webview, self.request(cmd, payload))
            .unwrap_or_else(|err| panic!("{cmd} failed: {}", error_text(err)))
            .deserialize::<T>()
            .unwrap_or_else(|err| panic!("{cmd} deserialize failed: {err}"))
    }

    pub fn invoke_err(&self, cmd: &str, payload: Value) -> String {
        let err = get_ipc_response(&self.webview, self.request(cmd, payload))
            .expect_err("invoke must fail");
        error_text(err)
    }

    pub fn artifact_dir(&self, project_id: &str) -> PathBuf {
        crate::storage::artifacts::project_artifact_dir(self.app.handle(), project_id)
            .expect("artifact dir")
    }

    pub async fn wait_for_plan_idle(&self, project_id: &str) {
        for _ in 0..100 {
            let status: Option<serde_json::Value> = self.invoke_ok(
                "query_plan_status",
                serde_json::json!({ "projectId": project_id }),
            );
            if status.is_none() && self.artifact_dir(project_id).join("plan.md").exists() {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        panic!("plan session did not settle for {project_id}");
    }

    pub async fn wait_for_completion(&self, project_id: &str) {
        for _ in 0..100 {
            let status = {
                let db = self.app.state::<DbState>();
                let conn = db.0.lock().expect("db lock");
                conn.query_row(
                    "SELECT status FROM projects WHERE id = ?1",
                    rusqlite::params![project_id],
                    |row: &rusqlite::Row| row.get::<_, String>(0),
                )
                .expect("project status")
            };
            if status == "completed" {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        panic!("loop did not complete for {project_id}");
    }

    fn request(&self, cmd: &str, payload: Value) -> tauri::webview::InvokeRequest {
        tauri::webview::InvokeRequest {
            cmd: cmd.to_string(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "http://tauri.localhost".parse().expect("invoke url"),
            body: InvokeBody::Json(payload),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        }
    }
}

fn error_text(value: Value) -> String {
    match value {
        Value::String(text) => text,
        other => other.to_string(),
    }
}
