use crate::projects::{NotificationPrefs, ProjectError};
use tauri::{AppHandle, Manager};

pub async fn get_notification_prefs(
    app: AppHandle,
    project_id: String,
) -> Result<NotificationPrefs, ProjectError> {
    let db = app.state::<crate::db::DbState>();
    let conn = db
        .0
        .lock()
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

pub async fn save_notification_prefs(
    app: AppHandle,
    project_id: String,
    prefs: NotificationPrefs,
) -> Result<(), ProjectError> {
    let db = app.state::<crate::db::DbState>();
    let conn = db
        .0
        .lock()
        .map_err(|_| ProjectError::Db("Lock poisoned".into()))?;
    let json = serde_json::to_string(&prefs)?;
    conn.execute(
        "UPDATE projects SET notification_prefs = ?1 WHERE id = ?2",
        rusqlite::params![json, project_id],
    )
    .map_err(|err| ProjectError::Db(err.to_string()))?;
    Ok(())
}
