use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlanEngineError {
    #[error("Shell error: {0}")]
    Shell(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path error: {0}")]
    Path(String),
    #[error("No active plan session for project: {0}")]
    NoSession(String),
    #[error("Plan session lock poisoned")]
    LockPoisoned,
    #[error("Plan already running for project: {0}")]
    AlreadyRunning(String),
}

impl Serialize for PlanEngineError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
