use loopforge_ui::MonitorView;

fn main() {
    let mut monitor_view = MonitorView::seeded();
    let _ = monitor_view.select_sidebar_row(1);
    monitor_view.cycle_focus();
    drop(monitor_view.render_snapshot());
}
