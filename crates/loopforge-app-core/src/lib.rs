pub mod atomizer;
pub mod events;
pub mod loop_session;
pub mod plan;
pub mod projects;
pub mod wizard;

pub use atomizer::{AtomizerEvent, AtomizerRequest, AtomizerService, AtomizerStage};
pub use events::{AppEvent, EventFanout};
pub use loop_session::{LoopCommand, LoopSessionEvent, LoopSessionHandle, LoopSessionService};
pub use plan::{PlanSessionEvent, PlanSessionHandle, PlanSessionService, PlanSessionStatus};
pub use projects::{ProjectCommand, ProjectService, ProjectSummary};
pub use wizard::{RuntimeWizardService, WizardRepository, WizardService};

#[cfg(test)]
mod tests {
    use super::*;
    use app_services::{SaveDraftCommand, WizardSnapshot};

    struct NoopProjectService;
    struct NoopPlanService;
    struct NoopAtomizerService;
    struct NoopLoopSessionService;
    struct NoopWizardService;

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

    impl WizardService for NoopWizardService {
        fn snapshot(
            &self,
            project_id: &str,
        ) -> app_services::ServiceResult<Option<WizardSnapshot>> {
            Ok(Some(WizardSnapshot {
                project_id: project_id.to_string(),
                ..Default::default()
            }))
        }

        fn save_state(
            &self,
            command: app_services::SaveWizardStateCommand,
        ) -> app_services::ServiceResult<WizardSnapshot> {
            Ok(WizardSnapshot {
                project_id: command.project_id,
                ..Default::default()
            })
        }

        fn save_draft(
            &self,
            command: SaveDraftCommand,
        ) -> app_services::ServiceResult<WizardSnapshot> {
            Ok(WizardSnapshot {
                project_id: command.project_id,
                ..Default::default()
            })
        }
    }

    #[test]
    fn exports_service_boundaries() {
        let project_service = NoopProjectService;
        let plan_service = NoopPlanService;
        let atomizer_service = NoopAtomizerService;
        let loop_service = NoopLoopSessionService;
        let wizard_service = NoopWizardService;
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
        let wizard_snapshot = wizard_service
            .save_draft(SaveDraftCommand {
                project_id: String::from("project-1"),
                draft_json: String::from("{}"),
            })
            .expect("save draft");
        assert_eq!(summary.name, "LoopForge");
        assert_eq!(plan.status, PlanSessionStatus::Idle);
        assert_eq!(stages.last(), Some(&AtomizerStage::WriteStories));
        assert!(loop_handle.active);
        assert_eq!(wizard_snapshot.project_id, "project-1");
    }
}
