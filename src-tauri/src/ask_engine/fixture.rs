use crate::ask_engine::types::{AskCompletePayload, AskStreamPayload};
use crate::events::{EVENT_ASK_COMPLETE, EVENT_ASK_STREAM};
use crate::test_support::TestRuntime;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use tauri::{AppHandle, Emitter};

static ASK_CALL_COUNT: AtomicU32 = AtomicU32::new(0);

pub fn reset_call_count() {
    ASK_CALL_COUNT.store(0, Ordering::SeqCst);
}

pub async fn spawn_fixture_ask(
    app: AppHandle,
    project_id: String,
    message_id: String,
    question: &str,
    runtime: &TestRuntime,
) {
    let idx = ASK_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
    let response = load_fixture_response(&runtime.dialog_dir, idx, question);

    for chunk in split_chunks(&response) {
        let _ = app.emit(
            EVENT_ASK_STREAM,
            AskStreamPayload {
                project_id: project_id.clone(),
                message_id: message_id.clone(),
                chunk: chunk.to_string(),
            },
        );
    }

    persist_interaction(&runtime.data_dir, idx, question, &response);

    let _ = app.emit(
        EVENT_ASK_COMPLETE,
        AskCompletePayload {
            project_id,
            message_id,
            full_content: response,
            agent: "fixture".to_string(),
            model: Some("deterministic".to_string()),
        },
    );
}

fn load_fixture_response(dialog_dir: &Path, idx: u32, _question: &str) -> String {
    let specific = dialog_dir.join(format!("ask_{idx}.txt"));
    if specific.exists() {
        return std::fs::read_to_string(&specific).unwrap_or_else(|_| default_response(idx));
    }

    let fallback = dialog_dir.join("ask_default.txt");
    if fallback.exists() {
        return std::fs::read_to_string(&fallback).unwrap_or_else(|_| default_response(idx));
    }

    default_response(idx)
}

fn default_response(idx: u32) -> String {
    format!("fixture: ask response {idx}")
}

fn split_chunks(content: &str) -> Vec<&str> {
    let mid = content.len() / 2;
    if mid == 0 {
        return vec![content];
    }
    vec![&content[..mid], &content[mid..]]
}

fn persist_interaction(data_dir: &Path, idx: u32, question: &str, response: &str) {
    let history_dir = data_dir.join("ask_history");
    let _ = std::fs::create_dir_all(&history_dir);

    let entry = format!(
        "---\nindex: {idx}\n---\n## Question\n{question}\n\n## Response\n{response}\n"
    );
    let path = history_dir.join(format!("ask_{idx}.md"));
    let _ = std::fs::write(&path, entry);
}
