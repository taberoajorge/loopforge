use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginEntry {
    pub name: String,
    pub slot: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub is_builtin: bool,
    pub config_json: String,
    pub health: bool,
}

#[derive(Debug)]
pub struct PluginError(String);

impl From<rusqlite::Error> for PluginError {
    fn from(err: rusqlite::Error) -> Self {
        Self(err.to_string())
    }
}

impl Serialize for PluginError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

fn builtin_plugins() -> Vec<PluginEntry> {
    vec![
        PluginEntry {
            name: "builtin-runtime".into(),
            slot: "runtime".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Default runtime lifecycle hooks".into(),
            enabled: true,
            is_builtin: true,
            config_json: "{}".into(),
            health: true,
        },
        PluginEntry {
            name: "builtin-notifier".into(),
            slot: "notifier".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Default stdout/OS notifier".into(),
            enabled: true,
            is_builtin: true,
            config_json: "{}".into(),
            health: true,
        },
        PluginEntry {
            name: "builtin-github-scm".into(),
            slot: "scm".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "GitHub SCM via gh CLI".into(),
            enabled: true,
            is_builtin: true,
            config_json: "{}".into(),
            health: true,
        },
        PluginEntry {
            name: "builtin-gitlab-scm".into(),
            slot: "scm".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "GitLab SCM via glab CLI".into(),
            enabled: true,
            is_builtin: true,
            config_json: "{}".into(),
            health: true,
        },
    ]
}

#[tauri::command]
pub async fn list_plugins(
    db: State<'_, DbState>,
) -> Result<Vec<PluginEntry>, String> {
    let mut entries = builtin_plugins();

    let conn = db.0.lock().map_err(|_| "Lock poisoned".to_string())?;
    let mut stmt = conn
        .prepare("SELECT plugin_name, slot, enabled, config_json FROM plugin_configs")
        .map_err(|err| err.to_string())?;

    let db_configs: Vec<(String, String, bool, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, bool>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|err| err.to_string())?
        .filter_map(|row| row.ok())
        .collect();

    for (plugin_name, _slot, enabled, config_json) in &db_configs {
        if let Some(entry) = entries.iter_mut().find(|entry| &entry.name == plugin_name) {
            entry.enabled = *enabled;
            entry.config_json = config_json.clone();
        }
    }

    Ok(entries)
}

#[tauri::command]
pub async fn save_plugin_config(
    db: State<'_, DbState>,
    plugin_name: String,
    slot: String,
    enabled: bool,
    config_json: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|_| "Lock poisoned".to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO plugin_configs (plugin_name, slot, enabled, config_json) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![plugin_name, slot, enabled, config_json],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}
