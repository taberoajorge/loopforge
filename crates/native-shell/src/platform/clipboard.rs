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
    pub fn write_text(&self, text: &str) -> Result<(), ClipboardError> {
        if cfg!(target_os = "macos") {
            return run_clipboard_command("pbcopy", &[], text).map_err(map_missing_command_error);
        }
        if cfg!(target_os = "linux") {
            return run_linux_clipboard_command(text);
        }
        if cfg!(target_os = "windows") {
            return run_clipboard_command("clip", &[], text).map_err(map_missing_command_error);
        }
        Err(ClipboardError::UnsupportedPlatform)
    }
}

pub fn write_text(text: &str) -> Result<(), ClipboardError> {
    ClipboardService.write_text(text)
}

fn run_linux_clipboard_command(text: &str) -> Result<(), ClipboardError> {
    let candidates: [(&str, &[&str]); 3] = [
        ("wl-copy", &["--trim-newline"]),
        ("xclip", &["-selection", "clipboard"]),
        ("xsel", &["--clipboard", "--input"]),
    ];
    for (command_name, arguments) in candidates {
        match run_clipboard_command(command_name, arguments, text) {
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

fn map_missing_command_error(error: ClipboardError) -> ClipboardError {
    if is_missing_command_error(&error) {
        return ClipboardError::MissingClipboardCommand;
    }
    error
}

fn is_missing_command_error(error: &ClipboardError) -> bool {
    match error {
        ClipboardError::Io(io_error) => io_error.kind() == ErrorKind::NotFound,
        _ => false,
    }
}
