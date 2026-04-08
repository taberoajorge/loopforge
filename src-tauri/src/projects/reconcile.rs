use crate::db::DbState;
use crate::projects::artifacts::artifact_dir;
use crate::projects::ProjectError;
use ralph_core::detection::failure_memory::FailureMemory;
use ralph_core::prd::Prd;
use std::collections::HashSet;
use std::path::Path;
use tauri::{AppHandle, Manager};

pub fn reconcile_project_prd(
    app: &AppHandle,
    project_id: &str,
    gutter_threshold: u32,
) -> Result<bool, ProjectError> {
    let project_dir = artifact_dir(app, project_id)?;
    let prd_path = project_dir.join("prd.json");
    if !prd_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&prd_path)?;
    let mut prd: Prd = serde_json::from_str(&content)?;
    let successful_story_ids = load_successful_story_ids(app, project_id)?;
    let blocked_story_ids = load_blocked_story_ids(&project_dir, gutter_threshold);
    let changed = reconcile_prd_state(&mut prd, &successful_story_ids, &blocked_story_ids);

    if changed {
        prd.save(&prd_path)
            .map_err(|err| ProjectError::Db(err.to_string()))?;
        let backup_path = project_dir.join("prd.backup.json");
        let _ = prd.save(&backup_path);
    }

    Ok(changed)
}

fn load_successful_story_ids(
    app: &AppHandle,
    project_id: &str,
) -> Result<HashSet<String>, ProjectError> {
    let db = app.state::<DbState>();
    let conn = db
        .0
        .lock()
        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT DISTINCT iterations.story_id
         FROM iterations
         INNER JOIN sessions ON sessions.id = iterations.session_id
         WHERE sessions.project_id = ?1 AND iterations.result = 'success'",
    )?;

    let rows = stmt.query_map(rusqlite::params![project_id], |row| row.get::<_, String>(0))?;
    Ok(rows.filter_map(|row| row.ok()).collect())
}

fn load_blocked_story_ids(project_dir: &Path, gutter_threshold: u32) -> HashSet<String> {
    let failure_memory = FailureMemory::load(&project_dir.join("failure_memory.json"));
    failure_memory
        .stories
        .iter()
        .filter(|record| {
            record
                .attempts
                .iter()
                .filter(|attempt| attempt.error_type != "spawn_error")
                .count() as u32
                >= gutter_threshold
                || record
                    .attempts
                    .iter()
                    .any(|attempt| attempt.error_type == "verification_exhausted")
        })
        .map(|record| record.story_id.clone())
        .collect()
}

pub(crate) fn reconcile_prd_state(
    prd: &mut Prd,
    successful_story_ids: &HashSet<String>,
    blocked_story_ids: &HashSet<String>,
) -> bool {
    let mut changed = false;

    for story in &mut prd.stories {
        let next_passes = successful_story_ids.contains(&story.id);
        let next_blocked = !next_passes && blocked_story_ids.contains(&story.id);

        if story.passes != next_passes {
            story.passes = next_passes;
            changed = true;
        }
        if story.blocked != next_blocked {
            story.blocked = next_blocked;
            changed = true;
        }
    }

    changed
}
