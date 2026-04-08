use crate::activity::types::PlanEventKind;

use super::ActivityClassifier;

impl ActivityClassifier {
    pub(super) fn codex_classify(&mut self, line: &str) -> Option<PlanEventKind> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        self.line_count += 1;

        if Self::is_stdin_wait_warning(trimmed) {
            self.reset_plan_state();
            return Some(PlanEventKind::Thinking);
        }

        if trimmed.contains("rmcp::transport")
            || trimmed.contains("AuthRequired(")
            || trimmed.contains("JsonRpcMessage")
        {
            self.reset_plan_state();
            return Some(PlanEventKind::McpCall);
        }

        if trimmed.contains(" ERROR ") || trimmed.starts_with("ERROR ") {
            self.in_plan_markdown = false;
            return Some(PlanEventKind::Error);
        }

        if self.is_codex_header(trimmed) {
            self.reset_plan_state();
            return Some(PlanEventKind::Thinking);
        }

        if trimmed == "exec"
            || trimmed.starts_with("exec ")
            || trimmed.starts_with("/bin/zsh -lc")
        {
            self.in_tool_output = true;
            self.in_plan_markdown = false;
            self.saw_agent_response = true;
            return Some(PlanEventKind::McpCall);
        }

        if (trimmed.starts_with("succeeded in ") || trimmed.starts_with("failed in "))
            && trimmed.contains("ms")
        {
            self.in_tool_output = true;
            self.in_plan_markdown = false;
            return Some(PlanEventKind::McpCall);
        }

        if trimmed == "codex" || trimmed.starts_with("codex ") {
            self.reset_plan_state();
            self.saw_agent_response = true;
            return Some(PlanEventKind::Thinking);
        }

        if trimmed == "user" || trimmed.starts_with("user ") {
            self.reset_plan_state();
            return Some(PlanEventKind::Thinking);
        }

        if trimmed.starts_with("Plan update") {
            self.reset_plan_state();
            self.saw_agent_response = true;
            return Some(PlanEventKind::Thinking);
        }

        if Self::is_prompt_echo(trimmed) {
            self.reset_plan_state();
            return Some(PlanEventKind::Thinking);
        }

        if self.in_tool_output {
            return Some(PlanEventKind::McpCall);
        }

        if self.patterns.search.iter().any(|re| re.is_match(trimmed)) {
            self.saw_agent_response = true;
            return Some(PlanEventKind::Search);
        }

        if self.patterns.docs_lookup.iter().any(|re| re.is_match(trimmed)) {
            self.saw_agent_response = true;
            return Some(PlanEventKind::DocsLookup);
        }

        if self.patterns.mcp_call.iter().any(|re| re.is_match(trimmed)) {
            return Some(PlanEventKind::McpCall);
        }

        let can_be_plan = self.saw_agent_response
            && self.line_count > super::PLAN_UNLOCK_LINE_THRESHOLD;

        if can_be_plan && Self::looks_like_markdown_plan_line(trimmed) {
            self.in_plan_markdown = true;
            return Some(PlanEventKind::PlanContent);
        }

        if can_be_plan && self.in_plan_markdown {
            return Some(PlanEventKind::PlanContent);
        }

        self.in_plan_markdown = false;
        Some(PlanEventKind::Thinking)
    }

    fn is_codex_header(&self, trimmed: &str) -> bool {
        trimmed.starts_with("OpenAI Codex")
            || trimmed.starts_with("workdir:")
            || trimmed.starts_with("model:")
            || trimmed.starts_with("provider:")
            || trimmed.starts_with("approval:")
            || trimmed.starts_with("sandbox:")
            || trimmed.starts_with("reasoning effort:")
            || trimmed.starts_with("reasoning summaries:")
            || trimmed.starts_with("session id:")
            || trimmed == "--------"
            || trimmed == "---"
    }

    fn reset_plan_state(&mut self) {
        self.in_tool_output = false;
        self.in_plan_markdown = false;
    }
}
