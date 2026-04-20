use crate::projects::notification_filter::{build_notification, AppNotification};
use crate::projects::{NotificationPrefs, ProjectError};
use tauri::{AppHandle, Emitter, Manager, Runtime};

#[derive(Debug, Clone)]
pub struct NotificationCreateInput {
    pub project_id: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
}

pub fn create_notification_and_emit<R: Runtime>(
    app: &AppHandle<R>,
    input: NotificationCreateInput,
) -> Result<AppNotification, ProjectError> {
    let notification = {
        let db = app.state::<crate::db::DbState>();
        let conn =
            db.0.lock()
                .map_err(|_| ProjectError::Db("Lock poisoned".into()))?;
        let created = build_notification(
            &input.project_id,
            &input.notification_type,
            input.title,
            input.message,
        );
        conn.execute(
            "INSERT INTO notifications (id, project_id, notification_type, title, message, ring_color, read, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                created.id,
                created.project_id,
                created.notification_type,
                created.title,
                created.message,
                created.ring_color,
                i32::from(created.read),
                created.timestamp,
            ],
        )?;
        conn.execute(
            "DELETE FROM notifications WHERE project_id = ?1 AND id NOT IN (SELECT id FROM notifications WHERE project_id = ?1 ORDER BY timestamp DESC LIMIT 200)",
            rusqlite::params![created.project_id],
        )?;
        created
    };
    let _ = app.emit(
        crate::events::EVENT_NOTIFICATION_ADDED,
        serde_json::json!({
            "projectId": notification.project_id,
            "notification": notification.clone(),
        }),
    );
    Ok(notification)
}

pub async fn get_notification_prefs<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
) -> Result<NotificationPrefs, ProjectError> {
    let db = app.state::<crate::db::DbState>();
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".into()))?;
    let prefs_json: Option<String> = conn
        .query_row(
            "SELECT notification_prefs FROM projects WHERE id = ?1",
            rusqlite::params![project_id],
            |row| row.get(0),
        )
        .map_err(|err| ProjectError::Db(err.to_string()))?;

    match prefs_json {
        Some(json) => Ok(serde_json::from_str(&json).unwrap_or_default()),
        None => Ok(NotificationPrefs::default()),
    }
}

pub async fn save_notification_prefs<R: Runtime>(
    app: AppHandle<R>,
    project_id: String,
    prefs: NotificationPrefs,
) -> Result<(), ProjectError> {
    let db = app.state::<crate::db::DbState>();
    let conn =
        db.0.lock()
            .map_err(|_| ProjectError::Db("Lock poisoned".into()))?;
    let json = serde_json::to_string(&prefs)?;
    conn.execute(
        "UPDATE projects SET notification_prefs = ?1 WHERE id = ?2",
        rusqlite::params![json, project_id],
    )
    .map_err(|err| ProjectError::Db(err.to_string()))?;
    Ok(())
}
