fn shell_quote_unix(value: &str) -> String {
    if value.is_empty() {
        "''".to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

#[cfg(windows)]
fn shell_quote_windows(value: &str) -> String {
    if value.is_empty() {
        "\"\"".to_string()
    } else {
        format!("\"{}\"", value.replace('"', "\"\""))
    }
}

fn resolved_path_env() -> String {
    let path = crate::agent_runtime_env::probe_path_env();
    if path.is_empty() {
        std::env::var("PATH").unwrap_or_default()
    } else {
        path
    }
}

pub fn resolve_shell() -> (String, Vec<String>) {
    #[cfg(windows)]
    {
        ("cmd.exe".to_string(), vec!["/C".to_string()])
    }
    #[cfg(not(windows))]
    {
        let path_env = resolved_path_env();
        if let Ok(shell) = std::env::var("SHELL") {
            if !shell.trim().is_empty() && std::path::Path::new(&shell).exists() {
                return (shell, vec!["-lc".to_string()]);
            }
        }
        for candidate in ["zsh", "sh", "bash"] {
            if crate::agent_runtime_env::resolve_binary_path(candidate, &path_env).is_some() {
                return (candidate.to_string(), vec!["-lc".to_string()]);
            }
        }
        ("sh".to_string(), vec!["-lc".to_string()])
    }
}

pub fn resolve_binary_via_shell(binary: &str) -> (String, Vec<String>) {
    #[cfg(windows)]
    {
        (
            "cmd.exe".to_string(),
            vec!["/C".to_string(), format!("where {binary}")],
        )
    }
    #[cfg(not(windows))]
    {
        let (shell, mut shell_args) = resolve_shell();
        shell_args.push(format!("command -v {binary}"));
        (shell, shell_args)
    }
}

pub fn build_null_stdin_command(binary: &str, args: &[String]) -> (String, Vec<String>) {
    #[cfg(windows)]
    {
        let mut parts = Vec::with_capacity(args.len() + 1);
        parts.push(shell_quote_windows(binary));
        for value in args {
            parts.push(shell_quote_windows(value));
        }
        (
            "cmd.exe".to_string(),
            vec!["/C".to_string(), format!("{} < NUL", parts.join(" "))],
        )
    }
    #[cfg(not(windows))]
    {
        let mut parts = Vec::with_capacity(args.len() + 1);
        parts.push(shell_quote_unix(binary));
        for value in args {
            parts.push(shell_quote_unix(value));
        }
        let (shell, mut shell_args) = resolve_shell();
        shell_args.push(format!("{} < /dev/null", parts.join(" ")));
        (shell, shell_args)
    }
}

pub fn is_shell_exec_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed == "exec" || trimmed.starts_with("exec ") {
        return true;
    }
    if trimmed.contains(" -lc") && (trimmed.contains("zsh") || trimmed.contains("/sh")) {
        return true;
    }
    trimmed.contains(" /C") && (trimmed.contains("cmd.exe") || trimmed.starts_with("cmd "))
}
