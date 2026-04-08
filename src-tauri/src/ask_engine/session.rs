use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri_plugin_shell::process::CommandChild;

#[derive(Debug, Default)]
pub struct AskSessions {
    pub(crate) sessions: HashMap<String, CommandChild>,
}

#[derive(Debug, Clone, Default)]
pub struct AskSessionsState(pub Arc<Mutex<AskSessions>>);

impl AskSessionsState {
    pub fn insert(&self, project_id: &str, child: CommandChild) -> Result<(), String> {
        let mut sessions = self.0.lock().map_err(|_| "Ask session lock poisoned".to_string())?;
        sessions.sessions.insert(project_id.to_string(), child);
        Ok(())
    }

    pub fn remove_and_kill(&self, project_id: &str) -> Result<bool, String> {
        let mut sessions = self.0.lock().map_err(|_| "Ask session lock poisoned".to_string())?;
        if let Some(child) = sessions.sessions.remove(project_id) {
            let _ = child.kill();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn has_active(&self, project_id: &str) -> bool {
        self.0
            .lock()
            .map(|sessions| sessions.sessions.contains_key(project_id))
            .unwrap_or(false)
    }

    pub fn kill_all(&self) {
        if let Ok(mut sessions) = self.0.lock() {
            let ids: Vec<String> = sessions.sessions.keys().cloned().collect();
            for project_id in ids {
                if let Some(child) = sessions.sessions.remove(&project_id) {
                    let _ = child.kill();
                }
            }
        }
    }
}
