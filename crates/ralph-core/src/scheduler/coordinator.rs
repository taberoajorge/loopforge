use crate::config::RalphConfig;
use crate::guardrails;
use crate::prd::Prd;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharedArtifactUpdate {
    MarkStoryPassed {
        story_id: String,
    },
    MarkStoryBlocked {
        story_id: String,
    },
    AppendGuardrail {
        story_id: String,
        error_message: String,
        iteration: u32,
    },
    BlockStoryAndAddGuardrail {
        story_id: String,
        error_message: String,
        iteration: u32,
    },
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct CoordinatorError {
    message: String,
}

impl CoordinatorError {
    pub fn apply(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Clone)]
pub struct ArtifactCoordinator {
    applier: Arc<dyn Fn(SharedArtifactUpdate) -> Result<(), CoordinatorError> + Send + Sync>,
    gate: Arc<Mutex<()>>,
}

impl ArtifactCoordinator {
    pub fn new<F>(applier: F) -> Self
    where
        F: Fn(SharedArtifactUpdate) -> Result<(), CoordinatorError> + Send + Sync + 'static,
    {
        Self {
            applier: Arc::new(applier),
            gate: Arc::new(Mutex::new(())),
        }
    }

    pub fn for_loop_engine(config: RalphConfig) -> Self {
        Self::new(move |update| apply_update(&config, update))
    }

    pub async fn submit(&self, update: SharedArtifactUpdate) -> Result<(), CoordinatorError> {
        let _guard = self.gate.lock().await;
        let applier = self.applier.clone();
        tokio::task::spawn_blocking(move || (applier)(update))
            .await
            .map_err(|err| CoordinatorError::apply(err.to_string()))?
    }
}

fn apply_update(
    config: &RalphConfig,
    update: SharedArtifactUpdate,
) -> Result<(), CoordinatorError> {
    match update {
        SharedArtifactUpdate::MarkStoryPassed { story_id } => {
            update_story_state(config, &story_id, true, false)
        }
        SharedArtifactUpdate::MarkStoryBlocked { story_id } => {
            update_story_state(config, &story_id, false, true)
        }
        SharedArtifactUpdate::AppendGuardrail {
            story_id,
            error_message,
            iteration,
        } => append_guardrail(config, &story_id, &error_message, iteration),
        SharedArtifactUpdate::BlockStoryAndAddGuardrail {
            story_id,
            error_message,
            iteration,
        } => {
            append_guardrail(config, &story_id, &error_message, iteration)?;
            update_story_state(config, &story_id, false, true)
        }
    }
}

fn update_story_state(
    config: &RalphConfig,
    story_id: &str,
    passed: bool,
    blocked: bool,
) -> Result<(), CoordinatorError> {
    let mut prd = load_or_restore_prd(config)?;
    if let Some(story) = prd.stories.iter_mut().find(|story| story.id == story_id) {
        story.passes = passed;
        story.blocked = blocked;
    }
    prd.save(&config.paths.prd_file)
        .map_err(|err| CoordinatorError::apply(err.to_string()))
}

fn append_guardrail(
    config: &RalphConfig,
    story_id: &str,
    error_message: &str,
    iteration: u32,
) -> Result<(), CoordinatorError> {
    guardrails::add_guardrail(
        &config.paths.guardrails_file,
        story_id,
        error_message,
        iteration,
    )
    .map_err(|err| CoordinatorError::apply(err.to_string()))
}

fn load_or_restore_prd(config: &RalphConfig) -> Result<Prd, CoordinatorError> {
    if Prd::is_valid_json(&config.paths.prd_file) {
        return Prd::load(&config.paths.prd_file)
            .map_err(|err| CoordinatorError::apply(err.to_string()));
    }
    if config.paths.prd_backup.exists() {
        std::fs::copy(&config.paths.prd_backup, &config.paths.prd_file)
            .map_err(|err| CoordinatorError::apply(err.to_string()))?;
        return Prd::load(&config.paths.prd_file)
            .map_err(|err| CoordinatorError::apply(err.to_string()));
    }
    Err(CoordinatorError::apply("PRD missing or corrupted"))
}
