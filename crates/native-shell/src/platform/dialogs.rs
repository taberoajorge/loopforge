use std::path::PathBuf;
use std::process::{Command, ExitStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogError {
    CommandFailed(String),
    InvalidResponse(String),
    UnsupportedPlatform,
}

impl std::fmt::Display for DialogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandFailed(message) => write!(formatter, "{message}"),
            Self::InvalidResponse(message) => write!(formatter, "{message}"),
            Self::UnsupportedPlatform => write!(formatter, "native dialog is not supported on this platform"),
        }
    }
}

impl std::error::Error for DialogError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeDialogButton {
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, Default)]
pub struct NativeDialogs;

impl NativeDialogs {
    pub fn new() -> Self {
        Self
    }

    pub fn confirm(&self, title: &str, message: &str) -> Result<bool, DialogError> {
        let button = self.confirm_with_labels(title, message, "Confirm", "Cancel")?;
        Ok(button == NativeDialogButton::Confirm)
    }

    pub fn confirm_with_labels(
        &self,
        title: &str,
        message: &str,
        confirm_label: &str,
        cancel_label: &str,
    ) -> Result<NativeDialogButton, DialogError> {
        #[cfg(target_os = "macos")]
        {
            return confirm_macos(title, message, confirm_label, cancel_label);
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (title, message, confirm_label, cancel_label);
            Err(DialogError::UnsupportedPlatform)
        }
    }

    pub fn pick_folder(&self, prompt: &str) -> Result<Option<PathBuf>, DialogError> {
        #[cfg(target_os = "macos")]
        {
            return pick_path_macos(prompt, true);
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = prompt;
            Err(DialogError::UnsupportedPlatform)
        }
    }

    pub fn pick_file(&self, prompt: &str) -> Result<Option<PathBuf>, DialogError> {
        #[cfg(target_os = "macos")]
        {
            return pick_path_macos(prompt, false);
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = prompt;
            Err(DialogError::UnsupportedPlatform)
        }
    }
}

#[cfg(target_os = "macos")]
fn confirm_macos(
    title: &str,
    message: &str,
    confirm_label: &str,
    cancel_label: &str,
) -> Result<NativeDialogButton, DialogError> {
    let script = format!(
        "display dialog \"{}\" with title \"{}\" buttons {{\"{}\", \"{}\"}} default button \"{}\"",
        escape_applescript(message),
        escape_applescript(title),
        escape_applescript(cancel_label),
        escape_applescript(confirm_label),
        escape_applescript(confirm_label)
    );
    let output = run_osascript(&script)?;
    if output.status.success() {
        let button = parse_button_label(&output.stdout)?;
        if button == confirm_label {
            return Ok(NativeDialogButton::Confirm);
        }
        if button == cancel_label {
            return Ok(NativeDialogButton::Cancel);
        }
        return Err(DialogError::InvalidResponse(format!(
            "unexpected dialog button '{button}'"
        )));
    }
    if output.stderr.contains("User canceled") {
        return Ok(NativeDialogButton::Cancel);
    }
    Err(DialogError::CommandFailed(output.stderr))
}

#[cfg(target_os = "macos")]
fn pick_path_macos(prompt: &str, folder: bool) -> Result<Option<PathBuf>, DialogError> {
    let picker = if folder { "choose folder" } else { "choose file" };
    let script = format!(
        "POSIX path of ({} with prompt \"{}\")",
        picker,
        escape_applescript(prompt)
    );
    let output = run_osascript(&script)?;
    if output.status.success() {
        let path = output.stdout.trim();
        if path.is_empty() {
            return Ok(None);
        }
        return Ok(Some(PathBuf::from(path)));
    }
    if output.stderr.contains("User canceled") {
        return Ok(None);
    }
    Err(DialogError::CommandFailed(output.stderr))
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
struct ScriptOutput {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

#[cfg(target_os = "macos")]
fn run_osascript(script: &str) -> Result<ScriptOutput, DialogError> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|error| DialogError::CommandFailed(error.to_string()))?;
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| DialogError::CommandFailed(error.to_string()))?;
    let stderr = String::from_utf8(output.stderr)
        .map_err(|error| DialogError::CommandFailed(error.to_string()))?;
    Ok(ScriptOutput {
        status: output.status,
        stdout,
        stderr,
    })
}

#[cfg(target_os = "macos")]
fn parse_button_label(output: &str) -> Result<&str, DialogError> {
    output
        .trim()
        .split(',')
        .find_map(|segment| segment.trim().strip_prefix("button returned:"))
        .map(str::trim)
        .ok_or_else(|| DialogError::InvalidResponse(output.to_string()))
}

#[cfg(target_os = "macos")]
fn escape_applescript(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
