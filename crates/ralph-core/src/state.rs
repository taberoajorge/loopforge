use crate::logger;
use std::path::Path;
use tokio::time::{sleep, Duration};

pub fn is_paused(pause_file: &Path) -> bool {
    pause_file.exists()
}

pub fn is_done(done_file: &Path) -> bool {
    done_file.exists()
}

pub fn check_and_clear_done(done_file: &Path) -> bool {
    if done_file.exists() {
        logger::log_success(".ralph-done detected, finishing");
        if let Err(remove_err) = std::fs::remove_file(done_file) {
            tracing::warn!(
                path = %done_file.display(),
                error = %remove_err,
                "failed to remove done file"
            );
        }
        return true;
    }
    false
}

pub async fn wait_while_paused(pause_file: &Path) {
    if !pause_file.exists() {
        return;
    }
    logger::log_warning("PAUSED: Remove .ralph-pause to continue");
    while pause_file.exists() {
        sleep(Duration::from_secs(5)).await;
    }
    logger::log_success("Resuming...");
}

pub fn save_state(state_file: &Path, content: &str) {
    if let Err(write_err) = crate::atomic_write::atomic_write(state_file, content.as_bytes()) {
        tracing::warn!(
            path = %state_file.display(),
            error = %write_err,
            "failed to save state"
        );
    }
}

pub fn clear_state(state_file: &Path) {
    if state_file.exists() {
        if let Err(remove_err) = std::fs::remove_file(state_file) {
            tracing::warn!(
                path = %state_file.display(),
                error = %remove_err,
                "failed to clear state file"
            );
        }
    }
}

pub fn reset_all(state_file: &Path, pause_file: &Path, done_file: &Path) {
    for path in [state_file, pause_file, done_file] {
        if path.exists() {
            if let Err(remove_err) = std::fs::remove_file(path) {
                tracing::warn!(
                    path = %path.display(),
                    error = %remove_err,
                    "failed to remove file during reset"
                );
            }
        }
    }
    logger::log_success("State reset");
}

pub async fn wait_with_countdown(seconds: u64, reason: &str) {
    tracing::warn!(
        duration = %logger::format_duration(seconds),
        reason,
        "Waiting"
    );

    let mut remaining = seconds;
    while remaining > 0 {
        let chunk = if remaining > 60 { 60 } else { 1 };
        tracing::debug!(remaining = %logger::format_duration(remaining), "countdown");
        sleep(Duration::from_secs(chunk)).await;
        remaining = remaining.saturating_sub(chunk);
    }
    tracing::info!("Wait complete, continuing");
}
