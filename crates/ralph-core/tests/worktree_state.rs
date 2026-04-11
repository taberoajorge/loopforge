use ralph_core::{LoopExecutionState, WorktreeExecutionState, PRIMARY_WORKTREE_ID};
use serde_json::json;

#[test]
fn single_worktree_state_matches_current_flow() {
    let state = LoopExecutionState::single("/repo");

    assert_eq!(state.primary_worktree_id, PRIMARY_WORKTREE_ID);
    assert_eq!(
        state.active_worktree_id.as_deref(),
        Some(PRIMARY_WORKTREE_ID)
    );
    assert_eq!(
        state.resolved_worktrees(),
        vec![WorktreeExecutionState::primary("/repo")]
    );
}

#[test]
fn legacy_state_without_worktree_fields_deserializes_to_primary_worktree() {
    let state: LoopExecutionState = serde_json::from_value(json!({
        "iteration": 4,
        "currentStoryId": "S-013",
        "workDir": "/repo"
    }))
    .unwrap();

    assert_eq!(state.primary_worktree_id, PRIMARY_WORKTREE_ID);
    assert_eq!(state.active_worktree().id, PRIMARY_WORKTREE_ID);
    assert_eq!(state.active_worktree().work_dir, "/repo");
    assert_eq!(state.resolved_worktrees().len(), 1);
}

#[test]
fn multi_worktree_state_preserves_distinct_identifiers() {
    let state: LoopExecutionState = serde_json::from_value(json!({
        "iteration": 9,
        "workDir": "/repo",
        "primaryWorktreeId": "main",
        "activeWorktreeId": "story-s013",
        "worktrees": [
            {"id": "main", "workDir": "/repo", "branch": "main", "iteration": 9},
            {
                "id": "story-s013",
                "workDir": "/repo/.loopforge/worktrees/story-s013",
                "branch": "loopforge/story-s013",
                "storyId": "S-013",
                "iteration": 2,
                "lastPromptHash": "abc123",
                "headCommit": "deadbeef"
            }
        ]
    }))
    .unwrap();

    let worktrees = state.resolved_worktrees();

    assert_eq!(worktrees.len(), 2);
    assert_eq!(state.active_worktree().id, "story-s013");
    assert_eq!(worktrees[0].id, "main");
    assert_eq!(worktrees[1].id, "story-s013");
    assert_eq!(worktrees[0].work_dir, "/repo");
    assert_eq!(
        worktrees[1].work_dir,
        "/repo/.loopforge/worktrees/story-s013"
    );
}
