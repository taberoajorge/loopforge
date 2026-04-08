use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_execute_agent")]
    pub execute_agent: String,
    #[serde(default)]
    pub execute_model: Option<String>,
    #[serde(default)]
    pub execute_effort: Option<String>,
    #[serde(default = "default_fallback_chain")]
    pub fallback_chain: Vec<String>,
    #[serde(default = "default_gutter_threshold")]
    pub gutter_threshold: u32,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    #[serde(default = "default_cooldown_seconds")]
    pub cooldown_seconds: u32,
    #[serde(default = "default_test_command")]
    pub test_command: String,
    #[serde(default = "default_verification_retries")]
    pub max_verification_retries: u32,
    #[serde(default = "default_scm_provider")]
    pub scm_provider: String,
    #[serde(default = "default_review_polling_interval")]
    pub review_polling_interval: u64,
    #[serde(default = "default_review_timeout")]
    pub review_timeout: u64,
}

fn default_schema_version() -> u32 {
    1
}

fn default_execute_agent() -> String {
    "cursor".to_string()
}

fn default_fallback_chain() -> Vec<String> {
    vec!["claude".to_string()]
}

fn default_gutter_threshold() -> u32 {
    3
}

fn default_max_iterations() -> u32 {
    50
}

fn default_cooldown_seconds() -> u32 {
    5
}

fn default_test_command() -> String {
    String::new()
}

fn default_verification_retries() -> u32 {
    3
}

fn default_scm_provider() -> String {
    "auto".to_string()
}

fn default_review_polling_interval() -> u64 {
    60
}

fn default_review_timeout() -> u64 {
    600
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            execute_agent: default_execute_agent(),
            execute_model: None,
            execute_effort: None,
            fallback_chain: default_fallback_chain(),
            gutter_threshold: default_gutter_threshold(),
            max_iterations: default_max_iterations(),
            cooldown_seconds: default_cooldown_seconds(),
            test_command: default_test_command(),
            max_verification_retries: default_verification_retries(),
            scm_provider: default_scm_provider(),
            review_polling_interval: default_review_polling_interval(),
            review_timeout: default_review_timeout(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPrefs {
    #[serde(default = "default_true")]
    pub story_blocked_ring: bool,
    #[serde(default = "default_true")]
    pub story_blocked_os: bool,
    #[serde(default = "default_true")]
    pub loop_completed_ring: bool,
    #[serde(default = "default_true")]
    pub loop_completed_os: bool,
    #[serde(default = "default_true")]
    pub rate_limited_ring: bool,
    #[serde(default)]
    pub rate_limited_os: bool,
    #[serde(default = "default_true")]
    pub review_comment_ring: bool,
    #[serde(default)]
    pub review_comment_os: bool,
}

fn default_true() -> bool {
    true
}

impl Default for NotificationPrefs {
    fn default() -> Self {
        Self {
            story_blocked_ring: true,
            story_blocked_os: true,
            loop_completed_ring: true,
            loop_completed_os: true,
            rate_limited_ring: true,
            rate_limited_os: false,
            review_comment_ring: true,
            review_comment_os: false,
        }
    }
}
