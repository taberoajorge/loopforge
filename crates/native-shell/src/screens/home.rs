use crate::platform::clipboard::ClipboardService;

#[derive(Debug, Default)]
pub struct HomeScreenClipboard;

impl HomeScreenClipboard {
    pub fn copy_dashboard_prompt(prompt_text: &str) -> Result<(), String> {
        copy_with_native_clipboard(prompt_text)
    }

    pub fn copy_project_identifier(project_identifier: &str) -> Result<(), String> {
        copy_with_native_clipboard(project_identifier)
    }
}

fn copy_with_native_clipboard(text: &str) -> Result<(), String> {
    ClipboardService.write_text(text).map_err(|error| error.to_string())
}
