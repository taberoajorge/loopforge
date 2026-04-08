use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FailureMemory {
    pub stories: Vec<StoryFailureRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryFailureRecord {
    pub story_id: String,
    pub attempts: Vec<FailureAttempt>,
    pub diversity_required: bool,
    pub banned_approaches: Vec<String>,
    pub gutter_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureAttempt {
    pub iteration: u32,
    pub error_type: String,
    pub files_changed: Vec<String>,
    pub approach_summary: String,
}

impl FailureMemory {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        crate::atomic_write::atomic_write(path, content.as_bytes())?;
        Ok(())
    }

    pub fn get_record(&self, story_id: &str) -> Option<&StoryFailureRecord> {
        self.stories.iter().find(|record| record.story_id == story_id)
    }

    pub fn record_failure(
        &mut self,
        story_id: &str,
        iteration: u32,
        error_type: &str,
        files_changed: Vec<String>,
        approach_summary: &str,
    ) {
        let counts_toward_gutter = error_type != "spawn_error";
        let record = self
            .stories
            .iter_mut()
            .find(|record| record.story_id == story_id);

        let attempt = FailureAttempt {
            iteration,
            error_type: error_type.to_string(),
            files_changed,
            approach_summary: approach_summary.to_string(),
        };

        match record {
            Some(existing) => {
                existing.attempts.push(attempt);
                if !counts_toward_gutter {
                    return;
                }

                existing.gutter_score += 1;

                let recent_approaches: Vec<&str> = existing
                    .attempts
                    .iter()
                    .rev()
                    .take(3)
                    .map(|attempt| attempt.approach_summary.as_str())
                    .collect();
                let all_same = recent_approaches.len() >= 2
                    && recent_approaches.windows(2).all(|window| window[0] == window[1]);

                if all_same {
                    existing.diversity_required = true;
                    if let Some(last_approach) = recent_approaches.first() {
                        if !existing.banned_approaches.contains(&last_approach.to_string()) {
                            existing
                                .banned_approaches
                                .push(last_approach.to_string());
                        }
                    }
                }
            }
            None => {
                self.stories.push(StoryFailureRecord {
                    story_id: story_id.to_string(),
                    attempts: vec![attempt],
                    diversity_required: false,
                    banned_approaches: Vec::new(),
                    gutter_score: u32::from(counts_toward_gutter),
                });
            }
        }
    }

    pub fn is_in_gutter(&self, story_id: &str, threshold: u32) -> bool {
        self.get_record(story_id)
            .map(|record| record.gutter_score >= threshold)
            .unwrap_or(false)
    }

    pub fn build_diversity_prompt(&self, story_id: &str) -> Option<String> {
        let record = self.get_record(story_id)?;
        if !record.diversity_required {
            return None;
        }

        let mut prompt = String::from(
            "\n## DIVERSITY REQUIREMENT (CRITICAL)\n\n\
             Previous attempts at this story FAILED with the same approach.\n\
             You MUST try a fundamentally different strategy.\n\n",
        );

        if !record.banned_approaches.is_empty() {
            prompt.push_str("BANNED APPROACHES (do NOT repeat these):\n");
            for approach in &record.banned_approaches {
                prompt.push_str(&format!("  * {approach}\n"));
            }
        }

        prompt.push_str(&format!(
            "\nPrevious attempts: {}\n",
            record.attempts.len()
        ));

        for attempt in record.attempts.iter().rev().take(3) {
            prompt.push_str(&format!(
                "  Iteration {}: {} (files: {})\n",
                attempt.iteration,
                attempt.error_type,
                attempt.files_changed.join(", ")
            ));
        }

        Some(prompt)
    }
}
