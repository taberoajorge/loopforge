#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectStatus {
    Running,
    Idle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    Healthy,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeProjectSummary {
    pub id: String,
    pub name: String,
    pub status: ProjectStatus,
    pub latest_session: SessionState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeSessionSummary {
    pub project_id: String,
    pub session_id: String,
    pub status: SessionState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeAction {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeViewModel {
    pub heading: String,
    pub strapline: String,
    pub active_projects: Vec<HomeProjectSummary>,
    pub recent_sessions: Vec<HomeSessionSummary>,
    pub primary_actions: Vec<HomeAction>,
}

impl HomeViewModel {
    pub fn seeded() -> Self {
        Self {
            heading: String::from("INITIALIZE SEQUENCE"),
            strapline: String::from("Autonomous AI loop orchestrator."),
            active_projects: vec![
                HomeProjectSummary {
                    id: String::from("proj-alpha"),
                    name: String::from("Dashboard Parity"),
                    status: ProjectStatus::Running,
                    latest_session: SessionState::Healthy,
                },
                HomeProjectSummary {
                    id: String::from("proj-beta"),
                    name: String::from("Shell Migration"),
                    status: ProjectStatus::Idle,
                    latest_session: SessionState::Blocked,
                },
            ],
            recent_sessions: vec![
                HomeSessionSummary {
                    project_id: String::from("proj-alpha"),
                    session_id: String::from("sess-104"),
                    status: SessionState::Healthy,
                },
                HomeSessionSummary {
                    project_id: String::from("proj-beta"),
                    session_id: String::from("sess-097"),
                    status: SessionState::Blocked,
                },
            ],
            primary_actions: vec![
                HomeAction {
                    id: String::from("start-project"),
                    label: String::from("Start new project"),
                },
                HomeAction {
                    id: String::from("resume-project"),
                    label: String::from("Resume project"),
                },
                HomeAction {
                    id: String::from("open-monitor"),
                    label: String::from("Open monitor"),
                },
            ],
        }
    }
}
