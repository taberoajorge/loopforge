use super::super::theme::palette::ThemePalette;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorStats {
    pub completed_iterations: u32,
    pub blocked_states: u32,
    pub rate_limit_events: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorState {
    pub project_id: Option<String>,
    pub project_name: String,
    pub session_id: Option<String>,
    pub running: bool,
    pub events: Vec<String>,
    pub stats: MonitorStats,
    pub last_error: Option<String>,
}

impl MonitorState {
    pub fn idle() -> Self {
        Self {
            project_id: None,
            project_name: String::new(),
            session_id: None,
            running: false,
            events: Vec::new(),
            stats: MonitorStats {
                completed_iterations: 0,
                blocked_states: 0,
                rate_limit_events: 0,
            },
            last_error: None,
        }
    }

    pub fn start(project_id: String, project_name: String) -> Self {
        Self {
            project_id: Some(project_id),
            project_name,
            session_id: None,
            running: true,
            events: Vec::new(),
            stats: MonitorStats {
                completed_iterations: 0,
                blocked_states: 0,
                rate_limit_events: 0,
            },
            last_error: None,
        }
    }

    pub fn apply_update(
        mut self,
        session_id: String,
        running: bool,
        events: Vec<String>,
        completed_iterations: u32,
        blocked_states: u32,
        rate_limit_events: u32,
    ) -> Self {
        self.session_id = Some(session_id);
        self.running = running;
        self.events.extend(events);
        self.stats.completed_iterations = completed_iterations;
        self.stats.blocked_states = blocked_states;
        self.stats.rate_limit_events = rate_limit_events;
        self.last_error = None;
        self
    }

    pub fn stop(mut self) -> Self {
        self.running = false;
        self
    }

    pub fn fail(mut self, error_message: String) -> Self {
        self.running = false;
        self.last_error = Some(error_message);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorScreen {
    pub shell_background: &'static str,
    pub surface_background: &'static str,
    pub text_primary: &'static str,
    pub accent: &'static str,
    pub heading: String,
    pub project_name: String,
    pub session_id: Option<String>,
    pub running: bool,
    pub events: Vec<String>,
    pub stats: MonitorStats,
    pub last_error: Option<String>,
}

impl MonitorScreen {
    pub fn themed(palette: ThemePalette, state: MonitorState) -> Self {
        Self {
            shell_background: palette.shell_background,
            surface_background: palette.surface_background,
            text_primary: palette.text_primary,
            accent: palette.accent,
            heading: String::from("LOOP MONITOR"),
            project_name: state.project_name,
            session_id: state.session_id,
            running: state.running,
            events: state.events,
            stats: state.stats,
            last_error: state.last_error,
        }
    }
}
