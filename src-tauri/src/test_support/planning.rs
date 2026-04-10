use super::runtime::{FixtureSet, TestRuntime};
use crate::activity::PlanEventKind;
use crate::plan_engine::payloads::{PlanActivityBatchPayload, PlanActivityPayload};

pub(crate) struct FixturePlanBatch {
    pub delay_ms: u64,
    pub payload: PlanActivityBatchPayload,
}

pub(crate) enum FixturePlanTerminal {
    Complete { plan_markdown: String },
    Error { detail: String },
}

pub(crate) struct FixturePlanRun {
    pub batches: Vec<FixturePlanBatch>,
    pub terminal: FixturePlanTerminal,
}

pub(crate) fn fixture_plan_run(runtime: &TestRuntime, project_id: &str) -> FixturePlanRun {
    match runtime.fixture_set() {
        FixtureSet::HappyPath => happy_path(project_id),
        FixtureSet::PlanError => plan_error(project_id),
        _ => happy_path(project_id),
    }
}

fn happy_path(project_id: &str) -> FixturePlanRun {
    let plan_markdown = [
        "# Fixture Planning Summary",
        "",
        "1. Confirm deterministic test runtime seams.",
        "2. Emit stable planning activity batches.",
        "3. Preserve write and stop session behavior.",
    ]
    .join("\n");
    FixturePlanRun {
        batches: vec![
            FixturePlanBatch {
                delay_ms: 20,
                payload: batch(
                    project_id,
                    vec![
                        event(
                            project_id,
                            PlanEventKind::Thinking,
                            "Inspecting planning fixture inputs",
                            "2026-04-09T10:00:00.000Z",
                        ),
                        event(
                            project_id,
                            PlanEventKind::Search,
                            "Searching existing planning hooks",
                            "2026-04-09T10:00:01.000Z",
                        ),
                    ],
                    "",
                ),
            },
            FixturePlanBatch {
                delay_ms: 10,
                payload: batch(
                    project_id,
                    vec![event(
                        project_id,
                        PlanEventKind::DocsLookup,
                        "Reviewing LoopForge planning contracts",
                        "2026-04-09T10:00:02.000Z",
                    )],
                    "",
                ),
            },
            FixturePlanBatch {
                delay_ms: 10,
                payload: batch(
                    project_id,
                    vec![
                        event(
                            project_id,
                            PlanEventKind::PlanContent,
                            "# Fixture Planning Summary",
                            "2026-04-09T10:00:03.000Z",
                        ),
                        event(
                            project_id,
                            PlanEventKind::PlanContent,
                            "1. Confirm deterministic test runtime seams.",
                            "2026-04-09T10:00:04.000Z",
                        ),
                        event(
                            project_id,
                            PlanEventKind::PlanContent,
                            "2. Emit stable planning activity batches.",
                            "2026-04-09T10:00:05.000Z",
                        ),
                        event(
                            project_id,
                            PlanEventKind::PlanContent,
                            "3. Preserve write and stop session behavior.",
                            "2026-04-09T10:00:06.000Z",
                        ),
                    ],
                    &plan_markdown,
                ),
            },
        ],
        terminal: FixturePlanTerminal::Complete { plan_markdown },
    }
}

fn plan_error(project_id: &str) -> FixturePlanRun {
    FixturePlanRun {
        batches: vec![FixturePlanBatch {
            delay_ms: 20,
            payload: batch(
                project_id,
                vec![event(
                    project_id,
                    PlanEventKind::Thinking,
                    "Loading deterministic plan failure fixture",
                    "2026-04-09T10:05:00.000Z",
                )],
                "",
            ),
        }],
        terminal: FixturePlanTerminal::Error {
            detail: plan_error_detail().to_string(),
        },
    }
}

pub(crate) fn plan_error_detail() -> &'static str {
    "fixture plan failed"
}

fn batch(
    project_id: &str,
    events: Vec<PlanActivityPayload>,
    plan_content_delta: &str,
) -> PlanActivityBatchPayload {
    PlanActivityBatchPayload {
        project_id: project_id.to_string(),
        events,
        plan_content_delta: plan_content_delta.to_string(),
    }
}

fn event(
    project_id: &str,
    kind: PlanEventKind,
    content: &str,
    timestamp: &str,
) -> PlanActivityPayload {
    PlanActivityPayload {
        project_id: project_id.to_string(),
        kind,
        content: content.to_string(),
        timestamp: timestamp.to_string(),
    }
}
