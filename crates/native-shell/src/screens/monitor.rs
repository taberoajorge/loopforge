use crate::platform::clipboard::write_text;

#[derive(Debug, Default)]
pub struct MonitorScreenClipboard;

impl MonitorScreenClipboard {
    pub fn copy_log_output(log_output: &str) -> Result<(), String> {
        write_text(log_output).map_err(|error| error.to_string())
    }

    pub fn copy_session_identifier(session_identifier: &str) -> Result<(), String> {
        write_text(session_identifier).map_err(|error| error.to_string())
    }
}
