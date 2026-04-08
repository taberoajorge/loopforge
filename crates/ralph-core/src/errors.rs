use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config at {path}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse TOML config at {path}")]
    ParseFailed {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
}

#[derive(Error, Debug)]
pub enum PrdError {
    #[error("failed to read PRD at {path}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse PRD JSON at {path}")]
    ParseFailed {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to serialize PRD")]
    SerializeFailed(#[from] serde_json::Error),
    #[error("failed to write PRD at {path}")]
    WriteFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("validation failed: {reason}")]
    ValidationFailed { reason: String },
}

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("failed to spawn agent process '{command}'")]
    SpawnFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("agent '{provider}' stalled after {timeout_secs}s and was killed")]
    StallKilled { provider: String, timeout_secs: u64 },
    #[error("agent '{provider}' exited with code {exit_code}")]
    NonZeroExit { provider: String, exit_code: i32 },
    #[error("rate limited by '{provider}', retry after: {retry_hint}")]
    RateLimited {
        provider: String,
        retry_hint: String,
    },
}

#[derive(Error, Debug)]
pub enum LoopError {
    #[error("PRD is unrecoverable: missing and no backup available")]
    PrdUnrecoverable,
    #[error("shutdown requested during loop execution")]
    ShutdownRequested,
    #[error(transparent)]
    Prd(#[from] PrdError),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("filesystem operation failed")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum GuardrailError {
    #[error("failed to create guardrails file at {path}")]
    CreateFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read guardrails at {path}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to append guardrail to {path}")]
    AppendFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
