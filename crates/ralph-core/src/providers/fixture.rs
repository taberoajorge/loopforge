use super::AgentResult;
use anyhow::Result;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum FixtureScenario {
    HappyPath,
    LoopFailure,
    Custom(Vec<FixtureStep>),
}

#[derive(Debug, Clone)]
pub struct FixtureStep {
    pub exit_code: i32,
    pub output_lines: Vec<String>,
    pub rate_limited: bool,
}

pub struct FixtureProvider {
    scenario: FixtureScenario,
    call_count: std::sync::atomic::AtomicU32,
}

impl FixtureProvider {
    pub fn new(scenario: FixtureScenario) -> Self {
        Self {
            scenario,
            call_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    pub fn call_count(&self) -> u32 {
        self.call_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn next_result(&self) -> AgentResult {
        let idx = self
            .call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        match &self.scenario {
            FixtureScenario::HappyPath => AgentResult {
                exit_code: 0,
                stall_killed: false,
                output_lines: vec![format!("fixture: iteration {idx} succeeded")],
                rate_limited: false,
                retry_after_message: None,
            },
            FixtureScenario::LoopFailure => AgentResult {
                exit_code: 1,
                stall_killed: false,
                output_lines: vec![format!("fixture: iteration {idx} failed")],
                rate_limited: false,
                retry_after_message: None,
            },
            FixtureScenario::Custom(steps) => {
                let step_idx = (idx as usize).min(steps.len().saturating_sub(1));
                let step = &steps[step_idx];
                let (rate_limited, retry_msg) = if step.rate_limited {
                    AgentResult::detect_rate_limit(&step.output_lines)
                } else {
                    (false, None)
                };
                AgentResult {
                    exit_code: step.exit_code,
                    stall_killed: false,
                    output_lines: step.output_lines.clone(),
                    rate_limited,
                    retry_after_message: retry_msg,
                }
            }
        }
    }
}

impl super::Provider for FixtureProvider {
    fn name(&self) -> &'static str {
        "fixture"
    }

    fn model(&self) -> &'static str {
        "deterministic"
    }

    async fn run_agent(
        &self,
        _prompt: &str,
        _story_id: &str,
        _work_dir: &Path,
        _stall_timeout_secs: u64,
        _shutdown_flag: Arc<AtomicBool>,
        _output_log: &Path,
    ) -> Result<AgentResult> {
        Ok(self.next_result())
    }
}
