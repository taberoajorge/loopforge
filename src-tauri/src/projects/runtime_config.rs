use crate::projects::artifacts::artifact_dir;
use crate::projects::{ProjectConfig, ProjectError};
use std::path::Path;
use tauri::{AppHandle, Manager, Runtime};

#[path = "../services/wizard_session_adapter.rs"]
pub(crate) mod wizard_session_adapter;

pub(crate) fn sanitize_config(mut config: ProjectConfig) -> ProjectConfig {
    let default_config = ProjectConfig::default();
    config.schema_version = default_config.schema_version;
    config.execute_agent = config.execute_agent.trim().to_string();
    if config.execute_agent.is_empty() {
        config.execute_agent = default_config.execute_agent;
    }
    config.execute_model = config
        .execute_model
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    config.execute_effort = config
        .execute_effort
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    config.fallback_chain = config
        .fallback_chain
        .into_iter()
        .map(|agent_name| agent_name.trim().to_string())
        .filter(|agent_name| !agent_name.is_empty())
        .collect();
    if config.fallback_chain.is_empty() {
        config.fallback_chain = default_config.fallback_chain;
    }
    if config.gutter_threshold == 0 {
        config.gutter_threshold = default_config.gutter_threshold;
    }
    if config.max_iterations == 0 {
        config.max_iterations = default_config.max_iterations;
    }
    if config.max_verification_retries == 0 {
        config.max_verification_retries = default_config.max_verification_retries;
    }
    if config.review_polling_interval == 0 {
        config.review_polling_interval = default_config.review_polling_interval;
    }
    if config.review_timeout == 0 {
        config.review_timeout = default_config.review_timeout;
    }
    config.scm_provider = config.scm_provider.trim().to_string();
    if config.scm_provider.is_empty() {
        config.scm_provider = default_config.scm_provider;
    }
    config
}

pub(crate) fn legacy_config_from_loop_args(content: &str) -> Result<ProjectConfig, ProjectError> {
    let args: serde_json::Value = serde_json::from_str(content)?;
    let default_config = ProjectConfig::default();
    let execute_agent = args
        .get("agent")
        .and_then(|value| value.as_str())
        .unwrap_or(default_config.execute_agent.as_str())
        .to_string();
    let execute_model = args
        .get("model")
        .and_then(|value| value.as_str())
        .map(ToString::to_string);
    let execute_effort = args
        .get("effort")
        .and_then(|value| value.as_str())
        .map(ToString::to_string);
    let fallback_chain = args
        .get("fallbackAgents")
        .and_then(|value| value.as_array())
        .map(|array| {
            array
                .iter()
                .filter_map(|value| value.as_str().map(String::from))
                .collect::<Vec<String>>()
        })
        .unwrap_or(default_config.fallback_chain.clone());
    let max_iterations = args
        .get("maxIterations")
        .and_then(serde_json::Value::as_u64)
        .map_or(default_config.max_iterations, |value| value as u32);
    let gutter_threshold = args
        .get("gutterThreshold")
        .and_then(serde_json::Value::as_u64)
        .map_or(default_config.gutter_threshold, |value| value as u32);
    let cooldown_seconds = args
        .get("cooldownSeconds")
        .and_then(serde_json::Value::as_u64)
        .map_or(default_config.cooldown_seconds, |value| value as u32);
    let test_command = args
        .get("testCommand")
        .and_then(|value| value.as_str())
        .unwrap_or(default_config.test_command.as_str())
        .to_string();
    let max_verification_retries = args
        .get("maxVerificationRetries")
        .and_then(serde_json::Value::as_u64)
        .map_or(default_config.max_verification_retries, |value| {
            value as u32
        });
    Ok(sanitize_config(ProjectConfig {
        schema_version: default_config.schema_version,
        execute_agent,
        execute_model,
        execute_effort,
        fallback_chain,
        gutter_threshold,
        max_iterations,
        cooldown_seconds,
        test_command,
        max_verification_retries,
        scm_provider: default_config.scm_provider,
        review_polling_interval: default_config.review_polling_interval,
        review_timeout: default_config.review_timeout,
    }))
}

pub fn save_project_config<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
    config: &ProjectConfig,
) -> Result<(), ProjectError> {
    let dir = artifact_dir(app, project_id)?;
    std::fs::create_dir_all(&dir)?;
    let config_json = serde_json::to_string_pretty(config)?;
    std::fs::write(dir.join("config.json"), config_json)?;
    Ok(())
}

