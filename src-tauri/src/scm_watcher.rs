use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScmProvider {
    Github,
    Gitlab,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewComment {
    pub comment_id: String,
    pub pr_number: u64,
    pub body: String,
    pub file_path: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub reviewer: String,
    pub created_at: String,
    pub status: CommentStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentStatus {
    Pending,
    Routed,
    Addressed,
    Escalated,
}

pub struct ScmWatcherState(pub Mutex<ScmWatcherInner>);

impl Default for ScmWatcherState {
    fn default() -> Self {
        Self(Mutex::new(ScmWatcherInner {
            processed_comment_ids: HashSet::new(),
        }))
    }
}

pub struct ScmWatcherInner {
    pub processed_comment_ids: HashSet<String>,
}

pub async fn detect_scm_provider(work_dir: &Path) -> ScmProvider {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(work_dir)
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            let url = String::from_utf8_lossy(&out.stdout).to_lowercase();
            classify_remote_url(&url)
        }
        _ => ScmProvider::Unknown,
    }
}

fn classify_remote_url(url: &str) -> ScmProvider {
    if url.contains("github.com") {
        ScmProvider::Github
    } else if url.contains("gitlab.com") || url.contains("gitlab") {
        ScmProvider::Gitlab
    } else {
        ScmProvider::Unknown
    }
}

