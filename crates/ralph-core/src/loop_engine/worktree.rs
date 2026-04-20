use serde::{Deserialize, Serialize};

pub const PRIMARY_WORKTREE_ID: &str = "primary";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct LoopExecutionState {
    pub iteration: u32,
    pub current_story_id: Option<String>,
    pub work_dir: String,
    pub primary_worktree_id: String,
    pub active_worktree_id: Option<String>,
    pub worktrees: Vec<WorktreeExecutionState>,
}

impl Default for LoopExecutionState {
    fn default() -> Self {
        Self {
            iteration: 0,
            current_story_id: None,
            work_dir: String::new(),
            primary_worktree_id: PRIMARY_WORKTREE_ID.to_string(),
            active_worktree_id: None,
            worktrees: Vec::new(),
        }
    }
}

impl LoopExecutionState {
    pub fn single(work_dir: impl Into<String>) -> Self {
        let work_dir = work_dir.into();
        Self {
            work_dir: work_dir.clone(),
            primary_worktree_id: PRIMARY_WORKTREE_ID.to_string(),
            active_worktree_id: Some(PRIMARY_WORKTREE_ID.to_string()),
            worktrees: vec![WorktreeExecutionState::primary(work_dir)],
            ..Self::default()
        }
    }

    pub fn resolved_worktrees(&self) -> Vec<WorktreeExecutionState> {
        if self.worktrees.is_empty() {
            return vec![self.primary_worktree()];
        }
        self.worktrees.clone()
    }

    pub fn primary_worktree(&self) -> WorktreeExecutionState {
        self.worktrees
            .iter()
            .find(|worktree| worktree.id == self.primary_worktree_id)
            .cloned()
            .or_else(|| self.worktrees.first().cloned())
            .unwrap_or_else(|| WorktreeExecutionState::primary(self.work_dir.clone()))
    }

    pub fn active_worktree(&self) -> WorktreeExecutionState {
        let active_id = self
            .active_worktree_id
            .as_deref()
            .unwrap_or(&self.primary_worktree_id);
        self.worktree(active_id)
            .unwrap_or_else(|| self.primary_worktree())
    }

    pub fn worktree(&self, worktree_id: &str) -> Option<WorktreeExecutionState> {
        self.resolved_worktrees()
            .into_iter()
            .find(|worktree| worktree.id == worktree_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct WorktreeExecutionState {
    pub id: String,
    pub work_dir: String,
    pub branch: Option<String>,
    pub story_id: Option<String>,
    pub iteration: u32,
    pub last_prompt_hash: Option<String>,
    pub head_commit: Option<String>,
}

impl WorktreeExecutionState {
    pub fn primary(work_dir: impl Into<String>) -> Self {
        Self {
            id: PRIMARY_WORKTREE_ID.to_string(),
            work_dir: work_dir.into(),
            ..Self::default()
        }
    }
}
