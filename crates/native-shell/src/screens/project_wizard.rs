use super::super::theme::palette::ThemePalette;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectWizardState {
    pub project_id: Option<String>,
    pub project_name: String,
    pub objective: String,
    pub draft_saved: bool,
    pub finalized: bool,
}

impl ProjectWizardState {
    pub fn idle() -> Self {
        Self {
            project_id: None,
            project_name: String::new(),
            objective: String::new(),
            draft_saved: false,
            finalized: false,
        }
    }

    pub fn completed(project_id: String, project_name: String, objective: String) -> Self {
        Self {
            project_id: Some(project_id),
            project_name,
            objective,
            draft_saved: true,
            finalized: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectWizardScreen {
    pub shell_background: &'static str,
    pub surface_background: &'static str,
    pub text_primary: &'static str,
    pub accent: &'static str,
    pub heading: String,
    pub project_name: String,
    pub objective: String,
    pub draft_saved: bool,
    pub finalized: bool,
}

impl ProjectWizardScreen {
    pub fn themed(palette: ThemePalette, state: ProjectWizardState) -> Self {
        Self {
            shell_background: palette.shell_background,
            surface_background: palette.surface_background,
            text_primary: palette.text_primary,
            accent: palette.accent,
            heading: String::from("PROJECT WIZARD"),
            project_name: state.project_name,
            objective: state.objective,
            draft_saved: state.draft_saved,
            finalized: state.finalized,
        }
    }
}
