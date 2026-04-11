use crate::prd::UserStory;
use crate::worktree::{self, WorktreeError, WorktreeInfo};
use std::future::Future;
use std::path::{Path, PathBuf};

const WORKTREE_BASE: &str = ".loopforge/worktrees";

#[derive(Debug, Clone)]
pub struct ProvisionedWorktree {
    pub story_id: String,
    pub worktree_path: PathBuf,
    pub branch: String,
    pub info: WorktreeInfo,
}

pub fn worktree_path_for(work_dir: &Path, story_id: &str) -> PathBuf {
    work_dir.join(WORKTREE_BASE).join(sanitize(story_id))
}

pub fn branch_for(story_id: &str) -> String {
    format!("loopforge/{}", sanitize(story_id))
}

fn sanitize(story_id: &str) -> String {
    story_id
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
}

pub async fn provision(
    work_dir: &Path,
    story_id: &str,
) -> Result<ProvisionedWorktree, WorktreeError> {
    let rel_path = format!("{WORKTREE_BASE}/{}", sanitize(story_id));
    let branch = branch_for(story_id);
    let info = worktree::create(work_dir, &rel_path, &branch).await?;
    Ok(ProvisionedWorktree {
        story_id: story_id.to_string(),
        worktree_path: PathBuf::from(&info.path),
        branch,
        info,
    })
}

pub async fn run_in_worktree<F, Fut, R>(
    work_dir: &Path,
    story: &UserStory,
    worker: F,
) -> Result<R, WorktreeError>
where
    F: FnOnce(ProvisionedWorktree) -> Fut,
    Fut: Future<Output = R>,
{
    let provisioned = provision(work_dir, &story.id).await?;
    Ok(worker(provisioned).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn different_stories_get_different_paths() {
        let work_dir = Path::new("/repo");
        let path_a = worktree_path_for(work_dir, "S-001");
        let path_b = worktree_path_for(work_dir, "S-002");
        assert_ne!(path_a, path_b);
    }

    #[test]
    fn same_story_gets_same_path() {
        let work_dir = Path::new("/repo");
        let first = worktree_path_for(work_dir, "S-001");
        let second = worktree_path_for(work_dir, "S-001");
        assert_eq!(first, second);
    }

    #[test]
    fn path_contains_sanitized_story_id() {
        let work_dir = Path::new("/repo");
        let path = worktree_path_for(work_dir, "S-001");
        assert!(path.to_string_lossy().contains("s-001"));
        assert!(path.to_string_lossy().contains(WORKTREE_BASE));
    }

    #[test]
    fn branch_contains_story_id() {
        let branch = branch_for("S-015");
        assert_eq!(branch, "loopforge/s-015");
    }

    #[test]
    fn sanitize_strips_special_chars() {
        assert_eq!(sanitize("S-001"), "s-001");
        assert_eq!(sanitize("S_002"), "s-002");
        assert_eq!(sanitize("My Story!"), "my-story-");
    }
}
