#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanSessionStatus {
    Idle,
    Running,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanSessionHandle {
    pub project_id: String,
    pub status: PlanSessionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanSessionEvent {
    Started(PlanSessionHandle),
    Output { project_id: String, chunk: String },
    Stopped { project_id: String },
}

pub trait PlanSessionService {
    fn open(&self, project_id: impl Into<String>) -> PlanSessionHandle;
}
