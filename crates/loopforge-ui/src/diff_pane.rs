use ralph_core::{UnifiedDiffRequest, unified_diff};
use std::path::PathBuf;

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffPaneState {
    repository_path: PathBuf,
    request_key: String,
    request_count: usize,
    is_loading: bool,
    lines: Vec<DiffLine>,
    scroll_top_line: usize,
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
        self.is_loading = true;
        let mut request = match base_ref {
            Some(base_ref_value) => {
                let head_ref_value = head_ref.unwrap_or("HEAD");
                UnifiedDiffRequest::between_refs(
                    self.repository_path.clone(),
                    base_ref_value.to_string(),
                    head_ref_value.to_string(),
                )
            }
            None => UnifiedDiffRequest::working_tree(self.repository_path.clone()),
        };
        request = request.with_context_lines(3);
        let patch_text = unified_diff(&request).unwrap_or_else(|error| error.to_string());
        self.request_count += 1;
        self.replace_patch_text(request_key, patch_text);
        self.is_loading = false;
    }

    pub fn replace_patch_text(&mut self, request_key: String, patch_text: String) {
        self.request_key = request_key;
        self.lines = parse_patch_lines(&patch_text);
        self.clamp_scroll_top_line();
    }

    pub fn set_scroll_top_line(&mut self, scroll_top_line: usize) {
        self.scroll_top_line = scroll_top_line;
        self.clamp_scroll_top_line();
    }

    pub fn snapshot(&self, viewport_rows: usize) -> DiffPaneSnapshot {
        let rows = viewport_rows.max(1);
        let start = self.scroll_top_line;
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

    fn clamp_scroll_top_line(&mut self) {
        if self.lines.is_empty() {
            self.scroll_top_line = 0;
            return;
        }
        let max_scroll_top_line = self.lines.len() - 1;
        self.scroll_top_line = self.scroll_top_line.min(max_scroll_top_line);
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
    DiffLine {
        text,
        kind,
        dark_token,
        light_token,
    }
}

fn classify_line_kind(line: &str) -> DiffLineKind {
    if line.starts_with("@@") {
        return DiffLineKind::Hunk;
    }
    if line.starts_with("diff --git")
        || line.starts_with("index ")
        || line.starts_with("--- ")
        || line.starts_with("+++ ")
    {
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

#[cfg(test)]
mod tests {
    use super::DiffPaneState;
    use std::path::PathBuf;

    #[test]
    fn large_patch_stays_scrollable_without_truncating_visible_rows() {
        let mut diff_pane = DiffPaneState::new(PathBuf::from("."));
        let large_patch = (0..6000)
            .map(|line_index| format!("+added line {line_index}"))
            .collect::<Vec<String>>()
            .join("\n");
        diff_pane.replace_patch_text("large".to_string(), large_patch);
        diff_pane.set_scroll_top_line(4900);
        let snapshot = diff_pane.snapshot(80);
        assert_eq!(snapshot.visible_lines.len(), 80);
        assert_eq!(snapshot.visible_lines[0].text, "+added line 4900");
        assert_eq!(snapshot.visible_lines[79].text, "+added line 4979");
    }

    #[test]
    fn dark_and_light_theme_tokens_follow_diff_line_kind() {
        let mut diff_pane = DiffPaneState::new(PathBuf::from("."));
        let patch = "diff --git a/file.txt b/file.txt\n@@ -1 +1 @@\n-line\n+line\n line";
        diff_pane.replace_patch_text("tokens".to_string(), patch.to_string());
        let snapshot = diff_pane.snapshot(10);
        assert_eq!(snapshot.visible_lines[0].dark_token, "diff-meta-dark");
        assert_eq!(snapshot.visible_lines[0].light_token, "diff-meta-light");
        assert_eq!(snapshot.visible_lines[2].dark_token, "diff-removed-dark");
        assert_eq!(snapshot.visible_lines[3].light_token, "diff-added-light");
        assert_eq!(snapshot.visible_lines[4].dark_token, "diff-context-dark");
    }
}
