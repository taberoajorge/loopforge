use ralph_core::prd::{Complexity, Priority, ScopeSpec, VerificationSpec};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AtomizerError {
    #[error("Template error: {0}")]
    Template(String),
    #[error("Path error: {0}")]
    Path(String),
    #[error("Agent invocation failed: {0}")]
    AgentFailed(String),
    #[error("JSON parse error at stage {stage}: {detail}")]
    JsonParse { stage: &'static str, detail: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PRD validation failed: {0}")]
    Validation(String),
}

impl Serialize for AtomizerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtomizeArgs {
    pub project_id: String,
    pub project_name: String,
    pub project_dir: PathBuf,
    pub agent: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub effort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtomizeProgress {
    pub stage: u8,
    pub stage_name: String,
    pub message: String,
    pub project_id: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct ChunkSection {
    pub(super) title: String,
    pub(super) content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AtomizedStoryDraft {
    pub(super) title: String,
    #[serde(default)]
    pub(super) description: Option<String>,
    #[serde(default)]
    pub(super) acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub(super) scope: ScopeSpec,
    #[serde(default)]
    pub(super) verification: VerificationSpec,
    #[serde(default)]
    pub(super) commit_message: Option<String>,
    #[serde(default)]
    pub(super) priority: Priority,
    #[serde(default)]
    pub(super) estimated_complexity: Complexity,
    #[serde(default)]
    pub(super) estimated_minutes: u32,
    #[serde(default)]
    pub(super) depends_on: Vec<String>,
}
