use ralph_core::prd::{Complexity, Prd, Priority};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn load_full_prd_from_fixture() {
    let prd = Prd::load(&fixture("full_prd.json")).expect("full_prd.json should load");
    assert_eq!(prd.project_name, "test-project");
    assert_eq!(prd.stories.len(), 3);
    assert_eq!(prd.feature, "authentication");
}

#[test]
fn load_full_prd_all_d6_scope_fields() {
    let prd = Prd::load(&fixture("full_prd.json")).unwrap();
    let story = &prd.stories[0];
    assert_eq!(story.id, "S-001");
    assert_eq!(story.scope.files_to_modify, vec!["src/auth.rs"]);
    assert_eq!(story.scope.files_to_create, vec!["src/jwt.rs"]);
    assert_eq!(story.scope.files_to_avoid, vec!["migrations/*"]);
    assert_eq!(story.verification.commands, vec!["cargo test"]);
    assert_eq!(
        story.verification.assertions,
        vec!["grep -q 'jwt' src/auth.rs"]
    );
    assert_eq!(
        story.commit_message.as_deref(),
        Some("feat(auth): implement JWT authentication")
    );
    assert_eq!(story.estimated_minutes, 60);
    assert_eq!(story.attempts, 0);
    assert!(!story.passes);
    assert!(!story.blocked);
}

#[test]
fn load_full_prd_priority_and_complexity() {
    let prd = Prd::load(&fixture("full_prd.json")).unwrap();
    assert_eq!(prd.stories[0].priority, Priority::Critical);
    assert_eq!(prd.stories[0].estimated_complexity, Complexity::Medium);
    assert_eq!(prd.stories[1].priority, Priority::High);
    assert_eq!(prd.stories[1].estimated_complexity, Complexity::Small);
    assert_eq!(prd.stories[2].priority, Priority::Medium);
}

#[test]
fn load_full_prd_dependency_references() {
    let prd = Prd::load(&fixture("full_prd.json")).unwrap();
    assert!(prd.stories[0].depends_on.is_empty());
    assert_eq!(prd.stories[1].depends_on, vec!["S-001"]);
    assert_eq!(prd.stories[2].depends_on, vec!["S-001", "S-002"]);
}

#[test]
fn total_estimated_minutes_sums_all_stories() {
    let prd = Prd::load(&fixture("full_prd.json")).unwrap();
    assert_eq!(prd.total_estimated_minutes(), 120);
}

#[test]
fn validate_atomicity_passes_for_valid_fixture() {
    let prd = Prd::load(&fixture("full_prd.json")).unwrap();
    assert!(prd.validate_atomicity().is_ok());
}

#[test]
fn validate_atomicity_rejects_missing_acceptance_criteria() {
    let mut prd = Prd::load(&fixture("full_prd.json")).unwrap();
    prd.stories[0].acceptance_criteria.clear();
    let err = prd.validate_atomicity().unwrap_err();
    assert!(
        err.to_string().contains("acceptance criteria"),
        "error should mention acceptance criteria, got: {err}"
    );
}

#[test]
fn validate_atomicity_rejects_empty_title() {
    let mut prd = Prd::load(&fixture("full_prd.json")).unwrap();
    prd.stories[0].title = String::new();
    let err = prd.validate_atomicity().unwrap_err();
    assert!(
        err.to_string().contains("empty title"),
        "error should mention empty title, got: {err}"
    );
}

#[test]
fn validate_atomicity_rejects_invalid_dependency_reference() {
    let mut prd = Prd::load(&fixture("full_prd.json")).unwrap();
    prd.stories[0].depends_on = vec!["S-999".to_string()];
    let err = prd.validate_atomicity().unwrap_err();
    assert!(
        err.to_string().contains("S-999"),
        "error should name the bad dependency id, got: {err}"
    );
}

#[test]
fn load_minimal_prd_backward_compat() {
    let prd = Prd::load(&fixture("minimal_prd.json")).expect("minimal_prd.json should load");
    assert_eq!(prd.project_name, "minimal-project");
    assert_eq!(prd.stories.len(), 1);
    assert_eq!(prd.stories[0].id, "S-001");
    assert_eq!(prd.stories[0].title, "Basic feature");
}

#[test]
fn minimal_prd_defaults_optional_fields() {
    let prd = Prd::load(&fixture("minimal_prd.json")).unwrap();
    let story = &prd.stories[0];
    assert_eq!(story.priority, Priority::Medium);
    assert_eq!(story.estimated_complexity, Complexity::Medium);
    assert_eq!(story.estimated_minutes, 0);
    assert!(story.depends_on.is_empty());
    assert_eq!(story.attempts, 0);
    assert!(story.acceptance_criteria.is_empty());
    assert!(story.scope.files_to_modify.is_empty());
    assert!(story.scope.files_to_create.is_empty());
    assert!(story.verification.commands.is_empty());
    assert!(story.commit_message.is_none());
    assert!(story.notes.is_none());
    assert!(!story.passes);
    assert!(!story.blocked);
}

#[test]
fn load_nonexistent_file_returns_error() {
    let result = Prd::load(&fixture("does_not_exist.json"));
    assert!(result.is_err());
}
