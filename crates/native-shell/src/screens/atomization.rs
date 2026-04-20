use super::super::theme::palette::ThemePalette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomizationArtifact {
    Prd,
    Prompt,
    Guardrails,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomizationStageUpdate {
    pub stage: String,
    pub detail: String,
}

impl AtomizationStageUpdate {
    pub fn new(stage: String, detail: String) -> Self {
        Self { stage, detail }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomizationState {
    pub project_id: Option<String>,
    pub project_name: String,
    pub running: bool,
    pub stage_updates: Vec<AtomizationStageUpdate>,
    pub prd_path: Option<String>,
    pub prompt_path: Option<String>,
    pub guardrails_path: Option<String>,
    pub last_error: Option<String>,
}

impl AtomizationState {
    pub fn idle() -> Self {
        Self {
            project_id: None,
            project_name: String::new(),
            running: false,
            stage_updates: Vec::new(),
            prd_path: None,
            prompt_path: None,
            guardrails_path: None,
            last_error: None,
        }
    }

    pub fn start(project_id: String, project_name: String) -> Self {
        Self {
            project_id: Some(project_id),
            project_name,
            running: true,
            stage_updates: Vec::new(),
            prd_path: None,
            prompt_path: None,
            guardrails_path: None,
            last_error: None,
        }
    }

    pub fn push_stage(mut self, update: AtomizationStageUpdate) -> Self {
        self.stage_updates.push(update);
        self.last_error = None;
        self
    }

    pub fn complete(mut self, prd_path: String, prompt_path: String, guardrails_path: String) -> Self {
        self.running = false;
        self.prd_path = Some(prd_path);
        self.prompt_path = Some(prompt_path);
        self.guardrails_path = Some(guardrails_path);
        self
    }

    pub fn fail(mut self, error_message: String) -> Self {
        self.running = false;
        self.last_error = Some(error_message);
        self
    }

    pub fn artifact_path(&self, artifact: AtomizationArtifact) -> Option<&str> {
        match artifact {
            AtomizationArtifact::Prd => self.prd_path.as_deref(),
            AtomizationArtifact::Prompt => self.prompt_path.as_deref(),
            AtomizationArtifact::Guardrails => self.guardrails_path.as_deref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomizationScreen {
    pub shell_background: &'static str,
    pub surface_background: &'static str,
    pub text_primary: &'static str,
    pub accent: &'static str,
    pub heading: String,
    pub project_name: String,
    pub running: bool,
    pub stage_updates: Vec<AtomizationStageUpdate>,
    pub prd_path: Option<String>,
    pub prompt_path: Option<String>,
    pub guardrails_path: Option<String>,
    pub last_error: Option<String>,
}

impl AtomizationScreen {
    pub fn themed(palette: ThemePalette, state: AtomizationState) -> Self {
        Self {
            shell_background: palette.shell_background,
            surface_background: palette.surface_background,
            text_primary: palette.text_primary,
            accent: palette.accent,
            heading: String::from("ATOMIZATION"),
            project_name: state.project_name,
            running: state.running,
            stage_updates: state.stage_updates,
            prd_path: state.prd_path,
            prompt_path: state.prompt_path,
            guardrails_path: state.guardrails_path,
            last_error: state.last_error,
        }
    }
}
