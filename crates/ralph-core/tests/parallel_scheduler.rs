use anyhow::Result;
use ralph_core::config::{PathConfig, RalphConfig, TuningConfig};
use ralph_core::events::NoopEventSink;
use ralph_core::prd::Prd;
use ralph_core::providers::{AgentResult, Provider};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Barrier;
use tokio::time::{Duration, timeout};

#[derive(Clone)]
struct ParallelProbeProvider {
    barrier: Arc<Barrier>,
    events: Arc<std::sync::Mutex<Vec<(String, String, String)>>>,
}

impl Provider for ParallelProbeProvider {
    fn name(&self) -> &str {
        "probe"
    }

    fn model(&self) -> &str {
        "test"
    }

    async fn run_agent(
        &self,
        _prompt: &str,
        story_id: &str,
        work_dir: &Path,
        _stall_timeout_secs: u64,
        _shutdown_flag: Arc<AtomicBool>,
        _output_log: &Path,
    ) -> Result<AgentResult> {
        self.events.lock().unwrap().push((
            "start".into(),
            story_id.to_string(),
            work_dir.display().to_string(),
        ));
        self.barrier.wait().await;
        tokio::time::sleep(Duration::from_millis(25)).await;
        self.events.lock().unwrap().push((
            "finish".into(),
            story_id.to_string(),
            work_dir.display().to_string(),
        ));
        Ok(AgentResult {
            exit_code: 0,
            stall_killed: false,
            output_lines: vec![format!("finished {story_id}")],
            rate_limited: false,
            retry_after_message: None,
        })
    }
}

fn git(repo_dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_dir)
        .output()
        .expect("git should run");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn init_repo() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let repo_dir = temp_dir.path().join("repo");
    std::fs::create_dir(&repo_dir).expect("repo dir");
    git(&repo_dir, &["init"]);
    git(&repo_dir, &["config", "user.name", "Test"]);
    git(&repo_dir, &["config", "user.email", "test@example.com"]);
    std::fs::write(repo_dir.join("README.md"), "init\n").expect("readme");
    git(&repo_dir, &["add", "README.md"]);
    git(&repo_dir, &["commit", "-m", "init"]);
    (temp_dir, repo_dir.canonicalize().expect("canonicalize"))
}

fn build_config(root: &Path, repo_dir: &Path) -> RalphConfig {
    let ralph_dir = root.join("ralph");
    std::fs::create_dir_all(&ralph_dir).expect("ralph dir");
    let prompt_file = ralph_dir.join("prompt.md");
    let guardrails_file = ralph_dir.join("guardrails.md");
    std::fs::write(&prompt_file, "# Prompt\n").expect("prompt");
    std::fs::write(&guardrails_file, "# Guardrails\n").expect("guardrails");
    RalphConfig {
        paths: PathConfig {
            ralph_dir: ralph_dir.clone(),
            work_dir: repo_dir.to_path_buf(),
            prd_file: ralph_dir.join("prd.json"),
            prd_backup: ralph_dir.join("prd.backup.json"),
            prompt_file,
            progress_file: ralph_dir.join("progress.log"),
            guardrails_file,
            error_log: ralph_dir.join("error.log"),
            activity_log: ralph_dir.join("activity.log"),
            state_file: ralph_dir.join("state.json"),
            pause_file: ralph_dir.join("pause"),
            done_file: ralph_dir.join("done"),
            failure_memory_file: ralph_dir.join("failure_memory.json"),
            last_rebase_file: ralph_dir.join("last_rebase"),
            codex_output_log: ralph_dir.join("codex_output.log"),
        },
        tuning: TuningConfig {
            max_iterations: 2,
            rate_limit_wait_secs: 1,
            gutter_threshold: 3,
            stall_timeout_secs: 10,
            cooldown_secs: 0,
            max_verification_retries: 1,
            test_command: None,
        },
        services: None,
    }
}

fn install_prd(config: &RalphConfig) {
    let prd = r#"{
        "project": "loopforge",
        "stories": [
            {
                "id": "S-017A",
                "title": "First",
                "acceptanceCriteria": ["first"],
                "verification": { "commands": ["true"] }
            },
            {
                "id": "S-017B",
                "title": "Second",
                "acceptanceCriteria": ["second"],
                "verification": { "commands": ["true"] }
            }
        ]
    }"#;
    std::fs::write(&config.paths.prd_file, prd).expect("prd");
    std::fs::write(&config.paths.prd_backup, prd).expect("prd backup");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ready_stories_execute_in_parallel_worktrees() {
    let (temp_dir, repo_dir) = init_repo();
    let config = build_config(temp_dir.path(), &repo_dir);
    install_prd(&config);
    let provider = ParallelProbeProvider {
        barrier: Arc::new(Barrier::new(2)),
        events: Arc::new(std::sync::Mutex::new(Vec::new())),
    };
    let shutdown = Arc::new(AtomicBool::new(false));
    let sink = NoopEventSink;

    timeout(
        Duration::from_secs(2),
        ralph_core::loop_engine::run(&config, &provider, shutdown, &sink),
    )
    .await
    .expect("parallel run should not hang")
    .expect("loop run should succeed");

    let events = provider.events.lock().unwrap().clone();
    assert_eq!(events.len(), 4, "expected start/finish for both stories");
    assert_eq!(events[0].0, "start");
    assert_eq!(events[1].0, "start");
    let work_dirs: std::collections::HashSet<String> =
        events.iter().take(2).map(|event| event.2.clone()).collect();
    assert_eq!(work_dirs.len(), 2, "stories should use distinct worktrees");
    assert!(
        work_dirs
            .iter()
            .all(|dir| dir.contains(".loopforge/worktrees")),
        "worktree paths should be provisioned under .loopforge/worktrees: {work_dirs:?}"
    );

    let updated = Prd::load(&config.paths.prd_file).expect("updated prd");
    assert!(updated.stories.iter().all(|story| story.passes));
    assert_eq!(updated.pending_count(), 0);
}
