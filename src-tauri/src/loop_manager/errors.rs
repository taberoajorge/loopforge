use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoopError {
    #[error("Loop already running for project: {0}")]
    AlreadyRunning(String),
    #[error("Database error: {0}")]
    Db(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path error: {0}")]
    Path(String),
    #[error("Lock poisoned")]
    LockPoisoned,
}

impl Serialize for LoopError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<rusqlite::Error> for LoopError {
    fn from(err: rusqlite::Error) -> Self {
        LoopError::Db(err.to_string())
    }
}
