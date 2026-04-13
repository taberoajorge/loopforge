use crate::agents::{AgentInfo, FallbackChain};
use rusqlite::Connection;

fn in_memory_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'draft',
            working_directory TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
    )
    .unwrap();
    conn
}

fn full_schema_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    conn.execute_batch(
        "CREATE TABLE projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'draft',
            working_directory TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            wizard_step TEXT DEFAULT NULL,
            wizard_state_json TEXT DEFAULT NULL,
            notification_prefs TEXT DEFAULT NULL
        );
        CREATE TABLE sessions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id),
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            ended_at TEXT,
            total_iterations INTEGER NOT NULL DEFAULT 0,
            stories_completed INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE iterations (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id),
            story_id TEXT NOT NULL,
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            duration_secs INTEGER NOT NULL DEFAULT 0,
            result TEXT NOT NULL DEFAULT 'pending',
            agent_used TEXT NOT NULL
        );
        CREATE TABLE connections (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE TABLE connection_repos (
            connection_id TEXT NOT NULL REFERENCES connections(id) ON DELETE CASCADE,
            repo_path TEXT NOT NULL,
            display_name TEXT,
            PRIMARY KEY (connection_id, repo_path)
        );
        CREATE TABLE plugin_configs (
            plugin_name TEXT PRIMARY KEY,
            slot TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            config_json TEXT NOT NULL DEFAULT '{}',
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .unwrap();
    conn
}

fn insert_project(conn: &Connection, id: &str, name: &str, status: &str) {
    let now = "2026-01-01T00:00:00Z";
    conn.execute(
        "INSERT INTO projects (id, name, description, status, working_directory, created_at, updated_at)
         VALUES (?1, ?2, '', ?3, '/tmp/test', ?4, ?5)",
        rusqlite::params![id, name, status, now, now],
    )
    .unwrap();
}

fn query_status(conn: &Connection, id: &str) -> String {
    conn.query_row(
        "SELECT status FROM projects WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    )
    .unwrap()
}

#[test]
fn create_project_persists_with_draft_status() {
    let conn = in_memory_db();
    insert_project(&conn, "proj-001", "Test Project", "draft");
    assert_eq!(query_status(&conn, "proj-001"), "draft");
}

#[test]
fn create_project_stores_name_and_description() {
    let conn = in_memory_db();
    let now = "2026-01-01T00:00:00Z";
    conn.execute(
        "INSERT INTO projects (id, name, description, status, working_directory, created_at, updated_at)
         VALUES ('p1', 'My App', 'A cool app', 'draft', '/tmp', ?1, ?2)",
        rusqlite::params![now, now],
    )
    .unwrap();

    let (name, description): (String, String) = conn
        .query_row(
            "SELECT name, description FROM projects WHERE id = 'p1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(name, "My App");
    assert_eq!(description, "A cool app");
}

#[test]
fn list_projects_groups_correctly_by_status() {
    let conn = in_memory_db();
    insert_project(&conn, "p-active", "Active", "active");
    insert_project(&conn, "p-paused", "Paused", "paused");
    insert_project(&conn, "p-draft", "Draft", "draft");
    insert_project(&conn, "p-completed", "Completed", "completed");
    insert_project(&conn, "p-archived", "Archived", "archived");

    let mut stmt = conn
        .prepare("SELECT status FROM projects ORDER BY updated_at DESC")
        .unwrap();
    let statuses: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|result| result.ok())
        .collect();

    let count = |target: &str| {
        statuses
            .iter()
            .filter(|status| status.as_str() == target)
            .count()
    };

    assert_eq!(count("active"), 1);
    assert_eq!(count("paused"), 1);
    assert_eq!(count("draft"), 1);
    assert_eq!(count("completed"), 1);
    assert_eq!(count("archived"), 1);
}

#[test]
fn pause_project_updates_status_to_paused() {
    let conn = in_memory_db();
    insert_project(&conn, "proj-001", "Test", "active");

    let now = "2026-01-02T00:00:00Z";
    let rows_updated = conn
        .execute(
            "UPDATE projects SET status = 'paused', updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, "proj-001"],
        )
        .unwrap();

    assert_eq!(rows_updated, 1);
    assert_eq!(query_status(&conn, "proj-001"), "paused");
}

#[test]
fn resume_project_updates_status_to_active() {
    let conn = in_memory_db();
    insert_project(&conn, "proj-001", "Test", "paused");

    let now = "2026-01-02T00:00:00Z";
    conn.execute(
        "UPDATE projects SET status = 'active', updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, "proj-001"],
    )
    .unwrap();

    assert_eq!(query_status(&conn, "proj-001"), "active");
}

#[test]
fn pause_then_resume_cycle_returns_to_active() {
    let conn = in_memory_db();
    insert_project(&conn, "proj-001", "Test", "active");

    let now = "2026-01-02T00:00:00Z";
    conn.execute(
        "UPDATE projects SET status = 'paused', updated_at = ?1 WHERE id = 'proj-001'",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "UPDATE projects SET status = 'active', updated_at = ?1 WHERE id = 'proj-001'",
        rusqlite::params![now],
    )
    .unwrap();

    assert_eq!(query_status(&conn, "proj-001"), "active");
}

#[test]
fn update_returns_zero_rows_for_unknown_id() {
    let conn = in_memory_db();
    let now = "2026-01-01T00:00:00Z";
    let rows_updated = conn
        .execute(
            "UPDATE projects SET status = 'paused', updated_at = ?1 WHERE id = 'nonexistent'",
            rusqlite::params![now],
        )
        .unwrap();
    assert_eq!(rows_updated, 0);
}

#[test]
fn detect_agents_result_is_vec_of_agent_info() {
    let agents: Vec<AgentInfo> = vec![
        AgentInfo {
            name: "claude".to_string(),
            binary: "claude".to_string(),
            version: None,
            available: false,
        },
        AgentInfo {
            name: "codex".to_string(),
            binary: "codex".to_string(),
            version: Some("1.0.0".to_string()),
            available: true,
        },
    ];
    assert_eq!(agents.len(), 2);
    assert!(!agents[0].available);
    assert!(agents[1].available);
    assert_eq!(agents[1].version.as_deref(), Some("1.0.0"));
}

#[test]
fn fallback_chain_filters_unavailable_agents() {
    let agents = vec![
        AgentInfo {
            name: "claude".to_string(),
            binary: "claude".to_string(),
            version: None,
            available: false,
        },
        AgentInfo {
            name: "codex".to_string(),
            binary: "codex".to_string(),
            version: None,
            available: true,
        },
    ];
    let chain = FallbackChain::new(agents);
    assert_eq!(chain.current_agent().unwrap().name, "codex");
}

#[test]
fn fallback_chain_exhausted_when_all_unavailable() {
    let agents = vec![AgentInfo {
        name: "claude".to_string(),
        binary: "claude".to_string(),
        version: None,
        available: false,
    }];
    let chain = FallbackChain::new(agents);
    assert!(chain.is_exhausted());
    assert!(chain.current_agent().is_none());
}

#[test]
fn fallback_chain_advances_to_next_agent() {
    let agents = vec![
        AgentInfo {
            name: "claude".to_string(),
            binary: "claude".to_string(),
            version: None,
            available: true,
        },
        AgentInfo {
            name: "codex".to_string(),
            binary: "codex".to_string(),
            version: None,
            available: true,
        },
    ];
    let mut chain = FallbackChain::new(agents);
    assert_eq!(chain.current_agent().unwrap().name, "claude");
    chain.next_agent();
    assert_eq!(chain.current_agent().unwrap().name, "codex");
}

#[test]
fn fallback_chain_reset_returns_to_first() {
    let agents = vec![
        AgentInfo {
            name: "claude".to_string(),
            binary: "claude".to_string(),
            version: None,
            available: true,
        },
        AgentInfo {
            name: "codex".to_string(),
            binary: "codex".to_string(),
            version: None,
            available: true,
        },
    ];
    let mut chain = FallbackChain::new(agents);
    chain.next_agent();
    chain.reset();
    assert_eq!(chain.current_agent().unwrap().name, "claude");
}

#[test]
fn wizard_draft_save_and_resume_cycle() {
    let conn = full_schema_db();
    let now = "2026-03-23T00:00:00Z";

    conn.execute(
        "INSERT INTO projects (id, name, status, working_directory, wizard_step, wizard_state_json, created_at, updated_at)
         VALUES ('draft-1', 'Draft Test', 'draft', '/tmp/test', 'plan', '{\"planComplete\":false}', ?1, ?2)",
        rusqlite::params![now, now],
    ).unwrap();

    let (step, state): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT wizard_step, wizard_state_json FROM projects WHERE id = 'draft-1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(step.as_deref(), Some("plan"));
    assert!(state.unwrap().contains("planComplete"));

    conn.execute(
        "UPDATE projects SET wizard_step = NULL, status = 'active', updated_at = ?1 WHERE id = 'draft-1'",
        rusqlite::params![now],
    ).unwrap();

    let finalized_step: Option<String> = conn
        .query_row(
            "SELECT wizard_step FROM projects WHERE id = 'draft-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(finalized_step.is_none());
    assert_eq!(query_status(&conn, "draft-1"), "active");
}

#[test]
fn ci_feedback_loop_verification_retries_tracked() {
    let conn = full_schema_db();
    let now = "2026-03-23T00:00:00Z";

    conn.execute(
        "INSERT INTO projects (id, name, status, working_directory, created_at, updated_at)
         VALUES ('proj-ci', 'CI Test', 'active', '/tmp', ?1, ?2)",
        rusqlite::params![now, now],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO sessions (id, project_id, started_at)
         VALUES ('sess-ci', 'proj-ci', ?1)",
        rusqlite::params![now],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO iterations (id, session_id, story_id, duration_secs, result, agent_used)
         VALUES ('iter-1', 'sess-ci', 'S-001', 10, 'failed', 'claude')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO iterations (id, session_id, story_id, duration_secs, result, agent_used)
         VALUES ('iter-2', 'sess-ci', 'S-001', 15, 'failed', 'claude')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO iterations (id, session_id, story_id, duration_secs, result, agent_used)
         VALUES ('iter-3', 'sess-ci', 'S-001', 20, 'success', 'claude')",
        [],
    )
    .unwrap();

    let attempts: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM iterations WHERE session_id = 'sess-ci' AND story_id = 'S-001'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(attempts, 3);

    let last_result: String = conn
        .query_row(
            "SELECT result FROM iterations WHERE session_id = 'sess-ci' AND story_id = 'S-001' ORDER BY rowid DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(last_result, "success");
}

#[test]
fn connection_crud_lifecycle() {
    let conn = full_schema_db();

    conn.execute(
        "INSERT INTO connections (id, name) VALUES ('conn-1', 'Monorepo Connection')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO connection_repos (connection_id, repo_path, display_name) VALUES ('conn-1', '/tmp/repo-a', 'Repo A')",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO connection_repos (connection_id, repo_path, display_name) VALUES ('conn-1', '/tmp/repo-b', 'Repo B')",
        [],
    ).unwrap();

    let repo_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM connection_repos WHERE connection_id = 'conn-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(repo_count, 2);

    conn.execute("DELETE FROM connections WHERE id = 'conn-1'", [])
        .unwrap();

    let leftover: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM connection_repos WHERE connection_id = 'conn-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(leftover, 0);
}

#[test]
fn notification_prefs_stored_and_retrieved() {
    let conn = full_schema_db();
    let now = "2026-03-23T00:00:00Z";

    conn.execute(
        "INSERT INTO projects (id, name, status, working_directory, created_at, updated_at)
         VALUES ('proj-notif', 'Notif Test', 'active', '/tmp', ?1, ?2)",
        rusqlite::params![now, now],
    )
    .unwrap();

    let prefs_json =
        r#"{"story_blocked":{"ring":true,"os":true},"loop_completed":{"ring":true,"os":false}}"#;
    conn.execute(
        "UPDATE projects SET notification_prefs = ?1 WHERE id = 'proj-notif'",
        rusqlite::params![prefs_json],
    )
    .unwrap();

    let stored: Option<String> = conn
        .query_row(
            "SELECT notification_prefs FROM projects WHERE id = 'proj-notif'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(stored.unwrap().contains("story_blocked"));
}

#[test]
fn plugin_config_round_trip() {
    let conn = full_schema_db();

    conn.execute(
        "INSERT OR REPLACE INTO plugin_configs (plugin_name, slot, enabled, config_json) VALUES ('builtin-runtime', 'runtime', 1, '{}')",
        [],
    ).unwrap();

    conn.execute(
        "INSERT OR REPLACE INTO plugin_configs (plugin_name, slot, enabled, config_json) VALUES ('builtin-runtime', 'runtime', 0, '{\"key\":\"value\"}')",
        [],
    ).unwrap();

    let (enabled, config): (bool, String) = conn
        .query_row(
            "SELECT enabled, config_json FROM plugin_configs WHERE plugin_name = 'builtin-runtime'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert!(!enabled);
    assert!(config.contains("key"));
}

#[test]
fn session_stats_calculation() {
    let conn = full_schema_db();
    let now = "2026-03-23T00:00:00Z";

    conn.execute(
        "INSERT INTO projects (id, name, status, working_directory, created_at, updated_at)
         VALUES ('proj-stats', 'Stats Test', 'active', '/tmp', ?1, ?2)",
        rusqlite::params![now, now],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO sessions (id, project_id, started_at, total_iterations)
         VALUES ('sess-stats', 'proj-stats', ?1, 0)",
        rusqlite::params![now],
    )
    .unwrap();

    for idx in 0..5 {
        let iter_id = format!("iter-s-{idx}");
        let result = if idx < 4 { "success" } else { "failed" };
        conn.execute(
            "INSERT INTO iterations (id, session_id, story_id, duration_secs, result, agent_used)
             VALUES (?1, 'sess-stats', ?2, ?3, ?4, 'claude')",
            rusqlite::params![iter_id, format!("S-{:03}", idx + 1), (idx + 1) * 60, result],
        )
        .unwrap();
    }

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM iterations WHERE session_id = 'sess-stats'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let success: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM iterations WHERE session_id = 'sess-stats' AND result = 'success'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let failed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM iterations WHERE session_id = 'sess-stats' AND result = 'failed'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(total, 5);
    assert_eq!(success, 4);
    assert_eq!(failed, 1);

    let success_rate = success as f64 / total as f64;
    assert!((success_rate - 0.8).abs() < 0.001);
}

#[cfg(feature = "frozen")]
#[test]
fn diagnostic_parser_handles_tsc_output() {
    let sample = "src/app.ts(12,5): error TS2304: Cannot find name 'foo'.\nsrc/lib.ts(3,1): error TS1005: ';' expected.";
    let diagnostics = crate::diagnostic_parser::parse_diagnostics(sample);
    assert!(diagnostics.len() >= 2);
    assert_eq!(diagnostics[0].error_type, "TS2304");
}

#[cfg(feature = "frozen")]
#[test]
fn diagnostic_parser_handles_empty_input() {
    let diagnostics = crate::diagnostic_parser::parse_diagnostics("");
    assert!(diagnostics.is_empty());
}

#[cfg(feature = "frozen")]
#[test]
fn summary_complexity_classification() {
    assert_eq!(
        crate::summary_generator::classify_complexity_for_test(5, 50, 1),
        "trivial"
    );
    assert_eq!(
        crate::summary_generator::classify_complexity_for_test(15, 300, 4),
        "small"
    );
    assert_eq!(
        crate::summary_generator::classify_complexity_for_test(30, 1000, 10),
        "medium"
    );
    assert_eq!(
        crate::summary_generator::classify_complexity_for_test(100, 5000, 20),
        "large"
    );
}
