use crate::config::RalphConfig;
use crate::detection::failure_memory::FailureMemory;
use crate::detection::progress;
use crate::events::{HeartbeatContext, LoopEvent, LoopEventSink};
use crate::git;
use crate::guardrails;
use crate::health;
use crate::logger;
use crate::prd::Prd;
use crate::prompt::PromptBuilder;
use crate::providers::Provider;
use crate::state;
use crate::verification;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const CIRCUIT_BREAKER_THRESHOLD: u32 = 3;
const HEARTBEAT_INTERVAL_SECS: u64 = 30;

pub async fn run<P: Provider>(
    config: &RalphConfig,
    provider: &P,
    shutdown_flag: Arc<AtomicBool>,
    event_sink: &dyn LoopEventSink,
) -> Result<()> {
    let mut iteration: u32 = 0;
    let initial_commit = git::get_head_hash(&config.work_dir).await.unwrap_or_default();
    let mut failure_memory = FailureMemory::load(&config.failure_memory_file);
    let mut consecutive_zero_progress: u32 = 0;

    while iteration < config.max_iterations {
        if shutdown_flag.load(Ordering::SeqCst) {
            logger::log_warning("Shutdown requested, exiting loop");
            break;
        }

        state::wait_while_paused(&config.pause_file).await;
        if state::check_and_clear_done(&config.done_file) {
            break;
        }

        let services_ok = health::check_all_services_parallel(config).await;
        if !services_ok {
            logger::log_error("Services unavailable, retrying in 30s", Some(&config.error_log));
            logger::log_activity("Iteration skipped: services down", &config.activity_log);
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            continue;
        }

        let mut prd = match load_or_restore_prd(config) {
            Some(prd) => prd,
            None => {
                logger::log_error("PRD unrecoverable, stopping", Some(&config.error_log));
                break;
            }
        };

        let total = prd.total_stories();
        let passed = prd.passed_count();
        let blocked = prd.blocked_count();
        let pending = prd.pending_count();

        if pending == 0 {
            logger::log_success(&format!(
                "All stories completed! {passed} passed, {blocked} blocked."
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

        iteration += 1;
        let iter_start = std::time::Instant::now();

        logger::log_iteration_header(iteration);
        logger::log_activity(
            &format!("=== Iteration {iteration} started ==="),
            &config.activity_log,
        );

        println!(
            "   Progress: {passed}/{total} | Pending: {pending} | Blocked: {blocked}"
        );
        println!("   Story: {}", next_story.title);

        if failure_memory.is_in_gutter(&next_story.id, config.gutter_threshold) {
            logger::log_warning(&format!(
                "GUTTER: {} has failed {}+ times, skipping",
                next_story.id, config.gutter_threshold
            ));
            event_sink.emit(LoopEvent::StorySkipped {
                story_id: next_story.id.clone(),
                reason: format!("gutter threshold ({} failures)", config.gutter_threshold),
            });
            guardrails::add_guardrail(
                &config.guardrails_file,
                &next_story.id,
                "Exceeded gutter threshold",
                iteration,
            );
            logger::log_activity(
                &format!("Story {} skipped (gutter threshold)", next_story.id),
                &config.activity_log,
            );

            prd.stories
                .iter_mut()
                .find(|story| story.id == next_story.id)
                .map(|story| story.blocked = true);
            prd.save(&config.prd_file)?;
            continue;
        }

        prd.save(&config.prd_backup)?;

        let progress_before = progress::get_progress_file_size(&config.progress_file);

        let head_before = git::get_head_hash(&config.work_dir)
            .await
            .unwrap_or_default();

        let prompt_builder = PromptBuilder::new(
            &config.prompt_file,
            &config.guardrails_file,
            &config.ralph_dir,
            &config.work_dir,
            iteration,
            &failure_memory,
        );
        let built = prompt_builder.build(&next_story, None);

        event_sink.emit(LoopEvent::PromptBuilt {
            story_id: next_story.id.clone(),
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

        if let Err(validation_errors) = built.validate(&next_story.id) {
            tracing::error!(
                story_id = %next_story.id,
                errors = ?validation_errors,
                "prompt validation failed, skipping story"
            );
            event_sink.emit(LoopEvent::StorySkipped {
                story_id: next_story.id.clone(),
                reason: format!("prompt validation failed: {}", validation_errors.join("; ")),
            });
            continue;
        }

        logger::log_info(&format!("Starting {} agent ({})...", provider.name(), provider.model()));

        let mut verification_attempt: u32 = 0;
        let max_retries = config.max_verification_retries;
        let mut current_prompt = built.text;
        let mut prev_prompt_hash: Option<u64> = Some(built.hash);
        let mut story_passed = false;

        let mut last_error_signature: Option<String> = None;
        let mut consecutive_same_error: u32 = 0;

        let agent_result = loop {
            verification_attempt += 1;

            let agent_result = provider
                .run_agent(
                    &current_prompt,
                    &next_story.id,
                    &config.work_dir,
                    config.stall_timeout_secs,
                    shutdown_flag.clone(),
                    &config.codex_output_log,
                )
                .await;

            git::remove_git_lock(&config.work_dir).await;

            match &agent_result {
                Ok(result) if result.rate_limited => {
                    break agent_result;
                }
                Ok(result) if !result.success() => {
                    break agent_result;
                }
                Err(_) => {
                    break agent_result;
                }
                Ok(_) => {}
            }

            if next_story.verification.commands.is_empty() {
                let prd_check = load_or_restore_prd(config);
                story_passed = prd_check
                    .as_ref()
                    .and_then(|prd| prd.stories.iter().find(|st| st.id == next_story.id))
                    .map(|st| st.passes)
                    .unwrap_or(false);
                break agent_result;
            }

            event_sink.emit(LoopEvent::VerificationStarted {
                story_id: next_story.id.clone(),
                attempt: verification_attempt,
                max_attempts: max_retries,
            });

            let verification_result = verification::run_verification(
                &next_story,
                &config.work_dir,
            )
            .await;

            if verification_result.passed {
                event_sink.emit(LoopEvent::VerificationPassed {
                    story_id: next_story.id.clone(),
                    attempt: verification_attempt,
                });
                logger::log_info(&format!(
                    "Verification passed for {} on attempt {verification_attempt}",
                    next_story.id
                ));
                story_passed = true;
                break agent_result;
            }

            let error_sig = verification::classify_error_signature(&verification_result.diagnostics);

            if last_error_signature.as_ref() == Some(&error_sig) {
                consecutive_same_error += 1;
            } else {
                consecutive_same_error = 1;
                last_error_signature = Some(error_sig.clone());
            }

            if consecutive_same_error >= CIRCUIT_BREAKER_THRESHOLD {
                event_sink.emit(LoopEvent::VerificationFailed {
                    story_id: next_story.id.clone(),
                    attempt: verification_attempt,
                    error_count: verification_result.diagnostics.len(),
                    circuit_breaker: true,
                });
                tracing::warn!(
                    story_id = %next_story.id,
                    signature = %error_sig,
                    consecutive = consecutive_same_error,
                    "circuit breaker: same error signature repeated"
                );
                logger::log_warning(&format!(
                    "Circuit breaker triggered for {}: same error {}x, marking blocked",
                    next_story.id, consecutive_same_error
                ));

                failure_memory.record_failure(
                    &next_story.id,
                    iteration,
                    "circuit_breaker",
                    vec![],
                    &format!(
                        "Same error signature {error_sig} repeated {consecutive_same_error}x"
                    ),
                );

                if let Some(mut prd) = load_or_restore_prd(config) {
                    if let Some(st) = prd.stories.iter_mut().find(|st| st.id == next_story.id) {
                        st.blocked = true;
                    }
                    let _ = prd.save(&config.prd_file);
                }

                guardrails::add_guardrail(
                    &config.guardrails_file,
                    &next_story.id,
                    &format!(
                        "Circuit breaker: same verification error {consecutive_same_error}x"
                    ),
                    iteration,
                );

                break agent_result;
            }

            event_sink.emit(LoopEvent::VerificationFailed {
                story_id: next_story.id.clone(),
                attempt: verification_attempt,
                error_count: verification_result.diagnostics.len(),
                circuit_breaker: false,
            });

            logger::log_warning(&format!(
                "Verification failed for {} (attempt {verification_attempt}/{max_retries}): {} diagnostics [sig={error_sig}]",
                next_story.id,
                verification_result.diagnostics.len()
            ));

            if verification_attempt >= max_retries {
                logger::log_error(
                    &format!(
                        "Story {} exceeded max verification retries ({max_retries}), marking blocked",
                        next_story.id
                    ),
                    Some(&config.error_log),
                );
                failure_memory.record_failure(
                    &next_story.id,
                    iteration,
                    "verification_exhausted",
                    vec![],
                    &format!(
                        "{} diagnostics after {max_retries} retries",
                        verification_result.diagnostics.len()
                    ),
                );

                if let Some(mut prd) = load_or_restore_prd(config) {
                    if let Some(st) = prd.stories.iter_mut().find(|st| st.id == next_story.id) {
                        st.blocked = true;
                    }
                    let _ = prd.save(&config.prd_file);
                }

                guardrails::add_guardrail(
                    &config.guardrails_file,
                    &next_story.id,
                    &format!(
                        "Verification failed after {max_retries} retries with {} diagnostics",
                        verification_result.diagnostics.len()
                    ),
                    iteration,
                );

                break agent_result;
            }

            let retry_prompt = verification::build_retry_prompt(
                &next_story,
                &verification_result.diagnostics,
                verification_attempt,
                max_retries,
                &config.work_dir,
            );

            let retry_hash = crate::prompt::prompt_content_hash(&retry_prompt);
            if let Some(prev) = prev_prompt_hash {
                if retry_hash != prev {
                    tracing::info!(
                        prev_hash = prev,
                        new_hash = retry_hash,
                        "prompt changed between attempts"
                    );
                }
            }
            prev_prompt_hash = Some(retry_hash);
            current_prompt = retry_prompt;

            logger::log_info(&format!(
                "Retrying {} with diagnostic context (attempt {})",
                next_story.id,
                verification_attempt + 1
            ));
        };

        if !story_passed {
            if let Some(prd_check) = load_or_restore_prd(config) {
                story_passed = prd_check
                    .stories
                    .iter()
                    .find(|st| st.id == next_story.id)
                    .map(|st| st.passes)
                    .unwrap_or(false);
            }
        }

        let progress_report = progress::analyze_iteration_progress(
            &config.work_dir,
            &head_before,
            story_passed,
            &config.progress_file,
            progress_before,
        )
        .await;

        match &agent_result {
            Ok(result) if result.rate_limited => {
                let wait_secs = compute_rate_limit_wait(result, config.rate_limit_wait_secs);
                let retry_hint = result
                    .retry_after_message
                    .as_deref()
                    .unwrap_or("unknown");
                logger::log_warning(&format!(
                    "RATE LIMITED! Provider says retry at: {retry_hint}. Sleeping {}m {}s...",
                    wait_secs / 60,
                    wait_secs % 60,
                ));
                logger::log_error(
                    &format!("Iteration {iteration} rate limited (retry at: {retry_hint})"),
                    Some(&config.error_log),
                );
                logger::log_activity(
                    &format!("Rate limited at iteration {iteration}, sleeping {wait_secs}s"),
                    &config.activity_log,
                );
                iteration = iteration.saturating_sub(1);
                if interruptible_sleep(wait_secs, &shutdown_flag, event_sink, HeartbeatContext::RateLimitWait).await {
                    logger::log_warning("Shutdown requested during rate limit wait");
                    break;
                }
                continue;
            }
            Ok(result) if result.success() => {
                if story_passed {
                    if prd.mark_story_passed(&next_story.id) {
                        let _ = prd.save(&config.prd_file);
                    }
                    logger::log_success(&format!("Story {} completed!", next_story.id));
                    consecutive_zero_progress = 0;
                } else if progress_report.zero_progress {
                    consecutive_zero_progress += 1;
                    logger::log_warning(&format!(
                        "Zero progress detected ({consecutive_zero_progress} consecutive)"
                    ));
                    let approach = summarize_approach(&result.output_lines);
                    failure_memory.record_failure(
                        &next_story.id,
                        iteration,
                        "zero_progress",
                        vec![],
                        &approach,
                    );
                    if consecutive_zero_progress >= 2 {
                        guardrails::add_guardrail(
                            &config.guardrails_file,
                            &next_story.id,
                            "Zero progress in consecutive iterations",
                            iteration,
                        );
                    }
                } else {
                    consecutive_zero_progress = 0;
                    logger::log_warning(&format!(
                        "Story {}: agent finished but not marked as passed",
                        next_story.id
                    ));
                }
            }
            Ok(result) => {
                let approach = summarize_approach(&result.output_lines);
                let error_type = if result.stall_killed {
                    "stall_killed"
                } else {
                    "nonzero_exit"
                };
                failure_memory.record_failure(
                    &next_story.id,
                    iteration,
                    error_type,
                    vec![],
                    &approach,
                );
                logger::log_error(
                    &format!(
                        "Iteration {iteration} failed (exit: {}, stall: {})",
                        result.exit_code, result.stall_killed
                    ),
                    Some(&config.error_log),
                );
            }
            Err(err) => {
                failure_memory.record_failure(
                    &next_story.id,
                    iteration,
                    "spawn_error",
                    vec![],
                    &err.to_string(),
                );
                logger::log_error(
                    &format!("Iteration {iteration} error: {err}"),
                    Some(&config.error_log),
                );
            }
        }

        failure_memory.save(&config.failure_memory_file)?;

        let elapsed = iter_start.elapsed().as_secs();
        let total_commits = git::count_commits_since(&config.work_dir, &initial_commit)
            .await
            .unwrap_or(0);

        logger::log_activity(
            &format!(
                "Iteration {iteration} completed in {} | Commits: {total_commits} | Story: {} | Passed: {story_passed}",
                logger::format_duration(elapsed),
                next_story.id,
            ),
            &config.activity_log,
        );
        println!(
            "   Duration: {} | Commits: {total_commits}",
            logger::format_duration(elapsed)
        );

        logger::log_info(&format!(
            "Cooldown {}s before next iteration...",
            config.cooldown_secs
        ));
        if interruptible_sleep(config.cooldown_secs, &shutdown_flag, event_sink, HeartbeatContext::CooldownWait).await {
            logger::log_warning("Shutdown requested during cooldown");
            break;
        }
    }

    print_session_summary(iteration, config, &initial_commit).await;
    state::clear_state(&config.state_file);
    Ok(())
}

async fn interruptible_sleep(
    total_secs: u64,
    shutdown_flag: &Arc<AtomicBool>,
    event_sink: &dyn LoopEventSink,
    context: HeartbeatContext,
) -> bool {
    let mut elapsed: u64 = 0;
    let mut since_last_heartbeat: u64 = 0;

    while elapsed < total_secs {
        if shutdown_flag.load(Ordering::SeqCst) {
            return true;
        }
        let chunk = (total_secs - elapsed).min(2);
        tokio::time::sleep(std::time::Duration::from_secs(chunk)).await;
        elapsed += chunk;
        since_last_heartbeat += chunk;

        if since_last_heartbeat >= HEARTBEAT_INTERVAL_SECS {
            event_sink.emit(LoopEvent::Heartbeat {
                elapsed_secs: elapsed,
                total_secs,
                context: context.clone(),
            });
            since_last_heartbeat = 0;
        }
    }
    false
}

fn compute_rate_limit_wait(result: &crate::providers::AgentResult, default_secs: u64) -> u64 {
    if let Some(ref retry_hint) = result.retry_after_message {
        if let Some(secs) = parse_retry_time_to_secs(retry_hint) {
            return secs.min(3600);
        }
    }
    default_secs
}

fn parse_retry_time_to_secs(time_str: &str) -> Option<u64> {
    let now = chrono::Local::now();
    let cleaned = time_str
        .trim()
        .replace(".", "")
        .to_uppercase();

    let is_pm = cleaned.contains("PM");
    let is_am = cleaned.contains("AM");
    let digits_only = cleaned
        .replace("PM", "")
        .replace("AM", "")
        .trim()
        .to_string();

    let parts: Vec<&str> = digits_only.split(':').collect();
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }

    let mut hour: u32 = parts[0].trim().parse().ok()?;
    let minute: u32 = if parts.len() == 2 {
        parts[1].trim().parse().ok()?
    } else {
        0
    };

    if is_pm && hour < 12 {
        hour += 12;
    } else if is_am && hour == 12 {
        hour = 0;
    }

    let target = now
        .date_naive()
        .and_hms_opt(hour, minute, 0)?;
    let target_dt = target
        .and_local_timezone(now.timezone())
        .single()?;

    let diff = target_dt.signed_duration_since(now);
    if diff.num_seconds() <= 0 {
        return Some(120);
    }
    Some(diff.num_seconds() as u64 + 30)
}

fn load_or_restore_prd(config: &RalphConfig) -> Option<Prd> {
    if Prd::is_valid_json(&config.prd_file) {
        return Prd::load(&config.prd_file).ok();
    }
    logger::log_warning("PRD missing or corrupted, restoring from backup");
    if config.prd_backup.exists() {
        let _ = std::fs::copy(&config.prd_backup, &config.prd_file);
        return Prd::load(&config.prd_file).ok();
    }
    None
}

fn summarize_approach(output_lines: &[String]) -> String {
    let meaningful: Vec<&str> = output_lines
        .iter()
        .rev()
        .take(20)
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && trimmed.len() > 10
        })
        .take(3)
        .map(|line| line.as_str())
        .collect();

    if meaningful.is_empty() {
        return "unknown approach".to_string();
    }
    meaningful.join(" | ")
}

async fn print_session_summary(iterations: u32, config: &RalphConfig, initial_commit: &str) {
    println!();
    println!("{}", "===============================================");
    println!("  Session ended after {iterations} iterations");
    println!("{}", "===============================================");

    let total_commits = git::count_commits_since(&config.work_dir, initial_commit)
        .await
        .unwrap_or(0);
    println!("   Total commits: {total_commits}");

    if let Some(last_hash) = git::load_last_rebase(&config.last_rebase_file) {
        println!("   Last rebase: {last_hash}");
    }

    logger::log_activity(
        &format!("=== Session ended after {iterations} iterations ==="),
        &config.activity_log,
    );
}
