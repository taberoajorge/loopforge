use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::process::Command;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentModelOption {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEffortOption {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    pub agent: String,
    pub source: String,
    pub supports_model: bool,
    pub supports_effort: bool,
    pub models: Vec<AgentModelOption>,
    pub efforts: Vec<AgentEffortOption>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub default_effort: Option<String>,
}

pub fn resolve_capabilities(
    agent: &str,
    binary_path: Option<&str>,
    path_env: &str,
) -> AgentCapabilities {
    let curated = curated_capabilities(agent);
    let discovered = discover_models(agent, binary_path, path_env);
    if discovered.is_empty() {
        return curated;
    }
    let models = discovered
        .into_iter()
        .map(|id| AgentModelOption {
            label: id.clone(),
            id,
        })
        .collect::<Vec<AgentModelOption>>();
    AgentCapabilities {
        source: "dynamic".to_string(),
        supports_model: !models.is_empty(),
        default_model: models.first().map(|entry| entry.id.clone()),
        models,
        ..curated
    }
}

fn curated_capabilities(agent: &str) -> AgentCapabilities {
    match agent {
        "claude" => AgentCapabilities {
            agent: agent.to_string(),
            source: "curated".to_string(),
            supports_model: true,
            supports_effort: true,
            models: model_options(&["sonnet", "opus", "haiku"]),
            efforts: effort_options(&["low", "medium", "high"]),
            default_model: Some("sonnet".to_string()),
            default_effort: Some("medium".to_string()),
        },
        "codex" => AgentCapabilities {
            agent: agent.to_string(),
            source: "curated".to_string(),
            supports_model: true,
            supports_effort: true,
            models: model_options(&[
                "gpt-5.4",
                "gpt-5.4-mini",
                "gpt-5-codex",
                "gpt-5.3-codex-high",
            ]),
            efforts: effort_options(&["low", "medium", "high"]),
            default_model: Some("gpt-5.4".to_string()),
            default_effort: None,
        },
        "cursor" => AgentCapabilities {
            agent: agent.to_string(),
            source: "curated".to_string(),
            supports_model: true,
            supports_effort: false,
            models: model_options(&["gpt-5.3-codex-high", "gpt-5.4", "gpt-5.4-mini"]),
            efforts: Vec::new(),
            default_model: Some("gpt-5.3-codex-high".to_string()),
            default_effort: None,
        },
        "gemini" => AgentCapabilities {
            agent: agent.to_string(),
            source: "curated".to_string(),
            supports_model: true,
            supports_effort: false,
            models: model_options(&[
                "auto",
                "gemini-2.5-pro",
                "gemini-2.5-flash",
                "gemini-2.5-flash-lite",
            ]),
            efforts: Vec::new(),
            default_model: Some("auto".to_string()),
            default_effort: None,
        },
        _ => AgentCapabilities {
            agent: agent.to_string(),
            source: "curated".to_string(),
            supports_model: false,
            supports_effort: false,
            models: Vec::new(),
            efforts: Vec::new(),
            default_model: None,
            default_effort: None,
        },
    }
}

fn model_options(models: &[&str]) -> Vec<AgentModelOption> {
    models
        .iter()
        .map(|id| AgentModelOption {
            id: (*id).to_string(),
            label: (*id).to_string(),
        })
        .collect()
}

fn effort_options(levels: &[&str]) -> Vec<AgentEffortOption> {
    levels
        .iter()
        .map(|id| AgentEffortOption {
            id: (*id).to_string(),
            label: (*id).to_string(),
        })
        .collect()
}

fn discover_models(agent: &str, binary_path: Option<&str>, path_env: &str) -> Vec<String> {
    let Some(binary) = binary_path else {
        return Vec::new();
    };
    let command_sets: &[&[&str]] = match agent {
        "cursor" => &[&["--list-models"], &["models"]],
        _ => &[],
    };
    for command_set in command_sets {
        let output = Command::new(binary)
            .args(*command_set)
            .env("PATH", path_env)
            .output();
        let Ok(result) = output else {
            continue;
        };
        if !result.status.success() {
            continue;
        }
        let stdout = String::from_utf8_lossy(&result.stdout).to_string();
        let parsed = parse_model_output(&stdout);
        if !parsed.is_empty() {
            return parsed;
        }
    }
    Vec::new()
}

fn parse_model_output(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            let mut items = BTreeSet::new();
            extract_ids(&value, &mut items);
            return items.into_iter().collect();
        }
    }
    let mut items = BTreeSet::new();
    for line in trimmed.lines() {
        let token = line.split_whitespace().next().unwrap_or("").trim();
        if token.len() < 3 {
            continue;
        }
        let valid = token
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'));
        if valid {
            items.insert(token.to_string());
        }
    }
    items.into_iter().collect()
}

fn extract_ids(value: &serde_json::Value, out: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                extract_ids(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(id)) = map.get("id") {
                out.insert(id.clone());
            }
            if let Some(models) = map.get("models") {
                extract_ids(models, out);
            }
        }
        _ => {}
    }
}
