use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartLoopArgs {
    pub project_id: String,
    #[serde(default)]
    pub project_name: Option<String>,
    #[serde(default)]
    pub working_directory: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub effort: Option<String>,
    #[serde(default)]
    pub fallback_agents: Vec<String>,
    #[serde(default)]
    pub max_iterations: Option<u32>,
    #[serde(default)]
    pub gutter_threshold: Option<u32>,
    #[serde(default)]
    pub cooldown_seconds: Option<u32>,
    #[serde(default)]
    pub test_command: Option<String>,
    #[serde(default)]
    pub max_verification_retries: Option<u32>,
}
