use crate::db::DbState;
use crate::projects::artifacts::artifact_dir;
use crate::projects::{IterationStory, ProjectError};
use ralph_core::prd::Prd;
use rusqlite::OptionalExtension;
use tauri::{AppHandle, State};

pub async fn get_project_stories(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
) -> Result<Vec<IterationStory>, ProjectError> {
    let dir = artifact_dir(&app, &project_id)?;
    let prd_path = dir.join("prd.json");

    let stories = if prd_path.exists() {
        Prd::load(&prd_path).map(|prd| prd.stories).unwrap_or_default()
    } else {
        return Ok(Vec::new());
    };

    let conn = db
        .0
        .lock()
        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;

    let latest_session: Option<(String, Option<String>)> = conn
        .query_row(
            "SELECT id, ended_at FROM sessions WHERE project_id = ?1 ORDER BY started_at DESC LIMIT 1",
            rusqlite::params![project_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .optional()
        .map_err(|err| ProjectError::Db(err.to_string()))?;
    let latest_session_id = latest_session.as_ref().map(|(session_id, _)| session_id.clone());
    let latest_session_is_open = latest_session
        .as_ref()
        .map(|(_, ended_at)| ended_at.is_none())
        .unwrap_or(false);

    let active_story_id: Option<String> = if latest_session_is_open {
        if let Some(ref sid) = latest_session_id {
            conn.query_row(
                "SELECT story_id FROM iterations WHERE session_id = ?1 ORDER BY started_at DESC LIMIT 1",
                rusqlite::params![sid],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| ProjectError::Db(err.to_string()))?
        } else {
            None
        }
    } else {
        None
    };
    let fallback_story_id = if latest_session_is_open && active_story_id.is_none() {
        stories
            .iter()
            .find(|story| !story.passes && !story.blocked)
            .map(|story| story.id.clone())
    } else {
        None
    };
    let effective_active_story_id = active_story_id.or(fallback_story_id);

    let iteration_stories: Vec<IterationStory> = stories
        .iter()
        .map(|story| {
            let (duration_secs, attempts, latest_result): (Option<i64>, i64, Option<String>) = conn
                .query_row(
                    "SELECT SUM(i.duration_secs), COUNT(i.id),
                            (SELECT i2.result FROM iterations i2
                             INNER JOIN sessions s2 ON s2.id = i2.session_id
                             WHERE s2.project_id = ?1 AND i2.story_id = ?2
                             ORDER BY i2.started_at DESC LIMIT 1)
                     FROM iterations i
                     INNER JOIN sessions s ON s.id = i.session_id
                     WHERE s.project_id = ?1 AND i.story_id = ?2",
                    rusqlite::params![project_id, story.id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .unwrap_or((None, 0, None));

            let is_active_story =
                latest_session_is_open && effective_active_story_id.as_deref() == Some(&story.id);
            let status = if is_active_story {
                "current".to_string()
            } else if story.passes || latest_result.as_deref() == Some("success") {
                "completed".to_string()
            } else if story.blocked {
                "blocked".to_string()
            } else {
                "pending".to_string()
            };

            IterationStory {
                id: story.id.clone(),
                title: story.title.clone(),
                status,
                duration_secs,
                attempts,
            }
        })
        .collect();

    Ok(iteration_stories)
}
