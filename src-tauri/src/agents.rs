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

fn resolve_test_runtime() -> Option<crate::test_support::runtime::TestRuntime> {
    crate::test_support::runtime::resolve_test_mode()
        .ok()
        .and_then(|mode| mode.runtime().cloned())
}

fn store_detected_agents(
    state: State<'_, AgentRegistryState>,
    detected: Vec<AgentInfo>,
) -> Result<Vec<AgentInfo>, AgentError> {
    let mut registry = state.0.lock().map_err(|_| AgentError::LockPoisoned)?;
    registry.agents = detected.clone();
    Ok(detected)
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
    if let Some(version) =
        crate::agent_runtime_env::run_version_probe(&binary_path, &path_env, "-v")
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
        let available: Vec<AgentInfo> =
            agents.into_iter().filter(|agent| agent.available).collect();
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
    if let Some(runtime) = resolve_test_runtime() {
        let detected = crate::test_support::agents::fixture_agents(&runtime);
        return store_detected_agents(state, detected);
    }

    let mut detected = Vec::with_capacity(KNOWN_AGENTS.len());

    for (name, binary) in KNOWN_AGENTS {
        let (available, version) = probe_agent(&app, binary).await;
        detected.push(AgentInfo {
            name: (*name).to_string(),
            binary: (*binary).to_string(),
            version,
            available,
        });
    }

    store_detected_agents(state, detected)
}

#[tauri::command]
pub async fn get_known_agents() -> Result<Vec<String>, AgentError> {
    Ok(KNOWN_AGENTS
        .iter()
        .map(|(name, _)| (*name).to_string())
        .collect())
}

#[tauri::command]
pub async fn get_agent_capabilities(
    agent: String,
) -> Result<crate::agent_profiles::AgentCapabilities, AgentError> {
    let normalized = agent.trim().to_lowercase();
    if let Some(runtime) = resolve_test_runtime() {
        return crate::test_support::agents::fixture_capabilities(&runtime, &normalized)
            .ok_or(AgentError::UnknownAgent(normalized));
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedAgentSelection {
    pub capabilities: crate::agent_profiles::AgentCapabilities,
    pub resolved_model: Option<String>,
    pub resolved_effort: Option<String>,
}

fn resolve_model(
    caps: &crate::agent_profiles::AgentCapabilities,
    current: Option<String>,
) -> Option<String> {
    if !caps.supports_model {
        return None;
    }
    let current_trimmed = current.filter(|val| !val.trim().is_empty());
    if let Some(ref model) = current_trimmed {
        if caps.models.iter().any(|entry| entry.id == *model) {
            return current_trimmed;
        }
    }
    caps.default_model
        .clone()
        .or_else(|| caps.models.first().map(|entry| entry.id.clone()))
}

fn resolve_effort(
    caps: &crate::agent_profiles::AgentCapabilities,
    current: Option<String>,
) -> Option<String> {
    if !caps.supports_effort {
        return None;
    }
    let current_trimmed = current.filter(|val| !val.trim().is_empty());
    if let Some(ref effort) = current_trimmed {
        if caps.efforts.iter().any(|entry| entry.id == *effort) {
            return current_trimmed;
        }
    }
    caps.default_effort
        .clone()
        .or_else(|| caps.efforts.first().map(|entry| entry.id.clone()))
}

#[tauri::command]
pub async fn resolve_agent_selection(
    agent: String,
    current_model: Option<String>,
    current_effort: Option<String>,
) -> Result<ResolvedAgentSelection, AgentError> {
    let caps = get_agent_capabilities(agent).await?;
    let resolved_model = resolve_model(&caps, current_model);
    let resolved_effort = resolve_effort(&caps, current_effort);
    Ok(ResolvedAgentSelection {
        capabilities: caps,
        resolved_model,
        resolved_effort,
    })
}

#[tauri::command]
pub async fn refresh_agents(
    app: AppHandle,
    state: State<'_, AgentRegistryState>,
) -> Result<Vec<AgentInfo>, AgentError> {
    detect_agents(app, state).await
}
