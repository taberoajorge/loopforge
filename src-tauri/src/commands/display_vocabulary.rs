use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatusMeta {
    pub badge_status: String,
    pub card_label: String,
    pub sidebar_label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanKindMeta {
    pub label: String,
    pub variant: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayVocabulary {
    pub project_status_meta: HashMap<String, ProjectStatusMeta>,
    pub status_labels: HashMap<String, String>,
    pub status_variants: HashMap<String, String>,
    pub notification_type_labels: HashMap<String, String>,
    pub notification_ring_variants: HashMap<String, String>,
    pub story_status_variants: HashMap<String, String>,
    pub activity_result_variants: HashMap<String, String>,
    pub monitor_status_badges: HashMap<String, String>,
    pub plan_kind_meta: HashMap<String, PlanKindMeta>,
    pub atomize_activity_kind_meta: HashMap<String, PlanKindMeta>,
    pub story_priority_variants: HashMap<String, String>,
    pub wizard_step_labels: HashMap<String, String>,
    pub stage_status_badges: HashMap<String, String>,
    pub stage_status_labels: HashMap<String, String>,
    pub step_indicator_variants: HashMap<String, String>,
    pub step_indicator_emphasis: HashMap<String, String>,
    pub agent_names: Vec<String>,
    pub inactive_statuses: Vec<String>,
    pub stall_threshold_secs: u32,
    pub max_visible_activity_events: usize,
    pub max_output_lines: usize,
}

fn string_map(entries: &[(&str, &str)]) -> HashMap<String, String> {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[tauri::command]
pub async fn get_display_vocabulary() -> DisplayVocabulary {
    let project_status_meta = HashMap::from([
        (
            "active".to_string(),
            ProjectStatusMeta {
                badge_status: "running".to_string(),
                card_label: "Running".to_string(),
                sidebar_label: "Running".to_string(),
            },
        ),
        (
            "paused".to_string(),
            ProjectStatusMeta {
                badge_status: "paused".to_string(),
                card_label: "Paused".to_string(),
                sidebar_label: "Paused".to_string(),
            },
        ),
        (
            "blocked".to_string(),
            ProjectStatusMeta {
                badge_status: "blocked".to_string(),
                card_label: "Blocked".to_string(),
                sidebar_label: "Blocked".to_string(),
            },
        ),
        (
            "completed".to_string(),
            ProjectStatusMeta {
                badge_status: "completed".to_string(),
                card_label: "Completed".to_string(),
                sidebar_label: "Completed".to_string(),
            },
        ),
        (
            "failed".to_string(),
            ProjectStatusMeta {
                badge_status: "failed".to_string(),
                card_label: "Failed".to_string(),
                sidebar_label: "Failed".to_string(),
            },
        ),
        (
            "draft".to_string(),
            ProjectStatusMeta {
                badge_status: "draft".to_string(),
                card_label: "Draft".to_string(),
                sidebar_label: "Draft".to_string(),
            },
        ),
        (
            "archived".to_string(),
            ProjectStatusMeta {
                badge_status: "archived".to_string(),
                card_label: "Archived".to_string(),
                sidebar_label: "Archived".to_string(),
            },
        ),
    ]);
    let plan_kind_meta = HashMap::from([
        (
            "search".to_string(),
            PlanKindMeta {
                label: "SRCH".to_string(),
                variant: "info".to_string(),
            },
        ),
        (
            "docsLookup".to_string(),
            PlanKindMeta {
                label: "DOCS".to_string(),
                variant: "warning".to_string(),
            },
        ),
        (
            "mcpCall".to_string(),
            PlanKindMeta {
                label: "TOOL".to_string(),
                variant: "neutral".to_string(),
            },
        ),
        (
            "thinking".to_string(),
            PlanKindMeta {
                label: "WAIT".to_string(),
                variant: "neutral".to_string(),
            },
        ),
        (
            "error".to_string(),
            PlanKindMeta {
                label: "ERR".to_string(),
                variant: "danger".to_string(),
            },
        ),
        (
            "planContent".to_string(),
            PlanKindMeta {
                label: "PLAN".to_string(),
                variant: "success".to_string(),
            },
        ),
    ]);
    let atomize_activity_kind_meta = HashMap::from([
        (
            "planLoaded".to_string(),
            PlanKindMeta {
                label: "LOAD".to_string(),
                variant: "info".to_string(),
            },
        ),
        (
            "templateRender".to_string(),
            PlanKindMeta {
                label: "TMPL".to_string(),
                variant: "neutral".to_string(),
            },
        ),
        (
            "agentStart".to_string(),
            PlanKindMeta {
                label: "CALL".to_string(),
                variant: "warning".to_string(),
            },
        ),
        (
            "agentComplete".to_string(),
            PlanKindMeta {
                label: "RECV".to_string(),
                variant: "success".to_string(),
            },
        ),
        (
            "chunkDetected".to_string(),
            PlanKindMeta {
                label: "SECT".to_string(),
                variant: "info".to_string(),
            },
        ),
        (
            "sectionProcess".to_string(),
            PlanKindMeta {
                label: "PROC".to_string(),
                variant: "warning".to_string(),
            },
        ),
        (
            "storyExtracted".to_string(),
            PlanKindMeta {
                label: "ATOM".to_string(),
                variant: "success".to_string(),
            },
        ),
        (
            "retry".to_string(),
            PlanKindMeta {
                label: "RTRY".to_string(),
                variant: "danger".to_string(),
            },
        ),
        (
            "validation".to_string(),
            PlanKindMeta {
                label: "VALD".to_string(),
                variant: "info".to_string(),
            },
        ),
        (
            "artifactSaved".to_string(),
            PlanKindMeta {
                label: "SAVE".to_string(),
                variant: "success".to_string(),
            },
        ),
    ]);
    DisplayVocabulary {
        project_status_meta,
        status_labels: string_map(&[
            ("draft", "Draft"),
            ("pending", "Pending"),
            ("current", "Current"),
            ("running", "Running"),
            ("paused", "Paused"),
            ("blocked", "Blocked"),
            ("completed", "Completed"),
            ("success", "Success"),
            ("error", "Error"),
            ("failed", "Failed"),
            ("archived", "Archived"),
        ]),
        status_variants: string_map(&[
            ("draft", "neutral"),
            ("pending", "neutral"),
            ("current", "info"),
            ("running", "info"),
            ("paused", "warning"),
            ("blocked", "danger"),
            ("completed", "success"),
            ("success", "success"),
            ("error", "danger"),
            ("failed", "danger"),
            ("archived", "neutral"),
        ]),
        notification_type_labels: string_map(&[
            ("story_blocked", "Blocked"),
            ("loop_completed", "Completed"),
            ("rate_limited", "Rate limit"),
            ("review_comment", "Review"),
            ("loop_error", "Error"),
            ("story_completed", "Story done"),
        ]),
        notification_ring_variants: string_map(&[
            ("red", "danger"),
            ("amber", "warning"),
            ("cyan", "info"),
            ("green", "success"),
        ]),
        story_status_variants: string_map(&[
            ("completed", "success"),
            ("current", "info"),
            ("blocked", "danger"),
            ("pending", "neutral"),
        ]),
        activity_result_variants: string_map(&[
            ("success", "success"),
            ("pending", "info"),
            ("failed", "danger"),
            ("blocked", "danger"),
            ("skipped", "warning"),
        ]),
        monitor_status_badges: string_map(&[
            ("ready", "pending"),
            ("running", "running"),
            ("paused", "paused"),
            ("blocked", "blocked"),
            ("failed", "failed"),
            ("completed", "completed"),
            ("archived", "archived"),
            ("draft", "draft"),
        ]),
        plan_kind_meta,
        atomize_activity_kind_meta,
        story_priority_variants: string_map(&[
            ("critical", "danger"),
            ("high", "warning"),
            ("medium", "info"),
            ("low", "neutral"),
        ]),
        wizard_step_labels: string_map(&[
            ("describe", "Describe"),
            ("plan", "Planning"),
            ("atomize", "Atomize"),
            ("configure", "Configure"),
            ("launch", "Launch"),
        ]),
        stage_status_badges: string_map(&[
            ("pending", "neutral"),
            ("running", "info"),
            ("done", "success"),
            ("error", "danger"),
        ]),
        stage_status_labels: string_map(&[
            ("pending", "Pending"),
            ("running", "Running"),
            ("done", "Done"),
            ("error", "Error"),
        ]),
        step_indicator_variants: string_map(&[
            ("upcoming", "neutral"),
            ("current", "info"),
            ("complete", "success"),
            ("stale", "warning"),
            ("error", "danger"),
        ]),
        step_indicator_emphasis: string_map(&[
            ("upcoming", "subtle"),
            ("current", "solid"),
            ("complete", "subtle"),
            ("stale", "subtle"),
            ("error", "subtle"),
        ]),
        agent_names: vec!["cursor", "codex", "claude", "gemini", "opencode"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        inactive_statuses: vec!["archived".to_string(), "failed".to_string()],
        stall_threshold_secs: 120,
        max_visible_activity_events: 200,
        max_output_lines: 5000,
    }
}
