use crate::ask_engine::types::AskMessage;
use rusqlite::Connection;
use std::path::Path;

const MAX_PLAN_CHARS: usize = 4000;
const MAX_GUARDRAILS_CHARS: usize = 2000;
const MAX_HISTORY_MESSAGES: usize = 20;
const MAX_ITERATIONS: usize = 10;

pub fn build_ask_context(
    artifact_dir: &Path,
    conn: &Connection,
    project_id: &str,
    history: &[AskMessage],
) -> String {
    let mut sections: Vec<String> = Vec::new();

    sections
        .push("You are a project assistant for a software project managed by LoopForge.".into());
    sections
        .push("Answer questions about the project state, progress, blockers, and stories.".into());
    sections.push("Be concise and direct. Reference story IDs when relevant.\n".into());

    if let Some(stories_section) = build_stories_section(artifact_dir) {
        sections.push(stories_section);
    }

    if let Some(plan_section) = build_plan_section(artifact_dir) {
        sections.push(plan_section);
    }

    if let Some(guardrails_section) = build_guardrails_section(artifact_dir) {
        sections.push(guardrails_section);
    }

    if let Some(iterations_section) = build_iterations_section(conn, project_id) {
        sections.push(iterations_section);
    }

    if !history.is_empty() {
        sections.push(build_history_section(history));
    }

    sections.join("\n")
}

fn build_stories_section(artifact_dir: &Path) -> Option<String> {
    let prd_path = artifact_dir.join("prd.json");
    let content = std::fs::read_to_string(&prd_path).ok()?;
    let prd: serde_json::Value = serde_json::from_str(&content).ok()?;
    let stories = prd.get("stories")?.as_array()?;

    let mut lines = vec!["## PRD Stories".to_string()];
    for story in stories {
        let story_id = story.get("id").and_then(|val| val.as_str()).unwrap_or("?");
        let title = story
            .get("title")
            .and_then(|val| val.as_str())
            .unwrap_or("untitled");
        let passes = story
            .get("passes")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let blocked = story
            .get("blocked")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let attempts = story
            .get("attempts")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let status = if passes {
            "done"
        } else if blocked {
            "blocked"
        } else {
            "pending"
        };
        lines.push(format!(
            "- {story_id}: {title} [{status}, {attempts} attempts]"
        ));
    }
    lines.push(String::new());
    Some(lines.join("\n"))
}

fn build_plan_section(artifact_dir: &Path) -> Option<String> {
    let plan_path = artifact_dir.join("plan.md");
    let content = std::fs::read_to_string(&plan_path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    let truncated = if trimmed.len() > MAX_PLAN_CHARS {
        &trimmed[..MAX_PLAN_CHARS]
    } else {
        trimmed
    };
    Some(format!("## Plan (truncated)\n{truncated}\n"))
}

fn build_guardrails_section(artifact_dir: &Path) -> Option<String> {
    let path = artifact_dir.join("guardrails.md");
    let content = std::fs::read_to_string(&path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    let truncated = if trimmed.len() > MAX_GUARDRAILS_CHARS {
        &trimmed[..MAX_GUARDRAILS_CHARS]
    } else {
        trimmed
    };
    Some(format!("## Guardrails\n{truncated}\n"))
}

fn build_iterations_section(conn: &Connection, project_id: &str) -> Option<String> {
    let mut stmt = conn
        .prepare(
            "SELECT i.story_id, i.result, i.agent_used, i.duration_secs
             FROM iterations i
             JOIN sessions s ON s.id = i.session_id
             WHERE s.project_id = ?1
             ORDER BY i.started_at DESC
             LIMIT ?2",
        )
        .ok()?;

    let rows: Vec<String> = stmt
        .query_map(
            rusqlite::params![project_id, MAX_ITERATIONS as i64],
            |row| {
                let story_id: String = row.get(0)?;
                let result: String = row.get(1)?;
                let agent: String = row.get(2)?;
                let duration: i64 = row.get(3)?;
                Ok(format!(
                    "- {story_id}: {result} (agent: {agent}, {duration}s)"
                ))
            },
        )
        .ok()?
        .filter_map(Result::ok)
        .collect();

    if rows.is_empty() {
        return None;
    }

    let mut lines = vec!["## Recent Iterations (newest first)".to_string()];
    lines.extend(rows);
    lines.push(String::new());
    Some(lines.join("\n"))
}

fn build_history_section(history: &[AskMessage]) -> String {
    let start = if history.len() > MAX_HISTORY_MESSAGES {
        history.len() - MAX_HISTORY_MESSAGES
    } else {
        0
    };
    let mut lines = vec!["## Conversation History".to_string()];
    for msg in &history[start..] {
        let role_label = if msg.role == "user" {
            "User"
        } else {
            "Assistant"
        };
        lines.push(format!("{role_label}: {}", msg.content));
    }
    lines.push(String::new());
    lines.join("\n")
}
