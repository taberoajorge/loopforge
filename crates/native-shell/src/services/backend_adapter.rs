use app_services::{IterationStory, ProjectDetail, ProjectQueryRecord, ProjectsByStatus};
use loopforge_app_core::atomizer::{
    AtomizerEvent, AtomizerProgress, AtomizerRequest, AtomizerRunResult, AtomizerStage,
};
use loopforge_app_core::events::atomizer_progress_payloads;
use loopforge_app_core::plan::{PlanSessionEvent, PlanSessionHandle};
use loopforge_app_core::projects::CreateProjectRequest;

pub type SharedProjects = ProjectsByStatus<ProjectQueryRecord>;
pub type SharedProjectDetail = ProjectDetail<ProjectQueryRecord, IterationStory>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BackendAdapter;

impl BackendAdapter {
    pub fn create_project<ResultData, ErrorData>(
        request: CreateProjectRequest,
        execute: impl FnOnce(CreateProjectRequest) -> Result<ResultData, ErrorData>,
    ) -> Result<ResultData, ErrorData> {
        execute(request)
    }

    pub fn list_projects<ErrorData>(
        execute: impl FnOnce() -> Result<SharedProjects, ErrorData>,
    ) -> Result<SharedProjects, ErrorData> {
        execute()
    }

    pub fn project_detail<ErrorData>(
        project_id: impl Into<String>,
        execute: impl FnOnce(String) -> Result<SharedProjectDetail, ErrorData>,
    ) -> Result<SharedProjectDetail, ErrorData> {
        execute(project_id.into())
    }

    pub fn open_plan_session<ErrorData>(
        project_id: impl Into<String>,
        execute: impl FnOnce(String) -> Result<PlanSessionHandle, ErrorData>,
    ) -> Result<PlanSessionHandle, ErrorData> {
        execute(project_id.into())
    }

    pub fn write_plan_input<ErrorData>(
        project_id: impl Into<String>,
        chunk: impl Into<String>,
        execute: impl FnOnce(String, String) -> Result<(), ErrorData>,
    ) -> Result<PlanSessionEvent, ErrorData> {
        let project_id = project_id.into();
        let chunk = chunk.into();
        execute(project_id.clone(), chunk.clone())?;
        Ok(PlanSessionEvent::Output { project_id, chunk })
    }

    pub fn stop_plan_session<ErrorData>(
        project_id: impl Into<String>,
        execute: impl FnOnce(String) -> Result<(), ErrorData>,
    ) -> Result<PlanSessionEvent, ErrorData> {
        let project_id = project_id.into();
        execute(project_id.clone())?;
        Ok(PlanSessionEvent::Stopped { project_id })
    }

    pub fn atomizer_stages<ErrorData>(
        request: AtomizerRequest,
        execute: impl FnOnce(AtomizerRequest) -> Result<Vec<AtomizerStage>, ErrorData>,
    ) -> Result<Vec<AtomizerStage>, ErrorData> {
        execute(request)
    }

    pub fn run_atomizer<OutputData, ErrorData>(
        request: AtomizerRequest,
        execute: impl FnOnce(AtomizerRequest) -> Result<AtomizerRunResult<OutputData>, ErrorData>,
    ) -> Result<AtomizerRunResult<OutputData>, ErrorData> {
        execute(request)
    }

    pub fn atomizer_progress(events: &[AtomizerEvent]) -> Vec<AtomizerProgress> {
        atomizer_progress_payloads(events)
    }
}

#[cfg(test)]
mod tests {
    use super::{BackendAdapter, SharedProjectDetail, SharedProjects};
    use app_services::{IterationStory, ProjectQueryRecord};
    use loopforge_app_core::atomizer::{AtomizerEvent, AtomizerRequest, AtomizerRunResult};
    use loopforge_app_core::plan::PlanSessionStatus;
    use loopforge_app_core::projects::CreateProjectRequest;

    #[test]
    fn projects_entry_points_use_shared_project_dtos() {
        let created = BackendAdapter::create_project(
            CreateProjectRequest {
                name: String::from("LoopForge"),
                description: String::from("Move orchestration"),
                working_directory: String::from("/tmp/loopforge"),
                wizard_step: Some(String::from("describe")),
            },
            |request| Ok::<_, ()>(request.name),
        )
        .expect("create project");
        let projects = BackendAdapter::list_projects(|| {
            Ok::<_, ()>(SharedProjects {
                draft: vec![ProjectQueryRecord {
                    id: String::from("project-1"),
                    name: String::from("LoopForge"),
                    status: String::from("draft"),
                    ..Default::default()
                }],
                ..Default::default()
            })
        })
        .expect("list projects");
        let detail = BackendAdapter::project_detail(String::from("project-1"), |project_id| {
            Ok::<_, ()>(SharedProjectDetail {
                project: ProjectQueryRecord {
                    id: project_id,
                    name: String::from("LoopForge"),
                    status: String::from("draft"),
                    ..Default::default()
                },
                total_stories: 1,
                pending_count: 1,
                stories: vec![IterationStory {
                    id: String::from("S-004"),
                    title: String::from("Add backend adapter"),
                    status: String::from("pending"),
                    ..Default::default()
                }],
                ..Default::default()
            })
        })
        .expect("project detail");

        assert_eq!(created, "LoopForge");
        assert_eq!(projects.draft[0].id, "project-1");
        assert_eq!(detail.project.name, "LoopForge");
        assert_eq!(detail.stories[0].id, "S-004");
    }

    #[test]
    fn planning_entry_points_use_shared_plan_dtos() {
        let handle = BackendAdapter::open_plan_session("project-1", |project_id| {
            Ok::<_, ()>(loopforge_app_core::plan::PlanSessionHandle {
                project_id,
                status: PlanSessionStatus::Running,
            })
        })
        .expect("open plan");
        let output = BackendAdapter::write_plan_input("project-1", "Continue", |_, _| Ok::<_, ()>(()))
            .expect("write plan");
        let stopped =
            BackendAdapter::stop_plan_session("project-1", |_| Ok::<_, ()>(())).expect("stop plan");

        assert_eq!(handle.project_id, "project-1");
        assert!(matches!(
            output,
            loopforge_app_core::plan::PlanSessionEvent::Output { project_id, chunk }
            if project_id == "project-1" && chunk == "Continue"
        ));
        assert!(matches!(
            stopped,
            loopforge_app_core::plan::PlanSessionEvent::Stopped { project_id }
            if project_id == "project-1"
        ));
    }

    #[test]
    fn atomization_entry_points_use_shared_atomizer_dtos() {
        let request = AtomizerRequest {
            project_id: String::from("project-1"),
        };
        let stages = BackendAdapter::atomizer_stages(request.clone(), |_| {
            Ok::<_, ()>(vec![loopforge_app_core::atomizer::AtomizerStage::CollectPlan])
        })
        .expect("stages");
        let run = BackendAdapter::run_atomizer(request.clone(), |request| {
            Ok::<_, ()>(AtomizerRunResult {
                output: request.project_id,
                events: vec![AtomizerEvent::StageStarted {
                    project_id: String::from("project-1"),
                    stage: loopforge_app_core::atomizer::AtomizerStage::CollectPlan,
                }],
            })
        })
        .expect("run");
        let progress = BackendAdapter::atomizer_progress(&run.events);

        assert_eq!(stages.len(), 1);
        assert_eq!(run.output, "project-1");
        assert_eq!(progress[0].project_id, "project-1");
    }
}
