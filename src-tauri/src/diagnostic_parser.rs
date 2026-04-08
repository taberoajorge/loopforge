use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub col: Option<u32>,
    pub error_type: String,
    pub message: String,
    pub source_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticReport {
    pub diagnostics: Vec<Diagnostic>,
    pub total: usize,
    pub has_errors: bool,
}

#[tauri::command]
pub fn parse_build_output(raw_output: String) -> DiagnosticReport {
    let diagnostics = parse_diagnostics(&raw_output);
    let total = diagnostics.len();
    let has_errors = diagnostics
        .iter()
        .any(|diag| diag.error_type != "warning");
    DiagnosticReport {
        diagnostics,
        total,
        has_errors,
    }
}

pub fn parse_diagnostics(output: &str) -> Vec<Diagnostic> {
    let mut results = Vec::new();

    results.extend(parse_typescript(output));
    results.extend(parse_eslint(output));
    results.extend(parse_cargo(output));
    results.extend(parse_jest(output));

    if results.is_empty() && !output.trim().is_empty() {
        results.push(parse_fallback(output));
    }

    results
}

fn parse_typescript(output: &str) -> Vec<Diagnostic> {
    let pattern = Regex::new(
        r"(?m)^(.+?)\((\d+),(\d+)\):\s*error\s+(TS\d+):\s*(.+)$"
    )
    .expect("valid regex");

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

fn parse_eslint(output: &str) -> Vec<Diagnostic> {
    let pattern = Regex::new(
        r"(?m)^\s*(\S+?):(\d+):(\d+):\s+(\S+)\s+(.+?)(?:\s{2,}|\t)(\S+)$"
    )
    .expect("valid regex");

    pattern
        .captures_iter(output)
        .filter(|cap| &cap[4] == "error" || &cap[4] == "warning")
        .map(|cap| Diagnostic {
            file: Some(cap[1].to_string()),
            line: cap[2].parse().ok(),
            col: cap[3].parse().ok(),
            error_type: cap[6].to_string(),
            message: cap[5].trim().to_string(),
            source_context: None,
        })
        .collect()
}

fn parse_cargo(output: &str) -> Vec<Diagnostic> {
    let error_pattern = Regex::new(
        r"(?m)^error(?:\[E(\d+)\])?:\s*(.+)\n\s*-->\s*(.+?):(\d+):(\d+)"
    )
    .expect("valid regex");

    error_pattern
        .captures_iter(output)
        .map(|cap| Diagnostic {
            file: Some(cap[3].to_string()),
            line: cap[4].parse().ok(),
            col: cap[5].parse().ok(),
            error_type: cap
                .get(1)
                .map(|matched| format!("E{}", matched.as_str()))
                .unwrap_or_else(|| "cargo_error".to_string()),
            message: cap[2].to_string(),
            source_context: None,
        })
        .collect()
}

fn parse_jest(output: &str) -> Vec<Diagnostic> {
    let fail_pattern = Regex::new(
        r"(?m)FAIL\s+(.+?)(?:\n|\r\n)"
    )
    .expect("valid regex");

    let assertion_pattern = Regex::new(
        r"(?m)Expected:?\s*(.+)\n\s*Received:?\s*(.+)"
    )
    .expect("valid regex");

    let location_pattern = Regex::new(
        r"(?m)at\s+.*?\((.+?):(\d+):(\d+)\)"
    )
    .expect("valid regex");

    let mut diagnostics = Vec::new();

    for fail_match in fail_pattern.captures_iter(output) {
        let test_file = fail_match[1].trim().to_string();

        let expected_received = assertion_pattern.captures(output).map(|cap| {
            format!("Expected: {}, Received: {}", &cap[1], &cap[2])
        });

        let (line, col) = location_pattern
            .captures(output)
            .map(|cap| (cap[2].parse::<u32>().ok(), cap[3].parse::<u32>().ok()))
            .unwrap_or((None, None));

        diagnostics.push(Diagnostic {
            file: Some(test_file),
            line,
            col,
            error_type: "jest_fail".to_string(),
            message: expected_received.unwrap_or_else(|| "Test assertion failed".to_string()),
            source_context: None,
        });
    }

    diagnostics
}

fn parse_fallback(output: &str) -> Diagnostic {
    let truncated = if output.len() > 2000 {
        format!("{}...[truncated]", &output[..2000])
    } else {
        output.to_string()
    };

    Diagnostic {
        file: None,
        line: None,
        col: None,
        error_type: "unstructured".to_string(),
        message: truncated,
        source_context: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typescript_parser_extracts_error() {
        let output = r#"src/auth.ts(42,5): error TS2322: Type 'string' is not assignable to type 'number'"#;
        let diags = parse_typescript(output);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].file.as_deref(), Some("src/auth.ts"));
        assert_eq!(diags[0].line, Some(42));
        assert_eq!(diags[0].col, Some(5));
        assert_eq!(diags[0].error_type, "TS2322");
    }

    #[test]
    fn typescript_parser_handles_multiple_errors() {
        let output = concat!(
            "src/a.ts(1,2): error TS1001: msg one\n",
            "src/b.ts(99,10): error TS2345: msg two\n",
        );
        let diags = parse_typescript(output);
        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].file.as_deref(), Some("src/a.ts"));
        assert_eq!(diags[1].error_type, "TS2345");
    }

    #[test]
    fn typescript_parser_ignores_non_matching_lines() {
        let output = "Compilation completed with 0 errors.\nAll good.";
        let diags = parse_typescript(output);
        assert!(diags.is_empty());
    }

    #[test]
    fn cargo_parser_extracts_error() {
        let output = "error[E0308]: mismatched types\n  --> src/main.rs:10:5\n";
        let diags = parse_cargo(output);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].file.as_deref(), Some("src/main.rs"));
        assert_eq!(diags[0].error_type, "E0308");
    }

    #[test]
    fn cargo_parser_handles_error_without_code() {
        let output = "error: could not compile `my_crate`\n  --> src/lib.rs:3:1\n";
        let diags = parse_cargo(output);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].error_type, "cargo_error");
        assert_eq!(diags[0].line, Some(3));
    }

    #[test]
    fn cargo_parser_handles_multiple_errors() {
        let output = concat!(
            "error[E0308]: mismatched types\n  --> src/a.rs:5:3\n",
            "error[E0599]: no method named `foo`\n  --> src/b.rs:12:8\n",
        );
        let diags = parse_cargo(output);
        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].file.as_deref(), Some("src/a.rs"));
        assert_eq!(diags[1].file.as_deref(), Some("src/b.rs"));
    }

    #[test]
    fn jest_parser_extracts_fail() {
        let output = "FAIL src/utils.test.ts\n  Expected: 42\n  Received: 43\n    at Object.<anonymous> (src/utils.test.ts:15:3)\n";
        let diags = parse_jest(output);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].file.as_deref(), Some("src/utils.test.ts"));
        assert_eq!(diags[0].error_type, "jest_fail");
    }

    #[test]
    fn jest_parser_extracts_expected_received() {
        let output = "FAIL src/math.test.ts\n  Expected: 100\n  Received: 99\n    at Object.<anonymous> (src/math.test.ts:8:5)\n";
        let diags = parse_jest(output);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("Expected: 100"));
        assert!(diags[0].message.contains("Received: 99"));
        assert_eq!(diags[0].line, Some(8));
        assert_eq!(diags[0].col, Some(5));
    }

    #[test]
    fn jest_parser_no_match_on_pass() {
        let output = "PASS src/utils.test.ts\n  ✓ adds correctly (5ms)\n";
        let diags = parse_jest(output);
        assert!(diags.is_empty());
    }

    #[test]
    fn fallback_parser_captures_unstructured() {
        let output = "Something went wrong\nNo pattern match";
        let diags = parse_diagnostics(output);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].error_type, "unstructured");
    }

    #[test]
    fn fallback_truncates_long_output() {
        let output = "x".repeat(3000);
        let diag = parse_fallback(&output);
        assert!(diag.message.len() < 2100);
        assert!(diag.message.contains("[truncated]"));
    }

    #[test]
    fn parse_diagnostics_empty_input() {
        let diags = parse_diagnostics("");
        assert!(diags.is_empty());
    }

    #[test]
    fn parse_diagnostics_whitespace_only() {
        let diags = parse_diagnostics("   \n  \n ");
        assert!(diags.is_empty());
    }

    #[test]
    fn parse_diagnostics_mixed_typescript_and_cargo() {
        let output = concat!(
            "src/auth.ts(10,3): error TS9999: something\n",
            "error[E0308]: mismatched types\n  --> src/main.rs:1:1\n",
        );
        let diags = parse_diagnostics(output);
        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].error_type, "TS9999");
        assert_eq!(diags[1].error_type, "E0308");
    }

    #[test]
    fn parse_build_output_command_returns_report() {
        let output = "src/x.ts(1,1): error TS0001: oops".to_string();
        let report = parse_build_output(output);
        assert_eq!(report.total, 1);
        assert!(report.has_errors);
        assert_eq!(report.diagnostics[0].error_type, "TS0001");
    }

    #[test]
    fn parse_build_output_empty_returns_no_errors() {
        let report = parse_build_output(String::new());
        assert_eq!(report.total, 0);
        assert!(!report.has_errors);
    }
}
