#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtomizerStage {
    CollectPlan,
    BuildPrompt,
    ReviewStories,
    WriteStories,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomizerRequest {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtomizerEvent {
    StageStarted {
        project_id: String,
        stage: AtomizerStage,
    },
    StageCompleted {
        project_id: String,
        stage: AtomizerStage,
    },
}

pub trait AtomizerService {
    fn stages(&self, request: &AtomizerRequest) -> Vec<AtomizerStage>;
}
