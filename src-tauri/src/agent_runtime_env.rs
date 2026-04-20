use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

pub fn ensure_full_path_env() {
    let full_path = probe_path_env();
    if !full_path.is_empty() {
        std::env::set_var("PATH", full_path);
    }
}

pub fn run_version_probe(binary_path: &str, path_env: &str, version_flag: &str) -> Option<String> {
    let output = Command::new(binary_path)
        .arg(version_flag)
        .env("PATH", path_env)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let line = stdout
        .lines()
        .find(|entry| !entry.trim().is_empty())
        .or_else(|| stderr.lines().find(|entry| !entry.trim().is_empty()))
        .map(|entry| entry.trim().to_string());
    line.filter(|entry| !entry.is_empty())
}

pub fn probe_path_env() -> String {
    let current_path = std::env::var_os("PATH")
        .map(PathBuf::from)
        .map(|value| std::env::split_paths(&value).collect::<Vec<PathBuf>>())
        .unwrap_or_default();
    let mut ordered_paths = current_path;
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        ordered_paths.push(home.join(".local/bin"));
        ordered_paths.push(home.join(".bun/bin"));
        ordered_paths.push(home.join(".cargo/bin"));
        ordered_paths.push(home.join(".npm-global/bin"));
        ordered_paths.push(home.join(".opencode/bin"));
    }
    ordered_paths.push(PathBuf::from("/opt/homebrew/bin"));
    ordered_paths.push(PathBuf::from("/usr/local/bin"));
    ordered_paths.push(PathBuf::from("/usr/bin"));
    ordered_paths.push(PathBuf::from("/bin"));
    let mut seen = HashSet::new();
    let unique_paths: Vec<PathBuf> = ordered_paths
        .into_iter()
        .filter(|path| path.exists())
        .filter(|path| seen.insert(path.clone()))
        .collect();
    std::env::join_paths(unique_paths)
        .ok()
        .and_then(|joined| joined.into_string().ok())
        .unwrap_or_default()
}

pub fn resolve_binary_path(binary: &str, path_env: &str) -> Option<String> {
    if binary.contains('/') {
        return std::path::Path::new(binary)
            .exists()
            .then_some(binary.to_string());
    }
    for entry in std::env::split_paths(path_env) {
        let candidate = entry.join(binary);
        if candidate.exists() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}
