use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, State};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfo {
    pub name: String,
    pub binary: String,
    pub version: Option<String>,
    pub available: bool,
}

#[derive(Debug, Default)]
pub struct AgentRegistry {
    pub agents: Vec<AgentInfo>,
}

#[derive(Debug, Default)]
pub struct AgentRegistryState(pub Mutex<AgentRegistry>);

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Registry lock poisoned")]
    LockPoisoned,
    #[error("Unknown agent: {0}")]
    UnknownAgent(String),
}

impl Serialize for AgentError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

const KNOWN_AGENTS: &[(&str, &str)] = &[
    ("claude", "claude"),
    ("codex", "codex"),
    ("cursor", "cursor-agent"),
    ("gemini", "gemini"),
    ("opencode", "opencode"),
];

fn known_agent_binary(agent: &str) -> Option<&'static str> {
    KNOWN_AGENTS
        .iter()
        .find(|(name, _)| *name == agent)
        .map(|(_, binary)| *binary)
}

async fn probe_agent(app: &AppHandle, binary: &str) -> (bool, Option<String>) {
    let _ = app;
    let path_env = crate::agent_runtime_env::probe_path_env();
    let Some(binary_path) = crate::agent_runtime_env::resolve_binary_path(binary, &path_env) else {
        return (false, None);
    };
    if let Some(version) =
        crate::agent_runtime_env::run_version_probe(&binary_path, &path_env, "--version")
    {
        return (true, Some(version));
    }
    if let Some(version) = crate::agent_runtime_env::run_version_probe(&binary_path, &path_env, "-v")
    {
        return (true, Some(version));
    }
    (true, None)
}

#[cfg(test)]
pub struct FallbackChain {
    agents: Vec<AgentInfo>,
    current_index: usize,
}

#[cfg(test)]
impl FallbackChain {
    pub fn new(agents: Vec<AgentInfo>) -> Self {
        let available: Vec<AgentInfo> = agents.into_iter().filter(|agent| agent.available).collect();
        Self {
            agents: available,
            current_index: 0,
        }
    }

    pub fn current_agent(&self) -> Option<&AgentInfo> {
        self.agents.get(self.current_index)
    }

    pub fn next_agent(&mut self) -> Option<&AgentInfo> {
        self.current_index += 1;
        self.agents.get(self.current_index)
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
    }

    pub fn is_exhausted(&self) -> bool {
        self.current_index >= self.agents.len()
    }
}

#[tauri::command]
pub async fn detect_agents(
    app: AppHandle,
    state: State<'_, AgentRegistryState>,
) -> Result<Vec<AgentInfo>, AgentError> {
    let mut detected = Vec::with_capacity(KNOWN_AGENTS.len());

    for (name, binary) in KNOWN_AGENTS {
        let (available, version) = probe_agent(&app, binary).await;
        detected.push(AgentInfo {
            name: name.to_string(),
            binary: binary.to_string(),
            version,
            available,
        });
    }

    let mut registry = state.0.lock().map_err(|_| AgentError::LockPoisoned)?;
    registry.agents = detected.clone();

    Ok(detected)
}

#[tauri::command]
pub async fn get_agent_capabilities(
    agent: String,
) -> Result<crate::agent_profiles::AgentCapabilities, AgentError> {
    let normalized = agent.trim().to_lowercase();
    let Some(binary_name) = known_agent_binary(&normalized) else {
        return Err(AgentError::UnknownAgent(normalized));
    };
    let path_env = crate::agent_runtime_env::probe_path_env();
    let binary_path = crate::agent_runtime_env::resolve_binary_path(binary_name, &path_env);
    Ok(crate::agent_profiles::resolve_capabilities(
        &normalized,
        binary_path.as_deref(),
        &path_env,
    ))
}

#[tauri::command]
pub async fn refresh_agents(
    app: AppHandle,
    state: State<'_, AgentRegistryState>,
) -> Result<Vec<AgentInfo>, AgentError> {
    detect_agents(app, state).await
}
