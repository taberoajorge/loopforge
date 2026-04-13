use crate::projects::artifacts::artifact_dir;
use crate::projects::ProjectError;
use ralph_core::prd::Prd;
pub use ralph_core::prd::UserStory;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{AppHandle, Runtime};

pub(crate) fn load_prd_from_paths(
    artifacts: &Path,
    working_directory: &Path,
) -> Result<Option<Prd>, ProjectError> {
    let artifact_path = artifacts.join("prd.json");
    if artifact_path.exists() {
        return Prd::load(&artifact_path)
            .map(Some)
            .map_err(|err| ProjectError::Path(err.to_string()));
    }

    let legacy_path = working_directory.join("prd.json");
    if legacy_path.exists() {
        return Prd::load(&legacy_path)
            .map(Some)
            .map_err(|err| ProjectError::Path(err.to_string()));
    }

    Ok(None)
}

fn load_prd<R: Runtime>(app: &AppHandle<R>, project_id: &str) -> Result<Prd, ProjectError> {
    let dir = artifact_dir(app, project_id)?;
    let prd_path = dir.join("prd.json");
    if prd_path.exists() {
        Prd::load(&prd_path)
            .map_err(|_| ProjectError::Path(format!("Failed to load prd.json for {project_id}")))
    } else {
        Ok(Prd {
            project_name: String::new(),
            feature: String::new(),
            working_directory: String::new(),
            branch_name: None,
            stories: Vec::new(),
            generated_at: Some(chrono::Utc::now().to_rfc3339()),
        })
    }
}

fn save_prd<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
    prd: &Prd,
) -> Result<(), ProjectError> {
    let dir = artifact_dir(app, project_id)?;
    std::fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(prd)?;
    std::fs::write(dir.join("prd.json"), json)?;
    Ok(())
}

fn total_estimated_minutes(prd: &Prd) -> u32 {
    prd.stories
        .iter()
        .map(|story| story.estimated_minutes)
        .sum()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoriesResponse {
    pub stories: Vec<UserStory>,
    pub total_estimated_minutes: u32,
}

pub fn add_story<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
) -> Result<StoriesResponse, ProjectError> {
    let mut prd = load_prd(app, project_id)?;
    let next_index = prd.stories.len() + 1;
    let padded_id = format!("S-{next_index:03}");

    let story = UserStory {
        id: padded_id,
        title: "New story".to_string(),
        description: None,
        acceptance_criteria: Vec::new(),
        scope: ralph_core::prd::ScopeSpec::default(),
        verification: ralph_core::prd::VerificationSpec::default(),
        commit_message: None,
        priority: ralph_core::prd::Priority::Medium,
        estimated_complexity: ralph_core::prd::Complexity::Medium,
        estimated_minutes: 30,
        depends_on: Vec::new(),
        passes: false,
        blocked: false,
        attempts: 0,
        notes: None,
    };

    prd.stories.push(story);
    let minutes = total_estimated_minutes(&prd);
    save_prd(app, project_id, &prd)?;

    Ok(StoriesResponse {
        stories: prd.stories,
        total_estimated_minutes: minutes,
    })
}

pub fn update_story<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
    story_id: &str,
    patch_json: &str,
) -> Result<StoriesResponse, ProjectError> {
    let mut prd = load_prd(app, project_id)?;
    let patch: serde_json::Value = serde_json::from_str(patch_json)?;

    let story = prd
        .stories
        .iter_mut()
        .find(|story| story.id == story_id)
        .ok_or_else(|| ProjectError::NotFound(format!("Story {story_id}")))?;

    if let Some(title) = patch.get("title").and_then(|val| val.as_str()) {
        story.title = title.to_string();
    }
    if let Some(description) = patch.get("description").and_then(|val| val.as_str()) {
        story.description = Some(description.to_string());
    }
    if let Some(priority) = patch.get("priority").and_then(|val| val.as_str()) {
        if let Ok(parsed) = serde_json::from_value::<ralph_core::prd::Priority>(
            serde_json::Value::String(priority.to_string()),
        ) {
            story.priority = parsed;
        }
    }
    if let Some(complexity) = patch
        .get("estimatedComplexity")
        .and_then(|val| val.as_str())
    {
        if let Ok(parsed) = serde_json::from_value::<ralph_core::prd::Complexity>(
            serde_json::Value::String(complexity.to_string()),
        ) {
            story.estimated_complexity = parsed;
        }
    }
    if let Some(minutes) = patch
        .get("estimatedMinutes")
        .and_then(serde_json::Value::as_u64)
    {
        story.estimated_minutes = minutes as u32;
    }

    let minutes = total_estimated_minutes(&prd);
    save_prd(app, project_id, &prd)?;

    Ok(StoriesResponse {
        stories: prd.stories,
        total_estimated_minutes: minutes,
    })
}

pub fn remove_story<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
    story_id: &str,
) -> Result<StoriesResponse, ProjectError> {
    let mut prd = load_prd(app, project_id)?;
    let original_count = prd.stories.len();
    prd.stories.retain(|story| story.id != story_id);

    if prd.stories.len() == original_count {
        return Err(ProjectError::NotFound(format!("Story {story_id}")));
    }

    let minutes = total_estimated_minutes(&prd);
    save_prd(app, project_id, &prd)?;

    Ok(StoriesResponse {
        stories: prd.stories,
        total_estimated_minutes: minutes,
    })
}

pub fn reorder_stories<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
    from_index: usize,
    to_index: usize,
) -> Result<StoriesResponse, ProjectError> {
    let mut prd = load_prd(app, project_id)?;

    if from_index >= prd.stories.len() || to_index >= prd.stories.len() {
        return Err(ProjectError::Path("Index out of bounds".to_string()));
    }

    let moved = prd.stories.remove(from_index);
    prd.stories.insert(to_index, moved);
    let minutes = total_estimated_minutes(&prd);
    save_prd(app, project_id, &prd)?;

    Ok(StoriesResponse {
        stories: prd.stories,
        total_estimated_minutes: minutes,
    })
}

pub fn get_stories<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
) -> Result<StoriesResponse, ProjectError> {
    let prd = load_prd(app, project_id)?;
    let minutes = total_estimated_minutes(&prd);
    Ok(StoriesResponse {
        total_estimated_minutes: minutes,
        stories: prd.stories,
    })
}
