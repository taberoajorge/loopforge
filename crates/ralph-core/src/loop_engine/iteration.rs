use crate::config::RalphConfig;
use crate::detection::failure_memory::FailureMemory;
use crate::events::{LoopEvent, LoopEventSink};
use crate::git;
use crate::logger;
use crate::providers::Provider;
use crate::verification;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use super::prd_lifecycle;

const CIRCUIT_BREAKER_THRESHOLD: u32 = 3;

pub struct VerificationOutcome {
    pub agent_result: anyhow::Result<crate::providers::AgentResult>,
    pub story_passed: bool,
}

#[allow(clippy::too_many_arguments)]
pub async fn run_with_verification<P: Provider>(
    config: &RalphConfig,
    provider: &P,
    story: &crate::prd::UserStory,
    initial_prompt: String,
    initial_hash: u64,
    shutdown_flag: &Arc<AtomicBool>,
    event_sink: &dyn LoopEventSink,
    failure_memory: &mut FailureMemory,
    iteration: u32,
) -> VerificationOutcome {
    let max_retries = config.tuning.max_verification_retries;
    let mut current_prompt = initial_prompt;
    let mut prev_prompt_hash: Option<u64> = Some(initial_hash);
    let mut story_passed = false;
    let mut verification_attempt: u32 = 0;
    let mut last_error_signature: Option<String> = None;
    let mut consecutive_same_error: u32 = 0;

    let agent_result = loop {
        verification_attempt += 1;

        let agent_result = provider
            .run_agent(
                &current_prompt,
                &story.id,
                &config.paths.work_dir,
                config.tuning.stall_timeout_secs,
                shutdown_flag.clone(),
                &config.paths.codex_output_log,
            )
            .await;

        git::remove_git_lock(&config.paths.work_dir).await;

        match &agent_result {
            Ok(result) if result.rate_limited => break agent_result,
            Ok(result) if !result.success() => break agent_result,
            Err(_) => break agent_result,
            Ok(_) => {}
        }

        if story.verification.commands.is_empty() {
            story_passed = super::check_story_passed_in_prd(config, &story.id);
            break agent_result;
        }

        event_sink.emit(LoopEvent::VerificationStarted {
            story_id: story.id.clone(),
            attempt: verification_attempt,
            max_attempts: max_retries,
        });

        let verification_result =
            verification::run_verification(story, &config.paths.work_dir).await;

        if verification_result.passed {
            event_sink.emit(LoopEvent::VerificationPassed {
                story_id: story.id.clone(),
                attempt: verification_attempt,
            });
            logger::log_info(&format!(
                "Verification passed for {} on attempt {verification_attempt}",
                story.id
            ));
            story_passed = true;
            break agent_result;
        }

        let error_sig =
            verification::classify_error_signature(&verification_result.diagnostics);

        if last_error_signature.as_ref() == Some(&error_sig) {
            consecutive_same_error += 1;
        } else {
            consecutive_same_error = 1;
            last_error_signature = Some(error_sig.clone());
        }

        if consecutive_same_error >= CIRCUIT_BREAKER_THRESHOLD {
            handle_circuit_breaker(
                config, &story.id, &error_sig, consecutive_same_error,
                iteration, event_sink, failure_memory, verification_attempt,
                verification_result.diagnostics.len(),
            );
            break agent_result;
        }

        event_sink.emit(LoopEvent::VerificationFailed {
            story_id: story.id.clone(),
            attempt: verification_attempt,
            error_count: verification_result.diagnostics.len(),
            circuit_breaker: false,
        });

        logger::log_warning(&format!(
            "Verification failed for {} (attempt {verification_attempt}/{max_retries}): {} diagnostics [sig={error_sig}]",
            story.id,
            verification_result.diagnostics.len()
        ));

        if verification_attempt >= max_retries {
            handle_retry_exhausted(
                config, &story.id, max_retries, &verification_result,
                iteration, failure_memory,
            );
            break agent_result;
        }

        let retry_prompt = verification::build_retry_prompt(
            story, &verification_result.diagnostics,
            verification_attempt, max_retries, &config.paths.work_dir,
        );

        let retry_hash = crate::prompt::prompt_content_hash(&retry_prompt);
        if let Some(prev) = prev_prompt_hash {
            if retry_hash != prev {
                tracing::info!(prev_hash = prev, new_hash = retry_hash, "prompt changed between attempts");
            }
        }
        prev_prompt_hash = Some(retry_hash);
        current_prompt = retry_prompt;

        logger::log_info(&format!(
            "Retrying {} with diagnostic context (attempt {})",
            story.id,
            verification_attempt + 1
        ));
    };

    VerificationOutcome {
        agent_result,
        story_passed,
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_circuit_breaker(
    config: &RalphConfig,
    story_id: &str,
    error_sig: &str,
    consecutive_same_error: u32,
    iteration: u32,
    event_sink: &dyn LoopEventSink,
    failure_memory: &mut FailureMemory,
    attempt: u32,
    error_count: usize,
) {
    event_sink.emit(LoopEvent::VerificationFailed {
        story_id: story_id.to_string(),
        attempt,
        error_count,
        circuit_breaker: true,
    });
    tracing::warn!(
        story_id, signature = error_sig, consecutive = consecutive_same_error,
        "circuit breaker: same error signature repeated"
    );
    logger::log_warning(&format!(
        "Circuit breaker triggered for {story_id}: same error {consecutive_same_error}x, marking blocked",
    ));

    failure_memory.record_failure(
        story_id, iteration, "circuit_breaker", vec![],
        &format!("Same error signature {error_sig} repeated {consecutive_same_error}x"),
    );
    prd_lifecycle::mark_story_blocked(config, story_id);

    if let Err(guardrail_err) = crate::guardrails::add_guardrail(
        &config.paths.guardrails_file, story_id,
        &format!("Circuit breaker: same verification error {consecutive_same_error}x"),
        iteration,
    ) {
        tracing::warn!(error = %guardrail_err, "failed to write guardrail");
    }
}

fn handle_retry_exhausted(
    config: &RalphConfig,
    story_id: &str,
    max_retries: u32,
    verification_result: &verification::VerificationResult,
    iteration: u32,
    failure_memory: &mut FailureMemory,
) {
    logger::log_error(
        &format!("Story {story_id} exceeded max verification retries ({max_retries}), marking blocked"),
        Some(&config.paths.error_log),
    );
    failure_memory.record_failure(
        story_id, iteration, "verification_exhausted", vec![],
        &format!("{} diagnostics after {max_retries} retries", verification_result.diagnostics.len()),
    );
    prd_lifecycle::mark_story_blocked(config, story_id);

    if let Err(guardrail_err) = crate::guardrails::add_guardrail(
        &config.paths.guardrails_file, story_id,
        &format!(
            "Verification failed after {max_retries} retries with {} diagnostics",
            verification_result.diagnostics.len()
        ),
        iteration,
    ) {
        tracing::warn!(error = %guardrail_err, "failed to write guardrail");
    }
}
