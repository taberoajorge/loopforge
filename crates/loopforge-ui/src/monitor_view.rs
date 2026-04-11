use crate::monitor_state::{MonitorState, MonitorSurface};
use crate::sidebar::{build_sidebar_entries, row_count, SidebarEntry};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorSnapshot {
    pub sidebar: Vec<SidebarEntry>,
    pub output_placeholder: String,
    pub diff_placeholder: String,
    pub focused_surface: MonitorSurface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorView {
    state: MonitorState,
    active_sidebar_row: usize,
}

impl MonitorView {
    pub fn seeded() -> Self {
        Self {
            state: MonitorState::seeded(),
            active_sidebar_row: 0,
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
        }
        changed
    }

    pub fn cycle_focus(&mut self) {
        self.state.cycle_focus();
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
    use crate::monitor_state::MonitorSurface;

    #[test]
    fn selecting_sidebar_row_updates_output_and_diff_placeholders() {
        let mut monitor_view = MonitorView::seeded();
        let changed = monitor_view.select_sidebar_row(2);
        assert!(changed);
        let snapshot = monitor_view.render_snapshot();
        assert_eq!(snapshot.output_placeholder, "Output pane: session-2026-04-11 via gpt-5-codex");
        assert_eq!(snapshot.diff_placeholder, "Diff pane: Current Session for Codex");
        let changed_agent = monitor_view.select_sidebar_row(3);
        assert!(changed_agent);
        let updated_snapshot = monitor_view.render_snapshot();
        assert_eq!(
            updated_snapshot.output_placeholder,
            "Output pane: session-2026-04-11 via claude-sonnet-4-5"
        );
        assert_eq!(
            updated_snapshot.diff_placeholder,
            "Diff pane: Current Session for Claude"
        );
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
        assert_eq!(
            second_snapshot.diff_placeholder,
            "Diff pane: Previous Session for Codex"
        );
        monitor_view.cycle_focus();
        let third_snapshot = monitor_view.render_snapshot();
        assert_eq!(third_snapshot.focused_surface, MonitorSurface::Sidebar);
        assert_eq!(
            third_snapshot.output_placeholder,
            "Output pane: session-2026-04-10 via gpt-5-codex"
        );
    }
}
