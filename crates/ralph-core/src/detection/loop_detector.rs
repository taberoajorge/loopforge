use std::collections::VecDeque;

const MAX_HISTORY: usize = 50;
const REPETITION_THRESHOLD: usize = 3;

pub struct LoopDetector {
    recent_lines: VecDeque<String>,
}

impl LoopDetector {
    pub fn new() -> Self {
        Self {
            recent_lines: VecDeque::with_capacity(MAX_HISTORY),
        }
    }

    pub fn feed_line(&mut self, line: &str) {
        let normalized = normalize_line(line);
        if normalized.is_empty() {
            return;
        }
        if self.recent_lines.len() >= MAX_HISTORY {
            self.recent_lines.pop_front();
        }
        self.recent_lines.push_back(normalized);
    }

    pub fn detect_repetition(&self) -> Option<String> {
        if self.recent_lines.len() < REPETITION_THRESHOLD {
            return None;
        }

        let last_lines: Vec<&String> = self.recent_lines.iter().rev().take(10).collect();

        for window_size in 1..=3 {
            if last_lines.len() < window_size * REPETITION_THRESHOLD {
                continue;
            }
            let pattern: Vec<&String> = last_lines[..window_size].to_vec();
            let mut repetitions = 1usize;

            for chunk_start in (window_size..last_lines.len()).step_by(window_size) {
                let chunk_end = (chunk_start + window_size).min(last_lines.len());
                let chunk: Vec<&String> = last_lines[chunk_start..chunk_end].to_vec();
                if chunk == pattern {
                    repetitions += 1;
                } else {
                    break;
                }
            }

            if repetitions >= REPETITION_THRESHOLD {
                return Some(pattern.iter().map(|line| line.as_str()).collect::<Vec<_>>().join(" | "));
            }
        }

        None
    }

    pub fn reset(&mut self) {
        self.recent_lines.clear();
    }
}

fn normalize_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("---")
        || trimmed.chars().all(|character| character == '=' || character == '-')
    {
        return String::new();
    }
    trimmed.to_lowercase()
}
