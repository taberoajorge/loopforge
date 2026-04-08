use std::path::Path;
use std::path::PathBuf;

pub(super) fn agent_env_vars(agent: &str) -> Vec<(String, String)> {
    let mut env_vars = vec![("TERM".to_string(), "xterm-256color".to_string())];
    if agent == "opencode" {
        env_vars.push((
            "OPENCODE_PERMISSION".to_string(),
            crate::agent_runtime::OPENCODE_PERMISSION_ALLOW_ALL.to_string(),
        ));
        env_vars.push((
            "OPENCODE_CONFIG_CONTENT".to_string(),
            crate::agent_runtime::OPENCODE_MCP_CONFIG_NO_SERVERS.to_string(),
        ));
    }
    env_vars
}

pub(super) fn agent_cli_args(
    agent: &str,
    prompt: &str,
    work_dir: &Path,
    model: Option<&str>,
    effort: Option<&str>,
) -> Vec<String> {
    let selected_model = model
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    let selected_effort = effort
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    match agent {
        "claude" => {
            let mut args = vec![
                "-p".to_string(),
                prompt.to_string(),
                "--output-format".to_string(),
                "text".to_string(),
                "--verbose".to_string(),
                "--dangerously-skip-permissions".to_string(),
                "--strict-mcp-config".to_string(),
                "--mcp-config".to_string(),
                crate::agent_runtime::CLAUDE_EMPTY_MCP_CONFIG.to_string(),
            ];
            if let Some(model_id) = selected_model {
                args.extend(["--model".to_string(), model_id]);
            }
            if let Some(effort_level) = selected_effort {
                args.extend(["--effort".to_string(), effort_level]);
            }
            args
        }
        "codex" => {
            let mut args = vec![
                "exec".to_string(),
                "--skip-git-repo-check".to_string(),
                "--dangerously-bypass-approvals-and-sandbox".to_string(),
            ];
            if let Some(model_id) = selected_model {
                args.extend(["--model".to_string(), model_id]);
            }
            if let Some(effort_level) = selected_effort {
                args.extend([
                    "-c".to_string(),
                    format!("model_reasoning_effort={effort_level}"),
                ]);
            }
            args.extend(crate::agent_runtime::codex_mcp_disable_args());
            args.extend([
                "-C".to_string(),
                work_dir.to_string_lossy().to_string(),
                prompt.to_string(),
            ]);
            args
        }
        "gemini" => {
            let mut args = vec![
                "-p".to_string(),
                prompt.to_string(),
                "--yolo".to_string(),
                "--allowed-mcp-server-names".to_string(),
                crate::agent_runtime::GEMINI_MCP_ALLOWLIST_NONE.to_string(),
            ];
            if let Some(model_id) = selected_model {
                args.extend(["--model".to_string(), model_id]);
            }
            args
        }
        "opencode" => vec![
            "run".to_string(),
            "--print-logs".to_string(),
            "--dir".to_string(),
            work_dir.to_string_lossy().to_string(),
            prompt.to_string(),
        ],
        "cursor" => {
            let model_id = selected_model.unwrap_or_else(crate::agent_runtime::cursor_model_arg);
            vec![
                "agent".to_string(),
                "--print".to_string(),
                "--force".to_string(),
                "--output-format".to_string(),
                "text".to_string(),
                "--model".to_string(),
                model_id,
                "--workspace".to_string(),
                work_dir.to_string_lossy().to_string(),
                "--sandbox".to_string(),
                "disabled".to_string(),
                "--trust".to_string(),
                prompt.to_string(),
            ]
        }
        _ => vec!["-p".to_string(), prompt.to_string()],
    }
}

pub(super) fn resolve_agent_binary_path(agent: &str) -> Option<PathBuf> {
    let binary = crate::agent_runtime::cli_binary_name(agent);
    if binary.contains('/') {
        let path = PathBuf::from(binary);
        return path.exists().then_some(path);
    }

    let path_var = std::env::var_os("PATH")?;
    for entry in std::env::split_paths(&path_var) {
        let candidate = entry.join(binary);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}
