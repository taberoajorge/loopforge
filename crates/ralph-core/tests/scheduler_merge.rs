use ralph_core::prd::{
    Complexity, Prd, Priority, ScopeSpec, UserStory, VerificationSpec,
};
use ralph_core::scheduler;

fn story(id: &str) -> UserStory {
    UserStory {
        id: id.to_string(),
        title: format!("Story {id}"),
        description: None,
        acceptance_criteria: vec!["works".to_string()],
        scope: ScopeSpec::default(),
        verification: VerificationSpec::default(),
        commit_message: None,
        priority: Priority::Medium,
        estimated_complexity: Complexity::Medium,
        estimated_minutes: 0,
        depends_on: Vec::new(),
        passes: false,
        blocked: false,
        attempts: 0,
        notes: None,
    }
}

fn prd(stories: Vec<UserStory>) -> Prd {
    Prd {
        project_name: "loopforge".to_string(),
        feature: String::new(),
        working_directory: String::new(),
        branch_name: None,
        stories,
        generated_at: None,
    }
}

fn scheduled_ids(prd: &Prd) -> Vec<String> {
    scheduler::ready_stories(prd)
        .into_iter()
        .map(|story| story.id.clone())
        .collect()
}

fn merge_schedule(existing: &[String], incoming: &[String]) -> Vec<String> {
    let mut merged = existing.to_vec();

    for story_id in incoming {
        if !merged.contains(story_id) {
            merged.push(story_id.clone());
        }
    }

    merged
}

#[test]
fn merge_schedule_keeps_existing_queue_when_new_ready_work_arrives() {
    let mut foundation = story("S-001");
    foundation.passes = true;

    let mut queued = story("S-002");
    queued.depends_on = vec!["S-001".to_string()];

    let initial_schedule = scheduled_ids(&prd(vec![foundation.clone(), queued.clone()]));

    let independent = story("S-003");
    let mut unlocked = story("S-004");
    unlocked.depends_on = vec!["S-002".to_string()];

    let incoming_schedule = scheduled_ids(&prd(vec![foundation, queued, independent, unlocked]));
    let merged = merge_schedule(&initial_schedule, &incoming_schedule);

    assert_eq!(initial_schedule, vec!["S-002"]);
    assert_eq!(incoming_schedule, vec!["S-002", "S-003"]);
    assert_eq!(merged, vec!["S-002", "S-003"]);
}

#[test]
fn merge_schedule_deduplicates_overlapping_ready_sets_in_story_order() {
    let mut foundation = story("S-001");
    foundation.passes = true;

    let mut queued_first = story("S-002");
    queued_first.depends_on = vec!["S-001".to_string()];

    let queued_second = story("S-003");
    let initial_schedule = scheduled_ids(&prd(vec![
        foundation.clone(),
        queued_first.clone(),
        queued_second.clone(),
    ]));

    queued_first.passes = true;

    let mut overlapping = story("S-004");
    overlapping.depends_on = vec!["S-001".to_string()];

    let mut newly_unlocked = story("S-005");
    newly_unlocked.depends_on = vec!["S-002".to_string()];

    let incoming_schedule = scheduled_ids(&prd(vec![
        foundation,
        queued_first,
        queued_second,
        overlapping,
        newly_unlocked,
    ]));
    let merged = merge_schedule(&initial_schedule, &incoming_schedule);

    assert_eq!(initial_schedule, vec!["S-002", "S-003"]);
    assert_eq!(incoming_schedule, vec!["S-003", "S-004", "S-005"]);
    assert_eq!(merged, vec!["S-002", "S-003", "S-004", "S-005"]);
}
