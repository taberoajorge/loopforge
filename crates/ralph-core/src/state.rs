use crate::logger;
use std::path::Path;
use tokio::time::{Duration, sleep};

pub fn is_paused(pause_file: &Path) -> bool {
    pause_file.exists()
}

pub fn is_done(done_file: &Path) -> bool {
    done_file.exists()
}

pub fn check_and_clear_done(done_file: &Path) -> bool {
    if done_file.exists() {
        logger::log_success(".ralph-done detected, finishing");
        let _ = std::fs::remove_file(done_file);
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
    let _ = crate::atomic_write::atomic_write(state_file, content.as_bytes());
}

pub fn clear_state(state_file: &Path) {
    let _ = std::fs::remove_file(state_file);
}

pub fn reset_all(state_file: &Path, pause_file: &Path, done_file: &Path) {
    let _ = std::fs::remove_file(state_file);
    let _ = std::fs::remove_file(pause_file);
    let _ = std::fs::remove_file(done_file);
    logger::log_success("State reset");
}

pub async fn wait_with_countdown(seconds: u64, reason: &str) {
    println!();
    logger::log_warning(&format!("Waiting: {reason}"));
    println!(
        "   Duration: {}...",
        logger::format_duration(seconds)
    );
    println!();

    let mut remaining = seconds;
    while remaining > 0 {
        let chunk = if remaining > 60 { 60 } else { 1 };
        print!(
            "\r   {} remaining...                              ",
            logger::format_duration(remaining)
        );
        sleep(Duration::from_secs(chunk)).await;
        remaining = remaining.saturating_sub(chunk);
    }
    println!("\r   {}                                          ", "Continuing...".to_string());
    println!();
}
