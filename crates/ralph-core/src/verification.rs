use crate::prd::UserStory;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use tokio::process::Command;

const DENIED_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "sudo rm -rf",
    "mkfs",
    "dd if=",
    ":(){ :|:& };:",
    "> /dev/sda",
    "chmod -R 777 /",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub col: Option<u32>,
    pub error_type: String,
    pub message: String,
    pub source_context: Option<String>,
}

#[derive(Debug)]
pub struct VerificationResult {
    pub passed: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub raw_output: String,
}

pub async fn run_verification(story: &UserStory, work_dir: &Path) -> VerificationResult {
    let commands = &story.verification.commands;
    if commands.is_empty() {
        return VerificationResult {
            passed: true,
            diagnostics: vec![],
            raw_output: String::new(),
        };
    }

    let mut all_diagnostics = Vec::new();
    let mut combined_output = String::new();
    let mut all_passed = true;

    for cmd in commands {
        if let Err(reason) = validate_command(cmd) {
            all_passed = false;
            all_diagnostics.push(Diagnostic {
                file: None,
                line: None,
                col: None,
                error_type: "blocked_command".into(),
                message: reason,
                source_context: None,
            });
            combined_output.push_str(&format!("command blocked: {cmd}\n"));
            continue;
        }

        let result = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(work_dir)
            .output()
            .await;

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let full_output = format!("{stdout}{stderr}");
                combined_output.push_str(&full_output);
                combined_output.push('\n');

                if !output.status.success() {
                    all_passed = false;
                    let mut parsed = parse_diagnostics(&full_output);
                    if parsed.is_empty() {
                        parsed.push(Diagnostic {
                            file: None,
                            line: None,
                            col: None,
                            error_type: "unstructured".into(),
                            message: full_output.chars().take(2000).collect(),
                            source_context: None,
                        });
                    }
                    all_diagnostics.extend(parsed);
                }
            }
            Err(spawn_err) => {
                all_passed = false;
                all_diagnostics.push(Diagnostic {
                    file: None,
                    line: None,
                    col: None,
                    error_type: "spawn_error".into(),
                    message: format!("Failed to run `{cmd}`: {spawn_err}"),
                    source_context: None,
                });
                combined_output.push_str(&format!("spawn error: {spawn_err}\n"));
            }
        }
    }

    VerificationResult {
        passed: all_passed,
        diagnostics: all_diagnostics,
        raw_output: combined_output,
    }
}

fn parse_diagnostics(output: &str) -> Vec<Diagnostic> {
    let mut results = Vec::new();
    results.extend(parse_typescript(output));
    results.extend(parse_cargo(output));
    results.extend(parse_eslint(output));
    results.extend(parse_jest(output));
    results
}

fn parse_typescript(output: &str) -> Vec<Diagnostic> {
    let pattern =
        Regex::new(r"(?m)^(.+?)\((\d+),(\d+)\):\s*error\s+(TS\d+):\s*(.+)$").expect("valid regex");

    pattern
        .captures_iter(output)
        .map(|cap| Diagnostic {
            file: Some(cap[1].to_string()),
            line: cap[2].parse().ok(),
            col: cap[3].parse().ok(),
            error_type: cap[4].to_string(),
            message: cap[5].to_string(),
            source_context: None,
        })
        .collect()
}

fn parse_cargo(output: &str) -> Vec<Diagnostic> {
    let pattern = Regex::new(r"(?m)^error\[([A-Z]\d+)\]:\s*(.+)\n\s*-->\s*(.+?):(\d+):(\d+)")
        .expect("valid regex");

    pattern
        .captures_iter(output)
        .map(|cap| Diagnostic {
            file: Some(cap[3].to_string()),
            line: cap[4].parse().ok(),
            col: cap[5].parse().ok(),
            error_type: cap[1].to_string(),
            message: cap[2].to_string(),
            source_context: None,
        })
        .collect()
}

