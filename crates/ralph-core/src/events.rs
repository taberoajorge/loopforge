use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LoopEvent {
    Heartbeat {
        elapsed_secs: u64,
        total_secs: u64,
        context: HeartbeatContext,
    },
    VerificationStarted {
        story_id: String,
        attempt: u32,
        max_attempts: u32,
    },
    VerificationFailed {
        story_id: String,
        attempt: u32,
        error_count: usize,
        circuit_breaker: bool,
    },
    VerificationPassed {
        story_id: String,
        attempt: u32,
    },
    PromptBuilt {
        story_id: String,
        size_bytes: usize,
        hash: u64,
        truncated: bool,
    },
    StorySkipped {
        story_id: String,
        reason: String,
    },
    HealthCheckWaiting {
        service: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeartbeatContext {
    RateLimitWait,
    CooldownWait,
    HealthCheckWait,
}

pub trait LoopEventSink: Send + Sync {
    fn emit(&self, event: LoopEvent);
}

pub struct NoopEventSink;

impl LoopEventSink for NoopEventSink {
    fn emit(&self, _event: LoopEvent) {}
}
