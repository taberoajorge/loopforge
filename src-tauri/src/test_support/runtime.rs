use std::path::{Path, PathBuf};

use thiserror::Error;

const TEST_MODE_ENV: &str = "LOOPFORGE_TEST_MODE";
const FIXTURE_SET_ENV: &str = "LOOPFORGE_TEST_FIXTURE_SET";
const DATA_DIR_ENV: &str = "LOOPFORGE_TEST_DATA_DIR";
const DIALOG_DIR_ENV: &str = "LOOPFORGE_TEST_DIALOG_DIR";
const DEFAULT_FIXTURE_SET: &str = "happy-path";
const DEFAULT_DATA_DIR: &str = "test-data";
const DEFAULT_DIALOG_DIR: &str = "test-dialogs";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FixtureSet {
    HappyPath,
    PlanError,
    AtomizeError,
    LoopFailure,
}

impl FixtureSet {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HappyPath => "happy-path",
            Self::PlanError => "plan-error",
            Self::AtomizeError => "atomize-error",
            Self::LoopFailure => "loop-failure",
        }
    }

    fn parse(raw: &str) -> Result<Self, TestConfigError> {
        match raw {
            "happy-path" => Ok(Self::HappyPath),
            "plan-error" => Ok(Self::PlanError),
            "atomize-error" => Ok(Self::AtomizeError),
            "loop-failure" => Ok(Self::LoopFailure),
            other => Err(TestConfigError::UnknownFixtureSet(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TestRuntime {
    fixture_set: FixtureSet,
    data_dir: PathBuf,
    dialog_dir: PathBuf,
}

impl TestRuntime {
    pub fn fixture_set(&self) -> FixtureSet {
        self.fixture_set
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn dialog_dir(&self) -> &Path {
        &self.dialog_dir
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TestMode {
    Disabled,
    Enabled(TestRuntime),
}

impl TestMode {
    pub fn runtime(&self) -> Option<&TestRuntime> {
        match self {
            Self::Enabled(runtime) => Some(runtime),
            Self::Disabled => None,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub(crate) enum TestConfigError {
    #[error(
        "test config error: invalid LOOPFORGE_TEST_MODE '{0}'; expected one of: 1, true, 0, false"
    )]
    InvalidTestMode(String),
    #[error(
        "test config error: unknown fixture set '{0}'; expected one of: happy-path, plan-error, atomize-error, loop-failure"
    )]
    UnknownFixtureSet(String),
    #[error("test config error: {0} is not valid UTF-8")]
    NonUnicode(&'static str),
}

pub(crate) fn resolve_test_mode() -> Result<TestMode, TestConfigError> {
    let Some(mode_value) = read_env(TEST_MODE_ENV)? else {
        return Ok(TestMode::Disabled);
    };

    if matches_false(&mode_value) {
        return Ok(TestMode::Disabled);
    }

    if !matches_true(&mode_value) {
        return Err(TestConfigError::InvalidTestMode(mode_value));
    }

    Ok(TestMode::Enabled(TestRuntime {
        fixture_set: parse_fixture_set()?,
        data_dir: env_path(DATA_DIR_ENV, DEFAULT_DATA_DIR)?,
        dialog_dir: env_path(DIALOG_DIR_ENV, DEFAULT_DIALOG_DIR)?,
    }))
}

fn parse_fixture_set() -> Result<FixtureSet, TestConfigError> {
    let raw = read_env(FIXTURE_SET_ENV)?.unwrap_or_else(|| DEFAULT_FIXTURE_SET.to_string());
    FixtureSet::parse(&raw)
}

fn env_path(key: &'static str, fallback: &str) -> Result<PathBuf, TestConfigError> {
    let raw = read_env(key)?.unwrap_or_else(|| fallback.to_string());
    Ok(PathBuf::from(raw))
}

fn read_env(key: &'static str) -> Result<Option<String>, TestConfigError> {
    match std::env::var(key) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(TestConfigError::NonUnicode(key)),
    }
}

fn matches_true(value: &str) -> bool {
    value == "1" || value.eq_ignore_ascii_case("true")
}

fn matches_false(value: &str) -> bool {
    value == "0" || value.eq_ignore_ascii_case("false")
}
