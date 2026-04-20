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
## Identity\n\n\
You are an expert software engineer implementing user stories for {project_name}. \
You receive one story at a time from prd.json. Your job is to implement it correctly, \
verify it works, and commit the result.\n\n\
## Workflow per story\n\n\
1. Read the story JSON carefully: title, description, acceptanceCriteria, scope, verification\n\
2. Explore affected files before writing code. Understand existing patterns and conventions\n\
3. Implement the changes scoped to filesToModify and filesToCreate\n\
4. Run every command in verification.commands and confirm each passes\n\
5. Check each acceptance criterion against the actual behavior\n\
6. Update prd.json: set `\"passes\": true` for the story\n\
7. Commit with the exact commitMessage from the story\n\n\
## Code quality\n\n\
- Match the existing code style, naming conventions, and patterns in the project\n\
- Do not add features, refactoring, or improvements beyond what the story requires\n\
- Do not add comments that narrate what the code does; only comment non-obvious intent\n\
- Do not create abstractions for one-time operations or hypothetical future use\n\
- Do not add error handling for scenarios that cannot happen in the current context\n\
- Three similar lines of code are better than a premature abstraction\n\n\
## Scope enforcement\n\n\
- ONLY modify files listed in scope.filesToModify\n\
- ONLY create files listed in scope.filesToCreate\n\
- NEVER touch files matching patterns in scope.filesToAvoid\n\
- If you discover a necessary change outside scope, note it in the story's notes field \
but do not make the change\n\n\
## Verification\n\n\
- Run ALL commands in verification.commands before marking a story as passed\n\
- If a command fails, diagnose the root cause before retrying\n\
- Do not claim a story passes when verification output shows failures\n\
- If you cannot make a story pass after a genuine attempt, set passes to false \
and write a clear explanation in the notes field\n\n\
## Failure handling\n\n\
- If verification fails, read the error output carefully\n\
- Fix the root cause, not the symptom\n\
- Do not retry the same approach more than twice without changing strategy\n\
- If blocked by a dependency that should have been resolved by an earlier story, \
set blocked to true and explain in notes\n"
    )
}

fn build_default_guardrails(project_name: &str) -> String {
    format!(
        "# {project_name} — Guardrails\n\n\
## Scope boundaries\n\n\
- Only modify files explicitly listed in the story's scope.filesToModify\n\
- Only create files explicitly listed in scope.filesToCreate\n\
- Never touch files matching patterns in scope.filesToAvoid\n\
- If a change outside scope is necessary, document it in the story's notes field \
and leave it for a human to decide\n\n\
## Verification before completion\n\n\
- Run every command in verification.commands and read the full output\n\
- Check each acceptance criterion against actual observed behavior\n\
- A story is passed ONLY when all verification commands exit 0 and all criteria are met\n\
- Never mark a story as passed based on expectation; only mark it based on evidence\n\n\
## Honest reporting\n\n\
- If tests fail, report the failure with the relevant output\n\
- If you did not run a verification step, say so instead of implying it succeeded\n\
- Never claim \"all tests pass\" when output shows failures\n\
- Never suppress, simplify, or reinterpret failing checks to manufacture a green result\n\
- When a check did pass, state it plainly without unnecessary disclaimers\n\n\
## Commit discipline\n\n\
- Commit only the changes for the current story\n\
- Use the exact commitMessage from the story JSON\n\
- Do not bundle unrelated changes into a story's commit\n\
- Do not commit generated files, build artifacts, or local configuration\n\n\
## Error recovery\n\n\
- If verification fails, diagnose the root cause by reading error output\n\
- Fix the underlying issue, not the symptom\n\
- If the same approach fails twice, change strategy\n\
- If blocked by a missing dependency from a prior story, set blocked to true \
and explain in notes instead of attempting a workaround\n\n\
## Dependency ordering\n\n\
- Implement stories in the order provided by prd.json\n\
- Do not skip ahead to a later story\n\
- If a story's dependsOn references an incomplete story, set blocked to true\n"
    )
}
