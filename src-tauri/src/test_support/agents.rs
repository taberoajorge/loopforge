use super::runtime::TestRuntime;
use crate::agent_profiles::{AgentCapabilities, AgentEffortOption, AgentModelOption};
use crate::agents::AgentInfo;

struct FixtureAgentSpec {
    name: &'static str,
    binary: &'static str,
    available: bool,
    models: &'static [&'static str],
    efforts: &'static [&'static str],
    default_model: Option<&'static str>,
    default_effort: Option<&'static str>,
}

const FIXTURE_AGENTS: &[FixtureAgentSpec] = &[
    FixtureAgentSpec {
        name: "claude",
        binary: "claude",
        available: true,
        models: &["fixture-sonnet", "fixture-opus"],
        efforts: &["low", "medium", "high"],
        default_model: Some("fixture-sonnet"),
        default_effort: Some("medium"),
    },
    FixtureAgentSpec {
        name: "codex",
        binary: "codex",
        available: true,
        models: &["fixture-gpt-5.4", "fixture-gpt-5.4-mini"],
        efforts: &["low", "medium", "high"],
        default_model: Some("fixture-gpt-5.4"),
        default_effort: Some("medium"),
    },
    FixtureAgentSpec {
        name: "cursor",
        binary: "cursor-agent",
        available: true,
        models: &["fixture-codex-high", "fixture-gpt-5.4"],
        efforts: &[],
        default_model: Some("fixture-codex-high"),
        default_effort: None,
    },
    FixtureAgentSpec {
        name: "gemini",
        binary: "gemini",
        available: true,
        models: &["fixture-auto", "fixture-pro"],
        efforts: &[],
        default_model: Some("fixture-auto"),
        default_effort: None,
    },
    FixtureAgentSpec {
        name: "opencode",
        binary: "opencode",
        available: true,
        models: &[],
        efforts: &[],
        default_model: None,
        default_effort: None,
    },
];

pub(crate) fn fixture_agents(runtime: &TestRuntime) -> Vec<AgentInfo> {
    if matches!(runtime.fixture_set(), super::runtime::FixtureSet::NoAgents) {
        return Vec::new();
    }
    let fixture_tag = runtime.fixture_set().as_str();
    FIXTURE_AGENTS
        .iter()
        .map(|spec| AgentInfo {
            name: spec.name.to_string(),
            binary: spec.binary.to_string(),
            version: Some(format!("{fixture_tag}-{}-1.0.0", spec.name)),
            available: spec.available,
        })
        .collect()
}

pub(crate) fn fixture_capabilities(
    runtime: &TestRuntime,
    agent: &str,
) -> Option<AgentCapabilities> {
    if matches!(runtime.fixture_set(), super::runtime::FixtureSet::NoAgents) {
        return None;
    }
    let spec = FIXTURE_AGENTS.iter().find(|spec| spec.name == agent)?;
    let source = format!("fixture:{}", runtime.fixture_set().as_str());
    Some(AgentCapabilities {
        agent: spec.name.to_string(),
        source,
        supports_model: !spec.models.is_empty(),
        supports_effort: !spec.efforts.is_empty(),
        models: model_options(spec.models),
        efforts: effort_options(spec.efforts),
        default_model: spec.default_model.map(str::to_string),
        default_effort: spec.default_effort.map(str::to_string),
    })
}

#[cfg(test)]
pub(crate) fn fixture_agent_names(_: super::runtime::FixtureSet) -> Vec<&'static str> {
    FIXTURE_AGENTS.iter().map(|spec| spec.name).collect()
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
