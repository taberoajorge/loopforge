use regex::Regex;

pub(super) struct ClassifierPatterns {
    pub(super) search: Vec<Regex>,
    pub(super) docs_lookup: Vec<Regex>,
    pub(super) mcp_call: Vec<Regex>,
    pub(super) thinking: Vec<Regex>,
    pub(super) error: Vec<Regex>,
}

impl ClassifierPatterns {
    pub(super) fn for_provider(provider: &str) -> Self {
        match provider {
            "codex" => Self::codex_patterns(),
            "cursor" => Self::cursor_patterns(),
            "gemini" => Self::gemini_patterns(),
            _ => Self::default_patterns(),
        }
    }

    fn default_patterns() -> Self {
        Self {
            search: vec![
                Regex::new(r"(?i)(searching|web search|looking up|fetching)\b").unwrap(),
                Regex::new(r"(?i)\bsearch\(").unwrap(),
            ],
            docs_lookup: vec![
                Regex::new(r"(?i)(reading docs|documentation|man page|reference)\b").unwrap(),
                Regex::new(r"(?i)\b(fetch|get)_docs?\b").unwrap(),
            ],
            mcp_call: vec![
                Regex::new(r"(?i)(mcp|tool_call|calling tool|use_mcp)\b").unwrap(),
                Regex::new(r"(?i)<tool_call>").unwrap(),
                Regex::new(r"(?i)\btool:\s*\w+").unwrap(),
            ],
            thinking: vec![
                Regex::new(r"(?i)(thinking|let me think|i need to|i'll|analyzing)\b").unwrap(),
                Regex::new(r"(?i)<thinking>").unwrap(),
                Regex::new(r"(?i)^\s*\.\.\.\s*$").unwrap(),
            ],
            error: vec![
                Regex::new(r"(?i)(error|exception|failed|failure|fatal)\b").unwrap(),
                Regex::new(r"(?i)^error:").unwrap(),
            ],
        }
    }

    fn codex_patterns() -> Self {
        let mut base = Self::default_patterns();
        base.search
            .push(Regex::new(r#"(?i)"type"\s*:\s*"search""#).unwrap());
        base.mcp_call
            .push(Regex::new(r#"(?i)"type"\s*:\s*"function""#).unwrap());
        base.thinking
            .push(Regex::new(r#"(?i)"type"\s*:\s*"reasoning""#).unwrap());
        base
    }

    fn cursor_patterns() -> Self {
        let mut base = Self::default_patterns();
        base.mcp_call
            .push(Regex::new(r"(?i)stream-json.*tool").unwrap());
        base
    }

    fn gemini_patterns() -> Self {
        let mut base = Self::default_patterns();
        base.search
            .push(Regex::new(r"(?i)google_search\(").unwrap());
        base.mcp_call
            .push(Regex::new(r"(?i)function_call\s*\{").unwrap());
        base.thinking
            .push(Regex::new(r"(?i)<gemini_thoughts>").unwrap());
        base
    }
}
