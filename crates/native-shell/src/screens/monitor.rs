use crate::platform::clipboard::ClipboardService;

#[derive(Debug, Default)]
pub struct MonitorScreenClipboard;

impl MonitorScreenClipboard {
    pub fn copy_log_output(log_output: &str) -> Result<(), String> {
        copy_with_native_clipboard(log_output)
    }

    pub fn copy_session_identifier(session_identifier: &str) -> Result<(), String> {
        copy_with_native_clipboard(session_identifier)
    }
}

fn copy_with_native_clipboard(text: &str) -> Result<(), String> {
    ClipboardService.write_text(text).map_err(|error| error.to_string())
}
