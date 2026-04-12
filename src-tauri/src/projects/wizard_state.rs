use serde::{Deserialize, Serialize};

const WIZARD_STEPS: &[&str] = &["describe", "plan", "atomize", "configure", "launch"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardStepState {
    pub current_step: u32,
    pub highest_step: u32,
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

    pub fn mark_stale(&mut self, from_step: u32) {
        self.stale_from_step = Some(from_step);
    }

    pub fn clear_stale(&mut self) {
        self.stale_from_step = None;
    }
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

    let step_name = step_number_to_name(target_step)
        .ok_or_else(|| format!("Unknown step: {target_step}"))?;

    Ok(AdvanceWizardResult {
        current_step: state.current_step,
        highest_step: state.highest_step,
        step_name: step_name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_updates_highest() {
        let mut state = WizardStepState::default();
        state.advance(3);
        assert_eq!(state.current_step, 3);
        assert_eq!(state.highest_step, 3);
    }

    #[test]
    fn advance_clears_stale() {
        let mut state = WizardStepState::default();
        state.mark_stale(2);
        state.advance(3);
        assert!(state.stale_from_step.is_none());
    }

    #[test]
    fn step_name_roundtrip() {
        assert_eq!(step_name_to_number("plan"), Some(2));
        assert_eq!(step_number_to_name(2), Some("plan"));
    }
}
