use app_services::{SaveDraftCommand, SaveWizardStateCommand, ServiceResult, WizardSnapshot};

pub trait WizardRepository {
    fn load_snapshot(&self, project_id: &str) -> ServiceResult<Option<WizardSnapshot>>;
    fn save_state(&self, command: SaveWizardStateCommand) -> ServiceResult<WizardSnapshot>;
    fn save_draft(&self, command: SaveDraftCommand) -> ServiceResult<WizardSnapshot>;
}

pub trait WizardService {
    fn snapshot(&self, project_id: &str) -> ServiceResult<Option<WizardSnapshot>>;
    fn save_state(&self, command: SaveWizardStateCommand) -> ServiceResult<WizardSnapshot>;
    fn save_draft(&self, command: SaveDraftCommand) -> ServiceResult<WizardSnapshot>;
}

pub struct RuntimeWizardService<R> {
    repository: R,
}

impl<R> RuntimeWizardService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: WizardRepository> WizardService for RuntimeWizardService<R> {
    fn snapshot(&self, project_id: &str) -> ServiceResult<Option<WizardSnapshot>> {
        load_wizard_snapshot(project_id, |project_id| {
            self.repository.load_snapshot(project_id)
        })
    }

    fn save_state(&self, command: SaveWizardStateCommand) -> ServiceResult<WizardSnapshot> {
        save_wizard_state(command, |command| self.repository.save_state(command))
    }

    fn save_draft(&self, command: SaveDraftCommand) -> ServiceResult<WizardSnapshot> {
        save_wizard_draft(command, |command| self.repository.save_draft(command))
    }
}

pub fn load_wizard_snapshot(
    project_id: &str,
    load: impl FnOnce(&str) -> ServiceResult<Option<WizardSnapshot>>,
) -> ServiceResult<Option<WizardSnapshot>> {
    load(project_id)
}

pub fn save_wizard_state(
    command: SaveWizardStateCommand,
    save: impl FnOnce(SaveWizardStateCommand) -> ServiceResult<WizardSnapshot>,
) -> ServiceResult<WizardSnapshot> {
    save(command)
}

pub fn save_wizard_draft(
    command: SaveDraftCommand,
    save: impl FnOnce(SaveDraftCommand) -> ServiceResult<WizardSnapshot>,
) -> ServiceResult<WizardSnapshot> {
    save(command)
}

#[cfg(test)]
mod tests {
    use super::{
        load_wizard_snapshot, save_wizard_draft, save_wizard_state, RuntimeWizardService,
        SaveDraftCommand, SaveWizardStateCommand, WizardRepository, WizardService, WizardSnapshot,
    };
    use app_services::{ServiceError, WizardAtomizeSnapshot, WizardConfigureSnapshot};

    #[test]
    fn load_snapshot_uses_shared_snapshot_output() {
        let snapshot = snapshot("project-1");

        let restored = load_wizard_snapshot("project-1", |project_id| {
            assert_eq!(project_id, "project-1");
            Ok(Some(snapshot.clone()))
        })
        .expect("load snapshot");

        assert_eq!(restored, Some(snapshot));
    }

    #[test]
    fn save_state_uses_shared_command_input() {
        let command = SaveWizardStateCommand {
            project_id: String::from("project-1"),
            wizard_step: String::from("plan"),
            wizard_state_json: String::from("{\"step\":\"plan\"}"),
        };
        let snapshot = snapshot("project-1");

        let saved = save_wizard_state(command.clone(), |received| {
            assert_eq!(received, command);
            Ok(snapshot.clone())
        })
        .expect("save wizard state");

        assert_eq!(saved.project_id, "project-1");
    }

    #[test]
    fn runtime_service_delegates_draft_save_through_repository() {
        let service = RuntimeWizardService::new(StubWizardRepository);
        let command = SaveDraftCommand {
            project_id: String::from("project-9"),
            draft_json: String::from("{\"draft\":true}"),
        };

        let saved = service.save_draft(command).expect("save draft");

        assert_eq!(saved.project_id, "project-9");
        assert_eq!(saved.atomize, WizardAtomizeSnapshot { stories_count: 4 });
        assert_eq!(saved.configure, WizardConfigureSnapshot::default());
    }

    struct StubWizardRepository;

    impl WizardRepository for StubWizardRepository {
        fn load_snapshot(&self, project_id: &str) -> Result<Option<WizardSnapshot>, ServiceError> {
            Ok(Some(snapshot(project_id)))
        }

        fn save_state(
            &self,
            command: SaveWizardStateCommand,
        ) -> Result<WizardSnapshot, ServiceError> {
            Ok(snapshot(&command.project_id))
        }

        fn save_draft(&self, command: SaveDraftCommand) -> Result<WizardSnapshot, ServiceError> {
            save_wizard_draft(command, |received| Ok(snapshot(&received.project_id)))
        }
    }

    fn snapshot(project_id: &str) -> WizardSnapshot {
        WizardSnapshot {
            project_id: project_id.to_string(),
            atomize: WizardAtomizeSnapshot { stories_count: 4 },
            ..Default::default()
        }
    }
}
