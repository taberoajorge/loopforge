use crate::platform::clipboard::write_text;

#[derive(Debug, Default)]
pub struct HomeScreenClipboard;

impl HomeScreenClipboard {
    pub fn copy_dashboard_prompt(prompt_text: &str) -> Result<(), String> {
        write_text(prompt_text).map_err(|error| error.to_string())
    }

    pub fn copy_project_identifier(project_identifier: &str) -> Result<(), String> {
        write_text(project_identifier).map_err(|error| error.to_string())
    }
}
