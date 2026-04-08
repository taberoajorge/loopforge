use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

#[tauri::command]
pub fn send_os_notification(app: AppHandle, title: String, body: String) {
    let _ = app
        .notification()
        .builder()
        .title(&title)
        .body(&body)
        .show();
}

pub fn notify_story_completed(app: &AppHandle, story_id: &str, project_name: &str) {
    let _ = app
        .notification()
        .builder()
        .title("Story Completed")
        .body(format!("{}: {} passed", project_name, story_id))
        .show();
}

pub fn notify_loop_completed(app: &AppHandle, project_name: &str) {
    let _ = app
        .notification()
        .builder()
        .title("Loop Completed")
        .body(format!("{} finished all stories", project_name))
        .show();
}

pub fn notify_all_rate_limited(app: &AppHandle, project_name: &str) {
    let _ = app
        .notification()
        .builder()
        .title("All Agents Rate Limited")
        .body(format!("{}: all agents are rate limited", project_name))
        .show();
}

pub fn notify_loop_error(app: &AppHandle, project_name: &str) {
    let _ = app
        .notification()
        .builder()
        .title("Loop Error")
        .body(format!("{}: loop encountered an error", project_name))
        .show();
}

pub fn notify_review_escalation(
    app: &AppHandle,
    project_name: &str,
    file_path: &str,
    reviewer: &str,
    timeout_minutes: u64,
) {
    let _ = app
        .notification()
        .builder()
        .title("Review Comment Unresolved")
        .body(format!(
            "{}: comment by {} on {} unresolved after {}m. Manual intervention may be needed.",
            project_name, reviewer, file_path, timeout_minutes
        ))
        .show();
}
