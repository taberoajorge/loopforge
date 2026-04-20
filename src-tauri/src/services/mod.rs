use crate::projects::{Project, ProjectDetail};
use app_services::{ProjectDetail as SharedProjectDetail, ProjectQueryRecord, SaveDraftCommand};
use ralph_core::prd::UserStory;

pub mod monitor_adapter;
pub mod session_adapter;

pub type InvokeProjectRecord = ProjectQueryRecord;
pub type InvokeProjectDetail = SharedProjectDetail<ProjectQueryRecord, UserStory>;

pub fn attach_runtime_aliases<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
    handler: impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static,
) -> tauri::Builder<R> {
    builder.invoke_handler(handler)
}

pub fn into_project_record(project: Project) -> InvokeProjectRecord {
    InvokeProjectRecord {
        id: project.id,
        name: project.name,
        description: project.description,
        status: project.status,
        working_directory: project.working_directory,
        created_at: project.created_at,
        updated_at: project.updated_at,
        wizard_step: project.wizard_step,
    }
}

pub fn into_project_detail(detail: ProjectDetail) -> InvokeProjectDetail {
    InvokeProjectDetail {
        project: into_project_record(detail.project),
        total_stories: detail.total_stories,
        passed_count: detail.passed_count,
        blocked_count: detail.blocked_count,
        pending_count: detail.pending_count,
        stories: detail.stories,
    }
}

pub fn save_draft_command(project_id: String, draft_json: String) -> SaveDraftCommand {
    SaveDraftCommand {
        project_id,
        draft_json,
    }
}
