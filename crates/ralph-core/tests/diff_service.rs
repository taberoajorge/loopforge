use ralph_core::{UnifiedDiffRequest, unified_diff};
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(repo_dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_dir)
        .output()
        .expect("git command should run");
    if !output.status.success() {
        panic!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
}

fn init_repo() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("tempdir should exist");
    let repo_dir = temp_dir.path().join("repo");
    std::fs::create_dir(&repo_dir).expect("repo dir should exist");
    git(&repo_dir, &["init"]);
    git(&repo_dir, &["config", "user.name", "LoopForge Test"]);
    git(
        &repo_dir,
        &["config", "user.email", "loopforge@example.com"],
    );
    (temp_dir, repo_dir)
}

#[test]
fn unified_diff_is_empty_when_working_tree_has_no_changes() {
    let (_temp_dir, repo_dir) = init_repo();
    std::fs::write(repo_dir.join("story.txt"), "alpha\nbeta\ngamma\n")
        .expect("fixture file should be written");
    git(&repo_dir, &["add", "story.txt"]);
    git(&repo_dir, &["commit", "-m", "initial commit"]);

    let request = UnifiedDiffRequest::working_tree(&repo_dir);
    let diff = unified_diff(&request).expect("diff should execute");

    assert!(diff.is_empty(), "expected no diff, got: {diff}");
}

#[test]
fn unified_diff_contains_patch_lines_for_file_changes() {
    let (_temp_dir, repo_dir) = init_repo();
    std::fs::write(repo_dir.join("story.txt"), "alpha\nbeta\ngamma\ndelta\n")
        .expect("fixture file should be written");
    git(&repo_dir, &["add", "story.txt"]);
    git(&repo_dir, &["commit", "-m", "initial commit"]);
    std::fs::write(
        repo_dir.join("story.txt"),
        "alpha\nbeta updated\ngamma\nepsilon\n",
    )
    .expect("modified file should be written");

    let request = UnifiedDiffRequest::working_tree(&repo_dir).with_context_lines(3);
    let diff = unified_diff(&request).expect("diff should execute");

    assert!(diff.contains("diff --git a/story.txt b/story.txt"));
    assert!(diff.contains("@@"));
    assert!(diff.contains("-beta"));
    assert!(diff.contains("+beta updated"));
    assert!(diff.contains(" gamma"));
}
