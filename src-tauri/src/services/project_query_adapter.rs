use app_services::{OutputEntry, ServiceError};
use ralph_core::prd::Prd;
use rusqlite::Connection;
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeListing {
    pub id: String,
    pub name: String,
    pub status: String,
    pub total_stories: Option<usize>,
    pub stories_completed: Option<usize>,
    pub current_agent: Option<String>,
}

pub type HomeListingGroups = BTreeMap<String, Vec<HomeListing>>;

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MonitorSnapshot {
    pub project_id: String,
    pub status: String,
    pub session_id: Option<String>,
    pub session_started_at: Option<String>,
    pub session_ended_at: Option<String>,
    pub has_active_handle: bool,
    pub is_stale: bool,
    pub stories_total: Option<usize>,
    pub stories_done: Option<usize>,
    pub current_story: Option<String>,
    pub has_partial_progress: bool,
    pub recent_output: Vec<OutputEntry>,
    pub events: Vec<String>,
}
pub fn home_listings(
    conn: &Connection,
    active: &HashSet<String>,
    artifacts: impl Fn(&str) -> Result<PathBuf, String>,
) -> Result<HomeListingGroups, ServiceError> {
    let mut groups = BTreeMap::from([
        ("draft".to_string(), vec![]),
        ("ready".to_string(), vec![]),
        ("paused".to_string(), vec![]),
    ]);
    let mut stmt = conn
        .prepare("SELECT id, name, status FROM projects ORDER BY updated_at DESC")
        .map_err(db)?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(db)?;
    for row in rows {
        let (id, name, status) = row.map_err(db)?;
        let dir = artifacts(&id).map_err(ServiceError::Internal)?;
        let prd = Prd::load(&dir.join("prd.json")).ok();
        let current_agent = config_agent(&dir);
        let key = if status == "draft"
            && prd.as_ref().is_some_and(|value| !value.stories.is_empty())
            && current_agent.is_some()
        {
            "ready"
        } else if status == "draft" && !active.contains(&id) {
            "draft"
        } else {
            "paused"
        };
        groups.get_mut(key).unwrap().push(HomeListing {
            id,
            name,
            status: key.to_string(),
            total_stories: prd.as_ref().map(Prd::total_stories),
            stories_completed: prd.as_ref().map(Prd::passed_count),
            current_agent,
        });
    }
    Ok(groups)
}

pub fn monitor_snapshot(
    conn: &Connection,
    dir: &Path,
    project_id: &str,
    has_active_handle: bool,
) -> Result<MonitorSnapshot, ServiceError> {
    let status = conn
        .query_row(
            "SELECT status FROM projects WHERE id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .map_err(db)?;
    let (session_id, started_at, ended_at) = conn
        .query_row(
            "SELECT id, started_at, ended_at FROM sessions WHERE project_id = ?1 ORDER BY started_at DESC LIMIT 1",
            [project_id],
            |row| Ok((Some(row.get::<_, String>(0)?), row.get(1).ok(), row.get(2).ok())),
        )
        .unwrap_or((None, None, None));
    let mut stmt = conn
        .prepare("SELECT story_id, started_at FROM iterations WHERE session_id = ?1 ORDER BY started_at ASC")
        .map_err(db)?;
    let mut current_story = None;
    let mut last_progress = None;
    let events = session_id
        .as_ref()
        .map(|id| {
            let mut names = started_at
                .iter()
                .map(|_| "session_started".to_string())
                .collect::<Vec<_>>();
            let rows = stmt
                .query_map([id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(db)?;
            for row in rows {
                let (story_id, started_at) = row.map_err(db)?;
                current_story = Some(story_id);
                last_progress = Some(started_at);
                names.push("iteration_completed".to_string());
            }
            if ended_at.is_some() {
                names.push("session_ended".to_string());
            }
            Ok(names)
        })
        .transpose()?
        .unwrap_or_default();
    let prd = Prd::load(&dir.join("prd.json")).ok();
    let has_partial_progress = prd.is_none() || current_story.is_some();
    Ok(MonitorSnapshot {
        project_id: project_id.to_string(),
        status: if !has_active_handle && status == "active" {
            "paused".into()
        } else {
            status
        },
        session_id,
        session_started_at: started_at,
        session_ended_at: ended_at.clone(),
        has_active_handle,
        is_stale: ended_at
            .as_ref()
            .is_some_and(|ended| last_progress.as_ref().map_or(true, |value| value < ended)),
        stories_total: prd.as_ref().map(Prd::total_stories),
        stories_done: prd.as_ref().map(Prd::passed_count),
        current_story,
        has_partial_progress,
        recent_output: recent_output(dir, project_id),
        events,
    })
}

fn config_agent(dir: &Path) -> Option<String> {
    std::fs::read_to_string(dir.join("config.json"))
        .ok()
        .and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok())
        .and_then(|value| {
            value
                .get("executeAgent")
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
}

fn recent_output(dir: &Path, project_id: &str) -> Vec<OutputEntry> {
    std::fs::read_to_string(dir.join("agent_output.log"))
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|content| OutputEntry {
            project_id: project_id.to_string(),
            content: content.to_string(),
            ..OutputEntry::default()
        })
        .collect()
}

fn db(error: rusqlite::Error) -> ServiceError {
    ServiceError::Internal(error.to_string())
}
