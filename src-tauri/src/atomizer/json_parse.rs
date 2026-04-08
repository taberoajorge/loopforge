use serde::de::DeserializeOwned;

pub(super) fn strip_json_fences(text: &str) -> &str {
    let trimmed = text.trim();
    if trimmed.starts_with("```json") {
        trimmed
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
    } else if trimmed.starts_with("```") {
        trimmed
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
    } else {
        trimmed
    }
}

pub(super) fn extract_json_candidates(text: &str, preferred_opening: char) -> Vec<String> {
    let stripped = strip_json_fences(text).trim();
    if stripped.is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    let indices = stripped.char_indices();

    for (start_index, ch) in indices {
        if ch != preferred_opening {
            continue;
        }

        let mut stack: Vec<char> = Vec::new();
        let mut in_string = false;
        let mut escaped = false;
        let slice = &stripped[start_index..];

        for (offset, current) in slice.char_indices() {
            if in_string {
                if escaped {
                    escaped = false;
                    continue;
                }
                if current == '\\' {
                    escaped = true;
                    continue;
                }
                if current == '"' {
                    in_string = false;
                }
                continue;
            }

            match current {
                '"' => in_string = true,
                '[' => stack.push(']'),
                '{' => stack.push('}'),
                ']' | '}' => {
                    if matches!(stack.last(), Some(expected) if *expected == current) {
                        stack.pop();
                    } else {
                        break;
                    }
                    if stack.is_empty() {
                        let end_index = start_index + offset + current.len_utf8();
                        candidates.push(stripped[start_index..end_index].to_string());
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    if candidates.is_empty() {
        return vec![stripped.to_string()];
    }

    candidates
}

pub(super) fn parse_json_from_candidates<T: DeserializeOwned>(
    text: &str,
    preferred_opening: char,
) -> Result<T, (String, String)> {
    let candidates = extract_json_candidates(text, preferred_opening);
    let mut last_error = String::new();
    let mut last_candidate = String::new();

    for candidate in candidates {
        match serde_json::from_str::<T>(&candidate) {
            Ok(parsed) => return Ok(parsed),
            Err(err) => {
                last_error = err.to_string();
                last_candidate = candidate;
            }
        }
    }

    Err((last_error, last_candidate))
}
