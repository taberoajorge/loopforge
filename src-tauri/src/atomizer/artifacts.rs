use crate::atomizer::AtomizerError;
use ralph_core::prd::Prd;
use std::path::Path;

pub(super) fn save_artifacts(project_dir: &Path, prd: &Prd) -> Result<(), AtomizerError> {
    let prd_path = project_dir.join("prd.json");
    let prd_json = serde_json::to_string_pretty(prd).map_err(|err| AtomizerError::JsonParse {
        stage: "save",
        detail: err.to_string(),
    })?;
    std::fs::write(&prd_path, prd_json)?;

    let prompt_path = project_dir.join("prompt.md");
    let missing_or_empty_prompt = std::fs::read_to_string(&prompt_path)
        .map(|content| content.trim().is_empty())
        .unwrap_or(true);
    if missing_or_empty_prompt {
        let prompt_content = build_default_prompt(&prd.project_name);
        std::fs::write(&prompt_path, prompt_content)?;
    }

    let guardrails_path = project_dir.join("guardrails.md");
    let missing_or_empty_guardrails = std::fs::read_to_string(&guardrails_path)
        .map(|content| content.trim().is_empty())
        .unwrap_or(true);
    if missing_or_empty_guardrails {
        let guardrails_content = build_default_guardrails(&prd.project_name);
        std::fs::write(&guardrails_path, guardrails_content)?;
    }

    Ok(())
}

fn build_default_prompt(project_name: &str) -> String {
    format!(
        "# {project_name} — Execution Prompt\n\n\
You are an expert software engineer implementing user stories for this project.\n\n\
For each story:\n\
1. Read the story and acceptance criteria carefully\n\
2. Implement the changes described in scope.filesToModify and scope.filesToCreate\n\
3. Run verification.commands to confirm your work\n\
4. Mark the story as passed by updating prd.json: set `\"passes\": true` for the story\n\
5. Commit with the provided commitMessage\n\n\
IMPORTANT: Only modify files listed in scope.filesToModify or scope.filesToCreate.\n\
Never modify files matching patterns in scope.filesToAvoid.\n"
    )
}

fn build_default_guardrails(project_name: &str) -> String {
    format!(
        "# {project_name} — Guardrails\n\n\
- Do not modify files outside the defined scope\n\
- Run all verification commands before marking a story as passed\n\
- Commit only the changes for the current story\n\
- If verification fails, fix the issue before proceeding\n"
    )
}
