use std::path::Path;

const GUARDRAILS_TEMPLATE: &str = r#"# Guardrails (Signs)

Lessons learned from previous iterations. The agent MUST read this FIRST.

## Active Guardrails

(None yet)

---

## How to Add a Guardrail

When something fails repeatedly, add a sign:

### Sign: [Short description]
- **Trigger**: [When it applies]
- **Instruction**: [What to do instead]
- **Added after**: Iteration N
"#;

pub fn ensure_exists(path: &Path) {
    if !path.exists() {
        let _ = std::fs::write(path, GUARDRAILS_TEMPLATE);
    }
}

pub fn read_content(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

pub fn add_guardrail(path: &Path, story_id: &str, error_msg: &str, iteration: u32) {
    if let Ok(content) = std::fs::read_to_string(path) {
        let marker = format!("Sign: Error in {story_id}");
        if content.contains(&marker) {
            return;
        }
    }

    let entry = format!(
        "\n### Sign: Error in {story_id}\n\
         - **Trigger**: When working on this story\n\
         - **Instruction**: Review previous errors before attempting: {error_msg}\n\
         - **Added after**: Iteration {iteration}\n"
    );

    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        use std::io::Write;
        let _ = file.write_all(entry.as_bytes());
    }
}

pub fn has_guardrail_for(path: &Path, story_id: &str) -> bool {
    std::fs::read_to_string(path)
        .map(|content| content.contains(&format!("Sign: Error in {story_id}")))
        .unwrap_or(false)
}
