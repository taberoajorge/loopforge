use std::collections::BTreeSet;
use std::path::PathBuf;

pub fn cli_binary_name(agent: &str) -> &str {
    match agent {
        "cursor" => "cursor-agent",
        other => other,
    }
}

pub const CLAUDE_EMPTY_MCP_CONFIG: &str = "{\"mcpServers\":{}}";
pub const GEMINI_MCP_ALLOWLIST_NONE: &str = "__loopforge_none__";
pub const OPENCODE_PERMISSION_ALLOW_ALL: &str =
    "{\"*\":\"allow\",\"external_directory\":\"allow\",\"doom_loop\":\"allow\"}";
pub const OPENCODE_MCP_CONFIG_NO_SERVERS: &str = "{\"mcp\":{\"openspec\":{\"enabled\":false},\"cocoindex-code\":{\"enabled\":false},\"claude-mem\":{\"enabled\":false},\"exa\":{\"enabled\":false},\"context7\":{\"enabled\":false},\"notion\":{\"enabled\":false},\"postgres\":{\"enabled\":false},\"dhtmlx-mcp\":{\"enabled\":false},\"lighthouse\":{\"enabled\":false},\"stitch\":{\"enabled\":false},\"mermaid\":{\"enabled\":false},\"mentisdb\":{\"enabled\":false}}}";
pub const DEFAULT_CURSOR_MODEL: &str = "gpt-5.3-codex-high";

const DEFAULT_CODEX_MCP_SERVERS: &[&str] = &[
    "claude-mem",
    "cocoindex-code",
    "openspec",
    "postgres",
    "Figma",
    "figma",
    "Sentry",
    "context7",
    "exa",
    "notion",
    "dhtmlx-mcp",
    "lighthouse",
    "stitch",
    "mermaid",
    "mentisdb",
];

pub fn codex_mcp_disable_args() -> Vec<String> {
    let mut server_names = configured_codex_mcp_servers();
    if server_names.is_empty() {
        server_names = DEFAULT_CODEX_MCP_SERVERS
            .iter()
            .map(|name| (*name).to_string())
            .collect();
    }

    let mut args = Vec::with_capacity(server_names.len() * 2);
    for server_name in server_names {
        args.push("-c".to_string());
        args.push(format!("mcp_servers.{server_name}.enabled=false"));
    }
    args
}

pub fn cursor_model_arg() -> String {
    std::env::var("LOOPFORGE_CURSOR_MODEL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_CURSOR_MODEL.to_string())
}

fn configured_codex_mcp_servers() -> Vec<String> {
    let Some(home_dir) = std::env::var_os("HOME") else {
        return Vec::new();
    };

    let config_path = PathBuf::from(home_dir).join(".codex").join("config.toml");
    let Ok(config_content) = std::fs::read_to_string(config_path) else {
        return Vec::new();
    };

    let mut names = BTreeSet::new();
    for line in config_content.lines() {
        if let Some(name) = extract_codex_server_name(line) {
            names.insert(name);
        }
    }

    names.into_iter().collect()
}

fn extract_codex_server_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("[mcp_servers.") || !trimmed.ends_with(']') {
        return None;
    }

    let raw_name = &trimmed["[mcp_servers.".len()..trimmed.len() - 1];
    let top_level = raw_name.split('.').next().unwrap_or("").trim();
    let normalized =
        if top_level.starts_with('"') && top_level.ends_with('"') && top_level.len() > 1 {
            &top_level[1..top_level.len() - 1]
        } else {
            top_level
        };

    if normalized.is_empty()
        || !normalized
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    {
        return None;
    }

    Some(normalized.to_string())
}

#[cfg(test)]
mod tests {
    use super::extract_codex_server_name;

    #[test]
    fn extracts_plain_codex_mcp_server_name() {
        let parsed = extract_codex_server_name("[mcp_servers.openspec]");
        assert_eq!(parsed.as_deref(), Some("openspec"));
    }

    #[test]
    fn extracts_quoted_codex_mcp_server_name() {
        let parsed = extract_codex_server_name("[mcp_servers.\"Figma\"]");
        assert_eq!(parsed.as_deref(), Some("Figma"));
    }

    #[test]
    fn extracts_top_level_name_from_nested_section() {
        let parsed = extract_codex_server_name("[mcp_servers.openspec.tools.search]");
        assert_eq!(parsed.as_deref(), Some("openspec"));
    }

    #[test]
    fn ignores_non_server_section() {
        let parsed = extract_codex_server_name("[projects.\"/tmp/demo\"]");
        assert!(parsed.is_none());
    }
}
