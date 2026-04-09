use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum FixtureSet {
    HappyPath,
    PlanError,
    AtomizeError,
    LoopFailure,
}

#[derive(Debug, Clone)]
pub struct TestRuntime {
    pub fixture_set: FixtureSet,
    pub data_dir: PathBuf,
    pub dialog_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub enum TestMode {
    Disabled,
    Enabled(TestRuntime),
}

impl TestMode {
    pub fn is_enabled(&self) -> bool {
        matches!(self, TestMode::Enabled(_))
    }

    pub fn runtime(&self) -> Option<&TestRuntime> {
        match self {
            TestMode::Enabled(rt) => Some(rt),
            TestMode::Disabled => None,
        }
    }
}

#[derive(Debug)]
pub struct TestConfigError(pub String);

impl std::fmt::Display for TestConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "test config error: {}", self.0)
    }
}

pub fn resolve_test_mode() -> Result<TestMode, TestConfigError> {
    let enabled = std::env::var("LOOPFORGE_TEST_MODE")
        .map(|val| val == "1" || val.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if !enabled {
        return Ok(TestMode::Disabled);
    }

    let fixture_set = parse_fixture_set()?;
    let data_dir = env_path("LOOPFORGE_TEST_DATA_DIR", "test-data")?;
    let dialog_dir = env_path("LOOPFORGE_TEST_DIALOG_DIR", "test-dialogs")?;

    Ok(TestMode::Enabled(TestRuntime {
        fixture_set,
        data_dir,
        dialog_dir,
    }))
}

fn parse_fixture_set() -> Result<FixtureSet, TestConfigError> {
    let raw = std::env::var("LOOPFORGE_TEST_FIXTURE_SET")
        .unwrap_or_else(|_| "happy-path".to_string());

    match raw.as_str() {
        "happy-path" => Ok(FixtureSet::HappyPath),
        "plan-error" => Ok(FixtureSet::PlanError),
        "atomize-error" => Ok(FixtureSet::AtomizeError),
        "loop-failure" => Ok(FixtureSet::LoopFailure),
        other => Err(TestConfigError(format!(
            "unknown fixture set '{other}'; expected one of: \
             happy-path, plan-error, atomize-error, loop-failure"
        ))),
    }
}

fn env_path(key: &str, fallback: &str) -> Result<PathBuf, TestConfigError> {
    let raw = std::env::var(key).unwrap_or_else(|_| fallback.to_string());
    let path = PathBuf::from(&raw);
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_when_unset() {
        std::env::remove_var("LOOPFORGE_TEST_MODE");
        let mode = resolve_test_mode().unwrap();
        assert!(!mode.is_enabled());
        assert!(mode.runtime().is_none());
    }

    #[test]
    fn rejects_unknown_fixture_set() {
        std::env::set_var("LOOPFORGE_TEST_MODE", "1");
        std::env::set_var("LOOPFORGE_TEST_FIXTURE_SET", "bogus");
        let result = resolve_test_mode();
        assert!(result.is_err());
        std::env::remove_var("LOOPFORGE_TEST_MODE");
        std::env::remove_var("LOOPFORGE_TEST_FIXTURE_SET");
    }
}
