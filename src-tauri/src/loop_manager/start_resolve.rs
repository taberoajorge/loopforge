use super::{LoopError, StartLoopArgs};
use crate::db::DbState;
use crate::projects::ProjectConfig;
use tauri::AppHandle;

#[derive(Clone)]
pub(super) struct ResolvedStartLoop {
    pub(super) project_id: String,
    pub(super) project_name: String,
    pub(super) working_directory: String,
    pub(super) agent: String,
    pub(super) model: Option<String>,
    pub(super) effort: Option<String>,
    pub(super) fallback_agents: Vec<String>,
    pub(super) max_iterations: Option<u32>,
    pub(super) gutter_threshold: Option<u32>,
    pub(super) cooldown_seconds: Option<u32>,
    pub(super) test_command: Option<String>,
    pub(super) max_verification_retries: Option<u32>,
}

fn read_project_metadata(
    db: &DbState,
    project_id: &str,
) -> Result<(String, String), LoopError> {
    let conn = db.0.lock().map_err(|_| LoopError::LockPoisoned)?;
    conn.query_row(
        "SELECT name, working_directory FROM projects WHERE id = ?1",
        rusqlite::params![project_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    )
    .map_err(|err| LoopError::Db(err.to_string()))
}

fn legacy_config_from_args(args: &StartLoopArgs) -> ProjectConfig {
    let mut runtime_config = ProjectConfig::default();
    if let Some(agent_name) = args.agent.clone() {
        runtime_config.execute_agent = agent_name;
    }
    runtime_config.execute_model = args.model.clone();
    runtime_config.execute_effort = args.effort.clone();
    if !args.fallback_agents.is_empty() {
        runtime_config.fallback_chain = args.fallback_agents.clone();
    }
    if let Some(max_iterations) = args.max_iterations {
        runtime_config.max_iterations = max_iterations;
    }
    if let Some(gutter_threshold) = args.gutter_threshold {
        runtime_config.gutter_threshold = gutter_threshold;
    }
    if let Some(cooldown_seconds) = args.cooldown_seconds {
        runtime_config.cooldown_seconds = cooldown_seconds;
    }
    if let Some(test_command) = args.test_command.clone() {
        runtime_config.test_command = test_command;
    }
    if let Some(max_retries) = args.max_verification_retries {
        runtime_config.max_verification_retries = max_retries;
    }
    runtime_config
}

pub(super) async fn resolve_start_loop(
    app: &AppHandle,
    db: &DbState,
    args: &StartLoopArgs,
) -> Result<ResolvedStartLoop, LoopError> {
    let (db_project_name, db_working_directory) = read_project_metadata(db, &args.project_id)?;
    let runtime_config = match crate::projects::runtime_config::get_project_config(
        app.clone(),
        args.project_id.clone(),
    )
    .await
    .map_err(|err| LoopError::Db(err.to_string()))? {
        Some(config) => config,
        None => {
            let migrated_config = legacy_config_from_args(args);
            crate::projects::runtime_config::save_project_config(
                app,
                &args.project_id,
                &migrated_config,
            )
            .map_err(|err| LoopError::Db(err.to_string()))?;
            migrated_config
        }
    };
    let project_name = db_project_name;
    let working_directory = db_working_directory;
    let fallback_agents = runtime_config
        .fallback_chain
        .iter()
        .filter(|agent_name| agent_name.as_str() != runtime_config.execute_agent.as_str())
        .cloned()
        .collect::<Vec<String>>();
    let test_command = if runtime_config.test_command.trim().is_empty() {
        None
    } else {
        Some(runtime_config.test_command.clone())
    };
    Ok(ResolvedStartLoop {
        project_id: args.project_id.clone(),
        project_name,
        working_directory,
        agent: runtime_config.execute_agent.clone(),
        model: runtime_config.execute_model.clone(),
        effort: runtime_config.execute_effort.clone(),
        fallback_agents,
        max_iterations: Some(runtime_config.max_iterations),
        gutter_threshold: Some(runtime_config.gutter_threshold),
        cooldown_seconds: Some(runtime_config.cooldown_seconds),
        test_command,
        max_verification_retries: Some(runtime_config.max_verification_retries),
    })
}
