use super::codex::{has_substantive_output, is_stdin_wait_line};
use super::command_args::{agent_cli_args, resolve_agent_binary_path};
use std::path::Path;

#[test]
fn resolve_agent_binary_path_finds_available_binary() {
    let path = resolve_agent_binary_path("zsh");
    assert!(path.is_some());
}

#[test]
fn stdin_wait_line_is_detected() {
    assert!(is_stdin_wait_line("Reading additional input from stdin..."));
}

#[test]
fn stdin_wait_line_is_not_substantive_output() {
    let output = vec!["Reading additional input from stdin...".to_string()];
    assert!(!has_substantive_output(&output));
}

#[test]
fn codex_args_include_model_and_effort_flags() {
    let args = agent_cli_args(
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
