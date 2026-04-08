use crate::git::{self, DiffStat};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ProgressReport {
    pub diff_stat: DiffStat,
    pub new_progress_lines: u32,
    pub story_passed: bool,
    pub zero_progress: bool,
}

pub async fn analyze_iteration_progress(
    work_dir: &Path,
    head_before: &str,
    story_passed: bool,
    progress_file: &Path,
    previous_progress_len: usize,
) -> ProgressReport {
    let diff_stat = git::get_diff_stat(work_dir, head_before)
        .await
        .unwrap_or_default();

    let current_progress_len = std::fs::read_to_string(progress_file)
        .map(|content| content.len())
        .unwrap_or(0);
    let new_progress_lines = if current_progress_len > previous_progress_len {
        let delta = current_progress_len - previous_progress_len;
        (delta / 40).max(1) as u32
    } else {
        0
    };

    let zero_progress = !diff_stat.has_changes() && !story_passed && new_progress_lines == 0;

    ProgressReport {
        diff_stat,
        new_progress_lines,
        story_passed,
        zero_progress,
    }
}

pub fn get_progress_file_size(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .map(|content| content.len())
        .unwrap_or(0)
}
