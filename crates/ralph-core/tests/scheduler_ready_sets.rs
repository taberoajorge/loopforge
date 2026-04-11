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

#[test]
fn ready_set_excludes_blocked_and_unsatisfied_dependencies() {
    let mut completed = story("S-001");
    completed.passes = true;

    let ready = story("S-002");

    let mut blocked = story("S-003");
    blocked.blocked = true;

    let mut waiting = story("S-004");
    waiting.depends_on = vec!["S-003".to_string()];

    let prd = prd(vec![completed, ready, blocked, waiting]);
    let ready_ids: Vec<&str> = scheduler::ready_stories(&prd)
        .into_iter()
        .map(|story| story.id.as_str())
        .collect();

    assert_eq!(ready_ids, vec!["S-002"]);
}

#[test]
fn ready_set_returns_all_satisfied_stories_in_story_order() {
    let mut foundation = story("S-001");
    foundation.passes = true;

    let mut first_ready = story("S-002");
    first_ready.depends_on = vec!["S-001".to_string()];

    let second_ready = story("S-003");

    let mut waiting = story("S-004");
    waiting.depends_on = vec!["S-002".to_string()];

    let prd = prd(vec![foundation, first_ready, second_ready, waiting]);
    let ready_ids: Vec<&str> = scheduler::ready_stories(&prd)
        .into_iter()
        .map(|story| story.id.as_str())
        .collect();

    assert_eq!(ready_ids, vec!["S-002", "S-003"]);
}

#[test]
fn ready_set_unlocks_story_only_when_all_dependencies_pass() {
    let mut first = story("S-001");
    first.passes = true;

    let second = story("S-002");

    let mut gated = story("S-003");
    gated.depends_on = vec!["S-001".to_string(), "S-002".to_string()];

    let prd = prd(vec![first, second, gated]);
    let ready_ids: Vec<&str> = scheduler::ready_stories(&prd)
        .into_iter()
        .map(|story| story.id.as_str())
        .collect();

    assert_eq!(ready_ids, vec!["S-002"]);
}
