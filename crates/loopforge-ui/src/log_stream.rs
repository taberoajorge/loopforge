use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogStream {
    log_path: PathBuf,
    read_offset: u64,
    trailing_fragment: String,
}

impl LogStream {
    pub fn new(log_path: impl AsRef<Path>) -> Self {
        Self {
            log_path: log_path.as_ref().to_path_buf(),
            read_offset: 0,
            trailing_fragment: String::new(),
        }
    }

    pub fn set_log_path(&mut self, log_path: impl AsRef<Path>) {
        self.log_path = log_path.as_ref().to_path_buf();
        self.read_offset = 0;
        self.trailing_fragment.clear();
    }

    pub fn poll_new_lines(&mut self) -> std::io::Result<Vec<String>> {
        let mut log_file = match File::open(&self.log_path) {
            Ok(log_file) => log_file,
            Err(open_error) if open_error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Vec::new());
            }
            Err(open_error) => return Err(open_error),
        };

        let file_length = log_file.metadata()?.len();
        if file_length < self.read_offset {
            self.read_offset = 0;
            self.trailing_fragment.clear();
        }

        log_file.seek(SeekFrom::Start(self.read_offset))?;
        let mut unread_bytes = Vec::new();
        log_file.read_to_end(&mut unread_bytes)?;
        self.read_offset += unread_bytes.len() as u64;
        Ok(self.decode_lines(unread_bytes))
    }

    fn decode_lines(&mut self, unread_bytes: Vec<u8>) -> Vec<String> {
        if unread_bytes.is_empty() {
            return Vec::new();
        }
        let decoded_chunk = String::from_utf8_lossy(&unread_bytes);
        let mut combined_chunk = String::new();
        combined_chunk.push_str(&self.trailing_fragment);
        combined_chunk.push_str(&decoded_chunk);
        self.trailing_fragment.clear();

        let mut extracted_lines = Vec::new();
        for segment in combined_chunk.split('\n') {
            extracted_lines.push(segment.to_string());
        }
        if !combined_chunk.ends_with('\n') {
            let trailing_segment = extracted_lines.pop().unwrap_or_default();
            self.trailing_fragment = trailing_segment;
        } else if extracted_lines.last().is_some_and(String::is_empty) {
            extracted_lines.pop();
        }

        extracted_lines
            .into_iter()
            .map(|line| line.trim_end_matches('\r').to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::LogStream;
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_log_path(label: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("current time should be after epoch")
            .as_nanos();
        let file_name = format!("loopforge-ui-{label}-{}-{timestamp}.log", std::process::id());
        std::env::temp_dir().join(file_name)
    }

    fn append_line(path: &PathBuf, line: &str) {
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("temp log file should open");
        writeln!(log_file, "{line}").expect("line should write");
    }

    #[test]
    fn polls_incremental_lines_without_repeating_previous_content() {
        let log_path = temp_log_path("polls-incremental-lines");
        append_line(&log_path, "first");
        append_line(&log_path, "second");
        let mut log_stream = LogStream::new(&log_path);
        let first_poll = log_stream.poll_new_lines().expect("first poll should succeed");
        assert_eq!(first_poll, vec!["first".to_string(), "second".to_string()]);
        append_line(&log_path, "third");
        let second_poll = log_stream.poll_new_lines().expect("second poll should succeed");
        assert_eq!(second_poll, vec!["third".to_string()]);
        let _ = fs::remove_file(log_path);
    }

    #[test]
    fn tracks_trailing_fragment_until_newline_arrives() {
        let log_path = temp_log_path("tracks-trailing-fragment");
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("temp log file should open");
        write!(log_file, "partial").expect("partial fragment should write");
        let mut log_stream = LogStream::new(&log_path);
        let first_poll = log_stream.poll_new_lines().expect("first poll should succeed");
        assert!(first_poll.is_empty());
        write!(log_file, " line\n").expect("remaining fragment should write");
        log_file.flush().expect("log file should flush");
        let second_poll = log_stream.poll_new_lines().expect("second poll should succeed");
        assert_eq!(second_poll, vec!["partial line".to_string()]);
        let _ = fs::remove_file(log_path);
    }
}
