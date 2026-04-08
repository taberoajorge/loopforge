use crate::atomizer::AtomizeProgress;
use tauri::{AppHandle, Emitter};

pub(super) fn emit_progress(app: &AppHandle, project_id: &str, stage: u8, stage_name: &str, message: &str) {
    let _ = app.emit(
        "atomization-progress",
        AtomizeProgress {
            stage,
            stage_name: stage_name.to_string(),
            message: message.to_string(),
            project_id: project_id.to_string(),
        },
    );
}
