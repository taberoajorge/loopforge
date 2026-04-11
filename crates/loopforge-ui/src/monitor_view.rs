use crate::monitor_state::{MonitorState, MonitorSurface};
use crate::sidebar::{SidebarEntry, build_sidebar_entries, row_count};
use std::path::PathBuf;

#[path = "diff_pane.rs"]
mod diff_pane;
use diff_pane::{DiffPaneSnapshot, DiffPaneState};
#[path = "log_stream.rs"]
mod log_stream;
#[path = "output_pane.rs"]
mod output_pane;
use output_pane::{OutputPaneSnapshot, OutputPaneState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorSnapshot {
    pub sidebar: Vec<SidebarEntry>,
    pub output_placeholder: String,
    pub output: OutputPaneSnapshot,
    pub diff: DiffPaneSnapshot,
    pub focused_surface: MonitorSurface,
}

#[derive(Debug)]
pub struct MonitorView {
    state: MonitorState,
    active_sidebar_row: usize,
    output_pane: OutputPaneState,
    diff_pane: DiffPaneState,
}

impl MonitorView {
    pub fn seeded() -> Self {
        let working_directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let repository_path = DiffPaneState::resolve_repository_path(working_directory);
        let mut monitor_view = Self {
            state: MonitorState::seeded(),
            active_sidebar_row: 0,
            output_pane: OutputPaneState::new(),
            diff_pane: DiffPaneState::new(repository_path),
        };
        monitor_view.refresh_output_pane();
        monitor_view.refresh_diff_pane();
        monitor_view
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
            self.refresh_output_pane();
            self.refresh_diff_pane();
        }
        changed
    }

    pub fn cycle_focus(&mut self) {
        self.state.cycle_focus();
    }

    pub fn set_diff_scroll_top_line(&mut self, scroll_top_line: usize) {
        self.diff_pane.set_scroll_top_line(scroll_top_line);
    }

    pub fn set_output_scroll_top_line(&mut self, scroll_top_line: usize) {
        self.output_pane.set_scroll_top_line(scroll_top_line);
    }

    pub fn begin_output_fixture_playback(&mut self, request_key: String, fixture_text: String) {
        self.output_pane.load_fixture_async(
            self.output_title(),
            request_key,
            fixture_text,
            200,
        );
    }

    pub fn begin_diff_fixture_playback(&mut self, request_key: String, patch_text: String) {
        self.diff_pane.load_fixture_async(request_key, patch_text);
    }

    pub fn poll_background_tasks(&mut self) -> bool {
        let output_changed = self.output_pane.poll_background_load();
        let diff_changed = self.diff_pane.poll_background_load();
        output_changed || diff_changed
    }

    pub fn is_loading(&self) -> bool {
        self.output_pane.is_loading() || self.diff_pane.is_loading()
    }

    pub fn drain_background_tasks(&mut self, max_polls: usize) {
        for _ in 0..max_polls {
            let changed = self.poll_background_tasks();
            if !self.is_loading() {
                return;
            }
            if !changed {
                std::thread::yield_now();
            }
        }
    }

    pub fn render_snapshot(&self) -> MonitorSnapshot {
        let output = self.output_pane.snapshot(120);
        MonitorSnapshot {
            sidebar: build_sidebar_entries(&self.state, self.active_sidebar_row),
            output_placeholder: output.title.clone(),
            output,
            diff: self.diff_pane.snapshot(120),
            focused_surface: self.state.active_surface.clone(),
        }
    }

    fn refresh_output_pane(&mut self) {
        self.output_pane.load_fixture_async(
            self.output_title(),
            self.state.active_selection_key(),
            self.output_fixture_text(),
            200,
        );
    }

    fn refresh_diff_pane(&mut self) {
        let (base_ref, head_ref) = self.state.active_diff_refs();
        self.diff_pane
            .load_for_selection(self.state.active_selection_key(), base_ref, head_ref);
    }
    fn output_title(&self) -> String {
        let active_session = self.state.active_session();
        let active_agent = self.state.active_agent();
        format!("Output pane: {} via {}", active_session.id, active_agent.model)
    }

    fn output_fixture_text(&self) -> String {
        let active_session = self.state.active_session();
        let active_agent = self.state.active_agent();
        (0..600)
            .map(|line_index| {
                format!(
                    "{} {} log line {}",
                    active_session.id, active_agent.model, line_index
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}
