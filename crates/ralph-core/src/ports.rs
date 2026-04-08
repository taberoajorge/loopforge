use crate::config::RalphConfig;
use crate::errors::GuardrailError;
use std::path::Path;

pub trait GitOps: Send + Sync {
    fn get_head_hash(
        &self,
        work_dir: &Path,
    ) -> impl std::future::Future<Output = anyhow::Result<String>> + Send;

    fn count_commits_since(
        &self,
        work_dir: &Path,
        since_hash: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<u32>> + Send;

    fn remove_git_lock(
        &self,
        work_dir: &Path,
    ) -> impl std::future::Future<Output = ()> + Send;

    fn load_last_rebase(&self, path: &Path) -> Option<String>;
}

pub trait HealthChecker: Send + Sync {
    fn check_all_services(
        &self,
        config: &RalphConfig,
    ) -> impl std::future::Future<Output = bool> + Send;
}

pub trait GuardrailStore: Send + Sync {
    fn read_content(&self, path: &Path) -> Result<String, GuardrailError>;

    fn add_guardrail(
        &self,
        path: &Path,
        story_id: &str,
        error_msg: &str,
        iteration: u32,
    ) -> Result<(), GuardrailError>;

    fn ensure_exists(&self, path: &Path) -> Result<(), GuardrailError>;

    fn has_guardrail_for(&self, path: &Path, story_id: &str) -> bool;
}

pub trait StateStore: Send + Sync {
    fn wait_while_paused(
        &self,
        pause_file: &Path,
    ) -> impl std::future::Future<Output = ()> + Send;

    fn check_and_clear_done(&self, done_file: &Path) -> bool;

    fn save_state(&self, state_file: &Path, content: &str);

    fn clear_state(&self, state_file: &Path);
}

pub trait ActivityLogger: Send + Sync {
    fn log_error(&self, message: &str, error_log: Option<&Path>);
    fn log_info(&self, message: &str);
    fn log_success(&self, message: &str);
    fn log_warning(&self, message: &str);
    fn log_activity(&self, message: &str, activity_log: &Path);
    fn log_iteration_header(&self, iteration: u32);
}

pub struct DefaultGitOps;

impl GitOps for DefaultGitOps {
    async fn get_head_hash(&self, work_dir: &Path) -> anyhow::Result<String> {
        crate::git::get_head_hash(work_dir).await
    }

    async fn count_commits_since(&self, work_dir: &Path, since_hash: &str) -> anyhow::Result<u32> {
        crate::git::count_commits_since(work_dir, since_hash).await
    }

    async fn remove_git_lock(&self, work_dir: &Path) {
        crate::git::remove_git_lock(work_dir).await;
    }

    fn load_last_rebase(&self, path: &Path) -> Option<String> {
        crate::git::load_last_rebase(path)
    }
}

pub struct DefaultHealthChecker;

impl HealthChecker for DefaultHealthChecker {
    async fn check_all_services(&self, config: &RalphConfig) -> bool {
        crate::health::check_all_services_parallel(config).await
    }
}

pub struct DefaultGuardrailStore;

impl GuardrailStore for DefaultGuardrailStore {
    fn read_content(&self, path: &Path) -> Result<String, GuardrailError> {
        crate::guardrails::read_content(path)
    }

    fn add_guardrail(
        &self,
        path: &Path,
        story_id: &str,
        error_msg: &str,
        iteration: u32,
    ) -> Result<(), GuardrailError> {
        crate::guardrails::add_guardrail(path, story_id, error_msg, iteration)
    }

    fn ensure_exists(&self, path: &Path) -> Result<(), GuardrailError> {
        crate::guardrails::ensure_exists(path)
    }

    fn has_guardrail_for(&self, path: &Path, story_id: &str) -> bool {
        crate::guardrails::has_guardrail_for(path, story_id)
    }
}

pub struct DefaultStateStore;

impl StateStore for DefaultStateStore {
    async fn wait_while_paused(&self, pause_file: &Path) {
        crate::state::wait_while_paused(pause_file).await;
    }

    fn check_and_clear_done(&self, done_file: &Path) -> bool {
        crate::state::check_and_clear_done(done_file)
    }

    fn save_state(&self, state_file: &Path, content: &str) {
        crate::state::save_state(state_file, content);
    }

    fn clear_state(&self, state_file: &Path) {
        crate::state::clear_state(state_file);
    }
}

pub struct DefaultActivityLogger;

impl ActivityLogger for DefaultActivityLogger {
    fn log_error(&self, message: &str, error_log: Option<&Path>) {
        crate::logger::log_error(message, error_log);
    }

    fn log_info(&self, message: &str) {
        crate::logger::log_info(message);
    }

    fn log_success(&self, message: &str) {
        crate::logger::log_success(message);
    }

    fn log_warning(&self, message: &str) {
        crate::logger::log_warning(message);
    }

    fn log_activity(&self, message: &str, activity_log: &Path) {
        crate::logger::log_activity(message, activity_log);
    }

    fn log_iteration_header(&self, iteration: u32) {
        crate::logger::log_iteration_header(iteration);
    }
}
