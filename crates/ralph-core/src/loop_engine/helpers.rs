use crate::config::RalphConfig;
use crate::events::{HeartbeatContext, LoopEvent, LoopEventSink};
use crate::git;
use crate::logger;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const HEARTBEAT_INTERVAL_SECS: u64 = 30;

pub async fn interruptible_sleep(
    total_secs: u64,
    shutdown_flag: &Arc<AtomicBool>,
    event_sink: &dyn LoopEventSink,
    context: HeartbeatContext,
) -> bool {
    let mut elapsed: u64 = 0;
    let mut since_last_heartbeat: u64 = 0;

    while elapsed < total_secs {
        if shutdown_flag.load(Ordering::SeqCst) {
            return true;
        }
        let chunk = (total_secs - elapsed).min(2);
        tokio::time::sleep(std::time::Duration::from_secs(chunk)).await;
        elapsed += chunk;
        since_last_heartbeat += chunk;

        if since_last_heartbeat >= HEARTBEAT_INTERVAL_SECS {
            event_sink.emit(LoopEvent::Heartbeat {
                elapsed_secs: elapsed,
                total_secs,
                context: context.clone(),
            });
            since_last_heartbeat = 0;
        }
    }
    false
}

pub async fn log_session_summary(
    iterations: u32,
    config: &RalphConfig,
    initial_commit: &str,
) {
    let total_commits = git::count_commits_since(&config.paths.work_dir, initial_commit)
        .await
        .unwrap_or(0);

    let last_rebase = git::load_last_rebase(&config.paths.last_rebase_file).unwrap_or_default();

    tracing::info!(
        iterations,
        total_commits,
        last_rebase = %last_rebase,
        "Session ended"
    );

    logger::log_activity(
        &format!("=== Session ended after {iterations} iterations ==="),
        &config.paths.activity_log,
    );
}
