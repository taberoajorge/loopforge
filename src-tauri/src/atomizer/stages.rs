use crate::atomizer::agent_invoke::{invoke_agent, invoke_agent_with_heartbeat};
use crate::atomizer::chunking::chunk_large_plan;
use crate::atomizer::json_parse::parse_json_from_candidates;
use crate::atomizer::progress::emit_progress;
use crate::atomizer::types::{AtomizedStoryDraft, AtomizerError, ChunkSection};
use minijinja::{context, Environment};
use ralph_core::prd::Prd;
use std::path::Path;
use tauri::AppHandle;

pub(super) async fn stage_summarize(
    app: &AppHandle,
    project_id: &str,
    env: &Environment<'_>,
    plan_content: &str,
    agent: &str,
    model: Option<&str>,
    effort: Option<&str>,
    project_dir: &Path,
) -> Result<String, AtomizerError> {
    let plan_chunks = chunk_large_plan(plan_content);
    let mut condensed_parts = Vec::with_capacity(plan_chunks.len());

    for chunk in plan_chunks {
        let tmpl = env
            .get_template("summarize")
            .map_err(|err| AtomizerError::Template(err.to_string()))?;
        let prompt = tmpl
            .render(context! { plan_content => chunk })
            .map_err(|err| AtomizerError::Template(err.to_string()))?;
        let hb = Some((project_id.to_string(), 1, "summarize".to_string()));
        let result = invoke_agent_with_heartbeat(app, agent, model, effort, &prompt, project_dir, hb).await?;
        condensed_parts.push(result);
    }

    Ok(condensed_parts.join("\n\n"))
}

pub(super) async fn stage_chunk(
    app: &AppHandle,
    project_id: &str,
    env: &Environment<'_>,
    condensed_plan: &str,
    agent: &str,
    model: Option<&str>,
    effort: Option<&str>,
    project_dir: &Path,
) -> Result<Vec<ChunkSection>, AtomizerError> {
    let tmpl = env
        .get_template("chunk")
        .map_err(|err| AtomizerError::Template(err.to_string()))?;
    let prompt = tmpl
        .render(context! { condensed_plan => condensed_plan })
        .map_err(|err| AtomizerError::Template(err.to_string()))?;

    let hb = Some((project_id.to_string(), 2, "chunk".to_string()));
    let raw = invoke_agent_with_heartbeat(app, agent, model, effort, &prompt, project_dir, hb).await?;
    parse_json_from_candidates::<Vec<ChunkSection>>(&raw, '[').map_err(|(err, candidate)| {
        AtomizerError::JsonParse {
            stage: "chunk",
            detail: format!("{err}: raw={}...", &candidate[..candidate.len().min(200)]),
        }
    })
}

fn build_json_retry_prompt(section_title: &str, section_content: &str, failed_raw: &str) -> String {
    format!(
        "Your previous response was not valid JSON. You must return ONLY a JSON array of story objects.\n\n\
        Section: {section_title}\n\n\
        Previous (invalid) response (first 300 chars):\n{snippet}\n\n\
        Rewrite your response as a valid JSON array based on this section content:\n\n\
        {section_content}\n\n\
        CRITICAL: Output ONLY the JSON array. First character must be [, last character must be ].\n\
        No prose, no requests, no markdown fences.",
        snippet = &failed_raw[..failed_raw.len().min(300)]
    )
}

pub(super) async fn stage_atomize(
    app: &AppHandle,
    project_id: &str,
    env: &Environment<'_>,
    sections: &[ChunkSection],
    project_name: &str,
    agent: &str,
    model: Option<&str>,
    effort: Option<&str>,
    project_dir: &Path,
) -> Result<Vec<AtomizedStoryDraft>, AtomizerError> {
    let mut all_stories: Vec<AtomizedStoryDraft> = Vec::new();

    for section in sections {
        let tmpl = env
            .get_template("stories")
            .map_err(|err| AtomizerError::Template(err.to_string()))?;
        let prompt = tmpl
            .render(context! {
                project_name => project_name,
                section_title => section.title,
                section_content => section.content,
            })
            .map_err(|err| AtomizerError::Template(err.to_string()))?;

        let hb = Some((project_id.to_string(), 3, "atomize".to_string()));
        let raw = invoke_agent_with_heartbeat(app, agent, model, effort, &prompt, project_dir, hb).await?;

        let stories: Vec<AtomizedStoryDraft> = match parse_json_from_candidates(&raw, '[') {
            Ok(stories) => stories,
            Err((_first_err, first_candidate)) => {
                emit_progress(
                    app,
                    project_id,
                    3,
                    "atomize",
                    &format!("Retrying section '{}'...", section.title),
                );

                let retry_prompt =
                    build_json_retry_prompt(&section.title, &section.content, &first_candidate);
                let retry_raw =
                    invoke_agent(app, agent, model, effort, &retry_prompt, project_dir).await?;

                match parse_json_from_candidates(&retry_raw, '[') {
                    Ok(stories) => stories,
                    Err((retry_err, retry_candidate)) => {
                        return Err(AtomizerError::JsonParse {
                            stage: "atomize",
                            detail: format!(
                                "{retry_err} in section '{}' (after retry): raw={}...",
                                section.title,
                                &retry_candidate[..retry_candidate.len().min(200)]
                            ),
                        });
                    }
                }
            }
        };

        all_stories.extend(stories);
    }

    Ok(all_stories)
}

pub(super) async fn stage_merge(
    app: &AppHandle,
    project_id: &str,
    env: &Environment<'_>,
    stories: Vec<AtomizedStoryDraft>,
    project_name: &str,
    agent: &str,
    model: Option<&str>,
    effort: Option<&str>,
    project_dir: &Path,
) -> Result<Prd, AtomizerError> {
    let stories_json = serde_json::to_string_pretty(&stories).map_err(|err| AtomizerError::JsonParse {
        stage: "merge",
        detail: err.to_string(),
    })?;

    let generated_at = chrono::Utc::now().to_rfc3339();
    let tmpl = env
        .get_template("merge")
        .map_err(|err| AtomizerError::Template(err.to_string()))?;
    let prompt = tmpl
        .render(context! {
            project_name => project_name,
            generated_at => generated_at,
            stories_json => stories_json,
        })
        .map_err(|err| AtomizerError::Template(err.to_string()))?;

    let hb = Some((project_id.to_string(), 4, "merge".to_string()));
    let raw = invoke_agent_with_heartbeat(app, agent, model, effort, &prompt, project_dir, hb).await?;
    parse_json_from_candidates::<Prd>(&raw, '{').map_err(|(err, candidate)| AtomizerError::JsonParse {
        stage: "merge",
        detail: format!("{err}: raw={}...", &candidate[..candidate.len().min(400)]),
    })
}
