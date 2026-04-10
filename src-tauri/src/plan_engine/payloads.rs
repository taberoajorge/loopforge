use crate::activity::PlanEventKind;
use serde::{Deserialize, Serialize};

pub(super) const HEARTBEAT_INTERVAL_SECS: u64 = 15;
pub(super) const DEFAULT_STALL_THRESHOLD_SECS: u64 = 180;
pub(super) const CURSOR_INITIAL_GRACE_SECS: u64 = 20;
pub(super) const BATCH_FLUSH_INTERVAL_MS: u64 = 150;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanActivityPayload {
    pub project_id: String,
    pub kind: PlanEventKind,
    pub content: String,
    pub timestamp: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanActivityBatchPayload {
    pub project_id: String,
    pub events: Vec<PlanActivityPayload>,
    pub plan_content_delta: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanTerminalPayload {
    pub project_id: String,
    pub detail: String,
}
