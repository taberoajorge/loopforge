use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("Database error: {0}")]
    Db(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Project not found: {0}")]
    NotFound(String),
    #[error("Path resolution failed: {0}")]
    Path(String),
}

impl Serialize for ProjectError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<rusqlite::Error> for ProjectError {
    fn from(err: rusqlite::Error) -> Self {
        ProjectError::Db(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub working_directory: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub wizard_step: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsByStatus {
    pub active: Vec<Project>,
    pub paused: Vec<Project>,
    pub completed: Vec<Project>,
    pub draft: Vec<Project>,
    pub archived: Vec<Project>,
    pub blocked: Vec<Project>,
    pub failed: Vec<Project>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail {
    pub project: Project,
    pub total_stories: usize,
    pub passed_count: usize,
    pub blocked_count: usize,
    pub pending_count: usize,
    pub stories: Vec<ralph_core::prd::UserStory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardResumeState {
    pub project: Project,
    pub wizard_step: String,
    #[serde(default)]
    pub wizard_session: Option<crate::projects::wizard_state::CanonicalWizardSession>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wizard_state_json: Option<String>,
    #[serde(default)]
    pub has_plan: bool,
    #[serde(default)]
    pub has_prd: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardProjectData {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub working_directory: String,
    #[serde(default = "default_plan_agent")]
    pub plan_agent: String,
    #[serde(default)]
    pub plan_model: Option<String>,
    #[serde(default)]
    pub plan_effort: Option<String>,
}

fn default_plan_agent() -> String {
    "claude".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardHydrationResult {
    pub project: Project,
    pub wizard_step: String,
    #[serde(default)]
    pub highest_step: u32,
    pub project_data: WizardProjectData,
    #[serde(default)]
    pub plan_complete: bool,
    #[serde(default)]
    pub stories: Vec<ralph_core::prd::UserStory>,
    #[serde(default)]
    pub config: Option<super::config_types::ProjectConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IterationStory {
    pub id: String,
    pub title: String,
    pub status: String,
    pub duration_secs: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_label: Option<String>,
    pub attempts: i64,
}
