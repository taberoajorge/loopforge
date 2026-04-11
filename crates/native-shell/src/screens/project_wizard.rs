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
        self.dialogs.pick_directory("Select project directory")
    }

    pub fn pick_seed_plan_file(&self) -> Result<Option<PathBuf>, DialogError> {
        self.dialogs.pick_file("Select a plan file")
    }

    pub fn confirm_project_creation(&self, project_name: &str) -> Result<bool, DialogError> {
        let message = format!("Create project \"{project_name}\" in the selected directory?");
        self.dialogs
            .confirm("Create Project", &message, "Create", "Cancel")
    }
}
