mod models {
    #[derive(Debug, Clone)]
    pub enum ProjectStatus {
        Draft,
        Ready,
        Running,
        Paused,
        Blocked,
        Failed,
        Completed,
        Archived,
    }
    impl ProjectStatus {
        pub fn from_db_status(status: &str) -> Self {
            match status {
                "active" | "running" => Self::Running,
                "paused" => Self::Paused,
                "blocked" => Self::Blocked,
                "failed" => Self::Failed,
                "completed" => Self::Completed,
                "archived" => Self::Archived,
                "ready" => Self::Ready,
                _ => Self::Draft,
            }
        }
        pub fn resolve_canonical(
            db_status: &str,
            has_prd: bool,
            has_config: bool,
            has_active_session: bool,
        ) -> Self {
            let base_status = Self::from_db_status(db_status);
            if has_active_session {
                return Self::Running;
            }
            if matches!(base_status, Self::Running | Self::Paused) {
                return Self::Paused;
            }
            if matches!(
                base_status,
                Self::Blocked | Self::Failed | Self::Completed | Self::Archived
            ) {
                return base_status;
            }
            if !has_prd {
                return Self::Draft;
            }
            if has_config {
                Self::Ready
            } else {
                Self::Draft
            }
        }
        pub fn as_project_status(&self) -> &'static str {
            match self {
                Self::Draft => "draft",
                Self::Ready => "ready",
                Self::Running => "active",
                Self::Paused => "paused",
                Self::Blocked => "blocked",
                Self::Failed => "failed",
                Self::Completed => "completed",
                Self::Archived => "archived",
            }
        }
    }
}

#[path = "../src/services/project_query_adapter.rs"]
mod project_query_adapter;

use project_query_adapter::{home_listings, monitor_snapshot};
use rusqlite::Connection;
use serde_json::json;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn home_listing_groups_state_transitions() {
    let conn = schema();
    let root = temp_root();
    insert_project(&conn, "draft-1", "Draft", "ready");
    insert_project(&conn, "ready-1", "Ready", "draft");
    insert_project(&conn, "paused-1", "Paused", "active");
    write_project(&root, "draft-1", None, None, None);
    write_project(
        &root,
        "ready-1",
        Some(prd_json(2, 1)),
        Some(r#"{"executeAgent":"codex"}"#),
        None,
    );
    write_project(
        &root,
        "paused-1",
        Some(prd_json(1, 0)),
        Some(r#"{"executeAgent":"codex"}"#),
        None,
    );
    let groups = home_listings(&conn, &HashSet::new(), |id| Ok(root.join(id))).unwrap();
    assert_eq!(groups["draft"][0].id, "draft-1");
    assert_eq!(groups["ready"][0].id, "ready-1");
    assert_eq!(groups["paused"][0].id, "paused-1");
    assert!(groups["draft"][0].total_stories.is_none());
    assert_eq!(groups["ready"][0].stories_completed, Some(1));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn monitor_snapshot_handles_stale_and_partial_states() {
    let conn = schema();
    let root = temp_root();
    insert_project(&conn, "project-1", "Monitor", "active");
    insert_session(
        &conn,
        "session-1",
        "project-1",
        "2026-01-01T00:00:00Z",
        Some("2026-01-01T00:06:00Z"),
    );
    insert_iteration(
        &conn,
        "iter-1",
        "session-1",
        "S-001",
        "2026-01-01T00:05:00Z",
    );
    write_project(&root, "project-1", None, None, Some("line 1\nline 2\n"));
    let snapshot = monitor_snapshot(&conn, &root.join("project-1"), "project-1", false).unwrap();
    let value = serde_json::to_value(&snapshot).unwrap();
    assert_eq!(snapshot.status, "paused");
    assert!(snapshot.is_stale);
    assert!(snapshot.has_partial_progress);
    assert_eq!(snapshot.current_story.as_deref(), Some("S-001"));
    assert_eq!(
        snapshot.events,
        vec!["session_started", "iteration_completed", "session_ended"]
    );
    assert_eq!(value["recentOutput"][1]["content"], "line 2");
    let _ = fs::remove_dir_all(root);
}

fn schema() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE projects (id TEXT PRIMARY KEY, name TEXT NOT NULL, status TEXT NOT NULL, updated_at TEXT NOT NULL); CREATE TABLE sessions (id TEXT PRIMARY KEY, project_id TEXT NOT NULL, started_at TEXT, ended_at TEXT); CREATE TABLE iterations (id TEXT PRIMARY KEY, session_id TEXT NOT NULL, story_id TEXT NOT NULL, started_at TEXT NOT NULL);").unwrap();
    conn
}

fn insert_project(conn: &Connection, id: &str, name: &str, status: &str) {
    conn.execute("INSERT INTO projects (id, name, status, updated_at) VALUES (?1, ?2, ?3, '2026-01-01T00:00:00Z')", [id, name, status]).unwrap();
}
fn insert_session(
    conn: &Connection,
    id: &str,
    project_id: &str,
    started_at: &str,
    ended_at: Option<&str>,
) {
    conn.execute(
        "INSERT INTO sessions (id, project_id, started_at, ended_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, project_id, started_at, ended_at],
    )
    .unwrap();
}
fn insert_iteration(
    conn: &Connection,
    id: &str,
    session_id: &str,
    story_id: &str,
    started_at: &str,
) {
    conn.execute(
        "INSERT INTO iterations (id, session_id, story_id, started_at) VALUES (?1, ?2, ?3, ?4)",
        [id, session_id, story_id, started_at],
    )
    .unwrap();
}

fn temp_root() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("loopforge-query-{stamp}"));
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_project(
    root: &Path,
    id: &str,
    prd: Option<String>,
    config: Option<&str>,
    output: Option<&str>,
) {
    let dir = root.join(id);
    fs::create_dir_all(&dir).unwrap();
    if let Some(value) = prd {
        fs::write(dir.join("prd.json"), value).unwrap();
    }
    if let Some(value) = config {
        fs::write(dir.join("config.json"), value).unwrap();
    }
    if let Some(value) = output {
        fs::write(dir.join("agent_output.log"), value).unwrap();
    }
}

fn prd_json(total: usize, passed: usize) -> String {
    json!({"project":"LoopForge","userStories":(0..total).map(|index| json!({"id":format!("S-{index:03}"),"title":format!("Story {index}"),"acceptanceCriteria":["ok"],"passes":index < passed,"blocked":false})).collect::<Vec<_>>()}).to_string()
}
