pub const EVENT_AGENT_OUTPUT_STREAM: &str = "agent-output-stream";
pub const EVENT_ITERATION_STARTED: &str = "loop:iteration-started";
pub const EVENT_ITERATION_COMPLETED: &str = "loop:iteration-completed";
pub const EVENT_RATE_LIMIT_DETECTED: &str = "loop:rate-limit-detected";
pub const EVENT_AGENT_SWITCHED: &str = "loop:agent-switched";
pub const EVENT_SESSION_STARTED: &str = "loop:session-started";
pub const EVENT_SESSION_ENDED: &str = "loop:session-ended";
pub const EVENT_HEARTBEAT: &str = "loop:heartbeat";
pub const EVENT_VERIFICATION_STARTED: &str = "loop:verification-started";
pub const EVENT_VERIFICATION_FAILED: &str = "loop:verification-failed";
pub const EVENT_VERIFICATION_PASSED: &str = "loop:verification-passed";
pub const EVENT_PROMPT_BUILT: &str = "loop:prompt-built";
pub const EVENT_STORY_SKIPPED: &str = "loop:story-skipped";

pub const EVENT_ASK_STREAM: &str = "ask:stream";
pub const EVENT_ASK_COMPLETE: &str = "ask:complete";
pub const EVENT_ASK_ERROR: &str = "ask:error";

pub const EVENT_PLAN_ACTIVITY_BATCH: &str = "plan:activity-batch";
pub const EVENT_PLAN_COMPLETE: &str = "plan:complete";
pub const EVENT_PLAN_ERROR: &str = "plan:error";
pub const EVENT_PLAN_HEARTBEAT: &str = "plan:heartbeat";

#[allow(dead_code)]
pub const LOOP_EVENT_CATALOG: [&str; 13] = [
    EVENT_AGENT_OUTPUT_STREAM,
    EVENT_ITERATION_STARTED,
    EVENT_ITERATION_COMPLETED,
    EVENT_RATE_LIMIT_DETECTED,
    EVENT_AGENT_SWITCHED,
    EVENT_SESSION_STARTED,
    EVENT_SESSION_ENDED,
    EVENT_HEARTBEAT,
    EVENT_VERIFICATION_STARTED,
    EVENT_VERIFICATION_FAILED,
    EVENT_VERIFICATION_PASSED,
    EVENT_PROMPT_BUILT,
    EVENT_STORY_SKIPPED,
];
