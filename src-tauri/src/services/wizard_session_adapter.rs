use crate::projects::artifacts::{artifact_dir, non_empty_file_content};
use crate::projects::config_types::ProjectConfig;
use crate::projects::documents::load_text_artifact_with_legacy_fallback;
use crate::projects::runtime_config::load_project_config_from_paths;
use crate::projects::stories_crud::load_prd_from_paths;
use crate::projects::{Project, ProjectError, WizardProjectData};
use ralph_core::prd::UserStory;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use tauri::{AppHandle, Runtime};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardStepValidation {
    pub valid: bool,
    #[serde(default)]
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardSessionValidation {
    pub describe: WizardStepValidation,
    pub plan: WizardStepValidation,
    pub atomize: WizardStepValidation,
    pub configure: WizardStepValidation,
    pub launch: WizardStepValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalWizardSession {
    pub project: Project,
    pub wizard_step: String,
    pub highest_step: u32,
    pub project_data: WizardProjectData,
    pub plan: Option<String>,
    pub plan_complete: bool,
    pub stories: Vec<UserStory>,
    pub config: Option<ProjectConfig>,
    pub prompt: Option<String>,
    pub guardrails: Option<String>,
    pub validation: WizardSessionValidation,
}

pub fn hydrate_canonical_session<R: Runtime>(
    app: &AppHandle<R>,
    project: &Project,
    wizard_state_json: Option<&str>,
) -> Result<CanonicalWizardSession, ProjectError> {
    let artifacts = artifact_dir(app, &project.id)?;
    let working_directory = Path::new(&project.working_directory);
    let wizard_step = resolved_wizard_step(project, wizard_state_json);
    let mut session = CanonicalWizardSession {
        project: project.clone(),
        highest_step: crate::projects::wizard_state::step_name_to_number(&wizard_step).unwrap_or(1),
        wizard_step,
        project_data: WizardProjectData {
            name: project.name.clone(),
            description: project.description.clone(),
            working_directory: project.working_directory.clone(),
            plan_agent: "claude".to_string(),
            plan_model: None,
            plan_effort: None,
        },
        plan: None,
        plan_complete: false,
        stories: Vec::new(),
        config: None,
        prompt: None,
        guardrails: None,
        validation: valid_validation(),
    };

    hydrate_draft(&artifacts, wizard_state_json, &mut session);
    load_plan(&artifacts, working_directory, &mut session);
    load_stories(&artifacts, working_directory, &mut session);
    load_config(&artifacts, working_directory, &mut session);
    load_prompt(&artifacts, working_directory, &mut session);
    load_guardrails(&artifacts, working_directory, &mut session);

    Ok(session)
}

fn hydrate_draft(
    artifacts: &Path,
    wizard_state_json: Option<&str>,
    session: &mut CanonicalWizardSession,
) {
    let raw = match non_empty_file_content(&artifacts.join("draft.json")) {
        Ok(content) => content.or_else(|| wizard_state_json.map(str::to_string)),
        Err(err) => {
            invalidate(&mut session.validation.describe, err.to_string());
            wizard_state_json.map(str::to_string)
        }
    };
    let Some(raw) = raw else {
        return;
    };
    let Ok(draft) = serde_json::from_str::<Value>(&raw) else {
        invalidate(
            &mut session.validation.describe,
            "draft.json is not valid JSON",
        );
        return;
    };
    let describe = draft.get("describe").unwrap_or(&draft);
    set_text(&mut session.project_data.name, describe.get("name"));
    set_text(
        &mut session.project_data.description,
        describe.get("description"),
    );
    set_text(
        &mut session.project_data.working_directory,
        describe.get("workingDirectory"),
    );
    set_text(
        &mut session.project_data.plan_agent,
        describe.get("planAgent"),
    );
    session.project_data.plan_model = string_field(describe.get("planModel"));
    session.project_data.plan_effort = string_field(describe.get("planEffort"));
    session.highest_step = draft
        .get("highestStep")
        .and_then(Value::as_u64)
        .map_or(session.highest_step, |value| value as u32);
    if draft
        .get("plan")
        .and_then(|value| value.get("completed"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        session.plan_complete = true;
    }
    if let Some(configure) = draft.get("configure").cloned() {
        match serde_json::from_value::<ProjectConfig>(configure) {
            Ok(config) => session.config = Some(config),
            Err(err) => invalidate(&mut session.validation.configure, err.to_string()),
        }
    }
}

fn load_plan(artifacts: &Path, working_directory: &Path, session: &mut CanonicalWizardSession) {
    let mut failed = false;
    match load_text_artifact_with_legacy_fallback(artifacts, working_directory, "plan.md") {
        Ok(plan) => session.plan = plan,
        Err(err) => {
            failed = true;
            invalidate(&mut session.validation.plan, err.to_string());
        }
    }
    session.plan_complete |= session.plan.is_some();
    if session.plan.is_none() && !failed {
        invalidate(&mut session.validation.plan, "plan.md is missing");
    }
}

fn load_stories(artifacts: &Path, working_directory: &Path, session: &mut CanonicalWizardSession) {
    match load_prd_from_paths(artifacts, working_directory) {
        Ok(Some(prd)) => session.stories = prd.stories,
        Ok(None) => invalidate(&mut session.validation.atomize, "prd.json is missing"),
        Err(err) => invalidate(&mut session.validation.atomize, err.to_string()),
    }
}

fn load_config(artifacts: &Path, working_directory: &Path, session: &mut CanonicalWizardSession) {
    match load_project_config_from_paths(artifacts, working_directory) {
        Ok(Some(config)) => session.config = Some(config),
        Ok(None) if session.config.is_none() => {
            invalidate(&mut session.validation.configure, "config.json is missing");
        }
        Err(err) => invalidate(&mut session.validation.configure, err.to_string()),
        Ok(None) => {}
    }
}

fn load_prompt(artifacts: &Path, working_directory: &Path, session: &mut CanonicalWizardSession) {
    let mut failed = false;
    match load_text_artifact_with_legacy_fallback(artifacts, working_directory, "prompt.md") {
        Ok(prompt) => session.prompt = prompt,
        Err(err) => {
            failed = true;
            invalidate(&mut session.validation.launch, err.to_string());
        }
    }
    if session.prompt.is_none() && !failed {
        invalidate(&mut session.validation.launch, "prompt.md is missing");
    }
}

fn load_guardrails(
    artifacts: &Path,
    working_directory: &Path,
    session: &mut CanonicalWizardSession,
) {
    let mut failed = false;
    match load_text_artifact_with_legacy_fallback(artifacts, working_directory, "guardrails.md") {
        Ok(guardrails) => session.guardrails = guardrails,
        Err(err) => {
            failed = true;
            invalidate(&mut session.validation.launch, err.to_string());
        }
    }
    if session.guardrails.is_none() && !failed {
        invalidate(&mut session.validation.launch, "guardrails.md is missing");
    }
}

fn resolved_wizard_step(project: &Project, wizard_state_json: Option<&str>) -> String {
    project
        .wizard_step
        .clone()
        .or_else(|| wizard_state_json.and_then(extract_step))
        .or_else(|| (project.status == "draft").then(|| "describe".to_string()))
        .unwrap_or_else(|| "describe".to_string())
}

fn extract_step(raw: &str) -> Option<String> {
    serde_json::from_str::<Value>(raw).ok().and_then(|value| {
        value
            .get("currentStep")
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

fn string_field(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(str::to_string)
}

fn set_text(target: &mut String, value: Option<&Value>) {
    if let Some(value) = value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        *target = value.to_string();
    }
}

fn valid_validation() -> WizardSessionValidation {
    WizardSessionValidation {
        describe: valid_step(),
        plan: valid_step(),
        atomize: valid_step(),
        configure: valid_step(),
        launch: valid_step(),
    }
}

fn valid_step() -> WizardStepValidation {
    WizardStepValidation {
        valid: true,
        issues: Vec::new(),
    }
}

fn invalidate(step: &mut WizardStepValidation, issue: impl Into<String>) {
    step.valid = false;
    step.issues.push(issue.into());
}
