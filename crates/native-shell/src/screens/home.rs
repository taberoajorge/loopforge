use crate::platform::ClipboardService;

#[derive(Debug, Default)]
pub struct HomeScreenClipboard;

impl HomeScreenClipboard {
    pub fn copy_dashboard_prompt(prompt_text: &str) -> Result<(), String> {
        copy(prompt_text)
    }

    pub fn copy_project_identifier(project_identifier: &str) -> Result<(), String> {
        copy(project_identifier)
    }
}

fn copy(text: &str) -> Result<(), String> {
    ClipboardService::write_text(text).map_err(|error| error.to_string())
}