pub fn parse_owner_repo(remote_url: &str) -> Option<(String, String)> {
    let trimmed = remote_url.trim();

    let path_part = if let Some(rest) = trimmed.strip_prefix("git@") {
        rest.split_once(':').map(|(_, path)| path)
    } else {
        trimmed
            .strip_prefix("https://")
            .or_else(|| trimmed.strip_prefix("http://"))
            .and_then(|rest| rest.split_once('/').map(|(_, path)| path))
    };

    let cleaned = path_part?.trim_end_matches(".git").trim_end_matches('/');

    let parts: Vec<&str> = cleaned.splitn(3, '/').collect();
    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

pub async fn fetch_github_comments(
    owner: &str,
    repo: &str,
    pr_number: u64,
    work_dir: &Path,
) -> Result<Vec<ReviewComment>, String> {
    let endpoint = format!("repos/{owner}/{repo}/pulls/{pr_number}/comments");
    let output = Command::new("gh")
        .args(["api", &endpoint, "--paginate"])
        .current_dir(work_dir)
        .output()
        .await
        .map_err(|err| format!("Failed to run gh: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh api failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw_comments: Vec<GhComment> = serde_json::from_str(&stdout)
        .map_err(|err| format!("Failed to parse GitHub comments: {err}"))?;

    let review_comments = raw_comments
        .into_iter()
        .map(|gh| ReviewComment {
            comment_id: gh.id.to_string(),
            pr_number,
            body: gh.body,
            file_path: gh.path,
            line_start: gh.original_start_line.or(gh.original_line),
            line_end: gh.original_line,
            reviewer: gh.user.login,
            created_at: gh.created_at,
            status: CommentStatus::Pending,
        })
        .collect();

    Ok(review_comments)
}

pub async fn detect_github_pr(
    _owner: &str,
    _repo: &str,
    branch: &str,
    work_dir: &Path,
) -> Option<u64> {
    let output = Command::new("gh")
        .args([
            "pr", "list",
            "--head", branch,
            "--json", "number",
            "--limit", "1",
        ])
        .current_dir(work_dir)
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let prs: Vec<GhPrRef> = serde_json::from_str(&stdout).ok()?;
    prs.first().map(|pr| pr.number)
}

#[derive(Deserialize)]
struct GhComment {
    id: u64,
    body: String,
    path: Option<String>,
    original_line: Option<u32>,
    original_start_line: Option<u32>,
    user: GhUser,
    created_at: String,
}

#[derive(Deserialize)]
struct GhUser {
    login: String,
}

#[derive(Deserialize)]
struct GhPrRef {
    number: u64,
}

pub async fn fetch_gitlab_comments(
    project_id: &str,
    mr_iid: u64,
    work_dir: &Path,
) -> Result<Vec<ReviewComment>, String> {
    let endpoint = format!(
        "projects/{}/merge_requests/{mr_iid}/notes",
        urlencoding_simple(project_id)
    );
    let output = Command::new("glab")
        .args(["api", &endpoint])
        .current_dir(work_dir)
        .output()
        .await
        .map_err(|err| format!("Failed to run glab: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("glab api failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw_notes: Vec<GlabNote> = serde_json::from_str(&stdout)
        .map_err(|err| format!("Failed to parse GitLab notes: {err}"))?;

    let review_comments = raw_notes
        .into_iter()
        .filter(|note| !note.system)
        .map(|note| {
            let (file_path, line_start) = note
                .position
                .map(|pos| (pos.new_path, pos.new_line))
                .unwrap_or((None, None));
            ReviewComment {
                comment_id: note.id.to_string(),
                pr_number: mr_iid,
                body: note.body,
                file_path,
                line_start,
                line_end: line_start,
                reviewer: note.author.username,
                created_at: note.created_at,
                status: CommentStatus::Pending,
            }
        })
        .collect();

    Ok(review_comments)
}

pub async fn detect_gitlab_mr(
    project_id: &str,
    branch: &str,
    work_dir: &Path,
) -> Option<u64> {
    let endpoint = format!(
        "projects/{}/merge_requests?source_branch={branch}&state=opened&per_page=1",
        urlencoding_simple(project_id)
    );
    let output = Command::new("glab")
        .args(["api", &endpoint])
        .current_dir(work_dir)
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mrs: Vec<GlabMrRef> = serde_json::from_str(&stdout).ok()?;
    mrs.first().map(|mr| mr.iid)
}

fn urlencoding_simple(input: &str) -> String {
    input.replace('/', "%2F")
}

#[derive(Deserialize)]
struct GlabNote {
    id: u64,
    body: String,
    system: bool,
    author: GlabAuthor,
    created_at: String,
    position: Option<GlabPosition>,
}

#[derive(Deserialize)]
struct GlabAuthor {
    username: String,
}

#[derive(Deserialize)]
struct GlabPosition {
    new_path: Option<String>,
    new_line: Option<u32>,
}

#[derive(Deserialize)]
struct GlabMrRef {
    iid: u64,
}

pub struct ReviewRouter;

impl ReviewRouter {
    pub fn build_prompt(
        comment: &ReviewComment,
        diff_hunk: Option<&str>,
        file_content: Option<&str>,
    ) -> String {
        let mut prompt = String::with_capacity(4096);

        prompt.push_str(&format!(
            "You are addressing a code review comment from {}.\n\n",
            comment.reviewer
        ));

        prompt.push_str(&format!("## Review Comment\n\n**PR #{}**\n", comment.pr_number));

        if let Some(file_path) = &comment.file_path {
            prompt.push_str(&format!("**File:** `{file_path}`"));
            if let Some(start) = comment.line_start {
                if let Some(end) = comment.line_end {
                    if end != start {
                        prompt.push_str(&format!(" (lines {start}:{end})"));
                    } else {
                        prompt.push_str(&format!(" (line {start})"));
                    }
                } else {
                    prompt.push_str(&format!(" (line {start})"));
                }
            }
            prompt.push('\n');
        }

        prompt.push_str(&format!("**Reviewer:** {}\n\n", comment.reviewer));
        prompt.push_str(&format!("> {}\n\n", comment.body));

        if let Some(hunk) = diff_hunk {
            prompt.push_str(&format!("## Diff Context\n\n```diff\n{hunk}\n```\n\n"));
        }

        if let Some(content) = file_content {
            let file_path = comment.file_path.as_deref().unwrap_or("unknown");
            prompt.push_str(&format!(
                "## File Content\n\n### `{file_path}`\n```\n{content}\n```\n\n"
            ));
        }

        prompt.push_str("## Instructions\n\n");
        prompt.push_str("1. Read the reviewer's comment carefully\n");
        prompt.push_str("2. Make the requested change in the affected file(s)\n");
        prompt.push_str("3. Commit the fix with a message referencing this review comment\n");
        prompt.push_str("4. Verify your change does not break existing tests\n");

        prompt
    }

    pub async fn route_to_agent(
        prompt: &str,
        agent_binary: &str,
        work_dir: &Path,
    ) -> Result<String, String> {
        let mut child = Command::new(agent_binary)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .current_dir(work_dir)
            .spawn()
            .map_err(|err| format!("Failed to spawn {agent_binary}: {err}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .await
                .map_err(|err| format!("Failed to write to agent stdin: {err}"))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|err| format!("Agent process error: {err}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }

    pub fn load_file_content(work_dir: &Path, file_path: &str) -> Option<String> {
        let full_path = work_dir.join(file_path);
        std::fs::read_to_string(full_path).ok()
    }
}

pub struct CommitWatcher;

impl CommitWatcher {
    pub async fn get_head_hash(work_dir: &Path) -> Option<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(work_dir)
            .output()
            .await
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    pub async fn has_new_commit_since(
        work_dir: &Path,
        baseline_hash: &str,
    ) -> bool {
        let current = Self::get_head_hash(work_dir).await;
        match current {
            Some(hash) => hash != baseline_hash,
            None => false,
        }
    }

    pub async fn wait_for_commit(
        work_dir: &Path,
        baseline_hash: &str,
        timeout_secs: u64,
        poll_interval_secs: u64,
    ) -> CommitWatchResult {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);
        let interval = std::time::Duration::from_secs(poll_interval_secs);

        loop {
            if start.elapsed() >= timeout {
                return CommitWatchResult::TimedOut;
            }

            if Self::has_new_commit_since(work_dir, baseline_hash).await {
                let new_hash = Self::get_head_hash(work_dir)
                    .await
                    .unwrap_or_default();
                return CommitWatchResult::NewCommit(new_hash);
            }

            tokio::time::sleep(interval).await;
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "result")]
pub enum CommitWatchResult {
    NewCommit(String),
    TimedOut,
}

pub fn filter_new_comments(
    comments: Vec<ReviewComment>,
    state: &ScmWatcherState,
) -> Vec<ReviewComment> {
    let guard = state.0.lock().expect("scm watcher lock");
    comments
        .into_iter()
        .filter(|comment| !guard.processed_comment_ids.contains(&comment.comment_id))
        .collect()
}

pub fn mark_comment_processed(state: &ScmWatcherState, comment_id: &str) {
    let mut guard = state.0.lock().expect("scm watcher lock");
    guard.processed_comment_ids.insert(comment_id.to_string());
}

pub fn is_approval_only(comments: &[ReviewComment]) -> bool {
    comments.is_empty()
        || comments.iter().all(|comment| {
            let body_lower = comment.body.to_lowercase();
            body_lower.contains("lgtm")
                || body_lower.contains("approved")
                || body_lower.starts_with(":+1:")
                || body_lower.starts_with("👍")
        })
}

pub async fn handle_comment_lifecycle(
    comment: &mut ReviewComment,
    agent_binary: &str,
    work_dir: &Path,
    timeout_secs: u64,
    app: &tauri::AppHandle,
    project_name: &str,
) {
    let baseline_hash = CommitWatcher::get_head_hash(work_dir)
        .await
        .unwrap_or_default();

    let file_content = comment
        .file_path
        .as_deref()
        .and_then(|fp| ReviewRouter::load_file_content(work_dir, fp));

    let prompt = ReviewRouter::build_prompt(
        comment,
        None,
        file_content.as_deref(),
    );

    comment.status = CommentStatus::Routed;
    let _ = ReviewRouter::route_to_agent(&prompt, agent_binary, work_dir).await;

    let watch_result = CommitWatcher::wait_for_commit(
        work_dir,
        &baseline_hash,
        timeout_secs,
        15,
    )
    .await;

    match watch_result {
        CommitWatchResult::NewCommit(_) => {
            comment.status = CommentStatus::Addressed;
        }
        CommitWatchResult::TimedOut => {
            comment.status = CommentStatus::Escalated;
            let file_display = comment
                .file_path
                .as_deref()
                .unwrap_or("unknown file");
            crate::notifications::notify_review_escalation(
                app,
                project_name,
                file_display,
                &comment.reviewer,
                timeout_secs / 60,
            );
        }
    }
}

#[tauri::command]
pub async fn detect_project_scm(working_directory: String) -> ScmProvider {
    detect_scm_provider(Path::new(&working_directory)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_github_https() {
        assert_eq!(
            classify_remote_url("https://github.com/user/repo.git"),
            ScmProvider::Github
        );
    }

    #[test]
    fn classify_github_ssh() {
        assert_eq!(
            classify_remote_url("git@github.com:user/repo.git"),
            ScmProvider::Github
        );
    }

    #[test]
    fn classify_gitlab_https() {
        assert_eq!(
            classify_remote_url("https://gitlab.com/group/project.git"),
            ScmProvider::Gitlab
        );
    }

    #[test]
    fn classify_self_hosted_gitlab() {
        assert_eq!(
            classify_remote_url("https://gitlab.company.io/team/app.git"),
            ScmProvider::Gitlab
        );
    }

    #[test]
    fn classify_unknown_remote() {
        assert_eq!(
            classify_remote_url("https://bitbucket.org/user/repo.git"),
            ScmProvider::Unknown
        );
    }

    #[test]
    fn parse_github_https_owner_repo() {
        let result = parse_owner_repo("https://github.com/example/my-repo.git");
        assert_eq!(result, Some(("example".into(), "my-repo".into())));
    }

    #[test]
    fn parse_github_ssh_owner_repo() {
        let result = parse_owner_repo("git@github.com:example/my-repo.git");
        assert_eq!(result, Some(("example".into(), "my-repo".into())));
    }

    #[test]
    fn parse_gitlab_group_project() {
        let result = parse_owner_repo("https://gitlab.com/group/project.git");
        assert_eq!(result, Some(("group".into(), "project".into())));
    }

    #[test]
    fn parse_owner_repo_returns_none_for_bare_host() {
        assert_eq!(parse_owner_repo("https://github.com"), None);
    }

    #[test]
    fn filter_new_comments_removes_processed() {
        let state = ScmWatcherState::default();
        mark_comment_processed(&state, "101");

        let comments = vec![
            ReviewComment {
                comment_id: "101".into(),
                pr_number: 1,
                body: "old".into(),
                file_path: None,
                line_start: None,
                line_end: None,
                reviewer: "alice".into(),
                created_at: "2026-01-01T00:00:00Z".into(),
                status: CommentStatus::Pending,
            },
            ReviewComment {
                comment_id: "102".into(),
                pr_number: 1,
                body: "new".into(),
                file_path: None,
                line_start: None,
                line_end: None,
                reviewer: "bob".into(),
                created_at: "2026-01-01T00:01:00Z".into(),
                status: CommentStatus::Pending,
            },
        ];

        let filtered = filter_new_comments(comments, &state);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].comment_id, "102");
    }

    #[test]
    fn is_approval_only_detects_lgtm() {
        let comments = vec![ReviewComment {
            comment_id: "200".into(),
            pr_number: 5,
            body: "LGTM, looks good!".into(),
            file_path: None,
            line_start: None,
            line_end: None,
            reviewer: "reviewer".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            status: CommentStatus::Pending,
        }];
        assert!(is_approval_only(&comments));
    }

    #[test]
    fn is_approval_only_rejects_change_request() {
        let comments = vec![ReviewComment {
            comment_id: "201".into(),
            pr_number: 5,
            body: "Please rename this variable".into(),
            file_path: Some("src/api.ts".into()),
            line_start: Some(42),
            line_end: Some(42),
            reviewer: "reviewer".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            status: CommentStatus::Pending,
        }];
        assert!(!is_approval_only(&comments));
    }

    #[test]
    fn is_approval_only_returns_true_for_empty() {
        assert!(is_approval_only(&[]));
    }

    #[test]
    fn review_router_builds_prompt_with_file_context() {
        let comment = ReviewComment {
            comment_id: "300".into(),
            pr_number: 10,
            body: "Rename x to requestCount".into(),
            file_path: Some("src/handler.ts".into()),
            line_start: Some(42),
            line_end: Some(42),
            reviewer: "alice".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            status: CommentStatus::Pending,
        };
        let prompt = ReviewRouter::build_prompt(&comment, None, Some("const x = 0;"));
        assert!(prompt.contains("alice"));
        assert!(prompt.contains("src/handler.ts"));
        assert!(prompt.contains("line 42"));
        assert!(prompt.contains("Rename x to requestCount"));
        assert!(prompt.contains("const x = 0;"));
    }

    #[test]
    fn review_router_builds_prompt_without_file() {
        let comment = ReviewComment {
            comment_id: "301".into(),
            pr_number: 10,
            body: "General comment".into(),
            file_path: None,
            line_start: None,
            line_end: None,
            reviewer: "bob".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            status: CommentStatus::Pending,
        };
        let prompt = ReviewRouter::build_prompt(&comment, None, None);
        assert!(prompt.contains("General comment"));
        assert!(prompt.contains("bob"));
        assert!(!prompt.contains("File Content"));
    }

    #[test]
    fn review_router_includes_diff_hunk() {
        let comment = ReviewComment {
            comment_id: "302".into(),
            pr_number: 10,
            body: "This line is wrong".into(),
            file_path: Some("src/main.rs".into()),
            line_start: Some(5),
            line_end: Some(5),
            reviewer: "eve".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            status: CommentStatus::Pending,
        };
        let hunk = "@@ -3,5 +3,5 @@\n-old line\n+new line";
        let prompt = ReviewRouter::build_prompt(&comment, Some(hunk), None);
        assert!(prompt.contains("Diff Context"));
        assert!(prompt.contains("+new line"));
    }

    #[test]
    fn urlencoding_simple_encodes_slashes() {
        assert_eq!(urlencoding_simple("group/project"), "group%2Fproject");
    }

    #[test]
    fn mock_github_comment_json_parsing() {
        let json = r#"[{
            "id": 999,
            "body": "Please fix this",
            "path": "src/lib.rs",
            "original_line": 10,
            "original_start_line": null,
            "user": { "login": "reviewer1" },
            "created_at": "2026-03-23T12:00:00Z"
        }]"#;
        let parsed: Vec<GhComment> = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].body, "Please fix this");
        assert_eq!(parsed[0].path.as_deref(), Some("src/lib.rs"));
        assert_eq!(parsed[0].original_line, Some(10));
        assert_eq!(parsed[0].user.login, "reviewer1");
    }

    #[test]
    fn mock_gitlab_note_json_parsing() {
        let json = r#"[{
            "id": 888,
            "body": "Rename variable",
            "system": false,
            "author": { "username": "dev1" },
            "created_at": "2026-03-23T14:00:00Z",
            "position": { "new_path": "src/utils.ts", "new_line": 25 }
        }, {
            "id": 889,
            "body": "System merge note",
            "system": true,
            "author": { "username": "gitlab" },
            "created_at": "2026-03-23T14:01:00Z",
            "position": null
        }]"#;
        let parsed: Vec<GlabNote> = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(!parsed[0].system);
        assert!(parsed[1].system);
        assert_eq!(
            parsed[0].position.as_ref().unwrap().new_path.as_deref(),
            Some("src/utils.ts")
        );
    }
}
