use ralph_core::config::{PathConfig, RalphConfig, TuningConfig};
use ralph_core::events::RecordingEventSink;
use ralph_core::providers::fixture::{FixtureProvider, FixtureScenario};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn build_test_config(tmp: &tempfile::TempDir) -> RalphConfig {
    let ralph_dir = tmp.path().to_path_buf();
    let prd_file = ralph_dir.join("prd.json");
    let prd_backup = ralph_dir.join("prd_backup.json");
    let prompt_file = ralph_dir.join("prompt.md");
    let guardrails_file = ralph_dir.join("guardrails.md");

    std::fs::write(&prompt_file, "# Test prompt\nExecute story.\n").unwrap();
    std::fs::write(&guardrails_file, "# Guardrails\n").unwrap();

    RalphConfig {
        paths: PathConfig {
            ralph_dir: ralph_dir.clone(),
            work_dir: ralph_dir.clone(),
            prd_file,
            prd_backup,
            prompt_file,
            progress_file: ralph_dir.join("progress.txt"),
            guardrails_file,
            error_log: ralph_dir.join("error.log"),
            activity_log: ralph_dir.join("activity.log"),
            state_file: ralph_dir.join(".ralph_state"),
            pause_file: ralph_dir.join(".ralph-pause"),
            done_file: ralph_dir.join(".ralph-done"),
            failure_memory_file: ralph_dir.join("failure_memory.json"),
            last_rebase_file: ralph_dir.join(".ralph_last_rebase"),
            codex_output_log: ralph_dir.join("codex_output.log"),
        },
        tuning: TuningConfig {
            max_iterations: 5,
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

fn install_prd(config: &RalphConfig, fixture_name: &str) {
    let source = fixture(fixture_name);
    std::fs::copy(&source, &config.paths.prd_file).unwrap();
    std::fs::copy(&source, &config.paths.prd_backup).unwrap();
}

#[tokio::test]
async fn happy_path_emits_deterministic_events() {
    let tmp = tempfile::tempdir().unwrap();
    let config = build_test_config(&tmp);
    install_prd(&config, "happy_path_prd.json");

    let provider = FixtureProvider::new(FixtureScenario::HappyPath);
    let shutdown = Arc::new(AtomicBool::new(false));
    let sink = RecordingEventSink::new();

    let result = ralph_core::loop_engine::run(&config, &provider, shutdown, &sink).await;
    assert!(result.is_ok(), "loop should complete without error");

    assert_eq!(provider.call_count(), 1, "happy path should run exactly one iteration");

    let types = sink.event_types();
    assert!(
        types.contains(&"prompt_built".to_string()),
        "should emit prompt_built, got: {types:?}"
    );
}

#[tokio::test]
async fn happy_path_marks_story_passed_in_prd() {
    let tmp = tempfile::tempdir().unwrap();
    let config = build_test_config(&tmp);
    install_prd(&config, "happy_path_prd.json");

    let provider = FixtureProvider::new(FixtureScenario::HappyPath);
    let shutdown = Arc::new(AtomicBool::new(false));
    let sink = RecordingEventSink::new();

    ralph_core::loop_engine::run(&config, &provider, shutdown, &sink)
        .await
        .unwrap();

    let prd = ralph_core::prd::Prd::load(&config.paths.prd_file).unwrap();
    assert!(prd.stories[0].passes, "story should be marked as passed");
    assert_eq!(prd.pending_count(), 0, "no stories should remain pending");
}

#[tokio::test]
async fn loop_failure_stops_after_gutter_threshold() {
    let tmp = tempfile::tempdir().unwrap();
    let config = build_test_config(&tmp);
    install_prd(&config, "failure_prd.json");

    let provider = FixtureProvider::new(FixtureScenario::LoopFailure);
    let shutdown = Arc::new(AtomicBool::new(false));
    let sink = RecordingEventSink::new();

    let result = ralph_core::loop_engine::run(&config, &provider, shutdown, &sink).await;
    assert!(result.is_ok(), "loop should exit cleanly on failure path");

    let types = sink.event_types();
    assert!(
        types.contains(&"story_skipped".to_string()),
        "should skip story after gutter threshold, got: {types:?}"
    );

    let prd = ralph_core::prd::Prd::load(&config.paths.prd_file).unwrap();
    assert!(!prd.stories[0].passes, "failed story should not be marked passed");
    assert!(prd.stories[0].blocked, "failed story should be blocked after gutter");
}

#[tokio::test]
async fn fixture_provider_does_not_spawn_real_process() {
    let tmp = tempfile::tempdir().unwrap();
    let config = build_test_config(&tmp);
    install_prd(&config, "happy_path_prd.json");

    let provider = FixtureProvider::new(FixtureScenario::HappyPath);
    let shutdown = Arc::new(AtomicBool::new(false));
    let sink = RecordingEventSink::new();

    let start = std::time::Instant::now();
    ralph_core::loop_engine::run(&config, &provider, shutdown, &sink)
        .await
        .unwrap();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_secs() < 5,
        "fixture run should be near-instant, took {:?}",
        elapsed
    );
    assert_eq!(provider.call_count(), 1);
}

#[tokio::test]
async fn loop_failure_emits_prompt_built_per_attempt() {
    let tmp = tempfile::tempdir().unwrap();
    let config = build_test_config(&tmp);
    install_prd(&config, "failure_prd.json");

    let provider = FixtureProvider::new(FixtureScenario::LoopFailure);
    let shutdown = Arc::new(AtomicBool::new(false));
    let sink = RecordingEventSink::new();

    ralph_core::loop_engine::run(&config, &provider, shutdown, &sink)
        .await
        .unwrap();

    let prompt_count = sink
        .event_types()
        .iter()
        .filter(|typ| *typ == "prompt_built")
        .count();

    assert!(
        prompt_count >= 1,
        "should emit at least one prompt_built event, got {prompt_count}"
    );
    assert!(
        provider.call_count() >= 1,
        "should have called provider at least once"
    );
}
