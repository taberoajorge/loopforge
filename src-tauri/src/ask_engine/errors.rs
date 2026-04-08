use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AskEngineError {
    #[error("Shell error: {0}")]
    Shell(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path error: {0}")]
    Path(String),
    #[error("Database error: {0}")]
    Db(String),
    #[error("Ask session lock poisoned")]
    LockPoisoned,
    #[error("Ask already running for project: {0}")]
    #[allow(dead_code)]
    AlreadyRunning(String),
    #[error("No active ask session for project: {0}")]
    #[allow(dead_code)]
    NoSession(String),
}

impl Serialize for AskEngineError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
