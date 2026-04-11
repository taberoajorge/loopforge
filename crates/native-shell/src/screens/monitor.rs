use crate::platform::ClipboardService;

#[derive(Debug, Default)]
pub struct MonitorScreenClipboard;

impl MonitorScreenClipboard {
    pub fn copy_log_output(log_output: &str) -> Result<(), String> {
        copy(log_output)
    }

    pub fn copy_session_identifier(session_identifier: &str) -> Result<(), String> {
        copy(session_identifier)
    }
}

fn copy(text: &str) -> Result<(), String> {
    ClipboardService.write_text(text).map_err(|error| error.to_string())
}
