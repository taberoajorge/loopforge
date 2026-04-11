use crate::platform::DialogError;
use crate::platform::NativeDialogs;

#[derive(Debug, Clone)]
pub struct HomeScreen {
    dialogs: NativeDialogs,
}

impl HomeScreen {
    pub fn new(dialogs: NativeDialogs) -> Self {
        Self { dialogs }
    }

    pub fn confirm_project_creation(&self, project_name: &str) -> Result<bool, DialogError> {
        let message = format!("Create project \"{project_name}\"?");
        self.dialogs
            .confirm("Create Project", &message, "Create", "Cancel")
    }

    pub fn confirm_project_archive(&self, project_name: &str) -> Result<bool, DialogError> {
        let message = format!("Archive project \"{project_name}\"?");
        self.dialogs
            .confirm("Archive Project", &message, "Archive", "Cancel")
    }
}
