mod helpers;
mod iteration;
mod outcome;
mod prd_lifecycle;
mod rate_limiter;

use crate::config::RalphConfig;
use crate::detection::failure_memory::FailureMemory;
use crate::detection::progress;
use crate::errors::LoopError;
use crate::events::{HeartbeatContext, LoopEvent, LoopEventSink};
use crate::git;
use crate::guardrails;
use crate::health;
use crate::logger;
use crate::prompt::PromptBuilder;
use crate::providers::Provider;
use crate::state;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub async fn run<P: Provider>(
    config: &RalphConfig,
    provider: &P,
    shutdown_flag: Arc<AtomicBool>,
    event_sink: &dyn LoopEventSink,
) -> Result<(), LoopError> {
    let mut iter_count: u32 = 0;
    let initial_commit = git::get_head_hash(&config.paths.work_dir).await.unwrap_or_default();
    let mut failure_memory = FailureMemory::load(&config.paths.failure_memory_file);
    let mut consecutive_zero_progress: u32 = 0;

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
            logger::log_error("Services unavailable, retrying in 30s", Some(&config.paths.error_log));
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            continue;
        }

        let mut prd = match prd_lifecycle::load_or_restore(config) {
            Some(prd) => prd,
            None => {
                logger::log_error("PRD unrecoverable, stopping", Some(&config.paths.error_log));
                break;
            }
        };

        let pending = prd.pending_count();
        if pending == 0 {
            logger::log_success(&format!(
                "All stories completed! {} passed, {} blocked.",
                prd.passed_count(),
                prd.blocked_count()
            ));
            break;
        }

        let next_story = match prd.next_actionable_story() {
            Some(story) => story.clone(),
            None => {
                logger::log_success("No more actionable stories. Done.");
                break;
            }
        };

        iter_count += 1;
        let iter_start = std::time::Instant::now();
        logger::log_iteration_header(iter_count);
        log_progress(&prd, &next_story.title, config, iter_count);

        if failure_memory.is_in_gutter(&next_story.id, config.tuning.gutter_threshold) {
            handle_gutter_skip(config, &mut prd, &next_story.id, iter_count, event_sink)?;
            continue;
        }

        prd.save(&config.paths.prd_backup)?;
        let progress_before = progress::get_progress_file_size(&config.paths.progress_file);
        let head_before = git::get_head_hash(&config.paths.work_dir).await.unwrap_or_default();

        let built = build_prompt(config, iter_count, &failure_memory, &next_story, event_sink);
        if let Err(validation_errors) = built.validate(&next_story.id) {
            tracing::error!(story_id = %next_story.id, errors = ?validation_errors, "prompt validation failed");
            event_sink.emit(LoopEvent::StorySkipped {
                story_id: next_story.id.clone(),
                reason: format!("prompt validation failed: {}", validation_errors.join("; ")),
            });
            continue;
        }

        logger::log_info(&format!("Starting {} agent ({})...", provider.name(), provider.model()));

        let outcome = iteration::run_with_verification(
            config, provider, &next_story, built.text, built.hash,
            &shutdown_flag, event_sink, &mut failure_memory, iter_count,
        )
        .await;

        let final_passed = outcome.story_passed || check_story_passed_in_prd(config, &next_story.id);

        let progress_report = progress::analyze_iteration_progress(
            &config.paths.work_dir, &head_before, final_passed,
            &config.paths.progress_file, progress_before,
        )
        .await;

        outcome::handle(
            config, &outcome.agent_result, &mut prd, &next_story.id,
            final_passed, &progress_report, &mut failure_memory,
            &mut consecutive_zero_progress, &mut iter_count,
            &shutdown_flag, event_sink,
        )
        .await?;

        failure_memory.save(&config.paths.failure_memory_file)?;
        log_iteration_complete(config, iter_count, &initial_commit, &next_story.id, final_passed, &iter_start).await;

        logger::log_info(&format!("Cooldown {}s before next iteration...", config.tuning.cooldown_secs));
        if helpers::interruptible_sleep(config.tuning.cooldown_secs, &shutdown_flag, event_sink, HeartbeatContext::CooldownWait).await {
            logger::log_warning("Shutdown requested during cooldown");
            break;
        }
    }

    helpers::log_session_summary(iter_count, config, &initial_commit).await;
    state::clear_state(&config.paths.state_file);
    Ok(())
}

fn log_progress(prd: &crate::prd::Prd, story_title: &str, config: &RalphConfig, iteration: u32) {
    let total = prd.total_stories();
    let passed = prd.passed_count();
    let blocked = prd.blocked_count();
    let pending = prd.pending_count();
    tracing::info!(passed, total, pending, blocked, "Progress");
    tracing::info!(story = %story_title, "Current story");
    logger::log_activity(&format!("=== Iteration {iteration} started ==="), &config.paths.activity_log);
}

fn handle_gutter_skip(
    config: &RalphConfig,
    prd: &mut crate::prd::Prd,
    story_id: &str,
    iteration: u32,
    event_sink: &dyn LoopEventSink,
) -> Result<(), LoopError> {
    logger::log_warning(&format!("GUTTER: {story_id} has failed {}+ times, skipping", config.tuning.gutter_threshold));
    event_sink.emit(LoopEvent::StorySkipped {
        story_id: story_id.to_string(),
        reason: format!("gutter threshold ({} failures)", config.tuning.gutter_threshold),
    });
    if let Err(err) = guardrails::add_guardrail(&config.paths.guardrails_file, story_id, "Exceeded gutter threshold", iteration) {
        tracing::warn!(error = %err, "failed to write guardrail");
    }
    logger::log_activity(&format!("Story {story_id} skipped (gutter threshold)"), &config.paths.activity_log);
    prd.stories.iter_mut().find(|story| story.id == story_id).map(|story| story.blocked = true);
    prd.save(&config.paths.prd_file)?;
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
        &config.paths.prompt_file, &config.paths.guardrails_file, &config.paths.ralph_dir,
        &config.paths.work_dir, iteration, failure_memory,
    );
    let built = builder.build(story, None);
    event_sink.emit(LoopEvent::PromptBuilt {
        story_id: story.id.clone(),
        size_bytes: built.size_bytes,
        hash: built.hash,
        truncated: built.truncated,
    });
    tracing::debug!(size = built.size_bytes, hash = built.hash, truncated = built.truncated, "prompt built");
    built
}

fn check_story_passed_in_prd(config: &RalphConfig, story_id: &str) -> bool {
    prd_lifecycle::load_or_restore(config)
        .and_then(|prd| prd.stories.iter().find(|st| st.id == story_id).map(|st| st.passes))
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
    let total_commits = git::count_commits_since(&config.paths.work_dir, initial_commit).await.unwrap_or(0);
    logger::log_activity(
        &format!(
            "Iteration {iteration} completed in {} | Commits: {total_commits} | Story: {story_id} | Passed: {passed}",
            logger::format_duration(elapsed),
        ),
        &config.paths.activity_log,
    );
    tracing::info!(duration = %logger::format_duration(elapsed), commits = total_commits, "Iteration complete");
}
