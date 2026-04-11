use std::future::Future;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtomizerStage {
    CollectPlan,
    BuildPrompt,
    ReviewStories,
    WriteStories,
}

impl AtomizerStage {
    pub fn ordered() -> [Self; 4] {
        [
            Self::CollectPlan,
            Self::BuildPrompt,
            Self::ReviewStories,
            Self::WriteStories,
        ]
    }

    pub fn index(&self) -> u8 {
        match self {
            Self::CollectPlan => 1,
            Self::BuildPrompt => 2,
            Self::ReviewStories => 3,
            Self::WriteStories => 4,
        }
    }

    pub fn stage_name(&self) -> String {
        match self {
            Self::CollectPlan => String::from("summarize"),
            Self::BuildPrompt => String::from("chunk"),
            Self::ReviewStories => String::from("atomize"),
            Self::WriteStories => String::from("merge"),
        }
    }

    fn start_message(&self) -> String {
        match self {
            Self::CollectPlan => String::from("Summarizing plan..."),
            Self::BuildPrompt => String::from("Splitting into sections..."),
            Self::ReviewStories => String::from("Atomizing sections..."),
            Self::WriteStories => String::from("Merging and ordering stories..."),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomizerRequest {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomizerProgress {
    pub stage: u8,
    pub stage_name: String,
    pub message: String,
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
        story_count: Option<usize>,
    },
}

impl AtomizerEvent {
    pub fn as_progress_payload(&self) -> AtomizerProgress {
        match self {
            Self::StageStarted { project_id, stage } => AtomizerProgress {
                stage: stage.index(),
                stage_name: stage.stage_name(),
                message: stage.start_message(),
                project_id: project_id.clone(),
            },
            Self::StageCompleted {
                project_id,
                stage,
                story_count,
            } => AtomizerProgress {
                stage: stage.index(),
                stage_name: stage.stage_name(),
                message: match story_count {
                    Some(total) => format!("Done — {total} stories"),
                    None => String::from("Stage completed"),
                },
                project_id: project_id.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomizerRunResult<OutputData> {
    pub output: OutputData,
    pub events: Vec<AtomizerEvent>,
}

pub trait AtomizerService {
    fn stages(&self, request: &AtomizerRequest) -> Vec<AtomizerStage>;
}

pub async fn run_atomizer<FutureData, OutputData, ErrorData>(
    request: AtomizerRequest,
    execute: impl FnOnce(AtomizerRequest) -> FutureData,
    story_count: impl FnOnce(&OutputData) -> usize,
) -> Result<AtomizerRunResult<OutputData>, ErrorData>
where
    FutureData: Future<Output = Result<OutputData, ErrorData>>,
{
    let mut events = AtomizerStage::ordered()
        .into_iter()
        .map(|stage| AtomizerEvent::StageStarted {
            project_id: request.project_id.clone(),
            stage,
        })
        .collect::<Vec<_>>();
    let project_id = request.project_id.clone();
    let output = execute(request).await?;
    events.push(AtomizerEvent::StageCompleted {
        project_id,
        stage: AtomizerStage::WriteStories,
        story_count: Some(story_count(&output)),
    });
    Ok(AtomizerRunResult { output, events })
}
