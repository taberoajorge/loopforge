mod codex;
mod command_args;
mod lifecycle;
mod runner;

#[cfg(test)]
mod tests;

use crate::db::DbState;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AgentOutputLine {
    pub(super) project_id: String,
    pub(super) session_id: String,
    pub(super) line: String,
    pub(super) stream: String,
}

pub(super) struct ShellProvider {
    pub(super) app: AppHandle,
    pub(super) agent_name: Arc<Mutex<String>>,
    pub(super) primary_agent: String,
    pub(super) selected_model: Option<String>,
    pub(super) selected_effort: Option<String>,
    pub(super) fallback_agents: Vec<String>,
    pub(super) fallback_index: Arc<Mutex<usize>>,
    pub(super) project_id: String,
    pub(super) project_name: String,
    pub(super) session_id: String,
    pub(super) iteration_counter: Arc<Mutex<u32>>,
}

impl ShellProvider {
    pub(super) fn current_agent(&self) -> String {
        self.agent_name
            .lock()
            .map_or_else(|err| err.into_inner().clone(), |guard| guard.clone())
    }

    pub(super) fn try_advance_fallback(&self) -> Option<String> {
        let mut index = self
            .fallback_index
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *index += 1;
        let agent = self.fallback_agents.get(*index)?.clone();
        if let Ok(mut name) = self.agent_name.lock() {
            *name = agent.clone();
        }
        Some(agent)
    }

    pub(super) fn insert_iteration(
        &self,
        story_id: &str,
        agent: &str,
        duration_secs: u64,
        result: &str,
    ) {
        let iteration_id = Uuid::new_v4().to_string();
        let db_ref = self.app.state::<DbState>();
        let lock = db_ref.0.lock();
        if let Ok(conn) = lock {
            let _ = conn.execute(
                "INSERT INTO iterations (id, session_id, story_id, duration_secs, result, agent_used)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    iteration_id,
                    self.session_id,
                    story_id,
                    duration_secs as i64,
                    result,
                    agent
                ],
            );
            let _ = conn.execute(
                "UPDATE sessions SET total_iterations = total_iterations + 1 WHERE id = ?1",
                rusqlite::params![self.session_id],
            );
            if result == "success" {
                let _ = conn.execute(
                    "UPDATE sessions SET stories_completed = stories_completed + 1 WHERE id = ?1",
                    rusqlite::params![self.session_id],
                );
            }
        }
    }
}
