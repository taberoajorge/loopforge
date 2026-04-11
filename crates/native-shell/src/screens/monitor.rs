use crate::platform::ClipboardService;

#[derive(Debug, Default)]
pub struct MonitorScreenClipboard;

impl MonitorScreenClipboard {
    pub fn copy_log_output(log_output: &str) -> Result<(), String> {
        ClipboardService::copy_log_output(log_output).map_err(|error| error.to_string())
    }

    pub fn copy_session_identifier(session_identifier: &str) -> Result<(), String> {
        ClipboardService::copy_session_identifier(session_identifier)
            .map_err(|error| error.to_string())
    }
}
