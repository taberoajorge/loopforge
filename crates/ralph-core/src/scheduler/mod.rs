mod coordinator;
pub mod worktree_runner;

use crate::loop_engine::scheduler::{
    schedule as schedule_completions, MergeAction, WorktreeCompletion,
};
use crate::prd::{Prd, UserStory};
use std::collections::HashSet;

pub use coordinator::{ArtifactCoordinator, CoordinatorError, SharedArtifactUpdate};

pub fn ready_stories(prd: &Prd) -> Vec<&UserStory> {
    let passed_ids: HashSet<&str> = prd
        .stories
        .iter()
        .filter(|story| story.passes)
        .map(|story| story.id.as_str())
        .collect();

    prd.stories
        .iter()
        .filter(|story| is_ready(story, &passed_ids))
        .collect()
}

pub fn ready_story_batch(prd: &Prd) -> Vec<UserStory> {
    ready_stories(prd).into_iter().cloned().collect()
}

pub fn shared_updates_for(completions: Vec<WorktreeCompletion>) -> Vec<SharedArtifactUpdate> {
    schedule_completions(completions)
        .into_iter()
        .filter_map(|action| match action {
            MergeAction::UpdateStoryStatus {
                story_id,
                passed,
                blocked: _,
            } if passed => Some(SharedArtifactUpdate::MarkStoryPassed { story_id }),
            MergeAction::UpdateStoryStatus {
                story_id,
                passed: _,
                blocked,
            } if blocked => Some(SharedArtifactUpdate::MarkStoryBlocked { story_id }),
            MergeAction::AppendGuardrail { story_id, content } => {
                Some(SharedArtifactUpdate::AppendGuardrailContent { story_id, content })
            }
            _ => None,
        })
        .collect()
}

fn is_ready(story: &UserStory, passed_ids: &HashSet<&str>) -> bool {
    !story.passes
        && !story.blocked
        && story
            .depends_on
            .iter()
            .all(|dependency| passed_ids.contains(dependency.as_str()))
}
