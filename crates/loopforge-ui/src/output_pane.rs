use super::log_stream::LogStream;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputPaneSnapshot {
    pub visible_lines: Vec<String>,
    pub total_lines: usize,
    pub first_visible_line_index: usize,
    pub is_following_tail: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputPane {
    log_stream: LogStream,
    lines: Vec<String>,
    viewport_rows: usize,
    first_visible_line_index: usize,
    is_following_tail: bool,
}

impl OutputPane {
    pub fn new(log_path: impl AsRef<Path>, viewport_rows: usize) -> Self {
        let mut output_pane = Self {
            log_stream: LogStream::new(log_path),
            lines: Vec::new(),
            viewport_rows: viewport_rows.max(1),
            first_visible_line_index: 0,
            is_following_tail: true,
        };
        let _ = output_pane.refresh();
        output_pane
    }

    pub fn set_log_path(&mut self, log_path: impl AsRef<Path>) {
        self.log_stream.set_log_path(log_path);
        self.lines.clear();
        self.first_visible_line_index = 0;
        self.is_following_tail = true;
        let _ = self.refresh();
    }

    pub fn refresh(&mut self) -> std::io::Result<bool> {
        let appended_lines = self.log_stream.poll_new_lines()?;
        if appended_lines.is_empty() {
            return Ok(false);
        }
        self.lines.extend(appended_lines);
        if self.is_following_tail {
            self.follow_tail();
        }
        Ok(true)
    }

    pub fn scroll_up(&mut self, row_count: usize) {
        self.first_visible_line_index = self.first_visible_line_index.saturating_sub(row_count);
        self.is_following_tail = self.first_visible_line_index == self.max_first_visible_line_index();
    }

    pub fn scroll_down(&mut self, row_count: usize) {
        let candidate_line_index = self.first_visible_line_index.saturating_add(row_count);
        self.first_visible_line_index = candidate_line_index.min(self.max_first_visible_line_index());
        self.is_following_tail = self.first_visible_line_index == self.max_first_visible_line_index();
    }

    pub fn follow_tail(&mut self) {
        self.first_visible_line_index = self.max_first_visible_line_index();
        self.is_following_tail = true;
    }

    pub fn snapshot(&self) -> OutputPaneSnapshot {
        let visible_end = (self.first_visible_line_index + self.viewport_rows).min(self.lines.len());
        let visible_lines = self.lines[self.first_visible_line_index..visible_end].to_vec();
        OutputPaneSnapshot {
            visible_lines,
            total_lines: self.lines.len(),
            first_visible_line_index: self.first_visible_line_index,
            is_following_tail: self.is_following_tail,
        }
    }

    fn max_first_visible_line_index(&self) -> usize {
        self.lines.len().saturating_sub(self.viewport_rows)
    }
}

#[cfg(test)]
mod tests {
    use super::OutputPane;
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

    fn write_fixture(path: &PathBuf, line_count: usize) {
        let mut log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .expect("fixture log should open");
        for line_index in 0..line_count {
            writeln!(log_file, "line-{line_index}").expect("fixture line should write");
        }
    }

    #[test]
    fn supports_large_log_fixtures_with_scrollable_snapshot() {
        let log_path = temp_log_path("supports-large-log-fixtures");
        write_fixture(&log_path, 5_000);
        let output_pane = OutputPane::new(&log_path, 24);
        let snapshot = output_pane.snapshot();
        assert_eq!(snapshot.total_lines, 5_000);
        assert_eq!(snapshot.visible_lines.len(), 24);
        assert!(snapshot.is_following_tail);
        assert_eq!(snapshot.visible_lines.first(), Some(&"line-4976".to_string()));
        assert_eq!(snapshot.visible_lines.last(), Some(&"line-4999".to_string()));
        let _ = fs::remove_file(log_path);
    }
}
