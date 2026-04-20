use ralph_core::worktree::{self, WorktreeError};
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn init_repo() -> (tempfile::TempDir, PathBuf, String, String) {
    let temp_dir = tempfile::tempdir().expect("tempdir should exist");
    let repo_dir = temp_dir.path().join("repo");
    std::fs::create_dir(&repo_dir).expect("repo dir should exist");
    git(&repo_dir, &["init"]);
    git(&repo_dir, &["config", "user.name", "LoopForge Test"]);
    git(
        &repo_dir,
        &["config", "user.email", "loopforge@example.com"],
    );
    let tracked_contents = "tracked root contents\n".to_string();
    let untracked_contents = "untracked sentinel\n".to_string();
    std::fs::write(repo_dir.join("README.md"), &tracked_contents)
        .expect("tracked file should write");
    git(&repo_dir, &["add", "README.md"]);
    git(&repo_dir, &["commit", "-m", "init"]);
    std::fs::write(repo_dir.join("notes.txt"), &untracked_contents)
        .expect("untracked file should write");
    let canonical_repo_dir = repo_dir
        .canonicalize()
        .expect("repo dir should canonicalize");
    (
        temp_dir,
        canonical_repo_dir,
        tracked_contents,
        untracked_contents,
    )
}

fn branch_exists(repo_dir: &Path, branch: &str) -> bool {
    !git(repo_dir, &["branch", "--list", branch]).is_empty()
}

#[tokio::test]
async fn worktree_setup_and_teardown_are_reentrant() {
    let (_temp_dir, repo_dir, tracked_contents, untracked_contents) = init_repo();
    let repo_head = git(&repo_dir, &["rev-parse", "HEAD"]);
    let worktree_path = ".loopforge/worktrees/regression";
    let worktree_dir = repo_dir.join(worktree_path);
    let worktree_branch = "loopforge/regression";
    let expected_path = worktree_dir.to_string_lossy().into_owned();

    let first = worktree::create(&repo_dir, worktree_path, worktree_branch)
        .await
        .expect("first setup should succeed");
    assert_eq!(first.path, expected_path);
    assert_eq!(first.branch, worktree_branch);
    assert!(
        worktree_dir.exists(),
        "worktree directory must exist after setup"
    );
    assert_eq!(
        std::fs::read_to_string(worktree_dir.join("README.md")).unwrap(),
        tracked_contents
    );
    assert_eq!(git(&repo_dir, &["rev-parse", "HEAD"]), repo_head);
    assert_eq!(
        std::fs::read_to_string(repo_dir.join("README.md")).unwrap(),
        tracked_contents
    );
    assert_eq!(
        std::fs::read_to_string(repo_dir.join("notes.txt")).unwrap(),
        untracked_contents
    );
    let first_list = worktree::list(&repo_dir)
        .await
        .expect("worktree list should load");
    assert_eq!(
        first_list
            .iter()
            .filter(|entry| entry.path == expected_path)
            .count(),
        1
    );

    worktree::remove(&repo_dir, Path::new(worktree_path), None)
        .await
        .expect("first teardown should succeed");
    assert!(!worktree_dir.exists(), "worktree directory must be removed");
    assert!(
        branch_exists(&repo_dir, worktree_branch),
        "branch should remain after partial teardown"
    );
    let first_teardown = worktree::list(&repo_dir)
        .await
        .expect("list should succeed after first teardown");
    assert!(first_teardown
        .iter()
        .all(|entry| entry.path != expected_path));
    let inspect_error = worktree::inspect(&repo_dir, Path::new(worktree_path))
        .await
        .expect_err("removed worktree should not be inspectable");
    assert!(matches!(inspect_error, WorktreeError::NotFound(path) if path == expected_path));

    let second = worktree::create(&repo_dir, worktree_path, worktree_branch)
        .await
        .expect("second setup should reuse branch cleanly");
    assert_eq!(second.path, expected_path);
    assert_eq!(second.branch, worktree_branch);
    let second_list = worktree::list(&repo_dir)
        .await
        .expect("second list should load");
    assert_eq!(
        second_list
            .iter()
            .filter(|entry| entry.path == expected_path)
            .count(),
        1
    );
    assert_eq!(git(&repo_dir, &["rev-parse", "HEAD"]), repo_head);
    assert_eq!(
        std::fs::read_to_string(repo_dir.join("README.md")).unwrap(),
        tracked_contents
    );
    assert_eq!(
        std::fs::read_to_string(repo_dir.join("notes.txt")).unwrap(),
        untracked_contents
    );

    worktree::remove(&repo_dir, Path::new(worktree_path), Some(worktree_branch))
        .await
        .expect("final teardown should succeed");
    assert!(
        !worktree_dir.exists(),
        "worktree directory must stay removed"
    );
    assert!(
        !branch_exists(&repo_dir, worktree_branch),
        "branch should be deleted during final teardown"
    );
    let final_list = worktree::list(&repo_dir)
        .await
        .expect("final list should load");
    assert_eq!(final_list.len(), 1);
    assert_eq!(final_list[0].path, repo_dir.to_string_lossy());
    assert_eq!(git(&repo_dir, &["rev-parse", "HEAD"]), repo_head);
    assert_eq!(
        std::fs::read_to_string(repo_dir.join("README.md")).unwrap(),
        tracked_contents
    );
    assert_eq!(
        std::fs::read_to_string(repo_dir.join("notes.txt")).unwrap(),
        untracked_contents
    );
}
