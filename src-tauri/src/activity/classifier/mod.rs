mod codex;

use crate::activity::patterns::ClassifierPatterns;
use crate::activity::types::{PlanEvent, PlanEventKind};

const PLAN_UNLOCK_LINE_THRESHOLD: u32 = 30;

pub struct ActivityClassifier {
    pub(super) patterns: ClassifierPatterns,
    content_buffer: Vec<String>,
    pub(super) provider: String,
    pub(super) in_tool_output: bool,
    pub(super) in_plan_markdown: bool,
    pub(super) line_count: u32,
    pub(super) saw_agent_response: bool,
}

impl ActivityClassifier {
    pub fn new(provider: &str) -> Self {
        Self {
            patterns: ClassifierPatterns::for_provider(provider),
            content_buffer: Vec::new(),
            provider: provider.to_string(),
            in_tool_output: false,
            in_plan_markdown: false,
            line_count: 0,
            saw_agent_response: false,
        }
    }

    pub(super) fn looks_like_markdown_plan_line(line: &str) -> bool {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#')
            || trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("+ ")
            || trimmed.starts_with("```")
            || trimmed.starts_with("> ")
        {
            return true;
        }

        let mut chars = trimmed.chars().peekable();
        let mut has_digit = false;
        while let Some(ch) = chars.peek() {
            if ch.is_ascii_digit() {
                has_digit = true;
                chars.next();
                continue;
            }
            break;
        }
        if has_digit && matches!(chars.next(), Some('.')) {
            return true;
        }

        trimmed.matches('|').count() >= 2
    }

    pub(super) fn is_prompt_echo(line: &str) -> bool {
        line.starts_with("You are a senior software architect")
            || line.starts_with("Think through the problem carefully")
            || line.starts_with("Output a structured plan in markdown")
            || line.starts_with("Feature request:")
            || line.starts_with("name:")
            || line.starts_with("description: |")
            || line.starts_with("Use when user says:")
            || line.starts_with("Executes:")
    }

    pub(super) fn is_stdin_wait_warning(line: &str) -> bool {
        line.starts_with("Warning: no stdin data received")
            || line.starts_with("If piping from a slow command, redirect stdin explicitly:")
            || line.starts_with("Reading additional input from stdin")
    }

    pub fn classify(&mut self, line: &str) -> PlanEvent {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let kind = self.detect_kind(line);

        if kind == PlanEventKind::PlanContent {
            self.content_buffer.push(line.to_string());
        }

        PlanEvent {
            kind,
            content: line.to_string(),
            timestamp,
        }
    }

    fn detect_kind(&mut self, line: &str) -> PlanEventKind {
        let trimmed = line.trim();
        if Self::is_stdin_wait_warning(trimmed) {
            return PlanEventKind::Thinking;
        }

        if self.provider == "codex" || self.provider == "opencode" {
            if let Some(kind) = self.codex_classify(line) {
                return kind;
            }
        }

        if self.patterns.error.iter().any(|regex| regex.is_match(line)) {
            return PlanEventKind::Error;
        }
        if self
            .patterns
            .mcp_call
            .iter()
            .any(|regex| regex.is_match(line))
        {
            return PlanEventKind::McpCall;
        }
        if self
            .patterns
            .search
            .iter()
            .any(|regex| regex.is_match(line))
        {
            return PlanEventKind::Search;
        }
        if self
            .patterns
            .docs_lookup
            .iter()
            .any(|regex| regex.is_match(line))
        {
            return PlanEventKind::DocsLookup;
        }
        if self
            .patterns
            .thinking
            .iter()
            .any(|regex| regex.is_match(line))
        {
            return PlanEventKind::Thinking;
        }
        PlanEventKind::PlanContent
    }

    pub fn accumulated_plan(&self) -> String {
        if self.content_buffer.is_empty() {
            return String::new();
        }
        if let Some(start) = self.find_plan_after_token_boundary() {
            return self.content_buffer[start..].join("\n");
        }
        self.content_buffer.join("\n")
    }

    fn find_plan_after_token_boundary(&self) -> Option<usize> {
        let token_line = self.content_buffer.iter().rposition(|line| {
            let lower = line.trim().to_lowercase();
            lower == "tokens used" || lower.starts_with("tokens used")
        })?;
        let mut start = token_line + 1;
        if start < self.content_buffer.len() {
            let next = self.content_buffer[start].trim();
            if next
                .chars()
                .all(|ch| ch.is_ascii_digit() || ch == ',' || ch == '.')
            {
                start += 1;
            }
        }
        if start < self.content_buffer.len() {
            Some(start)
        } else {
            None
        }
    }

    #[cfg(test)]
    pub fn drain_buffer(&mut self) -> Vec<String> {
        std::mem::take(&mut self.content_buffer)
    }

    #[cfg(test)]
    pub fn is_in_tool_output(&self) -> bool {
        self.in_tool_output
    }
}
