use ralph_core::prd::UserStory;
use ralph_core::scheduler::worktree_runner::{
    branch_for, provision, run_in_worktree, worktree_path_for,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

fn git(repo_dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_dir)
        .output()
        .expect("git command should run");
    if output.status.success() {
        return String::from_utf8_lossy(&output.stdout).trim().to_string();
    }
    panic!(
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr).trim()
    );
}

fn init_repo() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let repo_dir = temp_dir.path().join("repo");
    std::fs::create_dir(&repo_dir).expect("repo dir");
    git(&repo_dir, &["init"]);
    git(&repo_dir, &["config", "user.name", "Test"]);
    git(&repo_dir, &["config", "user.email", "test@example.com"]);
    std::fs::write(repo_dir.join("README.md"), "init\n").expect("write");
    git(&repo_dir, &["add", "README.md"]);
    git(&repo_dir, &["commit", "-m", "init"]);
    let canonical = repo_dir.canonicalize().expect("canonicalize");
    (temp_dir, canonical)
}

fn stub_story(story_id: &str) -> UserStory {
    serde_json::from_value(serde_json::json!({
        "id": story_id,
        "title": format!("Story {story_id}"),
        "acceptanceCriteria": ["placeholder"]
    }))
    .expect("stub story should parse")
}

#[test]
fn different_stories_receive_different_worktree_paths() {
    let work_dir = Path::new("/repo");
    let stories = ["S-001", "S-002", "S-003", "S-015"];
    let paths: Vec<PathBuf> = stories
        .iter()
        .map(|story_id| worktree_path_for(work_dir, story_id))
        .collect();

    for (idx, path) in paths.iter().enumerate() {
        for (jdx, other) in paths.iter().enumerate() {
            if idx != jdx {
                assert_ne!(path, other, "{} and {} must differ", stories[idx], stories[jdx]);
            }
        }
    }
}

#[test]
fn different_stories_receive_different_branches() {
    let branches: Vec<String> = ["S-001", "S-002", "S-015"]
        .iter()
        .map(|story_id| branch_for(story_id))
        .collect();

    assert_ne!(branches[0], branches[1]);
    assert_ne!(branches[1], branches[2]);
}

#[tokio::test]
async fn provision_creates_worktree_directory() {
    let (_temp_dir, repo_dir) = init_repo();

    let provisioned = provision(&repo_dir, "S-015")
        .await
        .expect("provision should succeed");

    assert_eq!(provisioned.story_id, "S-015");
    assert!(provisioned.worktree_path.exists());
    assert_eq!(provisioned.branch, "loopforge/s-015");
    assert!(provisioned.worktree_path.join("README.md").exists());
}

#[tokio::test]
async fn provision_fails_gracefully_on_invalid_repo() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let bad_dir = temp_dir.path().join("not-a-repo");
    std::fs::create_dir(&bad_dir).expect("dir");

    let result = provision(&bad_dir, "S-001").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn two_stories_get_separate_worktrees() {
    let (_temp_dir, repo_dir) = init_repo();

    let first = provision(&repo_dir, "S-001")
        .await
        .expect("first provision");
    let second = provision(&repo_dir, "S-002")
        .await
        .expect("second provision");

    assert_ne!(first.worktree_path, second.worktree_path);
    assert_ne!(first.branch, second.branch);
    assert!(first.worktree_path.exists());
    assert!(second.worktree_path.exists());
}

#[tokio::test]
async fn worker_fixture_runs_sequentially_through_core_loop() {
    let (_temp_dir, repo_dir) = init_repo();
    let execution_log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let story_a = stub_story("S-001");
    let story_b = stub_story("S-002");

    let log_clone = Arc::clone(&execution_log);
    let result_a = run_in_worktree(&repo_dir, &story_a, |provisioned| {
        let log = log_clone;
        async move {
            log.lock().unwrap().push(provisioned.story_id.clone());
            provisioned.worktree_path
        }
    })
    .await
    .expect("run_in_worktree S-001");

    let log_clone = Arc::clone(&execution_log);
    let result_b = run_in_worktree(&repo_dir, &story_b, |provisioned| {
        let log = log_clone;
        async move {
            log.lock().unwrap().push(provisioned.story_id.clone());
            provisioned.worktree_path
        }
    })
    .await
    .expect("run_in_worktree S-002");

    let log = execution_log.lock().unwrap();
    assert_eq!(log.len(), 2);
    assert_eq!(log[0], "S-001");
    assert_eq!(log[1], "S-002");
    assert_ne!(result_a, result_b);
}

#[tokio::test]
async fn worktree_creation_failure_prevents_worker_execution() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let bad_dir = temp_dir.path().join("not-a-repo");
    std::fs::create_dir(&bad_dir).expect("dir");

    let story = stub_story("S-001");
    let worker_called = Arc::new(Mutex::new(false));
    let flag = Arc::clone(&worker_called);

    let result = run_in_worktree(&bad_dir, &story, |_provisioned| {
        let called = flag;
        async move {
            *called.lock().unwrap() = true;
        }
    })
    .await;

    assert!(result.is_err());
    assert!(!*worker_called.lock().unwrap());
}
