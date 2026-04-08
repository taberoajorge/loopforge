pub(super) fn required_trimmed(value: String, field_name: &str) -> Result<String, String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(format!("Missing required field: {field_name}"));
    }
    Ok(normalized.to_string())
}

pub(super) fn optional_trimmed(value: Option<String>) -> Option<String> {
    match value {
        Some(content) => {
            let normalized = content.trim();
            if normalized.is_empty() {
                None
            } else {
                Some(normalized.to_string())
            }
        }
        None => None,
    }
}

pub(super) fn non_empty_trimmed_list(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{non_empty_trimmed_list, optional_trimmed, required_trimmed};

    #[test]
    fn required_trimmed_rejects_empty_content() {
        let result = required_trimmed("   ".to_string(), "project_id");
        assert!(result.is_err());
    }

    #[test]
    fn required_trimmed_trims_surrounding_whitespace() {
        let result = required_trimmed("  demo  ".to_string(), "project_id").unwrap();
        assert_eq!(result, "demo");
    }

    #[test]
    fn optional_trimmed_converts_blank_to_none() {
        let result = optional_trimmed(Some("   ".to_string()));
        assert!(result.is_none());
    }

    #[test]
    fn non_empty_trimmed_list_drops_blank_values() {
        let normalized = non_empty_trimmed_list(vec![
            "alpha".to_string(),
            "  ".to_string(),
            " beta  ".to_string(),
        ]);
        assert_eq!(normalized, vec!["alpha".to_string(), "beta".to_string()]);
    }
}
