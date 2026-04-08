use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum StallVerdict {
    Completed,
    StalledAndKilled,
    ShutdownRequested,
}

pub async fn monitor_child_with_stall_detection(
    child: &mut Child,
    stall_timeout: Duration,
    shutdown_flag: &Arc<AtomicBool>,
    output_sender: Option<mpsc::UnboundedSender<String>>,
) -> StallVerdict {
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let _ = child.wait().await;
            return StallVerdict::Completed;
        }
    };

    let stderr = child.stderr.take();
    let mut stdout_reader = BufReader::new(stdout).lines();

    let stderr_task = if let Some(stderr_stream) = stderr {
        let sender_clone = output_sender.clone();
        Some(tokio::spawn(async move {
            let mut stderr_reader = BufReader::new(stderr_stream).lines();
            let mut collected = Vec::new();
            while let Ok(Some(line)) = stderr_reader.next_line().await {
                if let Some(ref sender) = sender_clone {
                    let _ = sender.send(format!("[stderr] {line}"));
                }
                collected.push(line);
            }
            collected
        }))
    } else {
        None
    };

    let mut stall_timer = tokio::time::interval(Duration::from_secs(1));
    let mut seconds_since_output: u64 = 0;
    let stall_threshold = stall_timeout.as_secs();

    loop {
        tokio::select! {
            line_result = stdout_reader.next_line() => {
                match line_result {
                    Ok(Some(line)) => {
                        seconds_since_output = 0;
                        if let Some(ref sender) = output_sender {
                            let _ = sender.send(line);
                        }
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
            _ = stall_timer.tick() => {
                seconds_since_output += 1;
                if seconds_since_output >= stall_threshold {
                    crate::logger::log_warning(&format!(
                        "Stall detected: no output for {stall_threshold}s, killing process"
                    ));
                    kill_child_gracefully(child).await;
                    if let Some(task) = stderr_task { let _ = task.await; }
                    return StallVerdict::StalledAndKilled;
                }
                if shutdown_flag.load(Ordering::SeqCst) {
                    crate::logger::log_info("Shutdown requested, killing agent process...");
                    kill_child_gracefully(child).await;
                    if let Some(task) = stderr_task { let _ = task.await; }
                    return StallVerdict::ShutdownRequested;
                }
            }
        }
    }

    let _ = child.wait().await;
    if let Some(task) = stderr_task {
        let _ = task.await;
    }
    StallVerdict::Completed
}

async fn kill_child_gracefully(child: &mut Child) {
    if let Some(pid) = child.id() {
        crate::health::manager::graceful_kill(pid).await;
    } else {
        let _ = child.kill().await;
    }
}
