use crate::errors::ConfigError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PathConfig {
    pub ralph_dir: PathBuf,
    pub work_dir: PathBuf,
    pub prd_file: PathBuf,
    pub prd_backup: PathBuf,
    pub prompt_file: PathBuf,
    pub progress_file: PathBuf,
    pub guardrails_file: PathBuf,
    pub error_log: PathBuf,
    pub activity_log: PathBuf,
    pub state_file: PathBuf,
    pub pause_file: PathBuf,
    pub done_file: PathBuf,
    pub failure_memory_file: PathBuf,
    pub last_rebase_file: PathBuf,
    pub codex_output_log: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TuningConfig {
    pub max_iterations: u32,
    pub rate_limit_wait_secs: u64,
    pub gutter_threshold: u32,
    pub stall_timeout_secs: u64,
    pub cooldown_secs: u64,
    pub max_verification_retries: u32,
    pub test_command: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServiceConfigs {
    pub legacy: Option<ServiceConfig>,
    pub new_service: Option<ServiceConfig>,
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub health_url: Option<String>,
    pub port: Option<u16>,
    pub start_command: Option<String>,
    pub stop_command: Option<String>,
    pub working_directory: Option<PathBuf>,
    pub optional: bool,
    pub service_type: Option<String>,
    pub build_command: Option<String>,
    pub container_name: Option<String>,
    pub image_name: Option<String>,
    pub max_start_wait_secs: u64,
}

#[derive(Debug, Clone)]
pub struct RalphConfig {
    pub paths: PathConfig,
    pub tuning: TuningConfig,
    pub services: Option<ServiceConfigs>,
}

impl RalphConfig {
    pub fn from_defaults(ralph_dir: &Path) -> Self {
        Self {
            paths: PathConfig {
                ralph_dir: ralph_dir.to_path_buf(),
                work_dir: PathBuf::new(),
                prd_file: PathBuf::new(),
                prd_backup: PathBuf::new(),
                prompt_file: PathBuf::new(),
                progress_file: PathBuf::new(),
                guardrails_file: ralph_dir.join("guardrails.md"),
                error_log: ralph_dir.join("error.log"),
                activity_log: ralph_dir.join("activity.log"),
                state_file: ralph_dir.join(".ralph_state"),
                pause_file: ralph_dir.join(".ralph-pause"),
                done_file: ralph_dir.join(".ralph-done"),
                failure_memory_file: ralph_dir.join("failure_memory.json"),
                last_rebase_file: ralph_dir.join(".ralph_last_rebase"),
                codex_output_log: ralph_dir.join("codex_output.log"),
            },
            tuning: TuningConfig {
                max_iterations: env_or("MAX_ITERATIONS", 9999) as u32,
                rate_limit_wait_secs: env_or("RATE_LIMIT_WAIT", 120),
                gutter_threshold: env_or("GUTTER_THRESHOLD", 3) as u32,
                stall_timeout_secs: env_or("CODEX_STALL_TIMEOUT", 300),
                cooldown_secs: 30,
                max_verification_retries: env_or("MAX_VERIFICATION_RETRIES", 3) as u32,
                test_command: None,
            },
            services: None,
        }
    }

    pub fn load_toml_overlay(&mut self, config_path: &Path) -> Result<(), ConfigError> {
        let content =
            std::fs::read_to_string(config_path).map_err(|source| ConfigError::ReadFailed {
                path: config_path.to_path_buf(),
                source,
            })?;
        let parsed: TomlConfig =
            toml::from_str(&content).map_err(|source| ConfigError::ParseFailed {
                path: config_path.to_path_buf(),
                source,
            })?;

        let base_dir = parsed
            .project
            .as_ref()
            .and_then(|project| project.working_directory.as_deref())
            .map_or_else(|| self.paths.ralph_dir.clone(), PathBuf::from);

        if let Some(prd_cfg) = &parsed.prd {
            if let Some(path) = &prd_cfg.path {
                self.paths.prd_file = base_dir.join(path);
            }
            if let Some(backup) = &prd_cfg.backup {
                self.paths.prd_backup = base_dir.join(backup);
            }
            if let Some(prompt) = &prd_cfg.prompt {
                self.paths.prompt_file = base_dir.join(prompt);
            }
            if let Some(guardrails) = &prd_cfg.guardrails {
                self.paths.guardrails_file = base_dir.join(guardrails);
            }
            if let Some(progress) = &prd_cfg.progress {
                self.paths.progress_file = base_dir.join(progress);
            }
        }

        if let Some(services_cfg) = parsed.services {
            let legacy = services_cfg
                .legacy
                .map(|svc| to_service_config(svc, "legacy"));
            let new_service = services_cfg.new.map(|svc| to_service_config(svc, "new"));
            self.services = Some(ServiceConfigs {
                legacy,
                new_service,
            });
        }

        if let Some(test_cfg) = parsed.test {
            if let Some(command) = test_cfg.command {
                self.tuning.test_command = Some(command);
            }
        }

        Ok(())
    }
}

fn to_service_config(svc: TomlService, default_name: &str) -> ServiceConfig {
    ServiceConfig {
        name: svc.name.unwrap_or_else(|| default_name.into()),
        health_url: svc.health,
        port: svc.port,
        start_command: svc.start,
        stop_command: svc.stop,
        working_directory: svc.working_directory.map(PathBuf::from),
        optional: svc.optional.unwrap_or(false),
        service_type: svc.service_type,
        build_command: svc.build_command,
        container_name: svc.container_name,
        image_name: svc.image_name,
        max_start_wait_secs: svc.max_start_wait.unwrap_or(30),
    }
}

fn env_or(var_name: &str, default: u64) -> u64 {
    std::env::var(var_name)
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(default)
}

#[derive(Debug, Deserialize)]
struct TomlConfig {
    project: Option<TomlProject>,
    prd: Option<TomlPrd>,
    services: Option<TomlServices>,
    test: Option<TomlTest>,
}

#[derive(Debug, Deserialize)]
struct TomlProject {
    #[serde(rename = "workingDirectory")]
    working_directory: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TomlPrd {
    path: Option<String>,
    backup: Option<String>,
    prompt: Option<String>,
    guardrails: Option<String>,
    progress: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TomlServices {
    legacy: Option<TomlService>,
    new: Option<TomlService>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TomlService {
    name: Option<String>,
    health: Option<String>,
    port: Option<u16>,
    start: Option<String>,
    stop: Option<String>,
    #[serde(rename = "workingDirectory")]
    working_directory: Option<String>,
    optional: Option<bool>,
    #[serde(rename = "type")]
    service_type: Option<String>,
    #[serde(rename = "buildCommand")]
    build_command: Option<String>,
    #[serde(rename = "containerName")]
    container_name: Option<String>,
    #[serde(rename = "imageName")]
    image_name: Option<String>,
    #[serde(rename = "maxStartWait")]
    max_start_wait: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TomlTest {
    command: Option<String>,
}
