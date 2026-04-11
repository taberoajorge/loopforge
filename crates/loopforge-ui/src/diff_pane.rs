use ralph_core::{UnifiedDiffRequest, unified_diff};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiffLineKind {
    Added,
    Removed,
    Context,
    Metadata,
    Hunk,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub text: String,
    pub kind: DiffLineKind,
    pub dark_token: &'static str,
    pub light_token: &'static str,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffPaneSnapshot {
    pub request_key: String,
    pub request_count: usize,
    pub is_loading: bool,
    pub total_lines: usize,
    pub scroll_top_line: usize,
    pub visible_lines: Vec<DiffLine>,
}
#[derive(Debug)]
struct DiffLoadResult {
    request_ticket: usize,
    request_key: String,
    lines: Vec<DiffLine>,
}
#[derive(Debug)]
pub struct DiffPaneState {
    repository_path: PathBuf,
    request_key: String,
    request_count: usize,
    is_loading: bool,
    lines: Vec<DiffLine>,
    scroll_top_line: usize,
    request_ticket: usize,
    pending_receiver: Option<Receiver<DiffLoadResult>>,
}
impl DiffPaneState {
    pub fn new(repository_path: PathBuf) -> Self {
        Self {
            repository_path,
            request_key: String::new(),
            request_count: 0,
            is_loading: false,
            lines: Vec::new(),
            scroll_top_line: 0,
            request_ticket: 0,
            pending_receiver: None,
        }
    }
    pub fn resolve_repository_path(start_path: PathBuf) -> PathBuf {
        let mut candidate_path = start_path;
        loop {
            if candidate_path.join(".git").exists() {
                return candidate_path;
            }
            let Some(parent_path) = candidate_path.parent() else {
                return candidate_path;
            };
            candidate_path = parent_path.to_path_buf();
        }
    }
    pub fn load_for_selection(
        &mut self,
        request_key: String,
        base_ref: Option<&str>,
        head_ref: Option<&str>,
    ) {
        let repository_path = self.repository_path.clone();
        let base_ref_value = base_ref.map(str::to_string);
        let head_ref_value = head_ref.map(str::to_string);
        self.start_async_load(request_key, move || {
            let mut request = match base_ref_value {
                Some(base_ref_item) => UnifiedDiffRequest::between_refs(
                    repository_path,
                    base_ref_item,
                    head_ref_value.unwrap_or_else(|| "HEAD".to_string()),
                ),
                None => UnifiedDiffRequest::working_tree(repository_path),
            };
            request = request.with_context_lines(3);
            let patch_text = unified_diff(&request).unwrap_or_else(|error| error.to_string());
            parse_patch_lines(&patch_text)
        });
    }
    pub fn load_fixture_async(&mut self, request_key: String, patch_text: String) {
        self.start_async_load(request_key, move || parse_patch_lines(&patch_text));
    }
    pub fn poll_background_load(&mut self) -> bool {
        let Some(receiver) = self.pending_receiver.take() else {
            return false;
        };
        match receiver.try_recv() {
            Ok(result) => {
                if result.request_ticket == self.request_ticket {
                    self.request_key = result.request_key;
                    self.lines = result.lines;
                    self.is_loading = false;
                    self.clamp_scroll_top_line();
                }
                true
            }
            Err(TryRecvError::Empty) => {
                self.pending_receiver = Some(receiver);
                false
            }
            Err(TryRecvError::Disconnected) => {
                self.is_loading = false;
                false
            }
        }
    }
    pub fn set_scroll_top_line(&mut self, scroll_top_line: usize) {
        self.scroll_top_line = scroll_top_line;
        self.clamp_scroll_top_line();
    }
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }
    pub fn snapshot(&self, viewport_rows: usize) -> DiffPaneSnapshot {
        let rows = viewport_rows.max(1);
        let start = self.scroll_top_line.min(self.lines.len());
        let end = (start + rows).min(self.lines.len());
        DiffPaneSnapshot {
            request_key: self.request_key.clone(),
            request_count: self.request_count,
            is_loading: self.is_loading,
            total_lines: self.lines.len(),
            scroll_top_line: self.scroll_top_line,
            visible_lines: self.lines[start..end].to_vec(),
        }
    }
    fn start_async_load(&mut self, request_key: String, build_lines: impl FnOnce() -> Vec<DiffLine> + Send + 'static) {
        self.request_count += 1;
        self.request_ticket += 1;
        self.request_key = request_key.clone();
        self.is_loading = true;
        let request_ticket = self.request_ticket;
        let (sender, receiver) = channel();
        std::thread::spawn(move || {
            let lines = build_lines();
            let _ = sender.send(DiffLoadResult {
                request_ticket,
                request_key,
                lines,
            });
        });
        self.pending_receiver = Some(receiver);
    }
    fn clamp_scroll_top_line(&mut self) {
        if self.lines.is_empty() {
            self.scroll_top_line = 0;
            return;
        }
        self.scroll_top_line = self.scroll_top_line.min(self.lines.len() - 1);
    }
}
fn parse_patch_lines(patch_text: &str) -> Vec<DiffLine> {
    if patch_text.is_empty() {
        return vec![line_from_text("Working tree is clean".to_string())];
    }
    patch_text.lines().map(|line| line_from_text(line.to_string())).collect()
}
fn line_from_text(text: String) -> DiffLine {
    let kind = classify_line_kind(&text);
    let (dark_token, light_token) = match kind {
        DiffLineKind::Added => ("diff-added-dark", "diff-added-light"),
        DiffLineKind::Removed => ("diff-removed-dark", "diff-removed-light"),
        DiffLineKind::Context => ("diff-context-dark", "diff-context-light"),
        DiffLineKind::Metadata => ("diff-meta-dark", "diff-meta-light"),
        DiffLineKind::Hunk => ("diff-hunk-dark", "diff-hunk-light"),
    };
    DiffLine { text, kind, dark_token, light_token }
}
fn classify_line_kind(line: &str) -> DiffLineKind {
    if line.starts_with("@@") {
        return DiffLineKind::Hunk;
    }
    if line.starts_with("diff --git") || line.starts_with("index ") || line.starts_with("--- ") || line.starts_with("+++ ") {
        return DiffLineKind::Metadata;
    }
    if line.starts_with('+') {
        return DiffLineKind::Added;
    }
    if line.starts_with('-') {
        return DiffLineKind::Removed;
    }
    DiffLineKind::Context
}
