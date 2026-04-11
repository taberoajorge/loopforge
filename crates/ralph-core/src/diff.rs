use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnifiedDiffRequest {
    pub repository_path: PathBuf,
    pub base_ref: Option<String>,
    pub head_ref: Option<String>,
    pub context_lines: u16,
}

impl UnifiedDiffRequest {
    pub fn working_tree(repository_path: impl Into<PathBuf>) -> Self {
        Self {
            repository_path: repository_path.into(),
            base_ref: None,
            head_ref: None,
            context_lines: 3,
        }
    }

    pub fn between_refs(
        repository_path: impl Into<PathBuf>,
        base_ref: impl Into<String>,
        head_ref: impl Into<String>,
    ) -> Self {
        Self {
            repository_path: repository_path.into(),
            base_ref: Some(base_ref.into()),
            head_ref: Some(head_ref.into()),
            context_lines: 3,
        }
    }

    pub fn with_context_lines(mut self, context_lines: u16) -> Self {
        self.context_lines = context_lines;
        self
    }
}

#[derive(Debug, Error)]
pub enum DiffError {
    #[error("repository path does not exist: {path}")]
    RepositoryPathMissing { path: PathBuf },
    #[error("path is not a git repository: {path}")]
    NotGitRepository { path: PathBuf },
    #[error("head ref requires base ref")]
    InvalidRevisionRange,
    #[error("context lines must be greater than zero")]
    InvalidContextLines,
    #[error("failed to run git diff in {path}")]
    CommandFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("git diff failed in {path}: {stderr}")]
    GitDiffFailed { path: PathBuf, stderr: String },
}

pub fn unified_diff(request: &UnifiedDiffRequest) -> Result<String, DiffError> {
    validate_request(request)?;
    let mut command = Command::new("git");
    command
        .arg("diff")
        .arg("--no-color")
        .arg(format!("--unified={}", request.context_lines))
        .current_dir(&request.repository_path);

    if let Some(base_ref) = request.base_ref.as_deref() {
        command.arg(base_ref);
        if let Some(head_ref) = request.head_ref.as_deref() {
            command.arg(head_ref);
        }
    }

    let output = command.output().map_err(|source| DiffError::CommandFailed {
        path: request.repository_path.clone(),
        source,
    })?;

    if !output.status.success() {
        return Err(DiffError::GitDiffFailed {
            path: request.repository_path.clone(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn validate_request(request: &UnifiedDiffRequest) -> Result<(), DiffError> {
    if !request.repository_path.exists() {
        return Err(DiffError::RepositoryPathMissing {
            path: request.repository_path.clone(),
        });
    }
    let git_dir = request.repository_path.join(".git");
    if !git_dir.exists() {
        return Err(DiffError::NotGitRepository {
            path: request.repository_path.clone(),
        });
    }
    if request.base_ref.is_none() && request.head_ref.is_some() {
        return Err(DiffError::InvalidRevisionRange);
    }
    if request.context_lines == 0 {
        return Err(DiffError::InvalidContextLines);
    }
    Ok(())
}
