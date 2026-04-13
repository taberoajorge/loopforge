use super::agent_args::{build_agent_args, build_null_stdin_shell_command, shell_quote};
use super::chunking::{chunk_large_plan, CHUNK_SIZE_CHARS, MAX_PLAN_CHARS};
use ralph_core::prd::{Prd, ScopeSpec, UserStory, VerificationSpec};
use std::path::Path;

#[test]
fn chunk_large_plan_returns_single_chunk_for_small_plan() {
    let plan = "Short plan text that fits within the limit";
    let chunks = chunk_large_plan(plan);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], plan);
}

#[test]
fn chunk_large_plan_does_not_split_at_exact_boundary() {
    let plan = "x".repeat(MAX_PLAN_CHARS);
    let chunks = chunk_large_plan(&plan);
    assert_eq!(chunks.len(), 1);
}

#[test]
fn chunk_large_plan_splits_plan_exceeding_max_chars() {
    let plan = "x".repeat(MAX_PLAN_CHARS + 1);
    let chunks = chunk_large_plan(&plan);
    assert!(chunks.len() > 1);
}

#[test]
fn chunk_large_plan_preserves_all_content() {
    let plan = "x".repeat(MAX_PLAN_CHARS + CHUNK_SIZE_CHARS + 1);
    let chunks = chunk_large_plan(&plan);
    let rejoined: String = chunks.concat();
    assert_eq!(rejoined.len(), plan.len());
}

#[test]
fn build_agent_args_adds_codex_skip_repo_flag() {
    let args = build_agent_args(
        "codex",
        "Generate output",
        Path::new("/tmp/demo"),
        None,
        None,
    );
    assert!(args.iter().any(|value| value == "--skip-git-repo-check"));
}

#[test]
fn build_agent_args_adds_codex_bypass_flag() {
    let args = build_agent_args(
        "codex",
        "Generate output",
        Path::new("/tmp/demo"),
        None,
        None,
    );
    assert!(args
        .iter()
        .any(|value| value == "--dangerously-bypass-approvals-and-sandbox"));
}

#[test]
fn build_agent_args_includes_codex_working_directory() {
    let args = build_agent_args(
        "codex",
        "Generate output",
        Path::new("/tmp/demo"),
        None,
        None,
    );
    let has_directory_flag = args.windows(2).any(|pair| {
        pair.first().map(|value| value.as_str()) == Some("-C")
            && pair.get(1).map(|value| value.as_str()) == Some("/tmp/demo")
    });
    assert!(has_directory_flag);
}

#[test]
fn shell_quote_escapes_single_quotes() {
    let quoted = shell_quote("it's fine");
    assert_eq!(quoted, "'it'\"'\"'s fine'");
}

#[test]
fn build_null_stdin_shell_command_appends_redirection() {
    let args = vec!["exec".to_string(), "prompt body".to_string()];
    let command = build_null_stdin_shell_command("codex", &args);
    assert!(command.ends_with("< /dev/null"));
}

#[test]
fn build_agent_args_omits_removed_opencode_auto_share_flag() {
    let args = build_agent_args(
        "opencode",
        "Generate output",
        Path::new("/tmp/demo"),
        None,
        None,
    );
    assert!(!args.iter().any(|value| value == "--no-auto-share"));
}

#[test]
fn build_agent_args_sets_codex_model_and_effort_when_provided() {
    let args = build_agent_args(
        "codex",
        "Generate output",
        Path::new("/tmp/demo"),
        Some("gpt-5.4"),
        Some("high"),
    );
    let has_model = args.windows(2).any(|pair| {
        pair.first().map(|v| v.as_str()) == Some("--model")
            && pair.get(1).map(|v| v.as_str()) == Some("gpt-5.4")
    });
    let has_effort = args
        .windows(2)
        .any(|pair| pair.first().map(|v| v.as_str()) == Some("--reasoning-effort"));
    assert!(has_model);
    assert!(!has_effort, "codex should not receive --reasoning-effort");
}

#[test]
fn validate_atomicity_gate_rejects_story_without_acceptance_criteria() {
    let malformed = Prd {
        project_name: "test".to_string(),
        feature: String::new(),
        working_directory: String::new(),
        branch_name: None,
        generated_at: None,
        stories: vec![UserStory {
            id: "S-001".to_string(),
            title: "Do something".to_string(),
            description: None,
            acceptance_criteria: vec![],
            scope: ScopeSpec::default(),
            verification: VerificationSpec::default(),
            commit_message: None,
            priority: Default::default(),
            estimated_complexity: Default::default(),
            estimated_minutes: 0,
            depends_on: vec![],
            passes: false,
            blocked: false,
            attempts: 0,
            notes: None,
        }],
    };
    assert!(malformed.validate_atomicity().is_err());
}

#[test]
fn validate_atomicity_gate_rejects_story_with_empty_title() {
    let malformed = Prd {
        project_name: "test".to_string(),
        feature: String::new(),
        working_directory: String::new(),
        branch_name: None,
        generated_at: None,
        stories: vec![UserStory {
            id: "S-001".to_string(),
            title: String::new(),
            description: None,
            acceptance_criteria: vec!["Some criterion".to_string()],
            scope: ScopeSpec::default(),
            verification: VerificationSpec::default(),
            commit_message: None,
            priority: Default::default(),
            estimated_complexity: Default::default(),
            estimated_minutes: 0,
            depends_on: vec![],
            passes: false,
            blocked: false,
            attempts: 0,
            notes: None,
        }],
    };
    assert!(malformed.validate_atomicity().is_err());
}
