use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppNotification {
    pub id: String,
    pub project_id: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub timestamp: u64,
}

pub fn should_emit_verification_failed(attempt: u32, circuit_breaker: bool) -> bool {
    circuit_breaker || attempt >= 3
}

pub fn should_emit_iteration_completed(result: &str) -> bool {
    result == "success" || result == "passed"
}

pub fn build_notification(
    project_id: &str,
    notification_type: &str,
    title: String,
    message: String,
) -> AppNotification {
    AppNotification {
        id: format!(
            "notif_{}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            &uuid::Uuid::new_v4().to_string()[..8]
        ),
        project_id: project_id.to_string(),
        notification_type: notification_type.to_string(),
        title,
        message,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    }
}
