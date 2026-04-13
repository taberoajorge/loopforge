use crate::ask_engine::session::AskSessionsState;
use crate::ask_engine::types::{AskMessage, AskQuestionResult, StartAskArgs};
use crate::ask_engine::AskEngineError;
use crate::commands::validation::{optional_trimmed, required_trimmed};
use crate::db::DbState;
use tauri::{AppHandle, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use uuid::Uuid;

#[tauri::command]
pub async fn ask_question(
    app: AppHandle,
    sessions: State<'_, AskSessionsState>,
    db: State<'_, DbState>,
    args: StartAskArgs,
) -> Result<AskQuestionResult, AskEngineError> {
    let project_id =
        required_trimmed(args.project_id, "project_id").map_err(AskEngineError::Path)?;
    let question = required_trimmed(args.question, "question").map_err(AskEngineError::Path)?;
    let agent = required_trimmed(args.agent, "agent").map_err(AskEngineError::Path)?;
    let model = optional_trimmed(args.model);

    if sessions.has_active(&project_id) {
        let _ = sessions.remove_and_kill(&project_id);
    }

    let project_dir = {
        let conn = db.0.lock().map_err(|_| AskEngineError::LockPoisoned)?;
        let dir: String = conn
            .query_row(
                "SELECT working_directory FROM projects WHERE id = ?1",
                rusqlite::params![project_id],
                |row| row.get(0),
            )
            .map_err(|err| AskEngineError::Db(err.to_string()))?;
        std::path::PathBuf::from(dir)
    };

    let message_id = Uuid::new_v4().to_string();

    let user_message = {
        let conn = db.0.lock().map_err(|_| AskEngineError::LockPoisoned)?;
        let conversation =
            crate::ask_engine::storage::get_or_create_conversation(&conn, &project_id)
                .map_err(|err| AskEngineError::Db(err.to_string()))?;
        crate::ask_engine::storage::insert_message(
            &conn,
            &conversation.id,
            "user",
            &question,
            None,
            None,
        )
        .map_err(|err| AskEngineError::Db(err.to_string()))?
    };

    let normalized = StartAskArgs {
        project_id,
        question,
        agent,
        model,
    };

    crate::ask_engine::stream::spawn_ask(
        app,
        sessions.inner().clone(),
        normalized,
        message_id.clone(),
        project_dir,
    )
    .await?;

    Ok(AskQuestionResult {
        message_id,
        user_message,
    })
}

#[tauri::command]
pub async fn ask_history(
    db: State<'_, DbState>,
    project_id: String,
) -> Result<Vec<AskMessage>, AskEngineError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(AskEngineError::Path)?;
    let conn = db.0.lock().map_err(|_| AskEngineError::LockPoisoned)?;
    crate::ask_engine::storage::list_messages(&conn, &project_id)
        .map_err(|err| AskEngineError::Db(err.to_string()))
}

#[tauri::command]
pub async fn stop_ask(
    sessions: State<'_, AskSessionsState>,
    project_id: String,
) -> Result<(), AskEngineError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(AskEngineError::Path)?;
    sessions
        .remove_and_kill(&project_id)
        .map_err(AskEngineError::Shell)?;
    Ok(())
}

#[tauri::command]
pub async fn copy_ask_message(
    app: AppHandle,
    db: State<'_, DbState>,
    message_id: String,
) -> Result<String, AskEngineError> {
    let message_id = required_trimmed(message_id, "message_id").map_err(AskEngineError::Path)?;
    let conn = db.0.lock().map_err(|_| AskEngineError::LockPoisoned)?;
    let message = crate::ask_engine::storage::get_message_by_id(&conn, &message_id)
        .map_err(|err| AskEngineError::Db(err.to_string()))?
        .ok_or_else(|| AskEngineError::Db(format!("Message not found: {message_id}")))?;
    drop(conn);
    app.clipboard()
        .write_text(&message.content)
        .map_err(|err| AskEngineError::Shell(err.to_string()))?;
    Ok(message.content)
}

#[tauri::command]
pub async fn truncate_ask_from(
    db: State<'_, DbState>,
    project_id: String,
    message_id: String,
) -> Result<Vec<AskMessage>, AskEngineError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(AskEngineError::Path)?;
    let message_id = required_trimmed(message_id, "message_id").map_err(AskEngineError::Path)?;
    let conn = db.0.lock().map_err(|_| AskEngineError::LockPoisoned)?;
    crate::ask_engine::storage::truncate_from_message(&conn, &project_id, &message_id)
        .map_err(|err| AskEngineError::Db(err.to_string()))?;
    crate::ask_engine::storage::list_messages(&conn, &project_id)
        .map_err(|err| AskEngineError::Db(err.to_string()))
}

#[tauri::command]
pub async fn retry_ask(
    app: AppHandle,
    sessions: State<'_, AskSessionsState>,
    db: State<'_, DbState>,
    project_id: String,
    message_id: String,
    agent: String,
    model: Option<String>,
) -> Result<String, AskEngineError> {
    let project_id = required_trimmed(project_id, "project_id").map_err(AskEngineError::Path)?;
    let message_id = required_trimmed(message_id, "message_id").map_err(AskEngineError::Path)?;
    let agent = required_trimmed(agent, "agent").map_err(AskEngineError::Path)?;
    let model = optional_trimmed(model);

    let (question, project_dir) = {
        let conn = db.0.lock().map_err(|_| AskEngineError::LockPoisoned)?;
        let message = crate::ask_engine::storage::get_message_by_id(&conn, &message_id)
            .map_err(|err| AskEngineError::Db(err.to_string()))?
            .ok_or_else(|| AskEngineError::Db(format!("Message not found: {message_id}")))?;
        crate::ask_engine::storage::truncate_from_message(&conn, &project_id, &message_id)
            .map_err(|err| AskEngineError::Db(err.to_string()))?;
        let dir: String = conn
            .query_row(
                "SELECT working_directory FROM projects WHERE id = ?1",
                rusqlite::params![project_id],
                |row| row.get(0),
            )
            .map_err(|err| AskEngineError::Db(err.to_string()))?;
        (message.content, std::path::PathBuf::from(dir))
    };

    if sessions.has_active(&project_id) {
        let _ = sessions.remove_and_kill(&project_id);
    }

    let new_message_id = Uuid::new_v4().to_string();

    {
        let conn = db.0.lock().map_err(|_| AskEngineError::LockPoisoned)?;
        let conversation =
            crate::ask_engine::storage::get_or_create_conversation(&conn, &project_id)
                .map_err(|err| AskEngineError::Db(err.to_string()))?;
        crate::ask_engine::storage::insert_message(
            &conn,
            &conversation.id,
            "user",
            &question,
            None,
            None,
        )
        .map_err(|err| AskEngineError::Db(err.to_string()))?;
    }

    let args = StartAskArgs {
        project_id,
        question,
        agent,
        model,
    };

    crate::ask_engine::stream::spawn_ask(
        app,
        sessions.inner().clone(),
        args,
        new_message_id.clone(),
        project_dir,
    )
    .await?;

    Ok(new_message_id)
}
