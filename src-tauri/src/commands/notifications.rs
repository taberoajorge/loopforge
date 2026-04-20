use crate::projects::notification_filter::{
    build_project_summaries, ring_color_for_project, AppNotification, ProjectNotificationSummary,
};
use crate::projects::notifications::{create_notification_and_emit, NotificationCreateInput};
use crate::projects::ProjectError;
use crate::storage::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationListResponse {
    pub notifications: Vec<AppNotification>,
    pub unread_count: usize,
    pub ring_color: Option<String>,
    pub project_summaries: Vec<ProjectNotificationSummary>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddNotificationArgs {
    pub project_id: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
}

#[tauri::command]
pub async fn add_notification(
    app: AppHandle,
    args: AddNotificationArgs,
) -> Result<AppNotification, ProjectError> {
    create_notification_and_emit(
        &app,
        NotificationCreateInput {
            project_id: args.project_id,
            notification_type: args.notification_type,
            title: args.title,
            message: args.message,
        },
    )
}

#[tauri::command]
pub async fn get_notifications(
    db: State<'_, DbState>,
    project_id: Option<String>,
) -> Result<NotificationListResponse, ProjectError> {
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".into()))?;
    let notifications = query_notifications(&conn, project_id.as_deref())?;
    let unread_count = notifications.iter().filter(|notif| !notif.read).count();
    let ring = ring_color_for_project(&notifications).map(String::from);
    let project_summaries = build_project_summaries(&notifications);
    Ok(NotificationListResponse {
        notifications,
        unread_count,
        ring_color: ring,
        project_summaries,
    })
}

#[tauri::command]
pub async fn mark_notification_read(
    db: State<'_, DbState>,
    notification_id: String,
) -> Result<(), ProjectError> {
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".into()))?;
    conn.execute(
        "UPDATE notifications SET read = 1 WHERE id = ?1",
        rusqlite::params![notification_id],
    )?;
    Ok(())
}

#[tauri::command]
pub async fn mark_all_notifications_read(
    db: State<'_, DbState>,
    project_id: Option<String>,
) -> Result<(), ProjectError> {
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".into()))?;
    if let Some(ref pid) = project_id {
        conn.execute(
            "UPDATE notifications SET read = 1 WHERE project_id = ?1",
            rusqlite::params![pid],
        )?;
    } else {
        conn.execute("UPDATE notifications SET read = 1", [])?;
    }
    Ok(())
}

#[tauri::command]
pub async fn clear_notifications(
    db: State<'_, DbState>,
    project_id: Option<String>,
) -> Result<(), ProjectError> {
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".into()))?;
    if let Some(ref pid) = project_id {
        conn.execute(
            "DELETE FROM notifications WHERE project_id = ?1",
            rusqlite::params![pid],
        )?;
    } else {
        conn.execute("DELETE FROM notifications", [])?;
    }
    Ok(())
}

fn query_notifications(
    conn: &rusqlite::Connection,
    project_id: Option<&str>,
) -> Result<Vec<AppNotification>, ProjectError> {
    let sql = if project_id.is_some() {
        "SELECT id, project_id, notification_type, title, message, ring_color, read, timestamp FROM notifications WHERE project_id = ?1 ORDER BY timestamp DESC LIMIT 200"
    } else {
        "SELECT id, project_id, notification_type, title, message, ring_color, read, timestamp FROM notifications ORDER BY timestamp DESC LIMIT 200"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = match project_id {
        Some(pid) => stmt.query_map(rusqlite::params![pid], row_to_notification)?,
        None => stmt.query_map([], row_to_notification)?,
    };
    Ok(rows.filter_map(Result::ok).collect())
}

fn row_to_notification(row: &rusqlite::Row) -> rusqlite::Result<AppNotification> {
    Ok(AppNotification {
        id: row.get(0)?,
        project_id: row.get(1)?,
        notification_type: row.get(2)?,
        title: row.get(3)?,
        message: row.get(4)?,
        ring_color: row.get(5)?,
        read: row.get::<_, i32>(6)? != 0,
        timestamp: row.get(7)?,
    })
}
