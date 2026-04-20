use crate::detection::failure_memory::FailureMemory;
use crate::guardrails;
use crate::prd::UserStory;
use std::hash::{Hash, Hasher};
use std::path::Path;
use tracing;

const MAX_PROMPT_BYTES: usize = 100_000;
const MAX_GUARDRAILS_BYTES: usize = 8_000;
const MAX_DIVERSITY_BYTES: usize = 4_000;
const MAX_STORY_JSON_BYTES: usize = 16_000;

pub struct BuiltPrompt {
    pub text: String,
    pub size_bytes: usize,
    pub hash: u64,
    pub truncated: bool,
}

impl BuiltPrompt {
    pub fn validate(&self, story_id: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.text.trim().is_empty() {
            errors.push("prompt is empty".into());
        }

        if !self.text.contains(story_id) {
            errors.push(format!("prompt does not contain story id '{story_id}'"));
        }

        if self.size_bytes > MAX_PROMPT_BYTES + 20 {
            errors.push(format!(
                "prompt size {} exceeds budget {}",
                self.size_bytes, MAX_PROMPT_BYTES
            ));
        }

        if let Some(json_start) = self.text.find("Next story to implement:\n") {
            let json_offset = json_start + "Next story to implement:\n".len();
            if let Some(json_end) = self.text[json_offset..].find("\n\n---\n") {
                let json_slice = &self.text[json_offset..json_offset + json_end];
                if serde_json::from_str::<serde_json::Value>(json_slice).is_err() {
                    errors.push("story JSON section is not valid JSON".into());
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub struct PromptBuilder<'a> {
    prompt_file: &'a Path,
    guardrails_file: &'a Path,
    ralph_dir: &'a Path,
    work_dir: &'a Path,
    iteration: u32,
    failure_memory: &'a FailureMemory,
}

impl<'a> PromptBuilder<'a> {
    pub fn new(
        prompt_file: &'a Path,
        guardrails_file: &'a Path,
        ralph_dir: &'a Path,
        work_dir: &'a Path,
        iteration: u32,
        failure_memory: &'a FailureMemory,
    ) -> Self {
        Self {
            prompt_file,
            guardrails_file,
            ralph_dir,
            work_dir,
            iteration,
            failure_memory,
        }
    }

    pub fn build(&self, story: &UserStory, bulk_checkpoint: Option<&str>) -> BuiltPrompt {
        let mut prompt = String::with_capacity(8192);
        let mut truncated = false;

        let base_prompt = std::fs::read_to_string(self.prompt_file)
            .unwrap_or_else(|_| format!("No prompt file found at {}", self.prompt_file.display()));
        prompt.push_str(&base_prompt);

        prompt.push_str("\n\n## GUARDRAILS (READ FIRST!)\n\n");
        let guardrails_content = guardrails::read_content(self.guardrails_file).unwrap_or_default();
        let guardrails_section = truncate_section(&guardrails_content, MAX_GUARDRAILS_BYTES);
        if guardrails_section.len() < guardrails_content.len() {
            truncated = true;
            tracing::warn!(
                section = "guardrails",
                original_bytes = guardrails_content.len(),
                budget = MAX_GUARDRAILS_BYTES,
                "prompt section truncated"
            );
        }
        prompt.push_str(&guardrails_section);

        if let Some(diversity_section) = self.failure_memory.build_diversity_prompt(&story.id) {
            let diversity_trimmed = truncate_section(&diversity_section, MAX_DIVERSITY_BYTES);
            if diversity_trimmed.len() < diversity_section.len() {
                truncated = true;
                tracing::warn!(
                    section = "diversity",
                    original_bytes = diversity_section.len(),
                    budget = MAX_DIVERSITY_BYTES,
                    "prompt section truncated"
                );
            }
            prompt.push_str(&diversity_trimmed);
        }

        prompt.push_str("\n\n---\n\n");
        prompt.push_str("IMPORTANT: You have full file system access. You can use bash heredoc syntax freely.\n\n");
        prompt.push_str("Next story to implement:\n");

        let story_json =
            serde_json::to_string_pretty(story).unwrap_or_else(|_| format!("{story:?}"));
        let story_trimmed = truncate_section(&story_json, MAX_STORY_JSON_BYTES);
        if story_trimmed.len() < story_json.len() {
            truncated = true;
            tracing::warn!(
                section = "story_json",
                original_bytes = story_json.len(),
                budget = MAX_STORY_JSON_BYTES,
                "prompt section truncated"
            );
        }
        prompt.push_str(&story_trimmed);

        prompt.push_str("\n\n---\n");
        prompt.push_str("## DYNAMIC CONTEXT (this section changes per iteration)\n\n");
        prompt.push_str(&format!("RALPH_DIR: {}\n", self.ralph_dir.display()));
        prompt.push_str(&format!("WORK_DIR: {}\n", self.work_dir.display()));
        prompt.push_str(&format!(
            "Current working directory: {}\n",
            self.work_dir.display()
        ));
        prompt.push_str(&format!("Config directory: {}\n", self.ralph_dir.display()));
        prompt.push_str(&format!("ITERATION: {}\n", self.iteration));

        if let Some(checkpoint) = bulk_checkpoint {
            prompt.push_str(checkpoint);
        }

        if prompt.len() > MAX_PROMPT_BYTES {
            truncated = true;
            tracing::warn!(
                total_bytes = prompt.len(),
                budget = MAX_PROMPT_BYTES,
                "total prompt exceeds budget, truncating"
            );
            prompt.truncate(MAX_PROMPT_BYTES);
            prompt.push_str("\n[...truncated...]");
        }

        let size_bytes = prompt.len();
        let hash = prompt_content_hash(&prompt);

        BuiltPrompt {
            text: prompt,
            size_bytes,
            hash,
            truncated,
        }
    }
}

fn truncate_section(content: &str, max_bytes: usize) -> String {
    if content.len() <= max_bytes {
        return content.to_string();
    }
    let mut truncated = content[..max_bytes].to_string();
    truncated.push_str("\n[...truncated...]");
    truncated
}

pub fn prompt_content_hash(text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}
