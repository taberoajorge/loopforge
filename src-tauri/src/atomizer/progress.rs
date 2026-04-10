use crate::atomizer::AtomizeProgress;
use tauri::{AppHandle, Emitter, Runtime};

pub(super) fn emit_progress<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
    stage: u8,
    stage_name: &str,
    message: &str,
) {
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
