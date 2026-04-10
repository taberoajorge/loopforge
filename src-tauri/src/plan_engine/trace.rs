use chrono::Utc;
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum TraceEvent {
    SessionStart {
        agent: String,
        binary: String,
        args: Vec<String>,
        use_null_stdin: bool,
        stall_threshold_secs: u64,
    },
    Output {
        stream: &'static str,
        byte_len: usize,
        line_count: usize,
        first_line_preview: String,
    },
    HeartbeatCheck {
        since_last_activity_ms: u64,
        effective_threshold_ms: u64,
        triggered_stall: bool,
    },
    StallDetected {
        since_last_activity_ms: u64,
    },
    ProcessTerminated {
        exit_code: i32,
        has_plan_content: bool,
    },
    PlanComplete {
        plan_bytes: usize,
    },
    ErrorEmitted {
        detail: String,
    },
}

#[derive(Debug, Serialize)]
struct TraceEntry {
    ts: String,
    elapsed_ms: u64,
    #[serde(flatten)]
    event: TraceEvent,
}

#[derive(Debug, Clone)]
pub struct SessionTracer {
    inner: Arc<Mutex<TracerInner>>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct TracerInner {
    file: File,
    started_at: std::time::Instant,
    path: PathBuf,
}

impl SessionTracer {
    pub fn new(artifact_dir: &Path, project_id: &str) -> Option<Self> {
        let filename = format!("plan-trace-{}.jsonl", Utc::now().format("%Y%m%d-%H%M%S"));
        let path = artifact_dir.join(&filename);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .ok()?;

        let latest_link = artifact_dir.join("plan-trace-latest.jsonl");
        let _ = std::fs::remove_file(&latest_link);
        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink(&filename, &latest_link);

        log::info!("Plan trace for project {project_id}: {}", path.display());

        Some(Self {
            inner: Arc::new(Mutex::new(TracerInner {
                file,
                started_at: std::time::Instant::now(),
                path,
            })),
        })
    }

    pub fn log(&self, event: TraceEvent) {
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };
        let entry = TraceEntry {
            ts: Utc::now().format("%H:%M:%S%.3f").to_string(),
            elapsed_ms: inner.started_at.elapsed().as_millis() as u64,
            event,
        };
        if let Ok(line) = serde_json::to_string(&entry) {
            let _ = writeln!(inner.file, "{line}");
        }
    }

    #[allow(dead_code)]
    pub fn path(&self) -> PathBuf {
        self.inner
            .lock()
            .map(|g| g.path.clone())
            .unwrap_or_default()
    }
}
