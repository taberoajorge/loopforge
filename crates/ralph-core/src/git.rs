use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

pub async fn fetch_origin_main(work_dir: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["fetch", "origin", "main"])
        .current_dir(work_dir)
        .output()
        .await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git fetch failed: {stderr}");
    }
    Ok(())
}

pub async fn get_origin_main_hash(work_dir: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "origin/main"])
        .current_dir(work_dir)
        .output()
        .await?;
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(hash)
}

pub async fn get_head_hash(work_dir: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(work_dir)
        .output()
        .await?;
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(hash)
}

pub async fn get_current_branch(work_dir: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(work_dir)
        .output()
        .await?;
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

pub async fn checkout_branch(work_dir: &Path, branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["checkout", branch_name])
        .current_dir(work_dir)
        .output()
        .await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git checkout {branch_name} failed: {stderr}");
    }
    Ok(())
}

pub async fn count_commits_between(work_dir: &Path, from_hash: &str, to_hash: &str) -> Result<u32> {
    let range = format!("{from_hash}..{to_hash}");
    let output = Command::new("git")
        .args(["rev-list", &range])
        .current_dir(work_dir)
        .output()
        .await?;
    let count = String::from_utf8_lossy(&output.stdout)
        .lines()
        .count() as u32;
    Ok(count)
}

pub async fn count_commits_since(work_dir: &Path, since_hash: &str) -> Result<u32> {
    let range = format!("{since_hash}..HEAD");
    let output = Command::new("git")
        .args(["rev-list", &range])
        .current_dir(work_dir)
        .output()
        .await?;
    let count = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .count() as u32;
    Ok(count)
}

pub async fn get_diff_stat(work_dir: &Path, from_hash: &str) -> Result<DiffStat> {
    let head = get_head_hash(work_dir).await?;
    if from_hash == head {
        return Ok(DiffStat::default());
    }
    let range = format!("{from_hash}..HEAD");
    let output = Command::new("git")
        .args(["diff", "--stat", &range])
        .current_dir(work_dir)
        .output()
        .await?;
    let stat_text = String::from_utf8_lossy(&output.stdout);
    let files_changed = stat_text.lines().count().saturating_sub(1) as u32;

    let numstat_output = Command::new("git")
        .args(["diff", "--numstat", &range])
        .current_dir(work_dir)
        .output()
        .await?;
    let numstat_text = String::from_utf8_lossy(&numstat_output.stdout);
    let mut insertions: u32 = 0;
    let mut deletions: u32 = 0;
    let mut test_files: u32 = 0;
    for line in numstat_text.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            insertions += parts[0].parse::<u32>().unwrap_or(0);
            deletions += parts[1].parse::<u32>().unwrap_or(0);
            let filename = parts[2];
            if filename.contains("test") || filename.contains("spec") {
                test_files += 1;
            }
        }
    }

    Ok(DiffStat {
        files_changed,
        insertions,
        deletions,
        test_files_touched: test_files,
    })
}

pub fn save_last_rebase(rebase_file: &Path, hash: &str) {
    let _ = std::fs::write(rebase_file, hash);
}

pub fn load_last_rebase(rebase_file: &Path) -> Option<String> {
    std::fs::read_to_string(rebase_file)
        .ok()
        .map(|hash| hash.trim().to_string())
        .filter(|hash| !hash.is_empty())
}

pub async fn ensure_on_branch(work_dir: &Path, target_branch: &str) -> Result<()> {
    let current = get_current_branch(work_dir).await?;
    if current != target_branch {
        crate::logger::log_info(&format!(
            "Switching to branch {target_branch} (currently on {current})"
        ));
        checkout_branch(work_dir, target_branch).await?;
    }
    Ok(())
}

pub async fn remove_git_lock(work_dir: &Path) {
    let lock_file = work_dir.join(".git/index.lock");
    if lock_file.exists() {
        let _ = std::fs::remove_file(&lock_file);
        crate::logger::log_warning("Removed stale git index.lock");
    }
}

#[derive(Debug, Default, Clone)]
pub struct DiffStat {
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
    pub test_files_touched: u32,
}

impl DiffStat {
    pub fn has_changes(&self) -> bool {
        self.files_changed > 0 || self.insertions > 0 || self.deletions > 0
    }
}
