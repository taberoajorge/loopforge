use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Project not found: {0}")]
    ProjectNotFound(Uuid),

    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),

    #[error("Artifact error for '{path}': {message}")]
    ArtifactError { path: PathBuf, message: String },

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Service error in '{service}': {message}")]
    ServiceError { service: String, message: String },
}

impl AppError {
    pub fn artifact(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::ArtifactError {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn storage(message: impl Into<String>) -> Self {
        Self::StorageError(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError(message.into())
    }

    pub fn service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ServiceError {
            service: service.into(),
            message: message.into(),
        }
    }
}
