use std::fmt::{Display, Formatter};
use std::io::Write;
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

pub fn write_text(text: &str) -> Result<(), ClipboardError> {
    if cfg!(target_os = "macos") {
        return run_clipboard_command("pbcopy", &[], text);
    }
    if cfg!(target_os = "linux") {
        return run_linux_clipboard_command(text);
    }
    if cfg!(target_os = "windows") {
        return run_clipboard_command("clip", &[], text);
    }
    Err(ClipboardError::UnsupportedPlatform)
}

fn run_linux_clipboard_command(text: &str) -> Result<(), ClipboardError> {
    if command_exists("wl-copy") {
        return run_clipboard_command("wl-copy", &["--trim-newline"], text);
    }
    if command_exists("xclip") {
        return run_clipboard_command("xclip", &["-selection", "clipboard"], text);
    }
    if command_exists("xsel") {
        return run_clipboard_command("xsel", &["--clipboard", "--input"], text);
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

fn command_exists(command_name: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {command_name} >/dev/null 2>&1")])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
