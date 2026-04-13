mod helpers;
mod iteration;
mod outcome;
mod prd_lifecycle;
mod rate_limiter;
pub mod scheduler;
pub mod worktree;

pub use crate::scheduler::worktree_runner::{
    provision as provision_worktree, run_in_worktree, ProvisionedWorktree,
};
pub use worktree::{LoopExecutionState, WorktreeExecutionState, PRIMARY_WORKTREE_ID};

use crate::config::RalphConfig;
use crate::detection::failure_memory::{FailureMemory, StoryFailureRecord};
use crate::detection::progress;
use crate::errors::LoopError;
use crate::events::{HeartbeatContext, LoopEvent, LoopEventSink};
use crate::git;
use crate::health;
use crate::logger;
use crate::prompt::PromptBuilder;
use crate::providers::Provider;
use crate::scheduler as story_scheduler;
use crate::scheduler::{ArtifactCoordinator, SharedArtifactUpdate};
use crate::state;
use std::future::{poll_fn, Future};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::Poll;

use self::scheduler::WorktreeCompletion;

type WorkerFuture<'a> = Pin<Box<dyn Future<Output = Result<WorkerReport, LoopError>> + Send + 'a>>;

struct WorkerReport {
    completion: WorktreeCompletion,
    failure_record: Option<StoryFailureRecord>,
}

