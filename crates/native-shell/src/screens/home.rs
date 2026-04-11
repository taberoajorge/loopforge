use crate::platform::DialogError;
use crate::platform::NativeDialogButton;
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
        let title = "Create Project";
        let message = format!("Create project \"{project_name}\"?");
        self.dialogs
            .confirm_with_labels(title, &message, "Create", "Cancel")
            .map(|button| button == NativeDialogButton::Confirm)
    }

    pub fn confirm_project_archive(&self, project_name: &str) -> Result<bool, DialogError> {
        let title = "Archive Project";
        let message = format!("Archive project \"{project_name}\"?");
        self.dialogs
            .confirm_with_labels(title, &message, "Archive", "Cancel")
            .map(|button| button == crate::platform::NativeDialogButton::Confirm)
    }
}
