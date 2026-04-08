use std::path::Path;

pub(super) fn is_safe_binary_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/'))
}

pub(super) fn agent_env_vars(agent: &str) -> Vec<(String, String)> {
    if agent == "opencode" {
        vec![
            (
                "OPENCODE_PERMISSION".to_string(),
                crate::agent_runtime::OPENCODE_PERMISSION_ALLOW_ALL.to_string(),
            ),
            (
                "OPENCODE_CONFIG_CONTENT".to_string(),
                crate::agent_runtime::OPENCODE_MCP_CONFIG_NO_SERVERS.to_string(),
            ),
        ]
    } else {
        Vec::new()
    }
}

pub(super) fn build_agent_args(
    agent: &str,
    prompt: &str,
    project_dir: &Path,
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
                "--dangerously-skip-permissions".to_string(),
                "--strict-mcp-config".to_string(),
                "--mcp-config".to_string(),
                crate::agent_runtime::CLAUDE_EMPTY_MCP_CONFIG.to_string(),
                "--tools".to_string(),
                "".to_string(),
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
                project_dir.to_string_lossy().to_string(),
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
            project_dir.to_string_lossy().to_string(),
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
                project_dir.to_string_lossy().to_string(),
                "--sandbox".to_string(),
                "disabled".to_string(),
                "--trust".to_string(),
                prompt.to_string(),
            ]
        }
        _ => vec!["-p".to_string(), prompt.to_string()],
    }
}

pub(super) fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        "''".to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

pub(super) fn build_null_stdin_shell_command(agent_binary: &str, args: &[String]) -> String {
    let mut command_parts = Vec::with_capacity(args.len() + 1);
    command_parts.push(shell_quote(agent_binary));
    for value in args {
        command_parts.push(shell_quote(value));
    }
    format!("{} < /dev/null", command_parts.join(" "))
}
