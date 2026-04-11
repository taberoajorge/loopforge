use loopforge_ui::{MonitorSurface, MonitorView};

fn wait_until_idle(monitor_view: &mut MonitorView) {
    monitor_view.drain_background_tasks(50_000);
    assert!(!monitor_view.is_loading());
}

fn build_log_fixture(line_count: usize) -> String {
    (0..line_count)
        .map(|line_index| format!("fixture log line {}", line_index))
        .collect::<Vec<String>>()
        .join("\n")
}

fn build_patch_fixture(line_count: usize) -> String {
    let mut lines = vec![
        "diff --git a/file.txt b/file.txt".to_string(),
        "--- a/file.txt".to_string(),
        "+++ b/file.txt".to_string(),
        "@@ -1 +1 @@".to_string(),
    ];
    lines.extend(
        (0..line_count)
            .map(|line_index| format!("+added line {}", line_index))
            .collect::<Vec<String>>(),
    );
    lines.join("\n")
}

#[test]
fn fixture_playback_keeps_window_interactive() {
    let mut monitor_view = MonitorView::seeded();
    monitor_view.begin_output_fixture_playback("fixture-output".to_string(), build_log_fixture(8_000));
    monitor_view.begin_diff_fixture_playback("fixture-diff".to_string(), build_patch_fixture(12_000));
    let loading_snapshot = monitor_view.render_snapshot();
    assert!(loading_snapshot.output.stream.is_loading || loading_snapshot.diff.is_loading);
    monitor_view.cycle_focus();
    assert_eq!(monitor_view.render_snapshot().focused_surface, MonitorSurface::Output);
    monitor_view.cycle_focus();
    assert_eq!(monitor_view.render_snapshot().focused_surface, MonitorSurface::Diff);
    monitor_view.set_output_scroll_top_line(400);
    monitor_view.set_diff_scroll_top_line(400);
    wait_until_idle(&mut monitor_view);
    let settled_snapshot = monitor_view.render_snapshot();
    assert_eq!(settled_snapshot.output.stream.total_lines, 8_000);
    assert!(settled_snapshot.diff.total_lines >= 12_000);
}

#[test]
fn scrolls_through_five_thousand_output_lines_repeatably() {
    let mut monitor_view = MonitorView::seeded();
    monitor_view.begin_output_fixture_playback("scroll-fixture".to_string(), build_log_fixture(6_200));
    monitor_view.begin_diff_fixture_playback("scroll-diff".to_string(), build_patch_fixture(20));
    wait_until_idle(&mut monitor_view);
    monitor_view.set_output_scroll_top_line(5_000);
    let snapshot = monitor_view.render_snapshot();
    assert_eq!(snapshot.output.stream.visible_lines.len(), 120);
    assert_eq!(snapshot.output.stream.visible_lines[0], "fixture log line 5000");
}

#[test]
fn renders_large_patch_without_truncating_visible_rows() {
    let mut monitor_view = MonitorView::seeded();
    monitor_view.begin_diff_fixture_playback("large-diff".to_string(), build_patch_fixture(7_500));
    wait_until_idle(&mut monitor_view);
    monitor_view.set_diff_scroll_top_line(5_000);
    let snapshot = monitor_view.render_snapshot();
    assert_eq!(snapshot.diff.visible_lines.len(), 120);
    assert!(snapshot.diff.visible_lines[0].text.starts_with("+added line "));
}

#[test]
fn keyboard_focus_remains_reliable_after_repeated_cycles() {
    let mut monitor_view = MonitorView::seeded();
    for cycle_index in 0..300 {
        monitor_view.cycle_focus();
        let expected_surface = match cycle_index % 3 {
            0 => MonitorSurface::Output,
            1 => MonitorSurface::Diff,
            _ => MonitorSurface::Sidebar,
        };
        assert_eq!(monitor_view.render_snapshot().focused_surface, expected_surface);
    }
}

#[test]
fn dark_and_light_tokens_remain_in_parity_for_diff_lines() {
    let mut monitor_view = MonitorView::seeded();
    let patch = "diff --git a/file.txt b/file.txt\n@@ -1 +1 @@\n-line\n+line\n line";
    monitor_view.begin_diff_fixture_playback("token-parity".to_string(), patch.to_string());
    wait_until_idle(&mut monitor_view);
    let snapshot = monitor_view.render_snapshot();
    assert!(snapshot.diff.visible_lines.iter().all(|line| !line.dark_token.is_empty()));
    assert!(snapshot.diff.visible_lines.iter().all(|line| !line.light_token.is_empty()));
    assert!(snapshot
        .diff
        .visible_lines
        .iter()
        .any(|line| line.text.starts_with("diff --git") && line.dark_token == "diff-meta-dark"));
    assert!(snapshot
        .diff
        .visible_lines
        .iter()
        .any(|line| line.text.starts_with("@@") && line.light_token == "diff-hunk-light"));
    assert!(snapshot
        .diff
        .visible_lines
        .iter()
        .any(|line| line.text.starts_with('+') && line.dark_token == "diff-added-dark"));
    assert!(snapshot
        .diff
        .visible_lines
        .iter()
        .any(|line| line.text.starts_with('-') && line.light_token == "diff-removed-light"));
}
