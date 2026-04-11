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
        self.dialogs.confirm_project_creation(project_name)
    }

    pub fn confirm_project_archive(&self, project_name: &str) -> Result<bool, DialogError> {
        self.dialogs.confirm_project_archive(project_name)
    }
}
