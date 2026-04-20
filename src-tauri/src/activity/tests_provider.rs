use super::{ActivityClassifier, PlanEventKind};

#[test]
fn codex_provider_patterns_classify_json_search() {
    let mut classifier = ActivityClassifier::new("codex");
    let event = classifier.classify(r#"{"type": "search", "query": "rust async"}"#);
    assert_eq!(event.kind, PlanEventKind::Search);
}

#[test]
fn gemini_provider_patterns_classify_google_search() {
    let mut classifier = ActivityClassifier::new("gemini");
    let event = classifier.classify("google_search(\"rust tokio tutorial\")");
    assert_eq!(event.kind, PlanEventKind::Search);
}

#[test]
fn codex_classifies_function_type_as_mcp_call() {
    let mut classifier = ActivityClassifier::new("codex");
    let event = classifier.classify(r#"{"type": "function", "name": "shell"}"#);
    assert_eq!(event.kind, PlanEventKind::McpCall);
}

#[test]
fn codex_classifies_reasoning_type_as_thinking() {
    let mut classifier = ActivityClassifier::new("codex");
    let event =
        classifier.classify(r#"{"type": "reasoning", "summary": "Analyzing the codebase"}"#);
    assert_eq!(event.kind, PlanEventKind::Thinking);
}

#[test]
fn codex_classifies_search_type_as_search() {
    let mut classifier = ActivityClassifier::new("codex");
    let event = classifier.classify(r#"{"type": "search", "query": "rust async runtime"}"#);
    assert_eq!(event.kind, PlanEventKind::Search);
}

#[test]
fn codex_exec_line_starts_tool_output() {
    let mut classifier = ActivityClassifier::new("codex");
    let event = classifier.classify(r#"exec /bin/zsh -lc "rg -n foo" in /path"#);
    assert_eq!(event.kind, PlanEventKind::McpCall);
    assert!(classifier.is_in_tool_output());
}

#[test]
fn codex_exec_cmd_line_starts_tool_output() {
    let mut classifier = ActivityClassifier::new("codex");
    let event = classifier.classify(r#"exec cmd.exe /C "rg -n foo" in C:\repo"#);
    assert_eq!(event.kind, PlanEventKind::McpCall);
    assert!(classifier.is_in_tool_output());
}

#[test]
fn codex_tool_output_absorbed_until_narration() {
    let mut classifier = ActivityClassifier::new("codex");
    classifier.classify(r#"exec /bin/zsh -lc "rg foo" in /path"#);
    let status = classifier.classify("succeeded in 0ms:");
    assert_eq!(status.kind, PlanEventKind::McpCall);
    let output = classifier.classify("src/main.rs:10: fn foo() {}");
    assert_eq!(output.kind, PlanEventKind::McpCall);
    let narration = classifier.classify("codex Found the function definition.");
    assert_eq!(narration.kind, PlanEventKind::Thinking);
    assert!(!classifier.is_in_tool_output());
}

#[test]
fn codex_plan_content_only_after_tool_output_ends() {
    let mut classifier = ActivityClassifier::new("codex");
    classifier.classify(r#"exec /bin/zsh -lc "cat file" in /path"#);
    classifier.classify("succeeded in 0ms:");
    classifier.classify("line 1 of file");
    classifier.classify("line 2 of file");
    classifier.classify("codex I've reviewed the file.");
    for idx in 0..30 {
        classifier.classify(&format!("codex filler line {idx}"));
    }
    let plan_line = classifier.classify("## Implementation Plan");
    assert_eq!(plan_line.kind, PlanEventKind::PlanContent);
    assert!(classifier
        .accumulated_plan()
        .contains("## Implementation Plan"));
}

#[test]
fn codex_banner_classified_as_thinking() {
    let mut classifier = ActivityClassifier::new("codex");
    let banner = classifier.classify("OpenAI Codex v0.114.0 (research preview)");
    assert_eq!(banner.kind, PlanEventKind::Thinking);
    let meta = classifier.classify("workdir: /path model: gpt-5 provider: openai");
    assert_eq!(meta.kind, PlanEventKind::Thinking);
}

#[test]
fn codex_user_echo_classified_as_thinking() {
    let mut classifier = ActivityClassifier::new("codex");
    let event = classifier.classify("user Create a plan for authentication");
    assert_eq!(event.kind, PlanEventKind::Thinking);
}

#[test]
fn codex_plan_update_classified_as_thinking() {
    let mut classifier = ActivityClassifier::new("codex");
    let event = classifier.classify("Plan update → Inspect repo structure");
    assert_eq!(event.kind, PlanEventKind::Thinking);
}

#[test]
fn codex_no_tool_output_leak_into_plan_buffer() {
    let mut classifier = ActivityClassifier::new("codex");
    classifier.classify("codex Let me research the codebase.");
    classifier.classify(r#"exec /bin/zsh -lc "ls src/" in /path"#);
    classifier.classify("succeeded in 0ms:");
    classifier.classify("main.rs");
    classifier.classify("lib.rs");
    classifier.classify("codex Found the source files.");
    for idx in 0..30 {
        classifier.classify(&format!("codex padding {idx}"));
    }
    classifier.classify("## My Plan");
    classifier.classify("Step 1: Do the thing");
    let plan = classifier.accumulated_plan();
    assert!(plan.contains("## My Plan"));
    assert!(plan.contains("Step 1: Do the thing"));
    assert!(!plan.contains("main.rs"));
    assert!(!plan.contains("lib.rs"));
    assert!(!plan.contains("Let me research"));
}

#[test]
fn gemini_classifies_function_call_block_as_mcp_call() {
    let mut classifier = ActivityClassifier::new("gemini");
    let event = classifier.classify("function_call { name: read_file, args: {} }");
    assert_eq!(event.kind, PlanEventKind::McpCall);
}

#[test]
fn gemini_classifies_gemini_thoughts_tag_as_thinking() {
    let mut classifier = ActivityClassifier::new("gemini");
    let event = classifier.classify("<gemini_thoughts>Let me consider this</gemini_thoughts>");
    assert_eq!(event.kind, PlanEventKind::Thinking);
}

#[test]
fn gemini_classifies_google_search_call_as_search() {
    let mut classifier = ActivityClassifier::new("gemini");
    let event = classifier.classify(r#"google_search("best rust web frameworks")"#);
    assert_eq!(event.kind, PlanEventKind::Search);
}
