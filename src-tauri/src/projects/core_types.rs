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
    pub wizard_state_json: Option<String>,
    pub has_plan: bool,
    pub has_prd: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IterationStory {
    pub id: String,
    pub title: String,
    pub status: String,
    pub duration_secs: Option<i64>,
    pub attempts: i64,
}
