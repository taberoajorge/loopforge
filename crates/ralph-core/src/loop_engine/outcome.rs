use crate::config::RalphConfig;
use crate::detection::failure_memory::FailureMemory;
use crate::detection::progress::ProgressReport;
use crate::errors::LoopError;
use crate::events::{HeartbeatContext, LoopEventSink};
use crate::guardrails;
use crate::logger;
use crate::providers::AgentResult;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use super::{helpers, prd_lifecycle};

#[allow(clippy::too_many_arguments)]
pub async fn handle(
    config: &RalphConfig,
    agent_result: &anyhow::Result<AgentResult>,
    prd: &mut crate::prd::Prd,
    story_id: &str,
    story_passed: bool,
    progress_report: &ProgressReport,
    failure_memory: &mut FailureMemory,
    consecutive_zero_progress: &mut u32,
    iteration: &mut u32,
    shutdown_flag: &Arc<AtomicBool>,
    event_sink: &dyn LoopEventSink,
) -> Result<(), LoopError> {
    match agent_result {
        Ok(result) if result.rate_limited => {
            handle_rate_limit(config, result, iteration, shutdown_flag, event_sink).await;
        }
        Ok(result) if result.success() => {
            handle_success(
                config,
                prd,
                story_id,
                story_passed,
                progress_report,
                failure_memory,
                consecutive_zero_progress,
                result,
                *iteration,
            );
        }
        Ok(result) => {
            handle_failure(config, result, story_id, failure_memory, *iteration);
        }
        Err(err) => {
            handle_error(config, err, story_id, failure_memory, *iteration);
        }
    }
    Ok(())
}

async fn handle_rate_limit(
    config: &RalphConfig,
    result: &AgentResult,
    iteration: &mut u32,
    shutdown_flag: &Arc<AtomicBool>,
    event_sink: &dyn LoopEventSink,
) {
    let wait_secs = super::rate_limiter::compute_wait(result, config.tuning.rate_limit_wait_secs);
    let retry_hint = result.retry_after_message.as_deref().unwrap_or("unknown");
    logger::log_warning(&format!(
        "RATE LIMITED! Provider says retry at: {retry_hint}. Sleeping {}m {}s...",
        wait_secs / 60,
        wait_secs % 60,
    ));
    logger::log_error(
        &format!(
            "Iteration {} rate limited (retry at: {retry_hint})",
            *iteration
        ),
        Some(&config.paths.error_log),
    );
    logger::log_activity(
        &format!(
            "Rate limited at iteration {}, sleeping {wait_secs}s",
            *iteration
        ),
        &config.paths.activity_log,
    );
    *iteration = iteration.saturating_sub(1);
    if helpers::interruptible_sleep(
        wait_secs,
        shutdown_flag,
        event_sink,
        HeartbeatContext::RateLimitWait,
    )
    .await
    {
        logger::log_warning("Shutdown requested during rate limit wait");
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_success(
    config: &RalphConfig,
    prd: &mut crate::prd::Prd,
    story_id: &str,
    story_passed: bool,
    progress_report: &ProgressReport,
    failure_memory: &mut FailureMemory,
    consecutive_zero_progress: &mut u32,
    result: &AgentResult,
    iteration: u32,
) {
    if story_passed {
        if prd.mark_story_passed(story_id) {
            if let Err(save_err) = prd.save(&config.paths.prd_file) {
                tracing::warn!(error = %save_err, "failed to save PRD after passing story");
            }
        }
        logger::log_success(&format!("Story {story_id} completed!"));
        *consecutive_zero_progress = 0;
    } else if progress_report.zero_progress {
        *consecutive_zero_progress += 1;
        logger::log_warning(&format!(
            "Zero progress detected ({} consecutive)",
            *consecutive_zero_progress
        ));
        let approach = prd_lifecycle::summarize_approach(&result.output_lines);
        failure_memory.record_failure(story_id, iteration, "zero_progress", vec![], &approach);
        if *consecutive_zero_progress >= 2 {
            if let Err(guardrail_err) = guardrails::add_guardrail(
                &config.paths.guardrails_file,
                story_id,
                "Zero progress in consecutive iterations",
                iteration,
            ) {
                tracing::warn!(error = %guardrail_err, "failed to write guardrail");
            }
        }
    } else {
        *consecutive_zero_progress = 0;
        logger::log_warning(&format!(
            "Story {story_id}: agent finished but not marked as passed",
        ));
    }
}

fn handle_failure(
    config: &RalphConfig,
    result: &AgentResult,
    story_id: &str,
    failure_memory: &mut FailureMemory,
    iteration: u32,
) {
    let approach = prd_lifecycle::summarize_approach(&result.output_lines);
    let error_type = if result.stall_killed {
        "stall_killed"
    } else {
        "nonzero_exit"
    };
    failure_memory.record_failure(story_id, iteration, error_type, vec![], &approach);
    logger::log_error(
        &format!(
            "Iteration {iteration} failed (exit: {}, stall: {})",
            result.exit_code, result.stall_killed
        ),
        Some(&config.paths.error_log),
    );
}

fn handle_error(
    config: &RalphConfig,
    err: &anyhow::Error,
    story_id: &str,
    failure_memory: &mut FailureMemory,
    iteration: u32,
) {
    failure_memory.record_failure(story_id, iteration, "spawn_error", vec![], &err.to_string());
    logger::log_error(
        &format!("Iteration {iteration} error: {err}"),
        Some(&config.paths.error_log),
    );
}
