use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeCompletion {
    pub worktree_id: String,
    pub story_id: String,
    pub passed: bool,
    #[serde(default)]
    pub blocked: bool,
    #[serde(default)]
    pub head_commit: Option<String>,
    #[serde(default)]
    pub guardrail_append: Option<String>,
    #[serde(default)]
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MergeAction {
    UpdateStoryStatus {
        story_id: String,
        passed: bool,
        blocked: bool,
    },
    AppendGuardrail {
        story_id: String,
        content: String,
    },
    UpdateSessionHead {
        worktree_id: String,
        head_commit: String,
    },
}

#[derive(Debug, Default)]
pub struct CompletionScheduler {
    pending: Vec<WorktreeCompletion>,
}

impl CompletionScheduler {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    pub fn submit(&mut self, completion: WorktreeCompletion) {
        self.pending.push(completion);
    }

    pub fn submit_batch(&mut self, completions: Vec<WorktreeCompletion>) {
        self.pending.extend(completions);
    }

    pub fn drain_ordered(&mut self) -> Vec<MergeAction> {
        let mut batch = std::mem::take(&mut self.pending);
        batch.sort_by(deterministic_order);
        batch.into_iter().flat_map(expand_actions).collect()
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

pub fn schedule(completions: Vec<WorktreeCompletion>) -> Vec<MergeAction> {
    let mut scheduler = CompletionScheduler::new();
    scheduler.submit_batch(completions);
    scheduler.drain_ordered()
}

fn deterministic_order(left: &WorktreeCompletion, right: &WorktreeCompletion) -> Ordering {
    left.story_id
        .cmp(&right.story_id)
        .then_with(|| left.sequence.cmp(&right.sequence))
        .then_with(|| left.worktree_id.cmp(&right.worktree_id))
}

fn expand_actions(completion: WorktreeCompletion) -> Vec<MergeAction> {
    let mut actions = Vec::with_capacity(3);
    actions.push(MergeAction::UpdateStoryStatus {
        story_id: completion.story_id.clone(),
        passed: completion.passed,
        blocked: completion.blocked,
    });
    if let Some(content) = completion.guardrail_append {
        actions.push(MergeAction::AppendGuardrail {
            story_id: completion.story_id.clone(),
            content,
        });
    }
    if let Some(head_commit) = completion.head_commit {
        actions.push(MergeAction::UpdateSessionHead {
            worktree_id: completion.worktree_id,
            head_commit,
        });
    }
    actions
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn empty_scheduler_produces_no_actions() {
        let actions = schedule(vec![]);
        assert!(actions.is_empty());
    }

    #[test]
    fn single_completion_expands_to_status_action() {
        let actions = schedule(vec![completion("S-001", "wt-a", true, 0)]);
        assert_eq!(actions.len(), 1);
        assert_eq!(
            actions[0],
            MergeAction::UpdateStoryStatus {
                story_id: "S-001".into(),
                passed: true,
                blocked: false,
            }
        );
    }

    #[test]
    fn completion_with_guardrail_expands_to_two_actions() {
        let mut comp = completion("S-002", "wt-a", false, 0);
        comp.guardrail_append = Some("circuit breaker hit".into());
        let actions = schedule(vec![comp]);
        assert_eq!(actions.len(), 2);
        assert!(matches!(&actions[0], MergeAction::UpdateStoryStatus { .. }));
        assert!(matches!(&actions[1], MergeAction::AppendGuardrail { .. }));
    }

    #[test]
    fn completion_with_head_commit_expands_to_session_update() {
        let mut comp = completion("S-003", "wt-b", true, 0);
        comp.head_commit = Some("abc123".into());
        let actions = schedule(vec![comp]);
        assert_eq!(actions.len(), 2);
        assert_eq!(
            actions[1],
            MergeAction::UpdateSessionHead {
                worktree_id: "wt-b".into(),
                head_commit: "abc123".into(),
            }
        );
    }

    #[test]
    fn reverse_arrival_produces_same_order() {
        let forward = schedule(vec![
            completion("S-001", "wt-a", true, 0),
            completion("S-002", "wt-b", false, 1),
            completion("S-003", "wt-c", true, 2),
        ]);
        let reverse = schedule(vec![
            completion("S-003", "wt-c", true, 2),
            completion("S-002", "wt-b", false, 1),
            completion("S-001", "wt-a", true, 0),
        ]);
        assert_eq!(forward, reverse);
    }

    #[test]
    fn scheduler_state_drains_on_each_call() {
        let mut sched = CompletionScheduler::new();
        sched.submit(completion("S-001", "wt-a", true, 0));
        assert_eq!(sched.pending_count(), 1);
        let first = sched.drain_ordered();
        assert_eq!(first.len(), 1);
        assert_eq!(sched.pending_count(), 0);
        let second = sched.drain_ordered();
        assert!(second.is_empty());
    }
}