pub async fn run<P: Provider>(
    config: &RalphConfig,
    provider: &P,
    shutdown_flag: Arc<AtomicBool>,
    event_sink: &dyn LoopEventSink,
) -> Result<(), LoopError> {
    let mut iter_count: u32 = 0;
    let artifact_coordinator = ArtifactCoordinator::for_loop_engine(config.clone());
    let initial_commit = git::get_head_hash(&config.paths.work_dir)
        .await
        .unwrap_or_default();
    let mut failure_memory = FailureMemory::load(&config.paths.failure_memory_file);
    while iter_count < config.tuning.max_iterations {
        if shutdown_flag.load(Ordering::SeqCst) {
            logger::log_warning("Shutdown requested, exiting loop");
            break;
        }
        state::wait_while_paused(&config.paths.pause_file).await;
        if state::check_and_clear_done(&config.paths.done_file) {
            break;
        }
        if !health::check_all_services_parallel(config).await {
            logger::log_error(
                "Services unavailable, retrying in 30s",
                Some(&config.paths.error_log),
            );
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            continue;
        }
        let mut prd = if let Some(prd) = prd_lifecycle::load_or_restore(config) {
            prd
        } else {
            logger::log_error("PRD unrecoverable, stopping", Some(&config.paths.error_log));
            break;
        };
        if prd.pending_count() == 0 {
            logger::log_success(&format!(
                "All stories completed! {} passed, {} blocked.",
                prd.passed_count(),
                prd.blocked_count()
            ));
            break;
        }
        let ready_stories = story_scheduler::ready_story_batch(&prd);
        if ready_stories.is_empty() {
            logger::log_success("No more actionable stories. Done.");
            break;
        }
        iter_count += 1;
        let iter_start = std::time::Instant::now();
        logger::log_iteration_header(iter_count);
        log_progress(&prd, &ready_stories[0].title, config, iter_count);
        if ready_stories.len() == 1 {
            run_single_ready_story(
                config,
                provider,
                &artifact_coordinator,
                shutdown_flag.clone(),
                event_sink,
                &mut prd,
                ready_stories[0].clone(),
                &mut failure_memory,
                iter_count,
                &initial_commit,
                &iter_start,
            )
            .await?;
        } else {
            run_parallel_ready_stories(
                config,
                provider,
                &artifact_coordinator,
                shutdown_flag.clone(),
                event_sink,
                &mut prd,
                ready_stories,
                &mut failure_memory,
                iter_count,
                &initial_commit,
                &iter_start,
            )
            .await?;
        }
        logger::log_info(&format!(
            "Cooldown {}s before next iteration...",
            config.tuning.cooldown_secs
        ));
        if helpers::interruptible_sleep(
            config.tuning.cooldown_secs,
            &shutdown_flag,
            event_sink,
            HeartbeatContext::CooldownWait,
        )
        .await
        {
            logger::log_warning("Shutdown requested during cooldown");
            break;
        }
    }
    helpers::log_session_summary(iter_count, config, &initial_commit).await;
    state::clear_state(&config.paths.state_file);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_single_ready_story<P: Provider>(
    config: &RalphConfig,
    provider: &P,
    artifact_coordinator: &ArtifactCoordinator,
    shutdown_flag: Arc<AtomicBool>,
    event_sink: &dyn LoopEventSink,
    prd: &mut crate::prd::Prd,
    story: crate::prd::UserStory,
    failure_memory: &mut FailureMemory,
    iteration: u32,
    initial_commit: &str,
    iter_start: &std::time::Instant,
) -> Result<(), LoopError> {
    if failure_memory.is_in_gutter(&story.id, config.tuning.gutter_threshold) {
        handle_gutter_skip(
            artifact_coordinator,
            config,
            prd,
            &story.id,
            iteration,
            event_sink,
        )
        .await?;
        return Ok(());
    }
    prd.save(&config.paths.prd_backup)?;
    let progress_before = progress::get_progress_file_size(&config.paths.progress_file);
    let head_before = git::get_head_hash(&config.paths.work_dir)
        .await
        .unwrap_or_default();
    let built = build_prompt(config, iteration, failure_memory, &story, event_sink);
    if let Err(validation_errors) = built.validate(&story.id) {
        tracing::error!(story_id = %story.id, errors = ?validation_errors, "prompt validation failed");
        event_sink.emit(LoopEvent::StorySkipped {
            story_id: story.id.clone(),
            reason: format!("prompt validation failed: {}", validation_errors.join("; ")),
        });
        return Ok(());
    }
    logger::log_info(&format!(
        "Starting {} agent ({})...",
        provider.name(),
        provider.model()
    ));
    let outcome = iteration::run_with_verification(
        config,
        provider,
        &story,
        built.text,
        built.hash,
        &shutdown_flag,
        event_sink,
        failure_memory,
        iteration,
    )
    .await;
    let final_passed = outcome.story_passed || check_story_passed_in_prd(config, &story.id);
    let progress_report = progress::analyze_iteration_progress(
        &config.paths.work_dir,
        &head_before,
        final_passed,
        &config.paths.progress_file,
        progress_before,
    )
    .await;
    let mut consecutive_zero_progress = 0;
    let mut iteration_cursor = iteration;
    outcome::handle(
        config,
        &outcome.agent_result,
        prd,
        &story.id,
        final_passed,
        &progress_report,
        failure_memory,
        &mut consecutive_zero_progress,
        &mut iteration_cursor,
        &shutdown_flag,
        event_sink,
    )
    .await?;
    failure_memory.save(&config.paths.failure_memory_file)?;
    log_iteration_complete(
        config,
        iteration,
        initial_commit,
        &story.id,
        final_passed,
        iter_start,
    )
    .await;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_parallel_ready_stories<P: Provider>(
    config: &RalphConfig,
    provider: &P,
    artifact_coordinator: &ArtifactCoordinator,
    shutdown_flag: Arc<AtomicBool>,
    event_sink: &dyn LoopEventSink,
    prd: &mut crate::prd::Prd,
    ready_stories: Vec<crate::prd::UserStory>,
    failure_memory: &mut FailureMemory,
    iteration: u32,
    initial_commit: &str,
    iter_start: &std::time::Instant,
) -> Result<(), LoopError> {
    prd.save(&config.paths.prd_backup)?;
    let shared_failure_memory = failure_memory.clone();
    let mut workers = Vec::new();
    for story in ready_stories {
        if failure_memory.is_in_gutter(&story.id, config.tuning.gutter_threshold) {
            handle_gutter_skip(
                artifact_coordinator,
                config,
                prd,
                &story.id,
                iteration,
                event_sink,
            )
            .await?;
            continue;
        }
        let provisioned = provision_worktree(&config.paths.work_dir, &story.id)
            .await
            .map_err(|err| LoopError::Other(anyhow::anyhow!(err.to_string())))?;
        workers.push(Box::pin(execute_ready_story(
            config,
            provider,
            shutdown_flag.clone(),
            event_sink,
            story,
            provisioned,
            shared_failure_memory.clone(),
            iteration,
        )) as WorkerFuture<'_>);
    }
    if workers.is_empty() {
        return Ok(());
    }
    let reports = collect_worker_reports(workers).await?;
    for update in story_scheduler::shared_updates_for(
        reports
            .iter()
            .map(|report| report.completion.clone())
            .collect(),
    ) {
        artifact_coordinator
            .submit(update)
            .await
            .map_err(|err| LoopError::Other(anyhow::anyhow!(err.to_string())))?;
    }
    for report in reports {
        merge_failure_record(failure_memory, report.failure_record);
        if let Some(story) = prd
            .stories
            .iter_mut()
            .find(|story| story.id == report.completion.story_id)
        {
            story.passes = report.completion.passed;
            story.blocked = report.completion.blocked;
        }
        log_iteration_complete(
            config,
            iteration,
            initial_commit,
            &report.completion.story_id,
            report.completion.passed,
            iter_start,
        )
        .await;
    }
    failure_memory.save(&config.paths.failure_memory_file)?;
    Ok(())
}

async fn collect_worker_reports(
    mut workers: Vec<WorkerFuture<'_>>,
) -> Result<Vec<WorkerReport>, LoopError> {
    let mut reports = Vec::with_capacity(workers.len());
    poll_fn(|cx| {
        let mut index = 0;
        while index < workers.len() {
            match workers[index].as_mut().poll(cx) {
                Poll::Ready(Ok(report)) => {
                    reports.push(report);
                    let _ = workers.swap_remove(index);
                }
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                Poll::Pending => index += 1,
            }
        }
        if workers.is_empty() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    })
    .await?;
    Ok(reports)
}

#[allow(clippy::too_many_arguments)]
async fn execute_ready_story<P: Provider>(
    config: &RalphConfig,
    provider: &P,
    shutdown_flag: Arc<AtomicBool>,
    event_sink: &dyn LoopEventSink,
    story: crate::prd::UserStory,
    provisioned: ProvisionedWorktree,
    mut failure_memory: FailureMemory,
    iteration: u32,
) -> Result<WorkerReport, LoopError> {
    let worker_config =
        crate::scheduler::worktree_runner::prepare_worker_config(config, &provisioned)
            .map_err(|err| LoopError::Other(anyhow::anyhow!(err.to_string())))?;
    let mut worker_prd =
        prd_lifecycle::load_or_restore(&worker_config).ok_or(LoopError::PrdUnrecoverable)?;
    let guardrail_seed =
        std::fs::read_to_string(&worker_config.paths.guardrails_file).unwrap_or_default();
    let progress_before = progress::get_progress_file_size(&worker_config.paths.progress_file);
    let head_before = git::get_head_hash(&worker_config.paths.work_dir)
        .await
        .unwrap_or_default();
    let built = build_prompt(
        &worker_config,
        iteration,
        &failure_memory,
        &story,
        event_sink,
    );
    if let Err(validation_errors) = built.validate(&story.id) {
        tracing::error!(story_id = %story.id, errors = ?validation_errors, "prompt validation failed");
        event_sink.emit(LoopEvent::StorySkipped {
            story_id: story.id.clone(),
            reason: format!("prompt validation failed: {}", validation_errors.join("; ")),
        });
        return Ok(worker_report(
            &story.id,
            &provisioned.branch,
            &worker_prd,
            None,
            failure_memory.get_record(&story.id).cloned(),
            u64::from(iteration),
        ));
    }
    logger::log_info(&format!(
        "Starting {} agent ({}) in {}...",
        provider.name(),
        provider.model(),
        provisioned.worktree_path.display()
    ));
    let outcome = iteration::run_with_verification(
        &worker_config,
        provider,
        &story,
        built.text,
        built.hash,
        &shutdown_flag,
        event_sink,
        &mut failure_memory,
        iteration,
    )
    .await;
    let final_passed = outcome.story_passed || check_story_passed_in_prd(&worker_config, &story.id);
    let progress_report = progress::analyze_iteration_progress(
        &worker_config.paths.work_dir,
        &head_before,
        final_passed,
        &worker_config.paths.progress_file,
        progress_before,
    )
    .await;
    let mut consecutive_zero_progress = 0;
    let mut iteration_cursor = iteration;
    outcome::handle(
        &worker_config,
        &outcome.agent_result,
        &mut worker_prd,
        &story.id,
        final_passed,
        &progress_report,
        &mut failure_memory,
        &mut consecutive_zero_progress,
        &mut iteration_cursor,
        &shutdown_flag,
        event_sink,
    )
    .await?;
    Ok(worker_report(
        &story.id,
        &provisioned.branch,
        &worker_prd,
        guardrail_append(&guardrail_seed, &worker_config.paths.guardrails_file),
        failure_memory.get_record(&story.id).cloned(),
        u64::from(iteration),
    ))
}

fn worker_report(
    story_id: &str,
    worktree_id: &str,
    prd: &crate::prd::Prd,
    guardrail_append: Option<String>,
    failure_record: Option<StoryFailureRecord>,
    sequence: u64,
) -> WorkerReport {
    let state = prd.stories.iter().find(|story| story.id == story_id);
    WorkerReport {
        completion: WorktreeCompletion {
            worktree_id: worktree_id.to_string(),
            story_id: story_id.to_string(),
            passed: state.is_some_and(|story| story.passes),
            blocked: state.is_some_and(|story| story.blocked),
            head_commit: None,
            guardrail_append,
            sequence,
        },
        failure_record,
    }
}

fn merge_failure_record(failure_memory: &mut FailureMemory, record: Option<StoryFailureRecord>) {
    let Some(record) = record else {
        return;
    };
    if let Some(existing) = failure_memory
        .stories
        .iter_mut()
        .find(|existing| existing.story_id == record.story_id)
    {
        *existing = record;
    } else {
        failure_memory.stories.push(record);
    }
}

fn guardrail_append(seed: &str, path: &std::path::Path) -> Option<String> {
    let current = std::fs::read_to_string(path).ok()?;
    let appended = current
        .strip_prefix(seed)
        .unwrap_or(current.as_str())
        .trim();
    if appended.is_empty() {
        None
    } else {
        Some(format!("\n{appended}\n"))
    }
}

fn log_progress(prd: &crate::prd::Prd, story_title: &str, config: &RalphConfig, iteration: u32) {
    let total = prd.total_stories();
    let passed = prd.passed_count();
    let blocked = prd.blocked_count();
    let pending = prd.pending_count();
    tracing::info!(passed, total, pending, blocked, "Progress");
    tracing::info!(story = %story_title, "Current story");
    logger::log_activity(
        &format!("=== Iteration {iteration} started ==="),
        &config.paths.activity_log,
    );
}

async fn handle_gutter_skip(
    artifact_coordinator: &ArtifactCoordinator,
    config: &RalphConfig,
    prd: &mut crate::prd::Prd,
    story_id: &str,
    iteration: u32,
    event_sink: &dyn LoopEventSink,
) -> Result<(), LoopError> {
    logger::log_warning(&format!(
        "GUTTER: {story_id} has failed {}+ times, skipping",
        config.tuning.gutter_threshold
    ));
    event_sink.emit(LoopEvent::StorySkipped {
        story_id: story_id.to_string(),
        reason: format!(
            "gutter threshold ({} failures)",
            config.tuning.gutter_threshold
        ),
    });
    if let Err(err) = artifact_coordinator
        .submit(SharedArtifactUpdate::BlockStoryAndAddGuardrail {
            story_id: story_id.to_string(),
            error_message: "Exceeded gutter threshold".to_string(),
            iteration,
        })
        .await
    {
        tracing::warn!(error = %err, "failed to update shared artifacts");
    } else {
        prd.stories
            .iter_mut()
            .find(|story| story.id == story_id)
            .map(|story| story.blocked = true);
    }
    logger::log_activity(
        &format!("Story {story_id} skipped (gutter threshold)"),
        &config.paths.activity_log,
    );
    Ok(())
}

fn build_prompt(
    config: &RalphConfig,
    iteration: u32,
    failure_memory: &FailureMemory,
    story: &crate::prd::UserStory,
    event_sink: &dyn LoopEventSink,
) -> crate::prompt::BuiltPrompt {
    let builder = PromptBuilder::new(
        &config.paths.prompt_file,
        &config.paths.guardrails_file,
        &config.paths.ralph_dir,
        &config.paths.work_dir,
        iteration,
        failure_memory,
    );
    let built = builder.build(story, None);
    event_sink.emit(LoopEvent::PromptBuilt {
        story_id: story.id.clone(),
        size_bytes: built.size_bytes,
        hash: built.hash,
        truncated: built.truncated,
    });
    tracing::debug!(
        size = built.size_bytes,
        hash = built.hash,
        truncated = built.truncated,
        "prompt built"
    );
    built
}

fn check_story_passed_in_prd(config: &RalphConfig, story_id: &str) -> bool {
    prd_lifecycle::load_or_restore(config)
        .and_then(|prd| {
            prd.stories
                .iter()
                .find(|st| st.id == story_id)
                .map(|st| st.passes)
        })
        .unwrap_or(false)
}

async fn log_iteration_complete(
    config: &RalphConfig,
    iteration: u32,
    initial_commit: &str,
    story_id: &str,
    passed: bool,
    start: &std::time::Instant,
) {
    let elapsed = start.elapsed().as_secs();
    let total_commits = git::count_commits_since(&config.paths.work_dir, initial_commit)
        .await
        .unwrap_or(0);
    logger::log_activity(
        &format!(
            "Iteration {iteration} completed in {} | Commits: {total_commits} | Story: {story_id} | Passed: {passed}",
            logger::format_duration(elapsed)
        ),
        &config.paths.activity_log,
    );
    tracing::info!(
        duration = %logger::format_duration(elapsed),
        commits = total_commits,
        "Iteration complete"
    );
}
