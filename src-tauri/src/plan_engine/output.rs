use crate::activity::{ActivityClassifier, PlanEventKind};
use crate::plan_engine::payloads::{PlanActivityBatchPayload, PlanActivityPayload};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter, Runtime};

pub(super) fn buffer_text_segments(
    text: &str,
    classifier: &Arc<Mutex<ActivityClassifier>>,
    buffer: &Arc<Mutex<Vec<PlanActivityPayload>>>,
    project_id: &str,
    last_activity: &Arc<Mutex<Instant>>,
) {
    for segment in text.lines() {
        let trimmed = segment.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(mut activity) = last_activity.lock() {
            *activity = Instant::now();
        }
        if let Ok(mut guard) = classifier.lock() {
            let plan_event = guard.classify(trimmed);
            if let Ok(mut buf) = buffer.lock() {
                buf.push(PlanActivityPayload {
                    project_id: project_id.to_string(),
                    kind: plan_event.kind,
                    content: plan_event.content,
                    timestamp: plan_event.timestamp,
                });
            }
        }
    }
}

pub(super) fn flush_event_buffer<R: Runtime>(
    buffer: &Arc<Mutex<Vec<PlanActivityPayload>>>,
    classifier: &Arc<Mutex<ActivityClassifier>>,
    app: &AppHandle<R>,
    project_id: &str,
) {
    let events: Vec<PlanActivityPayload> = {
        let mut buf = match buffer.lock() {
            Ok(buf) => buf,
            Err(_) => return,
        };
        buf.drain(..).collect()
    };
    if events.is_empty() {
        return;
    }
    let plan_content_delta: String = events
        .iter()
        .filter(|evt| evt.kind == PlanEventKind::PlanContent)
        .map(|evt| evt.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let plan_content = classifier
        .lock()
        .map(|guard| guard.accumulated_plan())
        .unwrap_or_default();
    let raw_batch = PlanActivityBatchPayload {
        project_id: project_id.to_string(),
        events,
        plan_content,
        plan_content_delta,
    };
    let filtered_batch = crate::plan_engine::filters::filter_plan_batch(raw_batch);
    if filtered_batch.events.is_empty() && filtered_batch.plan_content.is_empty() {
        return;
    }
    let _ = app.emit(crate::events::EVENT_PLAN_ACTIVITY_BATCH, filtered_batch);
}
