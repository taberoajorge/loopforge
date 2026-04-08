use super::json_parse::{extract_json_candidates, parse_json_from_candidates, strip_json_fences};
use super::sanitize::sanitize_codex_plan_content;
use super::types::{AtomizedStoryDraft, ChunkSection};

#[test]
fn strip_json_fences_removes_backtick_json_fence() {
    let input = "```json\n{\"key\": \"value\"}\n```";
    assert_eq!(strip_json_fences(input), "{\"key\": \"value\"}");
}

#[test]
fn strip_json_fences_removes_plain_backtick_fence() {
    let input = "```\n[1, 2, 3]\n```";
    assert_eq!(strip_json_fences(input), "[1, 2, 3]");
}

#[test]
fn strip_json_fences_passes_through_unfenced_json() {
    let input = "{\"key\": \"value\"}";
    assert_eq!(strip_json_fences(input), "{\"key\": \"value\"}");
}

#[test]
fn strip_json_fences_trims_surrounding_whitespace() {
    let input = "  ```json\n{\"key\": \"value\"}\n```  ";
    assert_eq!(strip_json_fences(input), "{\"key\": \"value\"}");
}

#[test]
fn sanitize_codex_plan_content_removes_stdin_warning_lines() {
    let raw = "Warning: no stdin data received in 3s, proceeding without it.\nIf piping from a slow command, redirect stdin explicitly: < /dev/null to skip, or wait longer.\n## Plan\n- Step 1";
    let cleaned = sanitize_codex_plan_content(raw);
    assert!(!cleaned.contains("no stdin data received"));
    assert!(cleaned.contains("## Plan"));
}

#[test]
fn extract_json_candidates_ignores_preamble_before_array() {
    let input = "I have enough context to proceed.\n\n[{\"title\":\"Story\"}]";
    let candidates = extract_json_candidates(input, '[');
    assert_eq!(candidates[0], "[{\"title\":\"Story\"}]");
}

#[test]
fn extract_json_candidates_ignores_preamble_before_object() {
    let input = "Sure, here is the PRD.\n\n{\"projectName\":\"demo\",\"stories\":[]}";
    let candidates = extract_json_candidates(input, '{');
    assert_eq!(candidates[0], "{\"projectName\":\"demo\",\"stories\":[]}");
}

#[test]
fn parse_json_from_candidates_prefers_array_over_code_snippet_map() {
    let input = "The fix changes invoke(\"start_loop\", { projectId }) to use full args.\n\n[{\"title\":\"Story\",\"acceptanceCriteria\":[\"works\"]}]";
    let parsed = parse_json_from_candidates::<Vec<serde_json::Value>>(input, '[').unwrap();
    assert_eq!(parsed.len(), 1);
}

#[test]
fn strip_json_fences_fixture_produces_valid_json() {
    let fenced = include_str!("../../templates/test_fixtures/fenced_chunk_response.txt");
    let stripped = strip_json_fences(fenced);
    let parsed = serde_json::from_str::<serde_json::Value>(stripped);
    assert!(parsed.is_ok(), "stripped fixture should be valid JSON");
}

#[test]
fn chunk_sections_fixture_parses_as_array() {
    let json = include_str!("../../templates/test_fixtures/chunk_sections.json");
    let sections: Vec<ChunkSection> = serde_json::from_str(json).unwrap();
    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].title, "Database schema");
}

#[test]
fn stories_fixture_parses_as_atomized_story_draft_array() {
    let json = include_str!("../../templates/test_fixtures/stories_response.json");
    let stories: Vec<AtomizedStoryDraft> = serde_json::from_str(json).unwrap();
    assert_eq!(stories.len(), 1);
    assert!(!stories[0].acceptance_criteria.is_empty());
}
