use crate::db::DbState;
use crate::loop_manager::LoopManagerState;
use ralph_core::prd::Prd;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_shell::ShellExt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorktreeError {
    #[error("Git error: {0}")]
    Git(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path error: {0}")]
    Path(String),
    #[error("Worktree already exists: {0}")]
    AlreadyExists(String),
    #[error("Worktree not found: {0}")]
    NotFound(String),
    #[error("Lock poisoned")]
    LockPoisoned,
}

impl Serialize for WorktreeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub head: String,
    pub is_bare: bool,
    pub is_locked: bool,
    pub is_prunable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeOverlap {
    pub project_id_a: String,
    pub project_id_b: String,
    pub overlapping_files: Vec<String>,
}

async fn git_run(
    app: &AppHandle,
    work_dir: &str,
    args: &[&str],
) -> Result<(bool, String, String), WorktreeError> {
    let output = app
        .shell()
        .command("git")
        .args(args)
        .current_dir(work_dir)
        .output()
        .await
        .map_err(|err| WorktreeError::Git(err.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((output.status.success(), stdout, stderr))
}

fn parse_worktree_list(raw: &str) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut current_path = String::new();
    let mut current_head = String::new();
    let mut current_branch = String::new();
    let mut is_bare = false;
    let mut is_locked = false;
    let mut is_prunable = false;

    for line in raw.lines() {
        if line.starts_with("worktree ") {
            if !current_path.is_empty() {
                worktrees.push(WorktreeInfo {
                    path: current_path.clone(),
                    branch: current_branch.clone(),
                    head: current_head.clone(),
                    is_bare,
                    is_locked,
                    is_prunable,
                });
            }
            current_path = line.trim_start_matches("worktree ").to_string();
            current_head = String::new();
            current_branch = String::new();
            is_bare = false;
            is_locked = false;
            is_prunable = false;
        } else if line.starts_with("HEAD ") {
            current_head = line.trim_start_matches("HEAD ").to_string();
        } else if line.starts_with("branch ") {
            current_branch = line
                .trim_start_matches("branch refs/heads/")
                .trim_start_matches("branch ")
                .to_string();
        } else if line == "bare" {
            is_bare = true;
        } else if line == "locked" {
            is_locked = true;
        } else if line == "prunable" {
            is_prunable = true;
        }
    }

    if !current_path.is_empty() {
        worktrees.push(WorktreeInfo {
            path: current_path,
            branch: current_branch,
            head: current_head,
            is_bare,
            is_locked,
            is_prunable,
        });
    }

    worktrees
}

#[tauri::command]
pub async fn create_worktree(
    app: AppHandle,
    working_directory: String,
    loop_name: String,
) -> Result<WorktreeInfo, WorktreeError> {
    let worktree_path = format!(".loopforge/worktrees/{loop_name}");
    let branch = format!("loopforge/{loop_name}");

    let (ok, stdout, _stderr) =
        git_run(&app, &working_directory, &["worktree", "add", &worktree_path, "-b", &branch]).await?;

    if !ok {
        let (ok2, stdout2, stderr2) =
            git_run(&app, &working_directory, &["worktree", "add", &worktree_path, &branch]).await?;
        if !ok2 {
            return Err(WorktreeError::Git(stderr2));
        }
        return Ok(WorktreeInfo {
            path: format!("{working_directory}/{worktree_path}"),
            branch,
            head: stdout2.trim().to_string(),
            is_bare: false,
            is_locked: false,
            is_prunable: false,
        });
    }

    Ok(WorktreeInfo {
        path: format!("{working_directory}/{worktree_path}"),
        branch,
        head: stdout.trim().to_string(),
        is_bare: false,
        is_locked: false,
        is_prunable: false,
    })
}

#[tauri::command]
pub async fn list_worktrees(
    app: AppHandle,
    working_directory: String,
) -> Result<Vec<WorktreeInfo>, WorktreeError> {
    let (ok, stdout, stderr) =
        git_run(&app, &working_directory, &["worktree", "list", "--porcelain"]).await?;

    if !ok {
        return Err(WorktreeError::Git(stderr));
    }

    Ok(parse_worktree_list(&stdout))
}

#[tauri::command]
pub async fn remove_worktree(
    app: AppHandle,
    working_directory: String,
    worktree_path: String,
    delete_branch: bool,
) -> Result<(), WorktreeError> {
    let (ok, _stdout, stderr) =
        git_run(&app, &working_directory, &["worktree", "remove", &worktree_path, "--force"]).await?;

    if !ok {
        return Err(WorktreeError::Git(stderr));
    }

    if delete_branch {
        if let Some(branch_name) = worktree_path.split('/').last() {
            let branch = format!("loopforge/{branch_name}");
            let _ = git_run(&app, &working_directory, &["branch", "-d", &branch]).await;
        }
    }

    Ok(())
}

fn artifact_dir(app: &AppHandle, project_id: &str) -> Result<PathBuf, WorktreeError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| WorktreeError::Path(err.to_string()))?;
    Ok(data_dir.join("projects").join(project_id))
}

