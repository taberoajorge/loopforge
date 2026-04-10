#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopSessionHandle {
    pub project_id: String,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopCommand {
    Start { project_id: String },
    Stop { project_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopSessionEvent {
    IterationStarted { project_id: String, iteration: u64 },
    IterationCompleted { project_id: String, iteration: u64 },
    SessionEnded { project_id: String },
}

pub trait LoopSessionService {
    fn dispatch(&self, command: LoopCommand) -> LoopSessionHandle;
}
