use std::path::{Path, PathBuf};
use std::sync::MutexGuard;

const ENV_KEYS: [&str; 6] = [
    "HOME",
    "PATH",
    "XDG_CONFIG_HOME",
    "XDG_DATA_HOME",
    "LOOPFORGE_TEST_MODE",
    "LOOPFORGE_TEST_FIXTURE_SET",
];

pub struct FixtureGuard {
    _lock: MutexGuard<'static, ()>,
    root_dir: PathBuf,
    values: Vec<(&'static str, Option<String>)>,
    pub work_dir: PathBuf,
    pub loop_agent: String,
    pub atomizer_agent: String,
    pub broken_atomizer_agent: String,
}

impl FixtureGuard {
    pub fn new(fixture_set: Option<&str>) -> Self {
        let lock = crate::test_env_lock::ENV_LOCK
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let root_dir =
            std::env::temp_dir().join(format!("loopforge-invoke-{}", uuid::Uuid::new_v4()));
        let home_dir = root_dir.join("home");
        let bin_dir = root_dir.join("bin");
        let work_dir = root_dir.join("work");
        std::fs::create_dir_all(&home_dir).expect("home dir");
        std::fs::create_dir_all(&bin_dir).expect("bin dir");
        std::fs::create_dir_all(&work_dir).expect("work dir");

        let loop_agent = "fixture-loop-agent".to_string();
        let atomizer_agent = "fixture-atomizer-agent".to_string();
        let broken_atomizer_agent = "fixture-broken-atomizer-agent".to_string();
        install_loop_agent(&bin_dir, &loop_agent);
        install_atomizer_agent(&bin_dir, &atomizer_agent);
        install_broken_atomizer_agent(&bin_dir, &broken_atomizer_agent);

        let values = ENV_KEYS
            .into_iter()
            .map(|key| (key, std::env::var(key).ok()))
            .collect();
        let path = prepend_path(&bin_dir);
        std::env::set_var("HOME", &home_dir);
        std::env::set_var("XDG_CONFIG_HOME", home_dir.join(".config"));
        std::env::set_var("XDG_DATA_HOME", home_dir.join(".local").join("share"));
        std::env::set_var("PATH", path);
        match fixture_set {
            Some(name) => {
                std::env::set_var("LOOPFORGE_TEST_MODE", "1");
                std::env::set_var("LOOPFORGE_TEST_FIXTURE_SET", name);
            }
            None => {
                std::env::remove_var("LOOPFORGE_TEST_MODE");
                std::env::remove_var("LOOPFORGE_TEST_FIXTURE_SET");
            }
        }

        Self {
            _lock: lock,
            root_dir,
            values,
            work_dir,
            loop_agent,
            atomizer_agent,
            broken_atomizer_agent,
        }
    }
}

impl Drop for FixtureGuard {
    fn drop(&mut self) {
        for (key, value) in self.values.drain(..) {
            match value {
                Some(previous) => std::env::set_var(key, previous),
                None => std::env::remove_var(key),
            }
        }
        let _ = std::fs::remove_dir_all(&self.root_dir);
    }
}

fn install_loop_agent(bin_dir: &Path, agent_name: &str) {
    let script = loop_agent_script();
    install_script(bin_dir, agent_name, script);
}

fn install_atomizer_agent(bin_dir: &Path, agent_name: &str) {
    let script = atomizer_agent_script();
    install_script(bin_dir, agent_name, script);
}

fn install_broken_atomizer_agent(bin_dir: &Path, agent_name: &str) {
    let script = broken_atomizer_agent_script();
    install_script(bin_dir, agent_name, script);
}

fn install_script(bin_dir: &Path, name: &str, content: &str) {
    let script = bin_dir.join(script_name(name));
    std::fs::write(&script, content).expect("script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script).expect("metadata").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script, perms).expect("permissions");
    }
}

fn prepend_path(bin_dir: &Path) -> String {
    let mut path_items = vec![bin_dir.to_path_buf()];
    if let Some(existing) = std::env::var_os("PATH") {
        path_items.extend(std::env::split_paths(&existing));
    }
    std::env::join_paths(path_items)
        .ok()
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| bin_dir.display().to_string())
}

#[cfg(windows)]
fn script_name(name: &str) -> String {
    format!("{name}.cmd")
}

#[cfg(not(windows))]
fn script_name(name: &str) -> String {
    name.to_string()
}

#[cfg(windows)]
fn loop_agent_script() -> &'static str {
    "@echo off\r\necho fixture loop completed\r\n"
}

#[cfg(not(windows))]
fn loop_agent_script() -> &'static str {
    "#!/bin/sh\nprintf 'fixture loop completed\\n'\n"
}

