pub mod atomizer;
pub mod events;
pub mod loop_session;
pub mod plan;
pub mod projects;

pub use atomizer::{AtomizerEvent, AtomizerRequest, AtomizerService, AtomizerStage};
pub use events::{AppEvent, EventFanout};
pub use loop_session::{LoopCommand, LoopSessionEvent, LoopSessionHandle, LoopSessionService};
pub use plan::{PlanSessionEvent, PlanSessionHandle, PlanSessionService, PlanSessionStatus};
pub use projects::{ProjectCommand, ProjectService, ProjectSummary};

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopProjectService;
    struct NoopPlanService;
    struct NoopAtomizerService;
    struct NoopLoopSessionService;

    impl ProjectService for NoopProjectService {
        fn apply(&self, command: ProjectCommand) -> ProjectSummary {
            match command {
                ProjectCommand::Create { id, name } => ProjectSummary { id, name },
                ProjectCommand::Archive { id } => ProjectSummary {
                    id,
                    name: String::from("archived"),
                },
            }
        }
    }

    impl PlanSessionService for NoopPlanService {
        fn open(&self, project_id: impl Into<String>) -> PlanSessionHandle {
            PlanSessionHandle {
                project_id: project_id.into(),
                status: PlanSessionStatus::Idle,
            }
        }
    }

    impl AtomizerService for NoopAtomizerService {
        fn stages(&self, _: &AtomizerRequest) -> Vec<AtomizerStage> {
            vec![AtomizerStage::CollectPlan, AtomizerStage::WriteStories]
        }
    }

    impl LoopSessionService for NoopLoopSessionService {
        fn dispatch(&self, command: LoopCommand) -> LoopSessionHandle {
            match command {
                LoopCommand::Start { project_id } => LoopSessionHandle {
                    project_id,
                    active: true,
                },
                LoopCommand::Stop { project_id } => LoopSessionHandle {
                    project_id,
                    active: false,
                },
            }
        }
    }

    #[test]
    fn exports_service_boundaries() {
        let project_service = NoopProjectService;
        let plan_service = NoopPlanService;
        let atomizer_service = NoopAtomizerService;
        let loop_service = NoopLoopSessionService;
        let summary = project_service.apply(ProjectCommand::Create {
            id: String::from("project-1"),
            name: String::from("LoopForge"),
        });
        let plan = plan_service.open(summary.id.clone());
        let stages = atomizer_service.stages(&AtomizerRequest {
            project_id: summary.id.clone(),
        });
        let loop_handle = loop_service.dispatch(LoopCommand::Start {
            project_id: summary.id,
        });
        assert_eq!(summary.name, "LoopForge");
        assert_eq!(plan.status, PlanSessionStatus::Idle);
        assert_eq!(stages.last(), Some(&AtomizerStage::WriteStories));
        assert!(loop_handle.active);
    }
}
