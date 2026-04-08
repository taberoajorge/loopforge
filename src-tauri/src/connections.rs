use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Database error: {0}")]
    Db(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation: {0}")]
    Validation(String),
    #[error("IO error: {0}")]
    Io(String),
}

impl From<rusqlite::Error> for ConnectionError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Db(err.to_string())
    }
}

impl Serialize for ConnectionError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub repos: Vec<ConnectionRepo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRepo {
    pub repo_path: String,
    pub display_name: Option<String>,
}

fn validate_repo_paths(repos: &[ConnectionRepo]) -> Result<(), ConnectionError> {
    for repo in repos {
        let path = std::path::Path::new(&repo.repo_path);
        if !path.exists() {
            return Err(ConnectionError::Validation(format!(
                "Path does not exist: {}",
                repo.repo_path
            )));
        }
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            return Err(ConnectionError::Validation(format!(
                "Not a git repository: {}",
                repo.repo_path
            )));
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn create_connection(
    db: State<'_, DbState>,
    name: String,
    repos: Vec<ConnectionRepo>,
) -> Result<Connection, ConnectionError> {
    validate_repo_paths(&repos)?;

    let connection_id = Uuid::new_v4().to_string();
    let conn = db.0.lock().map_err(|_| ConnectionError::Db("Lock poisoned".into()))?;

    conn.execute(
        "INSERT INTO connections (id, name) VALUES (?1, ?2)",
        rusqlite::params![connection_id, name],
    )?;

    for repo in &repos {
        conn.execute(
            "INSERT INTO connection_repos (connection_id, repo_path, display_name) VALUES (?1, ?2, ?3)",
            rusqlite::params![connection_id, repo.repo_path, repo.display_name],
        )?;
    }

    let created_at: String = conn.query_row(
        "SELECT created_at FROM connections WHERE id = ?1",
        [&connection_id],
        |row| row.get(0),
    )?;

    Ok(Connection {
        id: connection_id,
        name,
        created_at,
        repos,
    })
}

#[tauri::command]
pub async fn list_connections(
    db: State<'_, DbState>,
) -> Result<Vec<Connection>, ConnectionError> {
    let conn = db.0.lock().map_err(|_| ConnectionError::Db("Lock poisoned".into()))?;

    let mut stmt = conn.prepare(
        "SELECT id, name, created_at FROM connections ORDER BY created_at DESC",
    )?;
    let connections: Vec<(String, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .filter_map(|row| row.ok())
        .collect();

    let mut result = Vec::with_capacity(connections.len());
    for (conn_id, name, created_at) in connections {
        let repos = load_repos(&conn, &conn_id)?;
        result.push(Connection {
            id: conn_id,
            name,
            created_at,
            repos,
        });
    }

    Ok(result)
}

fn load_repos(
    conn: &rusqlite::Connection,
    connection_id: &str,
) -> Result<Vec<ConnectionRepo>, ConnectionError> {
    let mut stmt = conn.prepare(
        "SELECT repo_path, display_name FROM connection_repos WHERE connection_id = ?1",
    )?;
    let repos = stmt
        .query_map([connection_id], |row| {
            Ok(ConnectionRepo {
                repo_path: row.get(0)?,
                display_name: row.get(1)?,
            })
        })?
        .filter_map(|row| row.ok())
        .collect();
    Ok(repos)
}

#[tauri::command]
pub async fn update_connection(
    db: State<'_, DbState>,
    connection_id: String,
    name: Option<String>,
    repos: Option<Vec<ConnectionRepo>>,
) -> Result<Connection, ConnectionError> {
    let conn = db.0.lock().map_err(|_| ConnectionError::Db("Lock poisoned".into()))?;

    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM connections WHERE id = ?1",
        [&connection_id],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count > 0)?;

    if !exists {
        return Err(ConnectionError::NotFound(connection_id));
    }

    if let Some(ref new_name) = name {
        conn.execute(
            "UPDATE connections SET name = ?1 WHERE id = ?2",
            rusqlite::params![new_name, connection_id],
        )?;
    }

    if let Some(ref new_repos) = repos {
        validate_repo_paths(new_repos)?;

        conn.execute(
            "DELETE FROM connection_repos WHERE connection_id = ?1",
            [&connection_id],
        )?;
        for repo in new_repos {
            conn.execute(
                "INSERT INTO connection_repos (connection_id, repo_path, display_name) VALUES (?1, ?2, ?3)",
                rusqlite::params![connection_id, repo.repo_path, repo.display_name],
            )?;
        }
    }

    let row = conn.query_row(
        "SELECT name, created_at FROM connections WHERE id = ?1",
        [&connection_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    )?;

    let final_repos = load_repos(&conn, &connection_id)?;
    Ok(Connection {
        id: connection_id,
        name: row.0,
        created_at: row.1,
        repos: final_repos,
    })
}

#[tauri::command]
pub async fn delete_connection(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    connection_id: String,
) -> Result<(), ConnectionError> {
    let workspace_dir = get_workspace_dir(&app, &connection_id);
    if workspace_dir.exists() {
        std::fs::remove_dir_all(&workspace_dir)
            .map_err(|err| ConnectionError::Io(format!("Failed to remove workspace: {err}")))?;
    }

    let conn = db.0.lock().map_err(|_| ConnectionError::Db("Lock poisoned".into()))?;
    conn.execute(
        "DELETE FROM connections WHERE id = ?1",
        [&connection_id],
    )?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoMergeSummary {
    pub display_name: String,
    pub repo_path: String,
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
    pub commit_count: u32,
}

#[tauri::command]
pub async fn connection_merge_summary(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    connection_id: String,
    base_ref: String,
) -> Result<Vec<RepoMergeSummary>, ConnectionError> {
    let repos = {
        let conn = db.0.lock().map_err(|_| ConnectionError::Db("Lock poisoned".into()))?;
        load_repos(&conn, &connection_id)?
    };

    let workspace_dir = get_workspace_dir(&app, &connection_id);
    let mut summaries = Vec::with_capacity(repos.len());

    for repo in &repos {
        let display = repo
            .display_name
            .as_deref()
            .unwrap_or_else(|| {
                std::path::Path::new(&repo.repo_path)
                    .file_name()
                    .and_then(|fname| fname.to_str())
                    .unwrap_or("repo")
            });
        let worktree_path = workspace_dir.join(display);

        let diff_output = tokio::process::Command::new("git")
            .args(["diff", "--stat", &base_ref, "HEAD"])
            .current_dir(&worktree_path)
            .output()
            .await;

        let (files_changed, insertions, deletions) = match diff_output {
            Ok(out) if out.status.success() => {
                parse_diff_stat(&String::from_utf8_lossy(&out.stdout))
            }
            _ => (0, 0, 0),
        };

        let commit_output = tokio::process::Command::new("git")
            .args(["rev-list", "--count", &format!("{base_ref}..HEAD")])
            .current_dir(&worktree_path)
            .output()
            .await;

        let commit_count = match commit_output {
            Ok(out) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .parse::<u32>()
                    .unwrap_or(0)
            }
            _ => 0,
        };

        summaries.push(RepoMergeSummary {
            display_name: display.to_string(),
            repo_path: repo.repo_path.clone(),
            files_changed,
            insertions,
            deletions,
            commit_count,
        });
    }

    Ok(summaries)
}

fn parse_diff_stat(output: &str) -> (u32, u32, u32) {
    let last_line = output.lines().last().unwrap_or("");
    let files = extract_stat_number(last_line, "file");
    let insertions = extract_stat_number(last_line, "insertion");
    let deletions = extract_stat_number(last_line, "deletion");
    (files, insertions, deletions)
}

fn extract_stat_number(line: &str, keyword: &str) -> u32 {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (idx, part) in parts.iter().enumerate() {
        if part.starts_with(keyword) && idx > 0 {
            return parts[idx.saturating_sub(1)].parse().unwrap_or(0);
        }
    }
    0
}

pub fn get_workspace_dir(app: &tauri::AppHandle, connection_id: &str) -> std::path::PathBuf {
    let config_dir = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("~/.config/loopforge"));
    config_dir
        .join("connections")
        .join(connection_id)
        .join("workspace")
}

pub fn build_workspace(
    workspace_dir: &std::path::Path,
    repos: &[ConnectionRepo],
) -> Result<(), ConnectionError> {
    if workspace_dir.exists() {
        std::fs::remove_dir_all(workspace_dir)
            .map_err(|err| ConnectionError::Io(format!("Failed to clean workspace: {err}")))?;
    }
    std::fs::create_dir_all(workspace_dir)
        .map_err(|err| ConnectionError::Io(format!("Failed to create workspace dir: {err}")))?;

    for repo in repos {
        let source = std::path::Path::new(&repo.repo_path);
        let link_name = repo
            .display_name
            .as_deref()
            .unwrap_or_else(|| {
                source
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("repo")
            });
        let link_path = workspace_dir.join(link_name);

        #[cfg(unix)]
        std::os::unix::fs::symlink(source, &link_path)
            .map_err(|err| ConnectionError::Io(format!(
                "Failed to symlink {} -> {}: {err}",
                link_path.display(),
                source.display()
            )))?;

        #[cfg(windows)]
        junction::create(source, &link_path)
            .map_err(|err| ConnectionError::Io(format!(
                "Failed to create junction {} -> {}: {err}",
                link_path.display(),
                source.display()
            )))?;
    }

    Ok(())
}

pub fn generate_workspace_manifest(
    workspace_dir: &std::path::Path,
    connection_name: &str,
    repos: &[ConnectionRepo],
) -> Result<(), ConnectionError> {
    let mut content = String::with_capacity(1024);
    content.push_str(&format!("# LoopForge Workspace: {connection_name}\n\n"));
    content.push_str("This workspace contains symlinks to the following repositories.\n");
    content.push_str("The agent can navigate between them as subdirectories.\n\n");
    content.push_str("## Repositories\n\n");

    for repo in repos {
        let display = repo
            .display_name
            .as_deref()
            .unwrap_or_else(|| {
                std::path::Path::new(&repo.repo_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("repo")
            });
        content.push_str(&format!("### {display}\n\n"));
        content.push_str(&format!("Path: `{}`\n\n", repo.repo_path));
    }

    content.push_str("## Instructions for Agent\n\n");
    content.push_str("When working across these repos, reference files with their repo prefix.\n");
    content.push_str("Changes in one repo may affect others; verify cross-repo imports.\n");

    let manifest_path = workspace_dir.join(".loopforge-workspace.md");
    std::fs::write(&manifest_path, content)
        .map_err(|err| ConnectionError::Io(format!("Failed to write manifest: {err}")))?;

    Ok(())
}

#[tauri::command]
pub async fn build_connection_workspace(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    connection_id: String,
) -> Result<String, ConnectionError> {
    let conn = db.0.lock().map_err(|_| ConnectionError::Db("Lock poisoned".into()))?;
    let repos = load_repos(&conn, &connection_id)?;
    drop(conn);

    let name: String = {
        let conn2 = db.0.lock().map_err(|_| ConnectionError::Db("Lock poisoned".into()))?;
        conn2.query_row(
            "SELECT name FROM connections WHERE id = ?1",
            [&connection_id],
            |row| row.get(0),
        )?
    };

    let workspace_dir = get_workspace_dir(&app, &connection_id);
    build_workspace(&workspace_dir, &repos)?;
    generate_workspace_manifest(&workspace_dir, &name, &repos)?;

    Ok(workspace_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn create_connection_worktrees(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    connection_id: String,
    branch_name: String,
) -> Result<String, ConnectionError> {
    let (repos, name) = {
        let conn = db.0.lock().map_err(|_| ConnectionError::Db("Lock poisoned".into()))?;
        let repos = load_repos(&conn, &connection_id)?;
        let name: String = conn.query_row(
            "SELECT name FROM connections WHERE id = ?1",
            [&connection_id],
            |row| row.get(0),
        )?;
        (repos, name)
    };

    let workspace_dir = get_workspace_dir(&app, &connection_id);
    if workspace_dir.exists() {
        std::fs::remove_dir_all(&workspace_dir)
            .map_err(|err| ConnectionError::Io(format!("Failed to clean workspace: {err}")))?;
    }
    std::fs::create_dir_all(&workspace_dir)
        .map_err(|err| ConnectionError::Io(format!("Failed to create workspace: {err}")))?;

    for repo in &repos {
        let display = repo
            .display_name
            .as_deref()
            .unwrap_or_else(|| {
                std::path::Path::new(&repo.repo_path)
                    .file_name()
                    .and_then(|fname| fname.to_str())
                    .unwrap_or("repo")
            });
        let worktree_path = workspace_dir.join(display);
        let worktree_branch = format!("loopforge/{branch_name}/{display}");

        let output = tokio::process::Command::new("git")
            .args([
                "worktree", "add",
                &worktree_path.to_string_lossy(),
                "-b", &worktree_branch,
            ])
            .current_dir(&repo.repo_path)
            .output()
            .await
            .map_err(|err| ConnectionError::Io(format!("git worktree failed for {display}: {err}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ConnectionError::Io(format!(
                "git worktree add failed for {display}: {stderr}"
            )));
        }
    }

    generate_workspace_manifest(&workspace_dir, &name, &repos)?;

    Ok(workspace_dir.to_string_lossy().to_string())
}
