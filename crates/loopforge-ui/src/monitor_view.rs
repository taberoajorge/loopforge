use crate::monitor_state::{MonitorState, MonitorSurface};
use crate::sidebar::{build_sidebar_entries, row_count, SidebarEntry};
use std::path::PathBuf;

#[path = "diff_pane.rs"]
mod diff_pane;
use diff_pane::{DiffPaneSnapshot, DiffPaneState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorSnapshot {
    pub sidebar: Vec<SidebarEntry>,
    pub output_placeholder: String,
    pub diff: DiffPaneSnapshot,
    pub focused_surface: MonitorSurface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorView {
    state: MonitorState,
    active_sidebar_row: usize,
    diff_pane: DiffPaneState,
}

impl MonitorView {
    pub fn seeded() -> Self {
        let working_directory =
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let repository_path = DiffPaneState::resolve_repository_path(working_directory);
        let mut monitor_view = Self {
            state: MonitorState::seeded(),
            active_sidebar_row: 0,
            diff_pane: DiffPaneState::new(repository_path),
        };
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

    pub fn render_snapshot(&self) -> MonitorSnapshot {
        let active_session = self.state.active_session();
        let active_agent = self.state.active_agent();
        MonitorSnapshot {
            sidebar: build_sidebar_entries(&self.state, self.active_sidebar_row),
            output_placeholder: format!(
                "Output pane: {} via {}",
                active_session.id, active_agent.model
            ),
            diff: self.diff_pane.snapshot(120),
            focused_surface: self.state.active_surface.clone(),
        }
    }

    fn refresh_diff_pane(&mut self) {
        let (base_ref, head_ref) = self.state.active_diff_refs();
        self.diff_pane
            .load_for_selection(self.state.active_selection_key(), base_ref, head_ref);
    }
}

#[cfg(test)]
mod tests {
    use super::MonitorView;
    use crate::monitor_state::MonitorSurface;

    #[test]
    fn selecting_sidebar_row_requests_new_unified_diff() {
        let mut monitor_view = MonitorView::seeded();
        let initial_snapshot = monitor_view.render_snapshot();
        assert_eq!(initial_snapshot.diff.request_count, 1);
        let first_request_key = initial_snapshot.diff.request_key.clone();
        let changed = monitor_view.select_sidebar_row(1);
        assert!(changed);
        let snapshot = monitor_view.render_snapshot();
        assert_eq!(snapshot.output_placeholder, "Output pane: session-2026-04-10 via gpt-5-codex");
        assert_eq!(snapshot.diff.request_count, 2);
        assert_ne!(snapshot.diff.request_key, first_request_key);
        let changed_agent = monitor_view.select_sidebar_row(3);
        assert!(changed_agent);
        let updated_snapshot = monitor_view.render_snapshot();
        assert_eq!(updated_snapshot.output_placeholder, "Output pane: session-2026-04-10 via claude-sonnet-4-5");
        assert_eq!(updated_snapshot.diff.request_count, 3);
        assert_ne!(updated_snapshot.diff.request_key, snapshot.diff.request_key);
    }

    #[test]
    fn focus_cycles_across_all_surfaces_without_losing_selection() {
        let mut monitor_view = MonitorView::seeded();
        let changed = monitor_view.select_sidebar_row(1);
        assert!(changed);
        monitor_view.cycle_focus();
        let first_snapshot = monitor_view.render_snapshot();
        assert_eq!(first_snapshot.focused_surface, MonitorSurface::Output);
        assert_eq!(
            first_snapshot.output_placeholder,
            "Output pane: session-2026-04-10 via gpt-5-codex"
        );
        monitor_view.cycle_focus();
        let second_snapshot = monitor_view.render_snapshot();
        assert_eq!(second_snapshot.focused_surface, MonitorSurface::Diff);
        assert!(!second_snapshot.diff.visible_lines.is_empty());
        monitor_view.cycle_focus();
        let third_snapshot = monitor_view.render_snapshot();
        assert_eq!(third_snapshot.focused_surface, MonitorSurface::Sidebar);
        assert_eq!(
            third_snapshot.output_placeholder,
            "Output pane: session-2026-04-10 via gpt-5-codex"
        );
    }

    #[test]
    fn diff_scroll_position_is_preserved_when_switching_selections() {
        let mut monitor_view = MonitorView::seeded();
        monitor_view.set_diff_scroll_top_line(20);
        let first_snapshot = monitor_view.render_snapshot();
        let first_scroll_top = first_snapshot.diff.scroll_top_line;
        let changed = monitor_view.select_sidebar_row(1);
        assert!(changed);
        let second_snapshot = monitor_view.render_snapshot();
        assert_eq!(second_snapshot.diff.scroll_top_line, first_scroll_top.min(second_snapshot.diff.total_lines.saturating_sub(1)));
    }
}
