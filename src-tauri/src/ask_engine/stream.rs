use crate::ask_engine::args::{agent_env_vars, build_ask_args, is_safe_binary_name};
use crate::ask_engine::context::build_ask_context;
use crate::ask_engine::session::AskSessionsState;
use crate::ask_engine::types::{AskCompletePayload, AskErrorPayload, AskStreamPayload, StartAskArgs};
use crate::ask_engine::AskEngineError;
use crate::db::DbState;
use crate::events::{EVENT_ASK_COMPLETE, EVENT_ASK_ERROR, EVENT_ASK_STREAM};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

pub async fn spawn_ask(
    app: AppHandle,
    sessions: AskSessionsState,
    args: StartAskArgs,
    message_id: String,
    project_dir: std::path::PathBuf,
) -> Result<(), AskEngineError> {
    if let Ok(crate::test_support::TestMode::Enabled(runtime)) =
        crate::test_support::resolve_test_mode()
    {
        let project_id = args.project_id.clone();
        let question = args.question.clone();
        let mid = message_id.clone();
        let app_clone = app.clone();
        tokio::spawn(async move {
            crate::ask_engine::fixture::spawn_fixture_ask(
                app_clone, project_id, mid, &question, &runtime,
            )
            .await;
        });
        return Ok(());
    }

    let agent_binary = resolve_agent_binary(&app, &args.agent).await?;

    let artifact_dir = crate::storage::artifacts::project_artifact_dir(&app, &args.project_id)
        .map_err(AskEngineError::Path)?;

    let context = {
        let db = app.state::<DbState>();
        let conn = db.0.lock().map_err(|_| AskEngineError::LockPoisoned)?;
        let history = crate::ask_engine::storage::list_messages(&conn, &args.project_id)
            .map_err(|err| AskEngineError::Db(err.to_string()))?;
        build_ask_context(&artifact_dir, &conn, &args.project_id, &history)
    };

    let full_prompt = format!("{context}\n\n## User Question\n{}", args.question);
    let agent_args = build_ask_args(&args.agent, &full_prompt, &project_dir, args.model.as_deref());
    let env_vars = agent_env_vars(&args.agent);

    let (mut event_rx, child) = if args.agent == "codex" {
        let shell_cmd = build_null_stdin_command(&agent_binary, &agent_args);
        app.shell()
            .command("/bin/zsh")
            .args(["-lc", &shell_cmd])
            .envs(env_vars)
            .current_dir(&project_dir)
            .spawn()
            .map_err(|err| AskEngineError::Shell(err.to_string()))?
    } else {
        app.shell()
            .command(&agent_binary)
            .args(agent_args)
            .envs(env_vars)
            .current_dir(&project_dir)
            .spawn()
            .map_err(|err| AskEngineError::Shell(err.to_string()))?
    };

    sessions
        .insert(&args.project_id, child)
        .map_err(|err| AskEngineError::Shell(err))?;

    let project_id = args.project_id.clone();
    let agent_name = args.agent.clone();
    let agent_model = args.model.clone();
    let sessions_arc = sessions.clone();
    let app_clone = app.clone();

    tokio::spawn(async move {
        let mut collected = String::new();

        while let Some(event) = event_rx.recv().await {
            match event {
                CommandEvent::Stdout(bytes) | CommandEvent::Stderr(bytes) => {
                    let chunk = String::from_utf8_lossy(&bytes).to_string();
                    if chunk.trim().is_empty() || is_agent_noise(&chunk) {
                        continue;
                    }
                    collected.push_str(&chunk);
                    let _ = app_clone.emit(
                        EVENT_ASK_STREAM,
                        AskStreamPayload {
                            project_id: project_id.clone(),
                            message_id: message_id.clone(),
                            chunk,
                        },
                    );
                }
                CommandEvent::Terminated(status) => {
                    let success = status.code.map(|code| code == 0).unwrap_or(false);
                    if success && !collected.trim().is_empty() {
                        save_assistant_message(&app_clone, &project_id, &collected, &agent_name, agent_model.as_deref());
                        let _ = app_clone.emit(
                            EVENT_ASK_COMPLETE,
                            AskCompletePayload {
                                project_id: project_id.clone(),
                                message_id: message_id.clone(),
                                full_content: collected.clone(),
                                agent: agent_name.clone(),
                                model: agent_model.clone(),
                            },
                        );
                    } else {
                        let error_msg = if collected.trim().is_empty() {
                            "Agent produced no output".to_string()
                        } else {
                            collected.clone()
                        };
                        let _ = app_clone.emit(
                            EVENT_ASK_ERROR,
                            AskErrorPayload {
                                project_id: project_id.clone(),
                                message_id: message_id.clone(),
                                error: error_msg,
                            },
                        );
                    }
                    break;
                }
                _ => {}
            }
        }

        let _ = sessions_arc.remove_and_kill(&project_id);
    });

    Ok(())
}

fn is_agent_noise(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with("Warning: no stdin data received")
        || trimmed.starts_with("If piping from a slow command, redirect stdin explicitly:")
        || trimmed.starts_with("Reading additional input from stdin")
        || trimmed.starts_with("error: unexpected argument")
        || trimmed.starts_with("tip: to pass '")
        || trimmed.starts_with("Usage: codex")
        || trimmed.starts_with("For more information, try '--help'")
}

fn save_assistant_message(
    app: &AppHandle,
    project_id: &str,
    content: &str,
    agent: &str,
    model: Option<&str>,
) {
    let db = app.state::<DbState>();
    let Ok(conn) = db.0.lock() else { return };
    let Ok(conversation) =
        crate::ask_engine::storage::get_or_create_conversation(&conn, project_id)
    else {
        return;
    };
    let _ = crate::ask_engine::storage::insert_message(
        &conn,
        &conversation.id,
        "assistant",
        content,
        Some(agent),
        model,
    );
}

fn build_null_stdin_command(binary: &str, args: &[String]) -> String {
    let escaped_args: Vec<String> = args
        .iter()
        .map(|arg| format!("'{}'", arg.replace('\'', "'\\''")))
        .collect();
    format!("{binary} {args} < /dev/null", binary = binary, args = escaped_args.join(" "))
}

async fn resolve_agent_binary(app: &AppHandle, agent: &str) -> Result<String, AskEngineError> {
    let binary = crate::agent_runtime::cli_binary_name(agent);
    if !is_safe_binary_name(binary) {
        return Err(AskEngineError::Shell(format!("Invalid agent name: {agent}")));
    }

    let lookup = format!("command -v {binary}");
    let output = app
        .shell()
        .command("/bin/zsh")
        .args(["-lc", &lookup])
        .output()
        .await
        .map_err(|err| AskEngineError::Shell(err.to_string()))?;

    if !output.status.success() {
        return Err(AskEngineError::Shell(format!("Agent '{agent}' not found")));
    }

    let resolved = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    if resolved.is_empty() {
        return Err(AskEngineError::Shell(format!("Agent '{agent}' not resolvable")));
    }

    Ok(resolved)
}
