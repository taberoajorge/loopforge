use ralph_core::config::RalphConfig;
use ralph_core::prd::Prd;
use ralph_core::scheduler::{ArtifactCoordinator, SharedArtifactUpdate};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    Start(String),
    Finish(String),
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_updates_are_applied_one_at_a_time() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let active = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));
    let coordinator = {
        let events = events.clone();
        let active = active.clone();
        let max_active = max_active.clone();
        ArtifactCoordinator::new(move |update| {
            let story_id = match update {
                SharedArtifactUpdate::MarkStoryBlocked { story_id } => story_id,
                _ => {
                    return Err(ralph_core::scheduler::CoordinatorError::apply(
                        "unexpected update",
                    ))
                }
            };
            let now_active = active.fetch_add(1, Ordering::SeqCst) + 1;
            max_active.fetch_max(now_active, Ordering::SeqCst);
            events.blocking_lock().push(Event::Start(story_id.clone()));
            std::thread::sleep(std::time::Duration::from_millis(20));
            events.blocking_lock().push(Event::Finish(story_id));
            active.fetch_sub(1, Ordering::SeqCst);
            Ok(())
        })
    };

    let mut tasks = Vec::new();
    for index in 0..6 {
        let coordinator = coordinator.clone();
        tasks.push(tokio::spawn(async move {
            coordinator
                .submit(SharedArtifactUpdate::MarkStoryBlocked {
                    story_id: format!("S-{index:03}"),
                })
                .await
        }));
    }
    for task in tasks {
        task.await
            .expect("task should join")
            .expect("update should apply");
    }

    assert_eq!(max_active.load(Ordering::SeqCst), 1);
    let events = events.lock().await.clone();
    assert_eq!(events.len(), 12);
    for pair in events.chunks_exact(2) {
        match (&pair[0], &pair[1]) {
            (Event::Start(started), Event::Finish(finished)) => assert_eq!(started, finished),
            other => panic!("unexpected event sequence: {other:?}"),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn loop_engine_gutter_skip_updates_flow_through_the_coordinator() {
    let temp = tempdir().expect("tempdir should exist");
    let ralph_dir = temp.path().join("ralph");
    std::fs::create_dir_all(&ralph_dir).expect("ralph dir should exist");
    let mut config = RalphConfig::from_defaults(&ralph_dir);
    config.paths.prd_file = ralph_dir.join("prd.json");
    config.paths.prd_backup = ralph_dir.join("prd.backup.json");
    config.paths.guardrails_file = ralph_dir.join("guardrails.md");

    let prd = r#"{
        "project": "loopforge",
        "stories": [
            { "id": "S-016", "title": "Serialize", "acceptanceCriteria": ["serialize"] },
            { "id": "S-017", "title": "Neighbor", "acceptanceCriteria": ["neighbor"] }
        ]
    }"#;
    std::fs::write(&config.paths.prd_file, prd).expect("prd should write");
    std::fs::write(&config.paths.prd_backup, prd).expect("prd backup should write");

    let coordinator = ArtifactCoordinator::for_loop_engine(config.clone());
    let first = coordinator.submit(SharedArtifactUpdate::BlockStoryAndAddGuardrail {
        story_id: "S-016".into(),
        error_message: "Exceeded gutter threshold".into(),
        iteration: 1,
    });
    let second = coordinator.submit(SharedArtifactUpdate::BlockStoryAndAddGuardrail {
        story_id: "S-017".into(),
        error_message: "Exceeded gutter threshold".into(),
        iteration: 2,
    });
    let (first, second) = tokio::join!(first, second);
    first.expect("first update should apply");
    second.expect("second update should apply");

    let updated = Prd::load(&config.paths.prd_file).expect("prd should load");
    assert!(updated.stories.iter().all(|story| story.blocked));
    let guardrails =
        std::fs::read_to_string(&config.paths.guardrails_file).expect("guardrails should exist");
    assert!(guardrails.contains("Sign: Error in S-016"));
    assert!(guardrails.contains("Sign: Error in S-017"));

    let source = std::fs::read_to_string(format!(
        "{}/src/loop_engine/mod.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("loop engine source should read");
    assert!(source.contains("artifact_coordinator"));
    assert!(source.contains(".submit("));
    assert!(source.contains("SharedArtifactUpdate::BlockStoryAndAddGuardrail"));
    assert!(!source.contains("guardrails::add_guardrail"));
}
