use std::path::PathBuf;

use crate::platform::DialogError;
use crate::platform::NativeDialogs;

#[derive(Debug, Clone)]
pub struct ProjectWizardScreen {
    dialogs: NativeDialogs,
}

impl ProjectWizardScreen {
    pub fn new(dialogs: NativeDialogs) -> Self {
        Self { dialogs }
    }

    pub fn pick_project_directory(&self) -> Result<Option<PathBuf>, DialogError> {
        self.dialogs.pick_project_directory()
    }

    pub fn pick_seed_plan_file(&self) -> Result<Option<PathBuf>, DialogError> {
        self.dialogs.pick_seed_plan_file()
    }

    pub fn confirm_project_creation(&self, project_name: &str) -> Result<bool, DialogError> {
        self.dialogs.confirm_wizard_project_creation(project_name)
    }
}
