use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Database lock poisoned")]
    LockPoisoned,
    #[error("Path resolution failed: {0}")]
    Path(String),
}

pub struct DbState(pub Mutex<Connection>);

impl DbState {
    pub fn open<R: Runtime>(app: &AppHandle<R>) -> Result<Self, DbError> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|err| DbError::Path(err.to_string()))?;
        std::fs::create_dir_all(&data_dir)
            .map_err(|err| DbError::Path(format!("Failed to create data dir: {err}")))?;
        let db_path = data_dir.join("loopforge.db");

        crate::storage::migration::migrate_legacy_storage_if_needed(&data_dir, &db_path)
            .map_err(DbError::Path)?;

        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Self::run_migrations(&conn)?;
        Ok(Self(Mutex::new(conn)))
    }

    fn run_migrations(conn: &Connection) -> Result<(), DbError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        let current_version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM _migrations",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if current_version < 1 {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS projects (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT NOT NULL DEFAULT '',
                    status TEXT NOT NULL DEFAULT 'draft',
                    working_directory TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT PRIMARY KEY,
                    project_id TEXT NOT NULL REFERENCES projects(id),
                    started_at TEXT NOT NULL DEFAULT (datetime('now')),
                    ended_at TEXT,
                    total_iterations INTEGER NOT NULL DEFAULT 0,
                    stories_completed INTEGER NOT NULL DEFAULT 0
                );
                CREATE TABLE IF NOT EXISTS iterations (
                    id TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL REFERENCES sessions(id),
                    story_id TEXT NOT NULL,
                    started_at TEXT NOT NULL DEFAULT (datetime('now')),
                    duration_secs INTEGER NOT NULL DEFAULT 0,
                    result TEXT NOT NULL DEFAULT 'pending',
                    agent_used TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS agent_configs (
                    id TEXT PRIMARY KEY,
                    project_id TEXT NOT NULL REFERENCES projects(id),
                    phase TEXT NOT NULL,
                    agent_name TEXT NOT NULL,
                    priority_order INTEGER NOT NULL DEFAULT 0,
                    model_override TEXT
                );
                CREATE TABLE IF NOT EXISTS app_state (
                    project_id TEXT PRIMARY KEY,
                    loop_args_json TEXT NOT NULL,
                    loop_status TEXT NOT NULL DEFAULT 'interrupted',
                    saved_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                INSERT INTO _migrations (version) VALUES (1);",
            )?;
        }

        if current_version < 2 {
            conn.execute_batch(
                "ALTER TABLE projects ADD COLUMN wizard_step TEXT DEFAULT NULL;
                ALTER TABLE projects ADD COLUMN wizard_state_json TEXT DEFAULT NULL;
                INSERT INTO _migrations (version) VALUES (2);",
            )?;
        }

        if current_version < 3 {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS connections (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                CREATE TABLE IF NOT EXISTS connection_repos (
                    connection_id TEXT NOT NULL REFERENCES connections(id) ON DELETE CASCADE,
                    repo_path TEXT NOT NULL,
                    display_name TEXT,
                    PRIMARY KEY (connection_id, repo_path)
                );
                INSERT INTO _migrations (version) VALUES (3);",
            )?;
        }

        if current_version < 4 {
            conn.execute_batch(
                "ALTER TABLE projects ADD COLUMN notification_prefs TEXT DEFAULT NULL;
                INSERT INTO _migrations (version) VALUES (4);",
            )?;
        }

        if current_version < 5 {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS plugin_configs (
                    plugin_name TEXT PRIMARY KEY,
                    slot TEXT NOT NULL,
                    enabled INTEGER NOT NULL DEFAULT 1,
                    config_json TEXT NOT NULL DEFAULT '{}',
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                INSERT INTO _migrations (version) VALUES (5);",
            )?;
        }

        if current_version < 6 {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS ask_conversations (
                    id TEXT PRIMARY KEY,
                    project_id TEXT NOT NULL REFERENCES projects(id),
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                CREATE TABLE IF NOT EXISTS ask_messages (
                    id TEXT PRIMARY KEY,
                    conversation_id TEXT NOT NULL REFERENCES ask_conversations(id) ON DELETE CASCADE,
                    role TEXT NOT NULL,
                    content TEXT NOT NULL,
                    agent TEXT,
                    model TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                CREATE INDEX IF NOT EXISTS idx_ask_messages_conv
                    ON ask_messages(conversation_id, created_at);
                INSERT INTO _migrations (version) VALUES (6);",
            )?;
        }

        if current_version < 7 {
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS notifications (
                    id TEXT PRIMARY KEY,
                    project_id TEXT NOT NULL REFERENCES projects(id),
                    notification_type TEXT NOT NULL,
                    title TEXT NOT NULL,
                    message TEXT NOT NULL,
                    ring_color TEXT NOT NULL DEFAULT 'cyan',
                    read INTEGER NOT NULL DEFAULT 0,
                    timestamp INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_notifications_project
                    ON notifications(project_id, timestamp DESC);
                INSERT INTO _migrations (version) VALUES (7);",
            )?;
        }

        Ok(())
    }

    pub fn save_loop_state(&self, project_id: &str, args_json: &str) -> Result<(), DbError> {
        let conn = self.0.lock().map_err(|_| DbError::LockPoisoned)?;
        conn.execute(
            "INSERT OR REPLACE INTO app_state (project_id, loop_args_json, loop_status)
             VALUES (?1, ?2, 'interrupted')",
            rusqlite::params![project_id, args_json],
        )?;
        Ok(())
    }

    pub fn load_loop_states(&self) -> Vec<(String, String)> {
        let Ok(conn) = self.0.lock() else {
            return vec![];
        };
        let Ok(mut stmt) = conn.prepare("SELECT project_id, loop_args_json FROM app_state") else {
            return vec![];
        };
        stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map(|rows| rows.filter_map(Result::ok).collect())
        .unwrap_or_default()
    }

    pub fn clear_loop_states(&self) {
        if let Ok(conn) = self.0.lock() {
            let _ = conn.execute("DELETE FROM app_state", []);
        }
    }
}
