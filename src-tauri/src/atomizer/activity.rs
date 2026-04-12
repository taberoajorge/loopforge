use crate::atomizer::types::{AtomizeActivity, AtomizeActivityKind};
use crate::events::EVENT_ATOMIZATION_ACTIVITY;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, Runtime};

const MAX_LOG_ENTRIES: usize = 300;

#[derive(Debug, Default)]
pub struct ActivityLog {
    entries: HashMap<String, VecDeque<AtomizeActivity>>,
}

impl ActivityLog {
    fn push(&mut self, entry: AtomizeActivity) {
        let ring = self
            .entries
            .entry(entry.project_id.clone())
            .or_insert_with(|| VecDeque::with_capacity(MAX_LOG_ENTRIES));
        if ring.len() >= MAX_LOG_ENTRIES {
            ring.pop_front();
        }
        ring.push_back(entry);
    }

    pub fn get(&self, project_id: &str) -> Vec<AtomizeActivity> {
        self.entries
            .get(project_id)
            .map(|ring| ring.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ActivityLogState(pub Arc<Mutex<ActivityLog>>);

pub(super) fn emit_activity<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
    kind: AtomizeActivityKind,
    content: &str,
) {
    let entry = AtomizeActivity {
        project_id: project_id.to_string(),
        kind,
        content: content.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    };

    if let Ok(mut log) = app.state::<ActivityLogState>().0.lock() {
        log.push(entry.clone());
    }

    let _ = app.emit(EVENT_ATOMIZATION_ACTIVITY, entry);
}
