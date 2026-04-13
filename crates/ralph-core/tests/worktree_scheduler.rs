use ralph_core::loop_engine::scheduler::{
    self, CompletionScheduler, MergeAction, WorktreeCompletion,
};

fn completion(
    story_id: &str,
    worktree_id: &str,
    passed: bool,
    sequence: u64,
) -> WorktreeCompletion {
    WorktreeCompletion {
        worktree_id: worktree_id.to_string(),
        story_id: story_id.to_string(),
        passed,
        blocked: false,
        head_commit: None,
        guardrail_append: None,
        sequence,
    }
}

fn story_ids(actions: &[MergeAction]) -> Vec<String> {
    actions
        .iter()
        .filter_map(|action| match action {
            MergeAction::UpdateStoryStatus { story_id, .. } => Some(story_id.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn conflicting_completions_produce_stable_ordering() {
    let completions = vec![
        completion("S-003", "wt-c", true, 2),
        completion("S-001", "wt-a", true, 0),
        completion("S-002", "wt-b", false, 1),
    ];
    let first_run = scheduler::schedule(completions.clone());
    let second_run = scheduler::schedule(completions.clone());
    let third_run = scheduler::schedule(completions);

    assert_eq!(first_run, second_run);
    assert_eq!(second_run, third_run);
    assert_eq!(story_ids(&first_run), vec!["S-001", "S-002", "S-003"]);
}

#[test]
fn same_story_from_different_worktrees_ordered_by_sequence_then_id() {
    let completions = vec![
        completion("S-001", "wt-b", false, 2),
        completion("S-001", "wt-a", true, 1),
    ];
    let actions = scheduler::schedule(completions);
    let worktree_order: Vec<&str> = actions
        .iter()
        .filter_map(|action| match action {
            MergeAction::UpdateStoryStatus { story_id, .. } if story_id == "S-001" => {
                Some("status")
            }
            _ => None,
        })
        .collect();
    assert_eq!(worktree_order.len(), 2);

    assert_eq!(
        actions[0],
        MergeAction::UpdateStoryStatus {
            story_id: "S-001".into(),
            passed: true,
            blocked: false,
        }
    );
    assert_eq!(
        actions[1],
        MergeAction::UpdateStoryStatus {
            story_id: "S-001".into(),
            passed: false,
            blocked: false,
        }
    );
}

#[test]
fn mixed_actions_interleave_correctly_per_story() {
    let mut comp_a = completion("S-002", "wt-a", true, 0);
    comp_a.head_commit = Some("aaa111".into());

    let mut comp_b = completion("S-001", "wt-b", false, 1);
    comp_b.guardrail_append = Some("retry limit".into());
    comp_b.head_commit = Some("bbb222".into());

    let actions = scheduler::schedule(vec![comp_a, comp_b]);

    assert_eq!(actions.len(), 5);
    assert_eq!(
        actions[0],
        MergeAction::UpdateStoryStatus {
            story_id: "S-001".into(),
            passed: false,
            blocked: false,
        }
    );
    assert_eq!(
        actions[1],
        MergeAction::AppendGuardrail {
            story_id: "S-001".into(),
            content: "retry limit".into(),
        }
    );
    assert_eq!(
        actions[2],
        MergeAction::UpdateSessionHead {
            worktree_id: "wt-b".into(),
            head_commit: "bbb222".into(),
        }
    );
    assert_eq!(
        actions[3],
        MergeAction::UpdateStoryStatus {
            story_id: "S-002".into(),
            passed: true,
            blocked: false,
        }
    );
    assert_eq!(
        actions[4],
        MergeAction::UpdateSessionHead {
            worktree_id: "wt-a".into(),
            head_commit: "aaa111".into(),
        }
    );
}

#[test]
fn scheduler_api_does_not_require_tauri_or_shell_types() {
    let mut sched = CompletionScheduler::new();
    sched.submit(completion("S-001", "wt-a", true, 0));
    sched.submit(completion("S-002", "wt-b", false, 1));
    assert_eq!(sched.pending_count(), 2);

    let actions = sched.drain_ordered();
    assert_eq!(actions.len(), 2);
    assert_eq!(sched.pending_count(), 0);
}

#[test]
fn batch_submit_matches_individual_submits() {
    let completions = vec![
        completion("S-003", "wt-c", true, 2),
        completion("S-001", "wt-a", true, 0),
        completion("S-002", "wt-b", false, 1),
    ];

    let batch_result = scheduler::schedule(completions.clone());

    let mut sched = CompletionScheduler::new();
    for comp in completions {
        sched.submit(comp);
    }
    let individual_result = sched.drain_ordered();

    assert_eq!(batch_result, individual_result);
}

#[test]
fn blocked_completion_propagates_blocked_flag() {
    let mut comp = completion("S-005", "wt-a", false, 0);
    comp.blocked = true;
    let actions = scheduler::schedule(vec![comp]);
    assert_eq!(
        actions[0],
        MergeAction::UpdateStoryStatus {
            story_id: "S-005".into(),
            passed: false,
            blocked: true,
        }
    );
}

#[test]
fn large_concurrent_set_stays_deterministic() {
    let mut completions: Vec<WorktreeCompletion> = (0..50)
        .map(|idx| {
            completion(
                &format!("S-{idx:03}"),
                &format!("wt-{idx}"),
                idx % 3 == 0,
                idx,
            )
        })
        .collect();
    let forward = scheduler::schedule(completions.clone());
    completions.reverse();
    let reversed = scheduler::schedule(completions);
    assert_eq!(forward, reversed);
}

#[test]
fn serialization_roundtrip_preserves_completion() {
    let mut comp = completion("S-010", "wt-x", true, 5);
    comp.head_commit = Some("deadbeef".into());
    comp.guardrail_append = Some("limit reached".into());

    let json = serde_json::to_string(&comp).expect("should serialize");
    let restored: WorktreeCompletion = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(comp, restored);
}

#[test]
fn serialization_roundtrip_preserves_merge_action() {
    let action = MergeAction::AppendGuardrail {
        story_id: "S-001".into(),
        content: "circuit breaker".into(),
    };
    let json = serde_json::to_string(&action).expect("should serialize");
    let restored: MergeAction = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(action, restored);
}
