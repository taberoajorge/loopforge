pub mod worktree_runner;

use crate::prd::{Prd, UserStory};
use std::collections::HashSet;

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

fn is_ready(story: &UserStory, passed_ids: &HashSet<&str>) -> bool {
    !story.passes
        && !story.blocked
        && story
            .depends_on
            .iter()
            .all(|dependency| passed_ids.contains(dependency.as_str()))
}
