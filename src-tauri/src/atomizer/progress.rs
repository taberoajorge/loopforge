use crate::atomizer::types::{PipelineSnapshot, StageStatus};
use crate::atomizer::AtomizeProgress;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager, Runtime};

#[derive(Debug)]
struct PipelineEntry {
    snapshot: PipelineSnapshot,
    started_instant: Instant,
}

#[derive(Debug, Default)]
pub struct PipelineRegistry {
    entries: HashMap<String, PipelineEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct PipelineRegistryState(pub Arc<Mutex<PipelineRegistry>>);

pub(super) fn mark_pipeline_start<R: Runtime>(app: &AppHandle<R>, project_id: &str) {
    let now_rfc = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let mut snapshot = PipelineSnapshot::initial();
    snapshot.started_at = Some(now_rfc);
    if let Ok(mut registry) = app.state::<PipelineRegistryState>().0.lock() {
        registry.entries.insert(
            project_id.to_string(),
            PipelineEntry {
                snapshot,
                started_instant: Instant::now(),
            },
        );
    }
}

pub(super) fn mark_pipeline_error<R: Runtime>(app: &AppHandle<R>, project_id: &str, error: &str) {
    if let Ok(mut registry) = app.state::<PipelineRegistryState>().0.lock() {
        if let Some(entry) = registry.entries.get_mut(project_id) {
            entry.snapshot.error = Some(error.to_string());
            for stage in &mut entry.snapshot.stages {
                if stage.status == StageStatus::Running {
                    stage.status = StageStatus::Error;
                }
            }
            entry.snapshot.elapsed_ms = entry.started_instant.elapsed().as_millis() as u64;
        }
    }
}

pub(super) fn mark_pipeline_done<R: Runtime>(app: &AppHandle<R>, project_id: &str) {
    if let Ok(mut registry) = app.state::<PipelineRegistryState>().0.lock() {
        if let Some(entry) = registry.entries.get_mut(project_id) {
            entry.snapshot.done = true;
            for stage in &mut entry.snapshot.stages {
                if stage.status != StageStatus::Error {
                    stage.status = StageStatus::Done;
                }
            }
            entry.snapshot.elapsed_ms = entry.started_instant.elapsed().as_millis() as u64;
        }
    }
}

pub fn get_pipeline_snapshot<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
) -> Option<PipelineSnapshot> {
    app.state::<PipelineRegistryState>()
        .0
        .lock()
        .ok()
        .and_then(|registry| {
            registry.entries.get(project_id).map(|entry| {
                let mut snap = entry.snapshot.clone();
                if !snap.done && snap.error.is_none() {
                    snap.elapsed_ms = entry.started_instant.elapsed().as_millis() as u64;
                }
                snap.recompute_derived();
                snap
            })
        })
}

pub(super) fn emit_progress<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
    stage: u8,
    stage_name: &str,
    message: &str,
) {
    if let Ok(mut registry) = app.state::<PipelineRegistryState>().0.lock() {
        if let Some(entry) = registry.entries.get_mut(project_id) {
            for snap_stage in &mut entry.snapshot.stages {
                if snap_stage.number == stage {
                    snap_stage.status = StageStatus::Running;
                } else if snap_stage.number < stage {
                    snap_stage.status = StageStatus::Done;
                }
            }
            entry.snapshot.elapsed_ms = entry.started_instant.elapsed().as_millis() as u64;
        }
    }

    let elapsed_ms = app
        .state::<PipelineRegistryState>()
        .0
        .lock()
        .ok()
        .and_then(|reg| {
            reg.entries
                .get(project_id)
                .map(|ent| ent.snapshot.elapsed_ms)
        })
        .unwrap_or(0);

    let _ = app.emit(
        "atomization-progress",
        AtomizeProgress {
            stage,
            stage_name: stage_name.to_string(),
            message: message.to_string(),
            project_id: project_id.to_string(),
            elapsed_ms,
        },
    );
}
