use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::db::DbState;
use crate::loop_manager::{LoopManagerState, SessionStats};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EphemeralAnswer {
    pub question: String,
    pub answer: String,
    pub source: AnswerSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnswerSource {
    Instant,
    Agent,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionContext {
    stats: SessionStats,
    is_running: bool,
    current_agent: Option<String>,
}

const INSTANT_PATTERNS: &[(&str, InstantQuery)] = &[
    ("current story", InstantQuery::CurrentStory),
    ("what story", InstantQuery::CurrentStory),
    ("which story", InstantQuery::CurrentStory),
    ("status", InstantQuery::SessionMetrics),
    ("metrics", InstantQuery::SessionMetrics),
    ("progress", InstantQuery::SessionMetrics),
    ("stats", InstantQuery::SessionMetrics),
    ("config", InstantQuery::LoopConfig),
    ("configuration", InstantQuery::LoopConfig),
    ("settings", InstantQuery::LoopConfig),
    ("agent", InstantQuery::CurrentAgent),
    ("which agent", InstantQuery::CurrentAgent),
];

#[derive(Debug, Clone, Copy)]
enum InstantQuery {
    CurrentStory,
    SessionMetrics,
    LoopConfig,
    CurrentAgent,
}

fn classify_question(question: &str) -> Option<InstantQuery> {
    let lower = question.to_lowercase();
    INSTANT_PATTERNS
        .iter()
        .find(|(pattern, _)| lower.contains(pattern))
        .map(|(_, query_type)| *query_type)
}

fn answer_instant(
    query_type: InstantQuery,
    stats: &SessionStats,
) -> String {
    match query_type {
        InstantQuery::CurrentStory => {
            if stats.is_running {
                format!(
                    "Iteration {} in progress. {} of {} stories completed, {} blocked, {} pending.",
                    stats.total_iterations,
                    stats.passed_stories,
                    stats.total_stories,
                    stats.blocked_stories,
                    stats.pending_stories
                )
            } else {
                "No loop is currently running.".to_string()
            }
        }
        InstantQuery::SessionMetrics => {
            format!(
                "Total iterations: {} | Success: {} | Failures: {} | Rate limited: {} | Success rate: {:.0}% | Stories/hr: {:.1}",
                stats.total_iterations,
                stats.success_count,
                stats.failure_count,
                stats.rate_limited_count,
                stats.success_rate * 100.0,
                stats.stories_per_hour
            )
        }
        InstantQuery::LoopConfig => {
            let agent = stats
                .current_agent
                .as_deref()
                .unwrap_or("none");
            format!(
                "Agent: {} | Running: {} | Stories: {}/{} completed",
                agent,
                stats.is_running,
                stats.passed_stories,
                stats.total_stories
            )
        }
        InstantQuery::CurrentAgent => {
            match &stats.current_agent {
                Some(agent) => format!("Current agent: {agent}"),
                None => "No agent is currently active.".to_string(),
            }
        }
    }
}

#[tauri::command]
pub async fn ephemeral_query(
    app: AppHandle,
    project_id: String,
    question: String,
) -> Result<EphemeralAnswer, String> {
    use tauri::Manager;
    let db_state = app.state::<DbState>();
    let loop_state = app.state::<LoopManagerState>();
    let stats = crate::loop_manager::session_stats(
        app.clone(),
        db_state,
        loop_state,
        project_id,
    )
    .await
    .map_err(|err| format!("Failed to get session context: {err}"))?;

    if let Some(query_type) = classify_question(&question) {
        let answer = answer_instant(query_type, &stats);
        return Ok(EphemeralAnswer {
            question,
            answer,
            source: AnswerSource::Instant,
        });
    }

    let context = serde_json::to_string_pretty(&stats)
        .unwrap_or_else(|_| "Unable to serialize context".to_string());

    Ok(EphemeralAnswer {
        question: question.clone(),
        answer: format!(
            "Complex query detected. Session context:\n{context}\n\nQuestion: {question}\n\n(Agent-based answering will be available in a future update.)"
        ),
        source: AnswerSource::Agent,
    })
}
