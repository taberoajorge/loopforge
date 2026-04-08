use super::{ActivityClassifier, PlanEventKind};

#[test]
fn classifies_search_line() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("Searching for relevant documentation...");
    assert_eq!(event.kind, PlanEventKind::Search);
}

#[test]
fn classifies_mcp_call_line() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("calling tool: read_file");
    assert_eq!(event.kind, PlanEventKind::McpCall);
}

#[test]
fn classifies_thinking_line() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("<thinking>");
    assert_eq!(event.kind, PlanEventKind::Thinking);
}

#[test]
fn classifies_stdin_wait_warning_as_thinking() {
    let mut classifier = ActivityClassifier::new("claude");
    let event =
        classifier.classify("Warning: no stdin data received in 3s, proceeding without it.");
    assert_eq!(event.kind, PlanEventKind::Thinking);
}

#[test]
fn classifies_error_line() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("error: command not found");
    assert_eq!(event.kind, PlanEventKind::Error);
}

#[test]
fn classifies_plain_line_as_plan_content() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("## Feature Plan\n\nWe will implement the following:");
    assert_eq!(event.kind, PlanEventKind::PlanContent);
}

#[test]
fn plan_content_accumulates_in_buffer() {
    let mut classifier = ActivityClassifier::new("claude");
    classifier.classify("## Introduction");
    classifier.classify("Some content here");
    classifier.classify("Searching for more...");
    let plan = classifier.accumulated_plan();
    assert!(plan.contains("## Introduction"));
    assert!(plan.contains("Some content here"));
    assert!(!plan.contains("Searching"));
}

#[test]
fn drain_buffer_clears_accumulated_content() {
    let mut classifier = ActivityClassifier::new("claude");
    classifier.classify("Line one");
    classifier.classify("Line two");
    let drained = classifier.drain_buffer();
    assert_eq!(drained.len(), 2);
    assert!(classifier.accumulated_plan().is_empty());
}

#[test]
fn classifies_claude_tool_call_xml_tag_as_mcp_call() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("<tool_call>write_file(path='src/main.rs')</tool_call>");
    assert_eq!(event.kind, PlanEventKind::McpCall);
}

#[test]
fn classifies_tool_colon_pattern_as_mcp_call() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("tool: read_file");
    assert_eq!(event.kind, PlanEventKind::McpCall);
}

#[test]
fn classifies_fetching_as_search() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("Fetching results from documentation...");
    assert_eq!(event.kind, PlanEventKind::Search);
}

#[test]
fn classifies_web_search_phrase_as_search() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("web search: rust async tokio tutorial");
    assert_eq!(event.kind, PlanEventKind::Search);
}

#[test]
fn classifies_markdown_heading_as_plan_content() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("## Implementation Strategy");
    assert_eq!(event.kind, PlanEventKind::PlanContent);
}

#[test]
fn accumulated_plan_collects_only_plan_content_lines() {
    let mut classifier = ActivityClassifier::new("claude");
    classifier.classify("## Section One");
    classifier.classify("Searching for documentation...");
    classifier.classify("Plan implementation details here");
    classifier.classify("error: build failed");
    classifier.classify("More implementation notes");
    let plan = classifier.accumulated_plan();
    assert!(plan.contains("## Section One"));
    assert!(plan.contains("Plan implementation details here"));
    assert!(plan.contains("More implementation notes"));
    assert!(!plan.contains("Searching"));
    assert!(!plan.contains("error: build failed"));
}

#[test]
fn classifies_exception_keyword_as_error() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("Exception: java.lang.NullPointerException");
    assert_eq!(event.kind, PlanEventKind::Error);
}

#[test]
fn classifies_fatal_keyword_as_error() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("fatal: repository not found");
    assert_eq!(event.kind, PlanEventKind::Error);
}

#[test]
fn classifies_failed_keyword_as_error() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("Build failed with exit code 1");
    assert_eq!(event.kind, PlanEventKind::Error);
}

#[test]
fn classifies_stderr_error_prefix_as_error() {
    let mut classifier = ActivityClassifier::new("claude");
    let event = classifier.classify("error: cannot find value `foo` in this scope");
    assert_eq!(event.kind, PlanEventKind::Error);
}
