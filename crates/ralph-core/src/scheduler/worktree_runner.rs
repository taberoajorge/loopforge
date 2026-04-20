use crate::config::RalphConfig;
use crate::prd::UserStory;
use crate::worktree::{self, WorktreeError, WorktreeInfo};
use std::future::Future;
use std::path::{Path, PathBuf};

const WORKTREE_BASE: &str = ".loopforge/worktrees";
const ARTIFACT_BASE: &str = ".loopforge/artifacts";

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

pub fn artifact_dir_for(worktree_path: &Path, story_id: &str) -> PathBuf {
    worktree_path.join(ARTIFACT_BASE).join(sanitize(story_id))
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

pub fn prepare_worker_config(
    config: &RalphConfig,
    provisioned: &ProvisionedWorktree,
) -> Result<RalphConfig, WorktreeError> {
    let artifact_dir = artifact_dir_for(&provisioned.worktree_path, &provisioned.story_id);
    std::fs::create_dir_all(&artifact_dir)?;
    let mut worker = config.clone();
    worker.paths.ralph_dir = artifact_dir.clone();
    worker.paths.work_dir = provisioned.worktree_path.clone();
    worker.paths.prd_file = artifact_dir.join("prd.json");
    worker.paths.prd_backup = artifact_dir.join("prd.backup.json");
    worker.paths.prompt_file = artifact_dir.join("prompt.md");
    worker.paths.progress_file = artifact_dir.join("progress.log");
    worker.paths.guardrails_file = artifact_dir.join("guardrails.md");
    worker.paths.error_log = artifact_dir.join("error.log");
    worker.paths.activity_log = artifact_dir.join("activity.log");
    worker.paths.state_file = artifact_dir.join("state.json");
    worker.paths.pause_file = artifact_dir.join("pause");
    worker.paths.done_file = artifact_dir.join("done");
    worker.paths.failure_memory_file = artifact_dir.join("failure_memory.json");
    worker.paths.last_rebase_file = artifact_dir.join("last_rebase");
    worker.paths.codex_output_log = artifact_dir.join("codex_output.log");
    copy_or_init(&config.paths.prd_file, &worker.paths.prd_file)?;
    let backup_source = if config.paths.prd_backup.exists() {
        &config.paths.prd_backup
    } else {
        &config.paths.prd_file
    };
    copy_or_init(backup_source, &worker.paths.prd_backup)?;
    copy_or_init(&config.paths.prompt_file, &worker.paths.prompt_file)?;
    copy_or_init(&config.paths.guardrails_file, &worker.paths.guardrails_file)?;
    copy_or_init(
        &config.paths.failure_memory_file,
        &worker.paths.failure_memory_file,
    )?;
    Ok(worker)
}

fn copy_or_init(source: &Path, target: &Path) -> std::io::Result<()> {
    if source.exists() {
        std::fs::copy(source, target)?;
    } else {
        std::fs::write(target, [])?;
    }
    Ok(())
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
