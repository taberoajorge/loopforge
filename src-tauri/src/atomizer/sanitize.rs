pub(super) fn is_stdin_wait_warning_line(line: &str) -> bool {
    line.starts_with("Warning: no stdin data received")
        || line.starts_with("If piping from a slow command, redirect stdin explicitly:")
}

pub(super) fn sanitize_codex_plan_content(raw: &str) -> String {
    let filtered_raw = raw
        .lines()
        .filter(|line| !is_stdin_wait_warning_line(line.trim()))
        .collect::<Vec<_>>()
        .join("\n");

    if filtered_raw.trim().is_empty() {
        return String::new();
    }

    let codex_style_output = filtered_raw.contains("workdir:")
        && filtered_raw.contains("model:")
        && filtered_raw.contains("reasoning effort:");
    if !codex_style_output {
        return filtered_raw.trim().to_string();
    }

    let mut cleaned_lines: Vec<String> = Vec::new();
    let mut in_exec_output = false;
    let mut previous_blank = true;

    for line in filtered_raw.lines() {
        let trimmed = line.trim();
        if is_stdin_wait_warning_line(trimmed) {
            continue;
        }
        if trimmed.is_empty() {
            if !previous_blank {
                cleaned_lines.push(String::new());
                previous_blank = true;
            }
            continue;
        }
        previous_blank = false;

        if crate::shell_resolve::is_shell_exec_line(trimmed) {
            in_exec_output = true;
            continue;
        }

        if (trimmed.starts_with("succeeded in ") || trimmed.starts_with("failed in "))
            && trimmed.contains("ms")
        {
            in_exec_output = true;
            continue;
        }

        if trimmed == "codex" || trimmed.starts_with("codex ") || trimmed.starts_with("Plan update")
        {
            in_exec_output = false;
            if trimmed == "codex" {
                continue;
            }
        }

        if trimmed.starts_with("OpenAI Codex")
            || trimmed.starts_with("workdir:")
            || trimmed.starts_with("model:")
            || trimmed.starts_with("provider:")
            || trimmed.starts_with("approval:")
            || trimmed.starts_with("sandbox:")
            || trimmed.starts_with("reasoning effort:")
            || trimmed.starts_with("reasoning summaries:")
            || trimmed.starts_with("session id:")
            || trimmed == "--------"
            || trimmed == "---"
            || trimmed == "user"
            || trimmed.starts_with("user ")
            || trimmed.starts_with("ERROR rmcp::transport")
            || trimmed.contains("AuthRequired(")
            || trimmed.contains("JsonRpcMessage")
        {
            continue;
        }

        if in_exec_output {
            continue;
        }

        cleaned_lines.push(line.to_string());
    }

    let cleaned = cleaned_lines.join("\n").trim().to_string();
    if cleaned.is_empty() {
        filtered_raw.trim().to_string()
    } else {
        cleaned
    }
}
