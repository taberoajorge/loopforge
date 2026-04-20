use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppNotification {
    pub id: String,
    pub project_id: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub ring_color: String,
    pub read: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectNotificationSummary {
    pub project_id: String,
    pub unread_count: usize,
    pub ring_color: Option<String>,
}

pub fn type_to_ring_color(notification_type: &str) -> &'static str {
    match notification_type {
        "story_blocked" | "loop_error" => "red",
        "loop_completed" | "story_completed" => "cyan",
        "rate_limited" | "review_comment" => "amber",
        _ => "cyan",
    }
}

pub fn ring_color_for_project(notifications: &[AppNotification]) -> Option<&'static str> {
    let unread: Vec<&AppNotification> = notifications.iter().filter(|notif| !notif.read).collect();
    if unread.is_empty() {
        return None;
    }
    let priority = ["red", "amber", "cyan", "green"];
    for color in &priority {
        if unread.iter().any(|notif| notif.ring_color == *color) {
            return Some(color);
        }
    }
    Some("cyan")
}

pub fn build_project_summaries(
    notifications: &[AppNotification],
) -> Vec<ProjectNotificationSummary> {
    let mut grouped: std::collections::BTreeMap<String, Vec<AppNotification>> =
        std::collections::BTreeMap::new();
    for notification in notifications {
        grouped
            .entry(notification.project_id.clone())
            .or_default()
            .push(notification.clone());
    }
    grouped
        .into_iter()
        .map(|(project_id, project_notifications)| {
            let unread_count = project_notifications
                .iter()
                .filter(|entry| !entry.read)
                .count();
            let ring_color = ring_color_for_project(&project_notifications).map(String::from);
            ProjectNotificationSummary {
                project_id,
                unread_count,
                ring_color,
            }
        })
        .collect()
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn build_notification(
    project_id: &str,
    notification_type: &str,
    title: String,
    message: String,
) -> AppNotification {
    let ring_color = type_to_ring_color(notification_type).to_string();
    AppNotification {
        id: format!(
            "notif_{}_{}",
            now_millis(),
            &uuid::Uuid::new_v4().to_string()[..8]
        ),
        project_id: project_id.to_string(),
        notification_type: notification_type.to_string(),
        title,
        message,
        ring_color,
        read: false,
        timestamp: now_millis(),
    }
}

pub fn should_emit_verification_failed(attempt: u32, circuit_breaker: bool) -> bool {
    circuit_breaker || attempt >= 3
}

pub fn should_emit_iteration_completed(result: &str) -> bool {
    result == "success" || result == "passed"
}