#[cfg(windows)]
fn atomizer_agent_script() -> &'static str {
    "@echo off\r\nset prompt=%2\r\necho %prompt% | findstr /C:\"Condense the following implementation plan\" >nul && (echo Create the project, atomize the plan, execute the story, and archive the result.& exit /b 0)\r\necho %prompt% | findstr /C:\"Split the following implementation plan\" >nul && (echo [{\"title\":\"Lifecycle\",\"content\":\"Create the project, atomize the plan, execute the story, and archive the result.\"}]& exit /b 0)\r\necho %prompt% | findstr /C:\"decomposing a plan section into atomic user stories\" >nul && (echo [{\"title\":\"Exercise invoke contracts\",\"description\":\"Cover invoke lifecycle commands.\",\"acceptanceCriteria\":[\"Lifecycle commands succeed\",\"Runtime history is recorded\"],\"scope\":{\"filesToModify\":[\"src-tauri/src/lib.rs\"],\"filesToCreate\":[\"src-tauri/src/contract_tests/invoke_handler.rs\"],\"filesToAvoid\":[]},\"verification\":{\"commands\":[\"cargo test -p loopforge contract_tests\"],\"assertions\":[]},\"commitMessage\":\"test(tauri): cover invoke handler contracts\",\"priority\":\"critical\",\"estimatedComplexity\":\"small\",\"estimatedMinutes\":30,\"dependsOn\":[]}]& exit /b 0)\r\necho %prompt% | findstr /C:\"technical lead finalizing\" >nul && (echo {\"projectName\":\"Invoke Contract\",\"feature\":\"Lifecycle coverage\",\"workingDirectory\":\"\",\"generatedAt\":\"2026-04-09T10:10:00.000Z\",\"stories\":[{\"id\":\"S-001\",\"title\":\"Exercise invoke contracts\",\"description\":\"Cover invoke lifecycle commands.\",\"acceptanceCriteria\":[\"Lifecycle commands succeed\",\"Runtime history is recorded\"],\"scope\":{\"filesToModify\":[\"src-tauri/src/lib.rs\"],\"filesToCreate\":[\"src-tauri/src/contract_tests/invoke_handler.rs\"],\"filesToAvoid\":[]},\"verification\":{\"commands\":[\"cargo test -p loopforge contract_tests\"],\"assertions\":[]},\"commitMessage\":\"test(tauri): cover invoke handler contracts\",\"priority\":\"critical\",\"estimatedComplexity\":\"small\",\"estimatedMinutes\":30,\"dependsOn\":[],\"passes\":false,\"blocked\":false,\"attempts\":0,\"notes\":null}]}& exit /b 0)\r\necho []\r\n"
}

#[cfg(not(windows))]
fn atomizer_agent_script() -> &'static str {
    r#"#!/bin/sh
prompt="$2"
if printf '%s' "$prompt" | grep -q "Condense the following implementation plan"; then
  printf '%s\n' 'Create the project, atomize the plan, execute the story, and archive the result.'
elif printf '%s' "$prompt" | grep -q "Split the following implementation plan"; then
  printf '%s\n' '[{"title":"Lifecycle","content":"Create the project, atomize the plan, execute the story, and archive the result."}]'
elif printf '%s' "$prompt" | grep -q "decomposing a plan section into atomic user stories"; then
  printf '%s\n' '[{"title":"Exercise invoke contracts","description":"Cover invoke lifecycle commands.","acceptanceCriteria":["Lifecycle commands succeed","Runtime history is recorded"],"scope":{"filesToModify":["src-tauri/src/lib.rs"],"filesToCreate":["src-tauri/src/contract_tests/invoke_handler.rs"],"filesToAvoid":[]},"verification":{"commands":["cargo test -p loopforge contract_tests"],"assertions":[]},"commitMessage":"test(tauri): cover invoke handler contracts","priority":"critical","estimatedComplexity":"small","estimatedMinutes":30,"dependsOn":[]}]'
elif printf '%s' "$prompt" | grep -q "technical lead finalizing"; then
  printf '%s\n' '{"projectName":"Invoke Contract","feature":"Lifecycle coverage","workingDirectory":"","generatedAt":"2026-04-09T10:10:00.000Z","stories":[{"id":"S-001","title":"Exercise invoke contracts","description":"Cover invoke lifecycle commands.","acceptanceCriteria":["Lifecycle commands succeed","Runtime history is recorded"],"scope":{"filesToModify":["src-tauri/src/lib.rs"],"filesToCreate":["src-tauri/src/contract_tests/invoke_handler.rs"],"filesToAvoid":[]},"verification":{"commands":["cargo test -p loopforge contract_tests"],"assertions":[]},"commitMessage":"test(tauri): cover invoke handler contracts","priority":"critical","estimatedComplexity":"small","estimatedMinutes":30,"dependsOn":[],"passes":false,"blocked":false,"attempts":0,"notes":null}]}'
else
  printf '%s\n' '[]'
fi
"#
}

#[cfg(windows)]
fn broken_atomizer_agent_script() -> &'static str {
    "@echo off\r\necho not valid json\r\n"
}

#[cfg(not(windows))]
fn broken_atomizer_agent_script() -> &'static str {
    "#!/bin/sh\nprintf '%s\\n' 'not valid json'\n"
}
