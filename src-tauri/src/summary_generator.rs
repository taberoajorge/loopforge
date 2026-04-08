use crate::db::DbState;
use ralph_core::prd::Prd;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryOutcome {
    pub story_id: String,
    pub title: String,
    pub status: String,
    pub duration_secs: i64,
    pub attempts: i64,
    pub agent_used: String,
    pub verification_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffStat {
    pub file: String,
    pub insertions: i64,
    pub deletions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUsage {
    pub agent: String,
    pub stories_handled: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoopSummary {
    pub project_id: String,
    pub project_name: String,
    pub session_id: String,
    pub completion_status: String,
    pub total_stories: usize,
    pub passed: usize,
    pub blocked: usize,
    pub skipped: usize,
    pub total_execution_secs: i64,
    pub agents: Vec<AgentUsage>,
    pub stories: Vec<StoryOutcome>,
    pub diff_stats: Vec<DiffStat>,
    pub total_insertions: i64,
    pub total_deletions: i64,
    pub total_files_changed: usize,
    pub complexity: String,
    pub narrative: String,
    pub generated_at: String,
}

#[derive(Debug)]
pub struct SummaryError(String);

impl Serialize for SummaryError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

fn collect_story_outcomes(
    conn: &rusqlite::Connection,
    session_id: &str,
    prd: &Prd,
) -> Vec<StoryOutcome> {
    prd.stories
        .iter()
        .map(|story| {
            let (total_duration, attempts, last_agent, last_result): (i64, i64, String, String) =
                conn.query_row(
                    "SELECT COALESCE(SUM(duration_secs), 0), COUNT(*),
                            COALESCE((SELECT agent_used FROM iterations WHERE session_id = ?1 AND story_id = ?2 ORDER BY started_at DESC LIMIT 1), ''),
                            COALESCE((SELECT result FROM iterations WHERE session_id = ?1 AND story_id = ?2 ORDER BY started_at DESC LIMIT 1), 'pending')
                     FROM iterations WHERE session_id = ?1 AND story_id = ?2",
                    rusqlite::params![session_id, story.id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .unwrap_or((0, 0, String::new(), "pending".into()));

            StoryOutcome {
                story_id: story.id.clone(),
                title: story.title.clone(),
                status: last_result,
                duration_secs: total_duration,
                attempts,
                agent_used: last_agent,
                verification_output: None,
            }
        })
        .collect()
}

fn collect_agent_usage(conn: &rusqlite::Connection, session_id: &str) -> Vec<AgentUsage> {
    let mut stmt = conn
        .prepare(
            "SELECT agent_used, COUNT(DISTINCT story_id) FROM iterations
             WHERE session_id = ?1 AND result = 'success'
             GROUP BY agent_used",
        )
        .unwrap_or_else(|_| {
            conn.prepare("SELECT '', 0 WHERE 0").unwrap()
        });

    stmt.query_map(rusqlite::params![session_id], |row| {
        Ok(AgentUsage {
            agent: row.get(0)?,
            stories_handled: row.get(1)?,
        })
    })
    .map(|rows| rows.filter_map(|row| row.ok()).collect())
    .unwrap_or_default()
}

fn collect_diff_stats(work_dir: &Path) -> Vec<DiffStat> {
    let output = std::process::Command::new("git")
        .args(["diff", "--stat", "--numstat", "HEAD~1..HEAD"])
        .current_dir(work_dir)
        .output()
        .ok();

    let Some(out) = output else {
        return vec![];
    };

    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let insertions = parts[0].parse::<i64>().unwrap_or(0);
                let deletions = parts[1].parse::<i64>().unwrap_or(0);
                Some(DiffStat {
                    file: parts[2].to_string(),
                    insertions,
                    deletions,
                })
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn classify_complexity_for_test(files_changed: usize, total_lines: i64, story_count: usize) -> &'static str {
    classify_complexity(files_changed, total_lines, story_count)
}

fn classify_complexity(files_changed: usize, total_lines: i64, story_count: usize) -> &'static str {
    if files_changed < 10 && total_lines < 100 && story_count <= 2 {
        "trivial"
    } else if files_changed < 20 && total_lines < 500 && story_count <= 5 {
        "small"
    } else if files_changed < 50 && total_lines < 2000 && story_count <= 15 {
        "medium"
    } else {
        "large"
    }
}

fn total_execution_time(conn: &rusqlite::Connection, session_id: &str) -> i64 {
    conn.query_row(
        "SELECT COALESCE(SUM(duration_secs), 0) FROM iterations WHERE session_id = ?1",
        rusqlite::params![session_id],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

pub fn generate_summary(
    conn: &rusqlite::Connection,
    project_id: &str,
    project_name: &str,
    session_id: &str,
    prd: &Prd,
    work_dir: &Path,
) -> LoopSummary {
    let stories = collect_story_outcomes(conn, session_id, prd);
    let agents = collect_agent_usage(conn, session_id);
    let diff_stats = collect_diff_stats(work_dir);
    let exec_secs = total_execution_time(conn, session_id);

    let passed = stories.iter().filter(|story| story.status == "success").count();
    let blocked = stories.iter().filter(|story| story.status == "failed" || story.status == "blocked").count();
    let skipped = stories.iter().filter(|story| story.status == "pending").count();

    let total_insertions: i64 = diff_stats.iter().map(|stat| stat.insertions).sum();
    let total_deletions: i64 = diff_stats.iter().map(|stat| stat.deletions).sum();
    let files_changed = diff_stats.len();

    let total_stories = stories.len();
    let complexity = classify_complexity(files_changed, total_insertions + total_deletions, total_stories).to_string();

    let completion_status = if blocked == 0 && skipped == 0 {
        "SUCCESS".to_string()
    } else if passed > 0 {
        "PARTIAL".to_string()
    } else {
        "FAILED".to_string()
    };

    LoopSummary {
        project_id: project_id.to_string(),
        project_name: project_name.to_string(),
        session_id: session_id.to_string(),
        completion_status,
        total_stories,
        passed,
        blocked,
        skipped,
        total_execution_secs: exec_secs,
        agents,
        stories,
        diff_stats,
        total_insertions,
        total_deletions,
        total_files_changed: files_changed,
        complexity,
        narrative: String::new(),
        generated_at: chrono::Utc::now().to_rfc3339(),
    }
}

fn render_narrative(summary: &LoopSummary) -> String {
    let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let template_path = templates_dir.join("summary_narrative.j2");

    let template_content = std::fs::read_to_string(&template_path).unwrap_or_default();
    if template_content.is_empty() {
        return fallback_narrative(summary);
    }

    let mut env = minijinja::Environment::new();
    env.add_template("summary", &template_content).ok();

    let ctx = minijinja::context! {
        project_name => summary.project_name,
        completion_status => summary.completion_status,
        passed => summary.passed,
        blocked => summary.blocked,
        skipped => summary.skipped,
        total_stories => summary.total_stories,
        total_execution_secs => summary.total_execution_secs,
        agents => summary.agents,
        total_files_changed => summary.total_files_changed,
        total_insertions => summary.total_insertions,
        total_deletions => summary.total_deletions,
        complexity => summary.complexity,
    };

    env.get_template("summary")
        .and_then(|tmpl| tmpl.render(ctx))
        .unwrap_or_else(|_| fallback_narrative(summary))
}

fn fallback_narrative(summary: &LoopSummary) -> String {
    let agent_list: Vec<String> = summary
        .agents
        .iter()
        .map(|agent| format!("{} ({} stories)", agent.agent, agent.stories_handled))
        .collect();

    format!(
        "Loop session for **{}** completed with status **{}**. \
         {}/{} stories passed, {} blocked, {} skipped. \
         Total execution time: {}s across {} changed files (+{}/−{}). \
         Complexity: **{}**. Agents: {}.",
        summary.project_name,
        summary.completion_status,
        summary.passed,
        summary.total_stories,
        summary.blocked,
        summary.skipped,
        summary.total_execution_secs,
        summary.total_files_changed,
        summary.total_insertions,
        summary.total_deletions,
        summary.complexity,
        if agent_list.is_empty() {
            "none".to_string()
        } else {
            agent_list.join(", ")
        },
    )
}

fn artifact_dir(app: &AppHandle, project_id: &str) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| err.to_string())?;
    Ok(data_dir.join("projects").join(project_id))
}

#[tauri::command]
pub async fn generate_loop_summary(
    app: AppHandle,
    db: State<'_, DbState>,
    project_id: String,
    project_name: String,
    session_id: String,
    working_directory: String,
) -> Result<LoopSummary, String> {
    let conn = db.0.lock().map_err(|_| "Lock poisoned".to_string())?;

    let artifact_path = artifact_dir(&app, &project_id)?;
    let prd_path = artifact_path.join("prd.json");
    let prd = Prd::load(&prd_path).map_err(|err| err.to_string())?;
    let work_dir = PathBuf::from(&working_directory);

    let mut summary = generate_summary(&conn, &project_id, &project_name, &session_id, &prd, &work_dir);
    summary.narrative = render_narrative(&summary);

    let summary_path = artifact_path.join("summary.json");
    let json = serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?;
    std::fs::write(&summary_path, &json).map_err(|err| err.to_string())?;

    Ok(summary)
}

#[tauri::command]
pub async fn load_summary(
    app: AppHandle,
    project_id: String,
) -> Result<Option<LoopSummary>, String> {
    let artifact_path = artifact_dir(&app, &project_id)?;
    let summary_path = artifact_path.join("summary.json");

    if !summary_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&summary_path).map_err(|err| err.to_string())?;
    let summary: LoopSummary = serde_json::from_str(&content).map_err(|err| err.to_string())?;
    Ok(Some(summary))
}

pub fn export_github_markdown(summary: &LoopSummary) -> String {
    let mut md = String::new();

    md.push_str(&format!("## {}\n\n", summary.project_name));
    md.push_str(&format!("{}\n\n", summary.narrative));

    md.push_str("| Story | Title | Status | Duration | Agent |\n");
    md.push_str("|-------|-------|--------|----------|-------|\n");
    for story in &summary.stories {
        md.push_str(&format!(
            "| {} | {} | {} | {}s | {} |\n",
            story.story_id, story.title, story.status, story.duration_secs, story.agent_used
        ));
    }

    md.push_str("\n<details>\n<summary>Changed Files</summary>\n\n");
    md.push_str("| File | Insertions | Deletions |\n");
    md.push_str("|------|------------|----------|\n");
    for stat in &summary.diff_stats {
        md.push_str(&format!(
            "| {} | +{} | −{} |\n",
            stat.file, stat.insertions, stat.deletions
        ));
    }
    md.push_str(&format!(
        "\n**Total**: {} files changed, +{}/−{}\n",
        summary.total_files_changed, summary.total_insertions, summary.total_deletions
    ));
    md.push_str("\n</details>\n\n");

    let agent_list: Vec<String> = summary
        .agents
        .iter()
        .map(|agent| format!("{} ({} stories)", agent.agent, agent.stories_handled))
        .collect();
    md.push_str(&format!(
        "**Agents**: {} | **Complexity**: {} | **Status**: {}\n",
        agent_list.join(", "),
        summary.complexity,
        summary.completion_status
    ));

    md
}

pub fn export_gitlab_markdown(summary: &LoopSummary) -> String {
    export_github_markdown(summary)
}

#[tauri::command]
pub async fn export_summary_markdown(
    app: AppHandle,
    project_id: String,
    format: String,
) -> Result<String, String> {
    let artifact_path = artifact_dir(&app, &project_id)?;
    let summary_path = artifact_path.join("summary.json");

    let content = std::fs::read_to_string(&summary_path).map_err(|err| err.to_string())?;
    let summary: LoopSummary = serde_json::from_str(&content).map_err(|err| err.to_string())?;

    let md = match format.as_str() {
        "gitlab" => export_gitlab_markdown(&summary),
        _ => export_github_markdown(&summary),
    };

    let md_path = artifact_path.join("summary.md");
    std::fs::write(&md_path, &md).map_err(|err| err.to_string())?;

    Ok(md)
}