fn parse_eslint(output: &str) -> Vec<Diagnostic> {
    let file_pattern = Regex::new(r"(?m)^(/[^\s]+|[A-Za-z]:\\[^\s]+)$").expect("valid regex");
    let rule_pattern = Regex::new(r"(?m)^\s+(\d+):(\d+)\s+(error|warning)\s+(.+?)\s{2,}(\S+)$")
        .expect("valid regex");

    let mut results = Vec::new();
    let mut current_file: Option<String> = None;

    for line in output.lines() {
        if let Some(cap) = file_pattern.captures(line) {
            current_file = Some(cap[0].to_string());
        } else if let Some(cap) = rule_pattern.captures(line) {
            results.push(Diagnostic {
                file: current_file.clone(),
                line: cap[1].parse().ok(),
                col: cap[2].parse().ok(),
                error_type: cap[5].to_string(),
                message: cap[4].to_string(),
                source_context: None,
            });
        }
    }
    results
}

fn parse_jest(output: &str) -> Vec<Diagnostic> {
    let pattern = Regex::new(r"(?m)●\s+(.+?)\s*\n.*?\n\s+at\s+.*?\((.+?):(\d+):(\d+)\)")
        .expect("valid regex");

    pattern
        .captures_iter(output)
        .map(|cap| Diagnostic {
            file: Some(cap[2].to_string()),
            line: cap[3].parse().ok(),
            col: cap[4].parse().ok(),
            error_type: "test_failure".to_string(),
            message: cap[1].to_string(),
            source_context: None,
        })
        .collect()
}

pub fn build_retry_prompt(
    story: &UserStory,
    diagnostics: &[Diagnostic],
    attempt: u32,
    max_attempts: u32,
    work_dir: &Path,
) -> String {
    let mut prompt = String::with_capacity(4096);

    prompt.push_str("You are fixing a story that failed verification. ");
    prompt.push_str("Study the diagnostics carefully and fix only the identified issues.\n\n");

    prompt.push_str("## Story Context\n\n");
    prompt.push_str(&format!("**ID:** {}\n", story.id));
    prompt.push_str(&format!("**Title:** {}\n", story.title));
    if let Some(desc) = &story.description {
        prompt.push_str(&format!("**Description:** {desc}\n"));
    }
    prompt.push_str(&format!(
        "\n## Verification Failure (Attempt {attempt} of {max_attempts})\n\n"
    ));

    for (idx, diag) in diagnostics.iter().enumerate() {
        prompt.push_str(&format!("### Diagnostic {}\n", idx + 1));
        if let Some(file) = &diag.file {
            prompt.push_str(&format!("**File:** `{file}`"));
            if let Some(line) = diag.line {
                prompt.push_str(&format!(":{line}"));
                if let Some(col) = diag.col {
                    prompt.push_str(&format!(":{col}"));
                }
            }
            prompt.push('\n');
        }
        prompt.push_str(&format!("**Type:** `{}`\n", diag.error_type));
        prompt.push_str(&format!("**Message:** {}\n\n", diag.message));
    }

    let affected_files: Vec<&String> = diagnostics
        .iter()
        .filter_map(|diag| diag.file.as_ref())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    if !affected_files.is_empty() {
        prompt.push_str("## Affected Source Files\n\n");
        for file_path in &affected_files {
            let abs_path = work_dir.join(file_path);
            if let Ok(content) = std::fs::read_to_string(&abs_path) {
                let truncated: String = content.chars().take(4000).collect();
                prompt.push_str(&format!("### `{file_path}`\n```\n{truncated}\n```\n\n"));
            }
        }
    }

    prompt.push_str("## Instructions\n\n");
    prompt.push_str("1. Analyze each diagnostic above\n");
    prompt.push_str("2. Fix the root cause in the affected files\n");
    prompt.push_str("3. Do NOT introduce new issues\n");
    prompt.push_str("4. Run the verification commands to confirm the fix:\n");
    for cmd in &story.verification.commands {
        prompt.push_str(&format!("   `{cmd}`\n"));
    }

    prompt
}

pub fn classify_error_signature(diagnostics: &[Diagnostic]) -> String {
    let mut tuples: Vec<String> = diagnostics
        .iter()
        .map(|d| {
            format!(
                "{}:{}:{}",
                d.error_type,
                d.file.as_deref().unwrap_or("?"),
                d.line.unwrap_or(0)
            )
        })
        .collect();
    tuples.sort();
    tuples.dedup();
    let mut hasher = DefaultHasher::new();
    tuples.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn validate_command(cmd: &str) -> Result<(), String> {
    let lower = cmd.to_lowercase();
    for pattern in DENIED_PATTERNS {
        if lower.contains(pattern) {
            return Err(format!(
                "blocked: command matches denied pattern '{pattern}'"
            ));
        }
    }
    Ok(())
}
