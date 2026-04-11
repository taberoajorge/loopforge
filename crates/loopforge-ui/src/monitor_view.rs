use crate::monitor_state::{MonitorState, MonitorSurface};
use crate::sidebar::{build_sidebar_entries, row_count, SidebarEntry};
use std::path::Path;

#[path = "log_stream.rs"]
mod log_stream;
#[path = "output_pane.rs"]
mod output_pane;
use output_pane::{OutputPane, OutputPaneSnapshot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorSnapshot {
    pub sidebar: Vec<SidebarEntry>,
    pub output: OutputPaneSnapshot,
    pub diff_placeholder: String,
    pub focused_surface: MonitorSurface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorView {
    state: MonitorState,
    active_sidebar_row: usize,
    output_pane: OutputPane,
}

impl MonitorView {
    pub fn seeded() -> Self {
        let state = MonitorState::seeded();
        let output_pane = OutputPane::new(state.active_session_log_path(), 24);
        Self {
            state,
            active_sidebar_row: 0,
            output_pane,
        }
    }

    pub fn select_sidebar_row(&mut self, row_index: usize) -> bool {
        if row_index >= row_count(&self.state) {
            return false;
        }
        let session_count = self.state.sessions.len();
        let changed = if row_index < session_count {
            self.state.set_active_session(row_index)
        } else {
            self.state.set_active_agent(row_index - session_count)
        };
        if changed {
            self.active_sidebar_row = row_index;
            if row_index < session_count {
                self.output_pane.set_log_path(self.state.active_session_log_path());
            }
        }
        changed
    }

    pub fn cycle_focus(&mut self) {
        self.state.cycle_focus();
    }

    pub fn refresh_output(&mut self) -> bool {
        self.output_pane.refresh().unwrap_or(false)
    }

    pub fn scroll_output_up(&mut self, row_count: usize) {
        self.output_pane.scroll_up(row_count);
    }

    pub fn scroll_output_down(&mut self, row_count: usize) {
        self.output_pane.scroll_down(row_count);
    }

    pub fn follow_output_tail(&mut self) {
        self.output_pane.follow_tail();
    }

    pub fn set_session_output_log_path(
        &mut self,
        session_index: usize,
        output_log_path: impl AsRef<Path>,
    ) -> bool {
        let changed = self
            .state
            .set_session_output_log_path(session_index, output_log_path);
        if changed && session_index == self.state.active_session_index {
            self.output_pane.set_log_path(self.state.active_session_log_path());
        }
        changed
    }

    pub fn render_snapshot(&self) -> MonitorSnapshot {
        let active_session = self.state.active_session();
        let active_agent = self.state.active_agent();
        MonitorSnapshot {
            sidebar: build_sidebar_entries(&self.state, self.active_sidebar_row),
            output: self.output_pane.snapshot(),
            diff_placeholder: format!(
                "Diff pane: {} for {}",
                active_session.title, active_agent.name
            ),
            focused_surface: self.state.active_surface.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MonitorView;
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
            .expect("log file should open");
        writeln!(log_file, "{line}").expect("line should write");
    }

    #[test]
    fn selecting_session_row_switches_the_active_log_stream() {
        let first_log_path = temp_log_path("switches-log-stream-first");
        let second_log_path = temp_log_path("switches-log-stream-second");
        append_line(&first_log_path, "current-session-line");
        append_line(&second_log_path, "previous-session-line");
        let mut monitor_view = MonitorView::seeded();
        assert!(monitor_view.set_session_output_log_path(0, &first_log_path));
        assert!(monitor_view.set_session_output_log_path(1, &second_log_path));
        let initial_snapshot = monitor_view.render_snapshot();
        assert_eq!(initial_snapshot.output.visible_lines.last(), Some(&"current-session-line".to_string()));
        assert!(monitor_view.select_sidebar_row(1));
        let switched_snapshot = monitor_view.render_snapshot();
        assert_eq!(switched_snapshot.output.visible_lines.last(), Some(&"previous-session-line".to_string()));
        let _ = fs::remove_file(first_log_path);
        let _ = fs::remove_file(second_log_path);
    }

    #[test]
    fn appending_log_lines_refreshes_output_within_the_same_session() {
        let log_path = temp_log_path("refreshes-output-same-session");
        append_line(&log_path, "first");
        let mut monitor_view = MonitorView::seeded();
        assert!(monitor_view.set_session_output_log_path(0, &log_path));
        let initial_snapshot = monitor_view.render_snapshot();
        assert_eq!(initial_snapshot.output.visible_lines.last(), Some(&"first".to_string()));
        append_line(&log_path, "second");
        assert!(monitor_view.refresh_output());
        let refreshed_snapshot = monitor_view.render_snapshot();
        assert_eq!(refreshed_snapshot.output.visible_lines.last(), Some(&"second".to_string()));
        let _ = fs::remove_file(log_path);
    }

    #[test]
    fn manual_scrollback_stays_stable_until_following_tail_again() {
        let log_path = temp_log_path("manual-scrollback-stable");
        for line_index in 0..40 {
            append_line(&log_path, &format!("line-{line_index}"));
        }
        let mut monitor_view = MonitorView::seeded();
        assert!(monitor_view.set_session_output_log_path(0, &log_path));
        monitor_view.scroll_output_up(10);
        let scrolled_snapshot = monitor_view.render_snapshot();
        let preserved_index = scrolled_snapshot.output.first_visible_line_index;
        assert!(!scrolled_snapshot.output.is_following_tail);
        append_line(&log_path, "line-40");
        append_line(&log_path, "line-41");
        assert!(monitor_view.refresh_output());
        let refreshed_snapshot = monitor_view.render_snapshot();
        assert_eq!(refreshed_snapshot.output.first_visible_line_index, preserved_index);
        assert!(!refreshed_snapshot.output.is_following_tail);
        monitor_view.follow_output_tail();
        let followed_snapshot = monitor_view.render_snapshot();
        assert!(followed_snapshot.output.is_following_tail);
        assert_eq!(followed_snapshot.output.visible_lines.last(), Some(&"line-41".to_string()));
        let _ = fs::remove_file(log_path);
    }

}
