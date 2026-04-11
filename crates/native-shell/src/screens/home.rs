use crate::platform::ClipboardService;

#[derive(Debug, Default)]
pub struct HomeScreenClipboard;

impl HomeScreenClipboard {
    pub fn copy_dashboard_prompt(prompt_text: &str) -> Result<(), String> {
        ClipboardService::copy_dashboard_prompt(prompt_text).map_err(|error| error.to_string())
    }

    pub fn copy_project_identifier(project_identifier: &str) -> Result<(), String> {
        ClipboardService::copy_project_identifier(project_identifier)
            .map_err(|error| error.to_string())
    }
}
