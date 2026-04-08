use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn log_error(message: &str, error_log: Option<&Path>) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    if let Some(log_path) = error_log {
        let formatted = format!("[{timestamp}] ERROR: {message}\n");
        if let Err(write_err) = append_to_file(log_path, &formatted) {
            tracing::warn!(
                path = %log_path.display(),
                error = %write_err,
                "failed to write error log"
            );
        }
    }
    tracing::error!("{message}");
}

pub fn log_info(message: &str) {
    tracing::info!("{message}");
}

pub fn log_success(message: &str) {
    tracing::info!(result = "success", "{message}");
}

pub fn log_warning(message: &str) {
    tracing::warn!("{message}");
}

pub fn log_activity(message: &str, activity_log: &Path) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let formatted = format!("[{timestamp}] {message}\n");
    if let Err(write_err) = append_to_file(activity_log, &formatted) {
        tracing::warn!(
            path = %activity_log.display(),
            error = %write_err,
            "failed to write activity log"
        );
    }
}

pub fn log_iteration_header(iteration: u32) {
    tracing::info!("=== Iteration {iteration} ===");
}

pub fn log_session_header(
    max_iterations: u32,
    poll_interval: u64,
    provider_name: &str,
    model: &str,
) {
    tracing::info!(
        max_iterations,
        poll_interval,
        provider_name,
        model,
        "Ralph session started"
    );
}

pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

fn append_to_file(path: &Path, content: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn rotate_log_if_large(path: &Path, max_lines: usize) {
    if let Ok(content) = std::fs::read_to_string(path) {
        let line_count = content.lines().count();
        if line_count > max_lines {
            let old_path = path.with_extension("log.old");
            if let Err(rename_err) = std::fs::rename(path, &old_path) {
                tracing::warn!(
                    path = %path.display(),
                    error = %rename_err,
                    "failed to rotate log"
                );
                return;
            }
            log_info("Log rotated");
        }
    }
}
