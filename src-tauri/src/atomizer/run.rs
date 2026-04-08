use crate::atomizer::artifacts::save_artifacts;
use crate::atomizer::io::{artifact_dir, load_plan_content};
use crate::atomizer::progress::emit_progress;
use crate::atomizer::sanitize::sanitize_codex_plan_content;
use crate::atomizer::stages::{stage_atomize, stage_chunk, stage_merge, stage_summarize};
use crate::atomizer::templates::load_templates;
use crate::atomizer::{AtomizeArgs, AtomizerError};
use ralph_core::prd::Prd;
use tauri::AppHandle;

pub async fn run_atomizer(app: AppHandle, args: AtomizeArgs) -> Result<Prd, AtomizerError> {
    if !args.project_dir.exists() {
        return Err(AtomizerError::Path(format!(
            "Working directory not found: {}",
            args.project_dir.display()
        )));
    }
    if !args.project_dir.is_dir() {
        return Err(AtomizerError::Path(format!(
            "Working directory is not a directory: {}",
            args.project_dir.display()
        )));
    }

    let env = load_templates()?;
    let artifact_path = artifact_dir(&app, &args.project_id)?;
    std::fs::create_dir_all(&artifact_path)?;

    let plan_content =
        sanitize_codex_plan_content(&load_plan_content(&artifact_path, &args.project_dir)?);
    if plan_content.trim().is_empty() {
        return Err(AtomizerError::Template(
            "Plan is empty. Reopen the draft from Plan or regenerate the plan before atomizing."
                .to_string(),
        ));
    }

    let pid = &args.project_id;

    emit_progress(&app, pid, 1, "summarize", "Summarizing plan...");
    let condensed = stage_summarize(
        &app,
        pid,
        &env,
        &plan_content,
        &args.agent,
        args.model.as_deref(),
        args.effort.as_deref(),
        &args.project_dir,
    )
    .await?;

    emit_progress(&app, pid, 2, "chunk", "Splitting into sections...");
    let sections = stage_chunk(
        &app,
        pid,
        &env,
        &condensed,
        &args.agent,
        args.model.as_deref(),
        args.effort.as_deref(),
        &args.project_dir,
    )
    .await?;

    emit_progress(
        &app,
        pid,
        3,
        "atomize",
        &format!("Atomizing {} sections...", sections.len()),
    );
    let all_stories = stage_atomize(
        &app,
        pid,
        &env,
        &sections,
        &args.project_name,
        &args.agent,
        args.model.as_deref(),
        args.effort.as_deref(),
        &args.project_dir,
    )
    .await?;

    emit_progress(&app, pid, 4, "merge", "Merging and ordering stories...");
    let prd = stage_merge(
        &app,
        pid,
        &env,
        all_stories,
        &args.project_name,
        &args.agent,
        args.model.as_deref(),
        args.effort.as_deref(),
        &args.project_dir,
    )
    .await?;

    prd.validate_atomicity()
        .map_err(|err| AtomizerError::Validation(err.to_string()))?;
    save_artifacts(&artifact_path, &prd)?;

    emit_progress(&app, pid, 4, "merge", &format!("Done — {} stories", prd.stories.len()));
    Ok(prd)
}
