use std::fmt::{Display, Formatter};
use std::io::{ErrorKind, Write};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub enum ClipboardError {
    UnsupportedPlatform,
    MissingClipboardCommand,
    MissingStdin,
    Io(std::io::Error),
    CommandFailed(String),
}

impl Display for ClipboardError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPlatform => formatter.write_str("unsupported platform"),
            Self::MissingClipboardCommand => {
                formatter.write_str("no clipboard command is available")
            }
            Self::MissingStdin => formatter.write_str("clipboard process stdin was unavailable"),
            Self::Io(error) => write!(formatter, "clipboard I/O error: {error}"),
            Self::CommandFailed(message) => write!(formatter, "clipboard command failed: {message}"),
        }
    }
}

impl std::error::Error for ClipboardError {}

#[derive(Debug, Default, Clone, Copy)]
pub struct ClipboardService;

impl ClipboardService {
    pub fn copy_dashboard_prompt(prompt_text: &str) -> Result<(), ClipboardError> {
        Self::write_text(prompt_text)
    }

    pub fn copy_project_identifier(project_identifier: &str) -> Result<(), ClipboardError> {
        Self::write_text(project_identifier)
    }

    pub fn copy_log_output(log_output: &str) -> Result<(), ClipboardError> {
        Self::write_text(log_output)
    }

    pub fn copy_session_identifier(session_identifier: &str) -> Result<(), ClipboardError> {
        Self::write_text(session_identifier)
    }

    pub fn write_text(text: &str) -> Result<(), ClipboardError> {
        #[cfg(target_os = "macos")]
        {
            return write_with_candidates(&[ClipboardCommand::new("pbcopy", &[])], text);
        }
        #[cfg(target_os = "linux")]
        {
            return write_with_candidates(
                &[
                    ClipboardCommand::new("wl-copy", &["--trim-newline"]),
                    ClipboardCommand::new("xclip", &["-selection", "clipboard"]),
                    ClipboardCommand::new("xsel", &["--clipboard", "--input"]),
                ],
                text,
            );
        }
        #[cfg(target_os = "windows")]
        {
            return write_with_candidates(
                &[
                    ClipboardCommand::new("clip", &[]),
                    ClipboardCommand::new(
                        "powershell",
                        &["-NoProfile", "-Command", "Set-Clipboard"],
                    ),
                ],
                text,
            );
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(ClipboardError::UnsupportedPlatform)
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ClipboardCommand {
    command_name: &'static str,
    arguments: &'static [&'static str],
}

impl ClipboardCommand {
    const fn new(command_name: &'static str, arguments: &'static [&'static str]) -> Self {
        Self {
            command_name,
            arguments,
        }
    }
}

fn write_with_candidates(candidates: &[ClipboardCommand], text: &str) -> Result<(), ClipboardError> {
    for candidate in candidates {
        match run_clipboard_command(candidate.command_name, candidate.arguments, text) {
            Ok(()) => return Ok(()),
            Err(error) if is_missing_command_error(&error) => continue,
            Err(error) => return Err(error),
        }
    }
    Err(ClipboardError::MissingClipboardCommand)
}

fn run_clipboard_command(
    command_name: &str,
    arguments: &[&str],
    text: &str,
) -> Result<(), ClipboardError> {
    let mut command = Command::new(command_name);
    command
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(ClipboardError::Io)?;
    let mut process_stdin = child.stdin.take().ok_or(ClipboardError::MissingStdin)?;
    process_stdin
        .write_all(text.as_bytes())
        .map_err(ClipboardError::Io)?;
    drop(process_stdin);
    let output = child.wait_with_output().map_err(ClipboardError::Io)?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        return Err(ClipboardError::CommandFailed(format!(
            "{command_name} exited with status {}",
            output.status
        )));
    }
    Err(ClipboardError::CommandFailed(stderr))
}

fn is_missing_command_error(error: &ClipboardError) -> bool {
    match error {
        ClipboardError::Io(io_error) => io_error.kind() == ErrorKind::NotFound,
        _ => false,
    }
}
