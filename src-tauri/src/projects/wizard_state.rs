use crate::projects::config_types::ProjectConfig;
use ralph_core::prd::UserStory;
use serde::{Deserialize, Serialize};

const WIZARD_STEPS: &[&str] = &["describe", "plan", "atomize", "configure", "launch"];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WizardDescribeState {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub working_directory: String,
    #[serde(default = "default_plan_agent")]
    pub plan_agent: String,
    #[serde(default)]
    pub plan_model: Option<String>,
    #[serde(default)]
    pub plan_effort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WizardPlanState {
    #[serde(default)]
    pub completed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WizardAtomizeState {
    #[serde(default)]
    pub stories_count: u32,
    #[serde(default)]
    pub stories: Vec<UserStory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalWizardSession {
    #[serde(default = "default_session_version")]
    pub version: u32,
    #[serde(default)]
    pub project_id: String,
    #[serde(default = "default_step_name")]
    pub current_step: String,
    #[serde(default = "default_first_step")]
    pub highest_step: u32,
    #[serde(default)]
    pub stale_from_step: Option<u32>,
    #[serde(default)]
    pub describe: WizardDescribeState,
    #[serde(default)]
    pub plan: WizardPlanState,
    #[serde(default)]
    pub atomize: WizardAtomizeState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configure: Option<ProjectConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guardrails: Option<String>,
}

impl Default for CanonicalWizardSession {
    fn default() -> Self {
        Self {
            version: default_session_version(),
            project_id: String::new(),
            current_step: default_step_name(),
            highest_step: default_first_step(),
            stale_from_step: None,
            describe: WizardDescribeState::default(),
            plan: WizardPlanState::default(),
            atomize: WizardAtomizeState::default(),
            configure: None,
            prompt: None,
            guardrails: None,
        }
    }
}

impl CanonicalWizardSession {
    pub fn from_payload(
        payload_json: &str,
        project_id: Option<&str>,
        fallback_step: Option<&str>,
    ) -> Result<Self, serde_json::Error> {
        let mut session = serde_json::from_str::<Self>(payload_json)?;
        session.normalize(project_id, fallback_step);
        Ok(session)
    }

    pub fn normalize(&mut self, project_id: Option<&str>, fallback_step: Option<&str>) {
        if self.project_id.trim().is_empty() {
            self.project_id = project_id.unwrap_or_default().to_string();
        }
        self.current_step = normalized_wizard_step(fallback_step, Some(self.current_step.as_str()));
        let current_step = step_name_to_number(&self.current_step).unwrap_or(1);
        if self.highest_step < current_step {
            self.highest_step = current_step;
        }
        self.atomize.stories_count = self
            .atomize
            .stories_count
            .max(self.atomize.stories.len() as u32);
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

fn default_plan_agent() -> String {
    "claude".to_string()
}

fn default_session_version() -> u32 {
    1
}

fn default_step_name() -> String {
    "describe".to_string()
}

fn default_first_step() -> u32 {
    1
}

pub fn normalized_wizard_step(fallback_step: Option<&str>, current_step: Option<&str>) -> String {
    fallback_step
        .map(str::trim)
        .filter(|step| !step.is_empty())
        .map(str::to_string)
        .or_else(|| {
            current_step
                .map(str::trim)
                .filter(|step| !step.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(default_step_name)
}

pub fn step_name_to_number(name: &str) -> Option<u32> {
    WIZARD_STEPS
        .iter()
        .position(|step| *step == name)
        .map(|index| (index as u32) + 1)
}

pub fn step_number_to_name(number: u32) -> Option<&'static str> {
    if number == 0 || number as usize > WIZARD_STEPS.len() {
        return None;
    }
    Some(WIZARD_STEPS[(number - 1) as usize])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardStepState {
    pub current_step: u32,
    pub highest_step: u32,
    #[serde(default)]
    pub stale_from_step: Option<u32>,
}

impl Default for WizardStepState {
    fn default() -> Self {
        Self {
            current_step: 1,
            highest_step: 1,
            stale_from_step: None,
        }
    }
}

impl WizardStepState {
    pub fn advance(&mut self, target_step: u32) {
        self.current_step = target_step;
        if target_step > self.highest_step {
            self.highest_step = target_step;
        }
        self.stale_from_step = None;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvanceWizardResult {
    pub current_step: u32,
    pub highest_step: u32,
    pub step_name: String,
}

pub fn advance_wizard_step(
    current_state: Option<&WizardStepState>,
    target_step: u32,
) -> Result<AdvanceWizardResult, String> {
    if target_step == 0 || target_step as usize > WIZARD_STEPS.len() {
        return Err(format!("Invalid step number: {target_step}"));
    }
    let mut state = current_state.cloned().unwrap_or_default();
    state.advance(target_step);
    let step_name =
        step_number_to_name(target_step).ok_or_else(|| format!("Unknown step: {target_step}"))?;
    Ok(AdvanceWizardResult {
        current_step: state.current_step,
        highest_step: state.highest_step,
        step_name: step_name.to_string(),
    })
}
