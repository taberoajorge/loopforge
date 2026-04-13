use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

pub fn ensure_full_path_env() {
    let full_path = probe_path_env();
    if !full_path.is_empty() {
        std::env::set_var("PATH", &full_path);
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
    append_user_tool_paths(&mut ordered_paths);
    append_system_tool_paths(&mut ordered_paths);
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
    if binary.contains('/') || binary.contains('\\') {
        return std::path::Path::new(binary)
            .exists()
            .then_some(binary.to_string());
    }
    for entry in std::env::split_paths(path_env) {
        let candidate = entry.join(binary);
        if candidate.exists() {
            return Some(candidate.to_string_lossy().to_string());
        }
        #[cfg(windows)]
        for extension in [".exe", ".cmd", ".bat"] {
            let candidate_with_extension = entry.join(format!("{binary}{extension}"));
            if candidate_with_extension.exists() {
                return Some(candidate_with_extension.to_string_lossy().to_string());
            }
        }
    }
    None
}

fn append_user_tool_paths(paths: &mut Vec<PathBuf>) {
    #[cfg(windows)]
    {
        if let Some(user_profile) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
            paths.push(user_profile.join("scoop").join("shims"));
            paths.push(
                user_profile
                    .join("AppData")
                    .join("Local")
                    .join("Microsoft")
                    .join("WinGet")
                    .join("Links"),
            );
            paths.push(user_profile.join(".cargo").join("bin"));
            paths.push(user_profile.join(".bun").join("bin"));
        }
        if let Some(app_data) = std::env::var_os("APPDATA").map(PathBuf::from) {
            paths.push(app_data.join("npm"));
        }
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
            paths.push(
                local_app_data
                    .join("Microsoft")
                    .join("WinGet")
                    .join("Links"),
            );
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(home_dir) = std::env::var_os("HOME").map(PathBuf::from) {
            paths.push(home_dir.join(".local/bin"));
            paths.push(home_dir.join(".bun/bin"));
            paths.push(home_dir.join(".cargo/bin"));
            paths.push(home_dir.join(".npm-global/bin"));
            paths.push(home_dir.join(".opencode/bin"));
        }
    }
}

fn append_system_tool_paths(paths: &mut Vec<PathBuf>) {
    #[cfg(windows)]
    {
        if let Some(program_files) = std::env::var_os("ProgramFiles").map(PathBuf::from) {
            paths.push(program_files.join("Git").join("cmd"));
            paths.push(program_files.join("nodejs"));
        }
        if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)").map(PathBuf::from) {
            paths.push(program_files_x86.join("Git").join("cmd"));
            paths.push(program_files_x86.join("nodejs"));
        }
    }
    #[cfg(not(windows))]
    {
        paths.push(PathBuf::from("/opt/homebrew/bin"));
        paths.push(PathBuf::from("/usr/local/bin"));
        paths.push(PathBuf::from("/usr/bin"));
        paths.push(PathBuf::from("/bin"));
    }
}
