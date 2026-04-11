#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectCommand {
    Create { id: String, name: String },
    Archive { id: String },
}

pub trait ProjectService {
    fn apply(&self, command: ProjectCommand) -> ProjectSummary;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: String,
    pub working_directory: String,
    pub wizard_step: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateProjectRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub working_directory: String,
    pub created_at: String,
    pub updated_at: String,
    pub wizard_step: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDetail<ProjectData, StoryData> {
    pub project: ProjectData,
    pub total_stories: usize,
    pub passed_count: usize,
    pub blocked_count: usize,
    pub pending_count: usize,
    pub stories: Vec<StoryData>,
}

pub trait StoryState {
    fn passes(&self) -> bool;
    fn blocked(&self) -> bool;
}

pub fn create_project<ResultData, ErrorData>(
    request: CreateProjectRequest,
    create_id: impl FnOnce() -> String,
    now: impl FnOnce() -> String,
    initialize_artifacts: impl FnOnce(&str, &str) -> Result<(), ErrorData>,
    persist: impl FnOnce(CreateProjectRecord) -> Result<ResultData, ErrorData>,
) -> Result<ResultData, ErrorData> {
    let project_id = create_id();
    let timestamp = now();
    initialize_artifacts(&project_id, &request.name)?;
    persist(CreateProjectRecord {
        id: project_id,
        name: request.name,
        description: request.description,
        status: "draft".to_string(),
        working_directory: request.working_directory,
        created_at: timestamp.clone(),
        updated_at: timestamp,
        wizard_step: request.wizard_step,
    })
}

pub fn list_projects<ResultData, ErrorData>(
    load: impl FnOnce() -> Result<ResultData, ErrorData>,
) -> Result<ResultData, ErrorData> {
    load()
}

pub fn finalize_draft<ErrorData>(
    project_id: String,
    now: impl FnOnce() -> String,
    clear_draft_state: impl FnOnce(&str, &str) -> Result<(), ErrorData>,
    remove_draft_artifact: impl FnOnce(&str) -> Result<(), ErrorData>,
) -> Result<(), ErrorData> {
    let updated_at = now();
    clear_draft_state(&project_id, &updated_at)?;
    remove_draft_artifact(&project_id)
}

pub fn discard_draft<ErrorData>(
    project_id: String,
    delete_draft: impl FnOnce(&str) -> Result<bool, ErrorData>,
    remove_artifacts: impl FnOnce(&str) -> Result<(), ErrorData>,
    not_found: impl FnOnce(String) -> ErrorData,
) -> Result<(), ErrorData> {
    if !delete_draft(&project_id)? {
        return Err(not_found(project_id));
    }
    remove_artifacts(&project_id)
}

pub fn get_project_detail<ProjectData, StoryData, ErrorData>(
    project_id: String,
    load_project: impl FnOnce(&str) -> Result<ProjectData, ErrorData>,
    load_stories: impl FnOnce(&str) -> Result<Vec<StoryData>, ErrorData>,
) -> Result<ProjectDetail<ProjectData, StoryData>, ErrorData>
where
    StoryData: StoryState,
{
    let project = load_project(&project_id)?;
    let stories = load_stories(&project_id)?;
    let total_stories = stories.len();
    let passed_count = stories.iter().filter(|story| story.passes()).count();
    let blocked_count = stories.iter().filter(|story| story.blocked()).count();
    Ok(ProjectDetail {
        project,
        total_stories,
        passed_count,
        blocked_count,
        pending_count: total_stories - passed_count - blocked_count,
        stories,
    })
}
