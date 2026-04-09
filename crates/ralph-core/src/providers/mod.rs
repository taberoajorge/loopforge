pub mod cli;
pub mod fixture;

use anyhow::Result;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AgentResult {
    pub exit_code: i32,
    pub stall_killed: bool,
    pub output_lines: Vec<String>,
    pub rate_limited: bool,
    pub retry_after_message: Option<String>,
}

const RATE_LIMIT_PATTERNS: &[&str] = &[
    "you've hit your usage limit",
    "rate limit exceeded",
    "too many requests",
    "quota exceeded",
    "rate_limit_error",
    "overloaded_error",
];

impl AgentResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0 && !self.stall_killed
    }

    pub fn detect_rate_limit(output_lines: &[String]) -> (bool, Option<String>) {
        for line in output_lines.iter().rev().take(30) {
            let lower = line.to_lowercase();
            let is_rate_limited = RATE_LIMIT_PATTERNS.iter().any(|pattern| lower.contains(pattern));
            if is_rate_limited {
                let retry_msg = extract_retry_time(line);
                return (true, retry_msg);
            }
        }
        (false, None)
    }
}

fn extract_retry_time(line: &str) -> Option<String> {
    if let Some(idx) = line.to_lowercase().find("try again at ") {
        let after = &line[idx + 13..];
        let end = after.find(|ch: char| ch == '"' || ch == '.' || ch == '}').unwrap_or(after.len());
        let time_str = after[..end].trim();
        if !time_str.is_empty() {
            return Some(time_str.to_string());
        }
    }
    None
}

pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    fn run_agent(
        &self,
        prompt: &str,
        story_id: &str,
        work_dir: &Path,
        stall_timeout_secs: u64,
        shutdown_flag: Arc<AtomicBool>,
        output_log: &Path,
    ) -> impl std::future::Future<Output = Result<AgentResult>> + Send;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProviderKind {
    Codex,
    Claude,
    Cursor,
    Gemini,
    OpenCode,
    OpenRouter,
}

impl ProviderKind {
    pub fn from_str(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "codex" => Some(Self::Codex),
            "claude" => Some(Self::Claude),
            "cursor" => Some(Self::Cursor),
            "gemini" => Some(Self::Gemini),
            "opencode" => Some(Self::OpenCode),
            "openrouter" => Some(Self::OpenRouter),
            _ => None,
        }
    }
}
