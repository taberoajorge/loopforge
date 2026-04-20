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

pub struct RecordingEventSink {
    events: std::sync::Mutex<Vec<LoopEvent>>,
}

impl Default for RecordingEventSink {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordingEventSink {
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn events(&self) -> Vec<LoopEvent> {
        self.events.lock().unwrap().clone()
    }

    pub fn event_types(&self) -> Vec<String> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .map(|event| match event {
                LoopEvent::Heartbeat { .. } => "heartbeat".into(),
                LoopEvent::VerificationStarted { .. } => "verification_started".into(),
                LoopEvent::VerificationFailed { .. } => "verification_failed".into(),
                LoopEvent::VerificationPassed { .. } => "verification_passed".into(),
                LoopEvent::PromptBuilt { .. } => "prompt_built".into(),
                LoopEvent::StorySkipped { .. } => "story_skipped".into(),
                LoopEvent::HealthCheckWaiting { .. } => "health_check_waiting".into(),
            })
            .collect()
    }
}

impl LoopEventSink for RecordingEventSink {
    fn emit(&self, event: LoopEvent) {
        self.events.lock().unwrap().push(event);
    }
}
