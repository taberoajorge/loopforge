use crate::projects::config_types::ProjectConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigLimits {
    pub gutter_threshold: (u32, u32),
    pub max_iterations: (u32, u32),
    pub cooldown_seconds: (u32, u32),
    pub max_verification_retries: (u32, u32),
    pub review_polling_interval: (u64, u64),
    pub review_timeout: (u64, u64),
}

impl Default for ConfigLimits {
    fn default() -> Self {
        Self {
            gutter_threshold: (1, 20),
            max_iterations: (1, 500),
            cooldown_seconds: (0, 300),
            max_verification_retries: (1, 10),
            review_polling_interval: (10, 600),
            review_timeout: (60, 3600),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigDefaultsResponse {
    pub config: ProjectConfig,
    pub limits: ConfigLimits,
}

pub fn get_config_defaults() -> ConfigDefaultsResponse {
    ConfigDefaultsResponse {
        config: ProjectConfig::default(),
        limits: ConfigLimits::default(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationErrors {
    pub errors: std::collections::HashMap<String, String>,
}

impl ValidationErrors {
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

pub fn validate_config(config: &ProjectConfig) -> ValidationErrors {
    let limits = ConfigLimits::default();
    let mut errors = std::collections::HashMap::new();

    if config.execute_agent.trim().is_empty() {
        errors.insert(
            "executeAgent".to_string(),
            "Execute agent is required".to_string(),
        );
    }

    validate_range(
        &mut errors,
        "gutterThreshold",
        "Gutter threshold",
        config.gutter_threshold as u64,
        limits.gutter_threshold.0 as u64,
        limits.gutter_threshold.1 as u64,
    );
    validate_range(
        &mut errors,
        "maxIterations",
        "Max iterations",
        config.max_iterations as u64,
        limits.max_iterations.0 as u64,
        limits.max_iterations.1 as u64,
    );
    validate_range(
        &mut errors,
        "cooldownSeconds",
        "Cooldown",
        config.cooldown_seconds as u64,
        limits.cooldown_seconds.0 as u64,
        limits.cooldown_seconds.1 as u64,
    );
    validate_range(
        &mut errors,
        "maxVerificationRetries",
        "Verification retries",
        config.max_verification_retries as u64,
        limits.max_verification_retries.0 as u64,
        limits.max_verification_retries.1 as u64,
    );
    validate_range(
        &mut errors,
        "reviewPollingInterval",
        "Poll interval",
        config.review_polling_interval,
        limits.review_polling_interval.0,
        limits.review_polling_interval.1,
    );
    validate_range(
        &mut errors,
        "reviewTimeout",
        "Timeout",
        config.review_timeout,
        limits.review_timeout.0,
        limits.review_timeout.1,
    );

    ValidationErrors { errors }
}

fn validate_range(
    errors: &mut std::collections::HashMap<String, String>,
    field: &str,
    label: &str,
    value: u64,
    min: u64,
    max: u64,
) {
    if value < min || value > max {
        errors.insert(
            field.to_string(),
            format!("{label} must be between {min} and {max}"),
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescribeInput {
    pub name: String,
    pub description: String,
    pub working_directory: String,
    pub plan_agent: String,
}

pub fn validate_describe(input: &DescribeInput, available_agents: &[String]) -> ValidationErrors {
    let mut errors = std::collections::HashMap::new();

    if input.name.trim().is_empty() {
        errors.insert("name".to_string(), "Project name is required".to_string());
    }
    if input.description.trim().is_empty() {
        errors.insert(
            "description".to_string(),
            "Feature description is required".to_string(),
        );
    }
    if input.working_directory.trim().is_empty() {
        errors.insert(
            "workingDirectory".to_string(),
            "Working directory is required".to_string(),
        );
    }
    if available_agents.is_empty() {
        errors.insert(
            "submit".to_string(),
            "No supported agent was detected. Install Claude, Codex, Gemini, or OpenCode."
                .to_string(),
        );
    } else if !available_agents.contains(&input.plan_agent) {
        errors.insert(
            "submit".to_string(),
            "Selected plan agent is not available in this environment.".to_string(),
        );
    }

    ValidationErrors { errors }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchReadiness {
    pub ready: bool,
    pub issues: Vec<String>,
}

pub fn validate_launch_readiness(
    project_name: &str,
    working_directory: &str,
    stories_count: usize,
    execute_agent: &str,
) -> LaunchReadiness {
    let mut issues = Vec::new();

    if project_name.trim().is_empty() {
        issues.push("Project name is missing.".to_string());
    }
    if working_directory.trim().is_empty() {
        issues.push("Working directory is missing.".to_string());
    }
    if stories_count == 0 {
        issues.push("Add at least one story before launching.".to_string());
    }
    if execute_agent.trim().is_empty() {
        issues.push("Execution agent is missing.".to_string());
    }

    LaunchReadiness {
        ready: issues.is_empty(),
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_passes_validation() {
        let config = ProjectConfig::default();
        let result = validate_config(&config);
        assert!(result.is_empty());
    }

    #[test]
    fn empty_agent_fails_validation() {
        let mut config = ProjectConfig::default();
        config.execute_agent = String::new();
        let result = validate_config(&config);
        assert!(result.errors.contains_key("executeAgent"));
    }

    #[test]
    fn launch_readiness_catches_missing_fields() {
        let result = validate_launch_readiness("", "", 0, "");
        assert!(!result.ready);
        assert_eq!(result.issues.len(), 4);
    }
}