pub(crate) fn load_project_config_from_paths(
    artifacts: &Path,
    working_directory: &Path,
) -> Result<Option<ProjectConfig>, ProjectError> {
    let config_path = artifacts.join("config.json");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let parsed_config: ProjectConfig = serde_json::from_str(&content)?;
        return Ok(Some(sanitize_config(parsed_config)));
    }

    let loop_args_path = artifacts.join("loop_args.json");
    if loop_args_path.exists() {
        let content = std::fs::read_to_string(&loop_args_path)?;
        return Ok(Some(legacy_config_from_loop_args(&content)?));
    }

    let legacy_config_path = working_directory.join("config.json");
    if legacy_config_path.exists() {
        let content = std::fs::read_to_string(&legacy_config_path)?;
        let parsed_config: ProjectConfig = serde_json::from_str(&content)?;
        return Ok(Some(sanitize_config(parsed_config)));
    }

    Ok(None)
}

pub async fn get_project_config<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
) -> Result<Option<ProjectConfig>, ProjectError> {
    let dir = artifact_dir(&app, &project_id)?;
    let db = app.state::<crate::db::DbState>();
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
    let working_directory = conn
        .query_row(
            "SELECT working_directory FROM projects WHERE id = ?1",
            rusqlite::params![project_id],
            |row: &rusqlite::Row| row.get::<_, String>(0),
        )
        .map_err(|_| ProjectError::NotFound(project_id.clone()))?;
    drop(conn);
    if let Some(config) = load_project_config_from_paths(&dir, Path::new(&working_directory))? {
        save_project_config(&app, &project_id, &config)?;
        return Ok(Some(config));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{legacy_config_from_loop_args, sanitize_config};
    use crate::projects::ProjectConfig;

    #[test]
    fn sanitize_config_restores_required_defaults() {
        let dirty = ProjectConfig {
            schema_version: 0,
            execute_agent: "   ".to_string(),
            execute_model: Some("  ".to_string()),
            execute_effort: Some("".to_string()),
            fallback_chain: vec!["   ".to_string()],
            gutter_threshold: 0,
            max_iterations: 0,
            cooldown_seconds: 0,
            test_command: String::new(),
            max_verification_retries: 0,
            scm_provider: " ".to_string(),
            review_polling_interval: 0,
            review_timeout: 0,
        };
        let sanitized = sanitize_config(dirty);
        assert_eq!(sanitized.schema_version, 1);
        assert_eq!(sanitized.execute_agent, "cursor");
        assert_eq!(sanitized.execute_model, None);
        assert_eq!(sanitized.execute_effort, None);
        assert_eq!(sanitized.fallback_chain, vec!["claude".to_string()]);
        assert_eq!(sanitized.gutter_threshold, 3);
        assert_eq!(sanitized.max_iterations, 50);
        assert_eq!(sanitized.max_verification_retries, 3);
        assert_eq!(sanitized.scm_provider, "auto");
        assert_eq!(sanitized.review_polling_interval, 60);
        assert_eq!(sanitized.review_timeout, 600);
    }

    #[test]
    fn legacy_config_from_loop_args_reads_runtime_fields() {
        let loop_args_json = r#"{
            "projectId": "proj-1",
            "agent": "codex",
            "model": "gpt-5.4",
            "effort": "high",
            "fallbackAgents": ["claude", "gemini"],
            "maxIterations": 120,
            "gutterThreshold": 7,
            "cooldownSeconds": 11,
            "testCommand": "bun test",
            "maxVerificationRetries": 5
        }"#;
        let config = legacy_config_from_loop_args(loop_args_json).unwrap();
        assert_eq!(config.execute_agent, "codex");
        assert_eq!(config.execute_model.as_deref(), Some("gpt-5.4"));
        assert_eq!(config.execute_effort.as_deref(), Some("high"));
        assert_eq!(
            config.fallback_chain,
            vec!["claude".to_string(), "gemini".to_string()]
        );
        assert_eq!(config.max_iterations, 120);
        assert_eq!(config.gutter_threshold, 7);
        assert_eq!(config.cooldown_seconds, 11);
        assert_eq!(config.test_command, "bun test");
        assert_eq!(config.max_verification_retries, 5);
    }
}
