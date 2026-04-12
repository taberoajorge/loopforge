use crate::plan_engine::payloads::{PlanActivityBatchPayload, PlanActivityPayload};
use crate::activity::PlanEventKind;
use std::collections::HashSet;
use std::sync::OnceLock;

static EXACT_NOISE: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn exact_noise_set() -> &'static HashSet<&'static str> {
    EXACT_NOISE.get_or_init(|| {
        let mut set = HashSet::new();
        for entry in &[
            "exec", "codex", "--------", "---", "WAIT", "reason", "effort", "found",
        ] {
            set.insert(*entry);
        }
        set
    })
}

const PREFIX_NOISE: &[&str] = &[
    "OpenAI Codex",
    "workdir:",
    "model:",
    "provider:",
    "approval:",
    "sandbox:",
    "reasoning effort:",
    "reasoning summaries:",
    "session id:",
    "user You are a senior software architect.",
    "error: unexpected argument",
    "tip: to pass",
    "Usage: codex",
    "For more information",
    "Usage:",
    "tip:",
    "Reading additional input from stdin",
    "Warning: no stdin data received",
    "If piping from a slow command",
];

fn is_noise_line(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return true;
    }
    if exact_noise_set().contains(trimmed) {
        return true;
    }
    PREFIX_NOISE.iter().any(|prefix| trimmed.starts_with(prefix))
}

fn normalize_line(content: &str) -> String {
    let trimmed = content.trim();
    if let Some(rest) = trimmed.strip_prefix("codex ") {
        return rest.trim().to_string();
    }
    if let Some(rest) = trimmed.strip_prefix("user ") {
        return rest.trim().to_string();
    }
    trimmed.to_string()
}

pub fn filter_plan_batch(batch: PlanActivityBatchPayload) -> PlanActivityBatchPayload {
    let mut last_signature = String::new();
    let filtered_events: Vec<PlanActivityPayload> = batch
        .events
        .into_iter()
        .filter_map(|mut event| {
            let normalized = normalize_line(&event.content);
            if is_noise_line(&normalized) {
                return None;
            }
            let signature = format!("{:?}:{normalized}", event.kind);
            if signature == last_signature {
                return None;
            }
            last_signature = signature;
            event.content = normalized;
            Some(event)
        })
        .collect();

    let plan_content_delta: String = filtered_events
        .iter()
        .filter(|evt| evt.kind == PlanEventKind::PlanContent)
        .map(|evt| evt.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    PlanActivityBatchPayload {
        project_id: batch.project_id,
        events: filtered_events,
        plan_content_delta,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(content: &str) -> PlanActivityPayload {
        PlanActivityPayload {
            project_id: "test".to_string(),
            kind: PlanEventKind::PlanContent,
            content: content.to_string(),
            timestamp: "12:00:00".to_string(),
        }
    }

    #[test]
    fn filters_noise_lines() {
        let batch = PlanActivityBatchPayload {
            project_id: "test".to_string(),
            events: vec![
                make_event("exec"),
                make_event("Real plan content"),
                make_event("OpenAI Codex v1.0"),
            ],
            plan_content_delta: String::new(),
        };
        let filtered = filter_plan_batch(batch);
        assert_eq!(filtered.events.len(), 1);
        assert_eq!(filtered.events[0].content, "Real plan content");
    }

    #[test]
    fn normalizes_codex_prefix() {
        let batch = PlanActivityBatchPayload {
            project_id: "test".to_string(),
            events: vec![make_event("codex Some plan output")],
            plan_content_delta: String::new(),
        };
        let filtered = filter_plan_batch(batch);
        assert_eq!(filtered.events[0].content, "Some plan output");
    }

    #[test]
    fn deduplicates_consecutive() {
        let batch = PlanActivityBatchPayload {
            project_id: "test".to_string(),
            events: vec![
                make_event("Same line"),
                make_event("Same line"),
                make_event("Different line"),
            ],
            plan_content_delta: String::new(),
        };
        let filtered = filter_plan_batch(batch);
        assert_eq!(filtered.events.len(), 2);
    }
}