#[tauri::command]
pub async fn check_scope_overlap(
    app: AppHandle,
    db: State<'_, DbState>,
    loop_state: State<'_, LoopManagerState>,
    working_directory: String,
) -> Result<Vec<ScopeOverlap>, WorktreeError> {
    let active_project_ids: Vec<String> = {
        let handles = loop_state.0.lock().map_err(|_| WorktreeError::LockPoisoned)?;
        handles.keys().cloned().collect()
    };

    let conn = db.0.lock().map_err(|_| WorktreeError::LockPoisoned)?;

    let mut projects_in_dir: Vec<String> = Vec::new();
    for pid in &active_project_ids {
        let matches: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM projects WHERE id = ?1 AND working_directory = ?2",
                rusqlite::params![pid, working_directory],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);
        if matches {
            projects_in_dir.push(pid.clone());
        }
    }
    drop(conn);

    let mut project_files: Vec<(String, Vec<String>)> = Vec::new();
    for pid in &projects_in_dir {
        let prd_path = artifact_dir(&app, pid)?.join("prd.json");
        if let Ok(prd) = Prd::load(&prd_path) {
            let pending_files: Vec<String> = prd
                .stories
                .iter()
                .filter(|story| !story.passes && !story.blocked)
                .flat_map(|story| story.scope.files_to_modify.iter().cloned())
                .collect();
            project_files.push((pid.clone(), pending_files));
        }
    }

    let mut overlaps: Vec<ScopeOverlap> = Vec::new();
    for outer in 0..project_files.len() {
        for inner in (outer + 1)..project_files.len() {
            let (ref pid_a, ref files_a) = project_files[outer];
            let (ref pid_b, ref files_b) = project_files[inner];

            let overlapping: Vec<String> = files_a
                .iter()
                .filter(|file| files_b.contains(file))
                .cloned()
                .collect();

            if !overlapping.is_empty() {
                overlaps.push(ScopeOverlap {
                    project_id_a: pid_a.clone(),
                    project_id_b: pid_b.clone(),
                    overlapping_files: overlapping,
                });
            }
        }
    }

    Ok(overlaps)
}

#[tauri::command]
pub async fn start_loop_with_worktree(
    app: AppHandle,
    db: State<'_, DbState>,
    loop_state: State<'_, LoopManagerState>,
    args: crate::loop_manager::StartLoopArgs,
) -> Result<String, crate::loop_manager::LoopError> {
    let base_working_directory = args.working_directory.clone().unwrap_or_default();
    let overlaps = check_scope_overlap(
        app.clone(),
        db.clone(),
        loop_state.clone(),
        base_working_directory.clone(),
    )
    .await
    .unwrap_or_default();

    let has_active_loop_in_dir = {
        let handles = loop_state.0.lock().map_err(|_| crate::loop_manager::LoopError::LockPoisoned)?;
        handles
            .keys()
            .any(|pid| pid != &args.project_id)
            && !overlaps.is_empty()
    };

    let effective_working_dir = if has_active_loop_in_dir {
        let loop_name = args.project_id.clone();
        match create_worktree(app.clone(), base_working_directory.clone(), loop_name).await {
            Ok(worktree) => worktree.path,
            Err(_) => base_working_directory.clone(),
        }
    } else {
        base_working_directory
    };

    let effective_args = crate::loop_manager::StartLoopArgs {
        working_directory: Some(effective_working_dir),
        ..args
    };

    crate::loop_manager::start_loop(app, effective_args).await
}

#[tauri::command]
pub async fn cleanup_worktree_after_loop(
    app: AppHandle,
    working_directory: String,
    project_id: String,
    delete_branch: bool,
) -> Result<(), WorktreeError> {
    let worktree_path = format!(".loopforge/worktrees/{project_id}");
    let full_path = format!("{working_directory}/{worktree_path}");

    if std::path::Path::new(&full_path).exists() {
        remove_worktree(app, working_directory, full_path, delete_branch).await
    } else {
        Ok(())
    }
}
