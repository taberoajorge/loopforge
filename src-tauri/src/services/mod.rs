pub mod monitor_adapter;
pub mod session_adapter;
#[path = "../../../crates/loopforge-app-core/src/atomizer.rs"]
pub mod app_core_atomizer;
#[path = "../../../crates/loopforge-app-core/src/loop_session.rs"]
pub mod app_core_loop_session;
#[path = "../../../crates/loopforge-app-core/src/projects.rs"]
pub mod app_core_projects;

use crate::projects::ProjectDetail;
use ralph_core::prd::UserStory;

pub fn build_create_project_request(
    name: String,
    description: String,
    working_directory: String,
    wizard_step: Option<String>,
) -> app_core_projects::CreateProjectRequest {
    app_core_projects::CreateProjectRequest {
        name,
        description,
        working_directory,
        wizard_step,
    }
}

pub fn build_atomizer_request(project_id: String) -> app_core_atomizer::AtomizerRequest {
    app_core_atomizer::AtomizerRequest { project_id }
}

pub fn build_start_loop_command(project_id: String) -> app_core_loop_session::LoopCommand {
    app_core_loop_session::LoopCommand::Start { project_id }
}

pub fn to_shared_project_detail(
    detail: ProjectDetail,
) -> app_core_projects::ProjectDetail<crate::projects::Project, UserStory> {
    app_core_projects::ProjectDetail {
        project: detail.project,
        total_stories: detail.total_stories,
        passed_count: detail.passed_count,
        blocked_count: detail.blocked_count,
        pending_count: detail.pending_count,
        stories: detail.stories,
    }
}

pub fn from_shared_project_detail(
    detail: app_core_projects::ProjectDetail<crate::projects::Project, UserStory>,
) -> ProjectDetail {
    ProjectDetail {
        project: detail.project,
        total_stories: detail.total_stories,
        passed_count: detail.passed_count,
        blocked_count: detail.blocked_count,
        pending_count: detail.pending_count,
        stories: detail.stories,
    }
}

pub fn attach_runtime_aliases<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
    handler: impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static,
) -> tauri::Builder<R> {
    builder.invoke_handler(handler)
}
