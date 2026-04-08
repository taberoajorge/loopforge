use crate::ask_engine::types::{AskConversation, AskMessage};
use rusqlite::{Connection, OptionalExtension};
use uuid::Uuid;

pub fn get_or_create_conversation(
    conn: &Connection,
    project_id: &str,
) -> Result<AskConversation, rusqlite::Error> {
    let existing: Option<AskConversation> = conn
        .query_row(
            "SELECT id, project_id, created_at FROM ask_conversations WHERE project_id = ?1",
            rusqlite::params![project_id],
            |row| {
                Ok(AskConversation {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    created_at: row.get(2)?,
                })
            },
        )
        .ok();

    if let Some(conversation) = existing {
        return Ok(conversation);
    }

    let conversation_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO ask_conversations (id, project_id) VALUES (?1, ?2)",
        rusqlite::params![conversation_id, project_id],
    )?;

    conn.query_row(
        "SELECT id, project_id, created_at FROM ask_conversations WHERE id = ?1",
        rusqlite::params![conversation_id],
        |row| {
            Ok(AskConversation {
                id: row.get(0)?,
                project_id: row.get(1)?,
                created_at: row.get(2)?,
            })
        },
    )
}

pub fn insert_message(
    conn: &Connection,
    conversation_id: &str,
    role: &str,
    content: &str,
    agent: Option<&str>,
    model: Option<&str>,
) -> Result<AskMessage, rusqlite::Error> {
    let message_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO ask_messages (id, conversation_id, role, content, agent, model)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![message_id, conversation_id, role, content, agent, model],
    )?;

    conn.query_row(
        "SELECT id, conversation_id, role, content, agent, model, created_at
         FROM ask_messages WHERE id = ?1",
        rusqlite::params![message_id],
        |row| {
            Ok(AskMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                agent: row.get(4)?,
                model: row.get(5)?,
                created_at: row.get(6)?,
            })
        },
    )
}

pub fn list_messages(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<AskMessage>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.conversation_id, m.role, m.content, m.agent, m.model, m.created_at
         FROM ask_messages m
         JOIN ask_conversations c ON c.id = m.conversation_id
         WHERE c.project_id = ?1
         ORDER BY m.created_at ASC",
    )?;

    let rows = stmt.query_map(rusqlite::params![project_id], |row| {
        Ok(AskMessage {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            agent: row.get(4)?,
            model: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    rows.collect()
}

pub fn get_message_by_id(
    conn: &Connection,
    message_id: &str,
) -> Result<Option<AskMessage>, rusqlite::Error> {
    conn.query_row(
        "SELECT id, conversation_id, role, content, agent, model, created_at
         FROM ask_messages WHERE id = ?1",
        rusqlite::params![message_id],
        |row| {
            Ok(AskMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                agent: row.get(4)?,
                model: row.get(5)?,
                created_at: row.get(6)?,
            })
        },
    )
    .optional()
}

pub fn truncate_from_message(
    conn: &Connection,
    project_id: &str,
    message_id: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM ask_messages
         WHERE conversation_id IN (SELECT id FROM ask_conversations WHERE project_id = ?1)
           AND created_at >= (SELECT created_at FROM ask_messages WHERE id = ?2)",
        rusqlite::params![project_id, message_id],
    )?;
    Ok(())
}
