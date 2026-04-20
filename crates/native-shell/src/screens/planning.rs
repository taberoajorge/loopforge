use super::super::theme::palette::ThemePalette;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningState {
    pub project_id: Option<String>,
    pub project_name: String,
    pub objective: String,
    pub session_active: bool,
    pub activity_lines: Vec<String>,
    pub plan_path: Option<String>,
    pub last_error: Option<String>,
}

impl PlanningState {
    pub fn idle() -> Self {
        Self {
            project_id: None,
            project_name: String::new(),
            objective: String::new(),
            session_active: false,
            activity_lines: Vec::new(),
            plan_path: None,
            last_error: None,
        }
    }

    pub fn start(project_id: String, project_name: String, objective: String) -> Self {
        Self {
            project_id: Some(project_id),
            project_name,
            objective,
            session_active: true,
            activity_lines: Vec::new(),
            plan_path: None,
            last_error: None,
        }
    }

    pub fn push_activity(mut self, activity: String) -> Self {
        self.activity_lines.push(activity);
        self.last_error = None;
        self
    }

    pub fn stop(mut self, plan_path: Option<String>) -> Self {
        self.session_active = false;
        if let Some(path) = plan_path {
            self.plan_path = Some(path);
        }
        self
    }

    pub fn fail(mut self, error_message: String) -> Self {
        self.last_error = Some(error_message);
        self.session_active = false;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningScreen {
    pub shell_background: &'static str,
    pub surface_background: &'static str,
    pub text_primary: &'static str,
    pub accent: &'static str,
    pub heading: String,
    pub project_name: String,
    pub objective: String,
    pub session_active: bool,
    pub activity_lines: Vec<String>,
    pub plan_path: Option<String>,
    pub last_error: Option<String>,
}

impl PlanningScreen {
    pub fn themed(palette: ThemePalette, state: PlanningState) -> Self {
        Self {
            shell_background: palette.shell_background,
            surface_background: palette.surface_background,
            text_primary: palette.text_primary,
            accent: palette.accent,
            heading: String::from("PLANNING SESSION"),
            project_name: state.project_name,
            objective: state.objective,
            session_active: state.session_active,
            activity_lines: state.activity_lines,
            plan_path: state.plan_path,
            last_error: state.last_error,
        }
    }
}
