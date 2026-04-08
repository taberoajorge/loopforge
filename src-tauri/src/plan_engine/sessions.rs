use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri_plugin_shell::process::CommandChild;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPlanArgs {
    pub project_id: String,
    pub project_dir: PathBuf,
    pub agent: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub effort: Option<String>,
    pub initial_prompt: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PlanSessionStatus {
    Running,
    Stalled,
}

pub struct PlanSessionEntry {
    pub child: CommandChild,
    pub status: PlanSessionStatus,
    pub agent_name: String,
    pub started_at: Instant,
    pub last_activity_at: Arc<Mutex<Instant>>,
}

impl std::fmt::Debug for PlanSessionEntry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PlanSessionEntry")
            .field("status", &self.status)
            .field("agent_name", &self.agent_name)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSessionInfo {
    pub status: PlanSessionStatus,
    pub agent_name: String,
    pub elapsed_secs: u64,
    pub last_activity_secs_ago: u64,
}

#[derive(Debug, Default)]
pub struct PlanSessions {
    pub(crate) sessions: HashMap<String, PlanSessionEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct PlanSessionsState(pub Arc<Mutex<PlanSessions>>);
