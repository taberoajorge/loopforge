use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum WorktreeError {
    #[error("git error: {0}")]
    Git(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("worktree not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub head: String,
    pub is_bare: bool,
    pub is_locked: bool,
    pub is_prunable: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct WorktreeDiff {
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
}

pub async fn create(
    work_dir: &Path,
    worktree_path: &str,
    branch: &str,
) -> Result<WorktreeInfo, WorktreeError> {
    let create_branch = ["worktree", "add", worktree_path, "-b", branch];
    if run_git(work_dir, &create_branch).await.is_err() {
        let existing_branch = ["worktree", "add", worktree_path, branch];
        run_git(work_dir, &existing_branch).await?;
    }
    inspect(work_dir, Path::new(worktree_path)).await
}

pub async fn list(work_dir: &Path) -> Result<Vec<WorktreeInfo>, WorktreeError> {
    let output = run_git(work_dir, &["worktree", "list", "--porcelain"]).await?;
    Ok(parse_worktree_list(&output))
}

pub async fn remove(
    work_dir: &Path,
    worktree_path: &Path,
    branch: Option<&str>,
) -> Result<(), WorktreeError> {
    let target = worktree_path.to_string_lossy().into_owned();
    run_git(work_dir, &["worktree", "remove", &target, "--force"]).await?;
    if let Some(branch_name) = branch {
        let _ = run_git(work_dir, &["branch", "-d", branch_name]).await;
    }
    Ok(())
}

pub async fn inspect(work_dir: &Path, worktree_path: &Path) -> Result<WorktreeInfo, WorktreeError> {
    let target = normalize_path(work_dir, worktree_path);
    list(work_dir)
        .await?
        .into_iter()
        .find(|worktree| worktree.path == target)
        .ok_or(WorktreeError::NotFound(target))
}

pub async fn diff(
    work_dir: &Path,
    from_ref: &str,
    to_ref: &str,
) -> Result<WorktreeDiff, WorktreeError> {
    let range = format!("{from_ref}..{to_ref}");
    let output = run_git(work_dir, &["diff", "--numstat", &range]).await?;
    Ok(parse_diff_numstat(&output))
}

pub fn parse_worktree_list(raw: &str) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut current = WorktreeInfo::default();
    for line in raw.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if !current.path.is_empty() {
                worktrees.push(current);
            }
            current = WorktreeInfo {
                path: path.to_string(),
                ..WorktreeInfo::default()
            };
            continue;
        }
        if let Some(head) = line.strip_prefix("HEAD ") {
            current.head = head.to_string();
        } else if let Some(branch) = line.strip_prefix("branch ") {
            current.branch = branch.trim_start_matches("refs/heads/").to_string();
        } else if line == "bare" {
            current.is_bare = true;
        } else if line == "locked" {
            current.is_locked = true;
        } else if line == "prunable" {
            current.is_prunable = true;
        }
    }
    if !current.path.is_empty() {
        worktrees.push(current);
    }
    worktrees
}

pub fn parse_diff_numstat(raw: &str) -> WorktreeDiff {
    raw.lines().fold(WorktreeDiff::default(), |mut diff, line| {
        let mut parts = line.splitn(3, '\t');
        let insertions = parts
            .next()
            .unwrap_or_default()
            .parse::<u32>()
            .unwrap_or_default();
        let deletions = parts
            .next()
            .unwrap_or_default()
            .parse::<u32>()
            .unwrap_or_default();
        if parts.next().is_some() {
            diff.files_changed += 1;
            diff.insertions += insertions;
            diff.deletions += deletions;
        }
        diff
    })
}

async fn run_git(work_dir: &Path, args: &[&str]) -> Result<String, WorktreeError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(work_dir)
        .output()
        .await?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(WorktreeError::Git(if stderr.is_empty() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        stderr
    }))
}

fn normalize_path(work_dir: &Path, worktree_path: &Path) -> String {
    let path = if worktree_path.is_absolute() {
        PathBuf::from(worktree_path)
    } else {
        work_dir.join(worktree_path)
    };
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::{parse_diff_numstat, parse_worktree_list, WorktreeDiff, WorktreeInfo};

    #[test]
    fn parses_worktree_porcelain() {
        let raw = "worktree /repo\nHEAD abc\nbranch refs/heads/main\n\nworktree /repo/w1\nHEAD def\nbranch refs/heads/feature\nlocked\nprunable\n";
        assert_eq!(
            parse_worktree_list(raw),
            vec![
                WorktreeInfo {
                    path: "/repo".into(),
                    branch: "main".into(),
                    head: "abc".into(),
                    ..WorktreeInfo::default()
                },
                WorktreeInfo {
                    path: "/repo/w1".into(),
                    branch: "feature".into(),
                    head: "def".into(),
                    is_locked: true,
                    is_prunable: true,
                    ..WorktreeInfo::default()
                },
            ]
        );
    }

    #[test]
    fn parses_numstat_safely() {
        let raw = "10\t2\ta.rs\n-\t-\tb.bin\n";
        assert_eq!(
            parse_diff_numstat(raw),
            WorktreeDiff {
                files_changed: 2,
                insertions: 10,
                deletions: 2
            }
        );
    }
}
