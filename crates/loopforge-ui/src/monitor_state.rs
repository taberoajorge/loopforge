use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MonitorSurface {
    Sidebar,
    Output,
    Diff,
}

impl MonitorSurface {
    pub fn next(&self) -> Self {
        match self {
            Self::Sidebar => Self::Output,
            Self::Output => Self::Diff,
            Self::Diff => Self::Sidebar,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorSession {
    pub id: String,
    pub title: String,
    pub status: String,
    pub output_log_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorAgent {
    pub id: String,
    pub name: String,
    pub model: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorState {
    pub sessions: Vec<MonitorSession>,
    pub agents: Vec<MonitorAgent>,
    pub active_session_index: usize,
    pub active_agent_index: usize,
    pub active_surface: MonitorSurface,
}

impl MonitorState {
    pub fn seeded() -> Self {
        Self {
            sessions: vec![
                MonitorSession {
                    id: "session-2026-04-11".to_string(),
                    title: "Current Session".to_string(),
                    status: "running".to_string(),
                    output_log_path: PathBuf::from(
                        ".loopforge/artifacts/s-028/fixtures/session-2026-04-11/agent_output.log",
                    ),
                },
                MonitorSession {
                    id: "session-2026-04-10".to_string(),
                    title: "Previous Session".to_string(),
                    status: "paused".to_string(),
                    output_log_path: PathBuf::from(
                        ".loopforge/artifacts/s-028/fixtures/session-2026-04-10/agent_output.log",
                    ),
                },
            ],
            agents: vec![
                MonitorAgent {
                    id: "agent-codex".to_string(),
                    name: "Codex".to_string(),
                    model: "gpt-5-codex".to_string(),
                },
                MonitorAgent {
                    id: "agent-claude".to_string(),
                    name: "Claude".to_string(),
                    model: "claude-sonnet-4-5".to_string(),
                },
            ],
            active_session_index: 0,
            active_agent_index: 0,
            active_surface: MonitorSurface::Sidebar,
        }
    }

    pub fn set_active_session(&mut self, index: usize) -> bool {
        if index >= self.sessions.len() {
            return false;
        }
        self.active_session_index = index;
        true
    }

    pub fn set_active_agent(&mut self, index: usize) -> bool {
        if index >= self.agents.len() {
            return false;
        }
        self.active_agent_index = index;
        true
    }

    pub fn cycle_focus(&mut self) {
        self.active_surface = self.active_surface.next();
    }

    pub fn active_session(&self) -> &MonitorSession {
        &self.sessions[self.active_session_index]
    }

    pub fn active_agent(&self) -> &MonitorAgent {
        &self.agents[self.active_agent_index]
    }

    pub fn active_session_log_path(&self) -> PathBuf {
        self.active_session().output_log_path.clone()
    }

    pub fn set_session_output_log_path(
        &mut self,
        session_index: usize,
        output_log_path: impl AsRef<Path>,
    ) -> bool {
        if session_index >= self.sessions.len() {
            return false;
        }
        self.sessions[session_index].output_log_path = output_log_path.as_ref().to_path_buf();
        true
    }
}
