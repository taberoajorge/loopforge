# Monitor POC Phase 1 Hard Gates

## Gate Results

| Hard gate | Result | Evidence |
| --- | --- | --- |
| Log ingestion and diff loading stay off the UI thread during fixture playback | Pass | `crates/loopforge-ui/tests/monitor_poc.rs` covers async log fixture playback + async diff fixture playback and confirms focus/scroll interactions while loading remains in progress. |
| 5,000-line scroll performance is repeatable | Pass | `scrolls_through_five_thousand_output_lines_repeatably` verifies stable viewport rendering at line 5,000 for a 6,200-line log fixture. |
| Large patch diff rendering is repeatable | Pass | `renders_large_patch_without_truncating_visible_rows` validates a 7,500-line patch fixture and confirms full viewport population at deep scroll offsets. |
| Keyboard focus reliability is repeatable | Pass | `keyboard_focus_remains_reliable_after_repeated_cycles` runs 300 focus transitions and validates the full Sidebar → Output → Diff loop without drift. |
| Dark/light parity is repeatable | Pass | `dark_and_light_tokens_remain_in_parity_for_diff_lines` checks both token families for metadata, hunk, added, and removed lines. |

## Phase 2 Decision

Floem passes every Phase 1 hard gate in this POC. Recommendation: continue to Phase 2 on Floem and keep `iced` as contingency only if later integration regressions break these checks.
