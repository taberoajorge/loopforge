use crate::{agents, commands, connections, ephemeral_query, services};
#[cfg(not(test))]
#[path = "../../crates/loopforge-app-core/src/atomizer.rs"]
mod app_core_atomizer;
#[cfg(not(test))]
#[path = "../../crates/loopforge-app-core/src/projects.rs"]
mod app_core_projects;

#[cfg(not(test))]
pub fn attach_app(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    services::attach_runtime_aliases(builder, {
        let fallback: fn(tauri::ipc::Invoke<tauri::Wry>) -> bool = tauri::generate_handler![
            agents::detect_agents,
            agents::refresh_agents,
            agents::check_system_readiness,
            agents::get_known_agents,
            agents::get_agent_capabilities,
            agents::resolve_agent_selection,
            commands::display_vocabulary::get_display_vocabulary,
            commands::planning::query_plan_status,
            commands::planning::resolve_plan_state,
            commands::planning::resolve_plan_action,
            commands::planning::plan_user_action,
            commands::planning::replan,
            commands::projects_artifacts::load_existing_plan,
            commands::projects_artifacts::load_existing_prd,
            commands::projects_artifacts::load_output_log,
            commands::projects_artifacts::save_plan,
            commands::projects_artifacts::save_prd,
            commands::projects_artifacts::save_config,
            commands::projects_artifacts::load_config,
            commands::atomization::run_atomizer,
            commands::atomization::get_atomizer_activity_log,
            commands::atomization::get_atomizer_pipeline_state,
            commands::projects_wizard::save_wizard_state,
            commands::projects_wizard::resume_wizard,
            commands::projects_wizard::hydrate_wizard,
            commands::projects_wizard::save_draft,
            commands::projects_wizard::load_draft,
            commands::projects_lifecycle::pause_project,
            commands::projects_lifecycle::resume_project,
            commands::projects_lifecycle::archive_project,
            commands::projects_lifecycle::get_project_stories,
            commands::projects_lifecycle::get_guardrails,
            commands::projects_lifecycle::get_project_config,
            commands::projects::get_project_snapshot,
            commands::projects_listing::list_projects_enriched,
            commands::projects_listing::list_projects_grouped,
            commands::projects_lifecycle::get_notification_prefs,
            commands::projects_lifecycle::save_notification_prefs,
            commands::execution::start_loop,
            commands::execution::stop_loop,
            commands::execution::get_activity_feed,
            commands::ask::ask_question,
            commands::ask::ask_history,
            commands::ask::stop_ask,
            commands::ask::copy_ask_message,
            commands::ask::truncate_ask_from,
            commands::ask::retry_ask,
            ephemeral_query::ephemeral_query,
            connections::list_connections,
            connections::build_connection_workspace,
            commands::wizard_logic::advance_wizard_step,
            commands::wizard_logic::get_default_config,
            commands::wizard_logic::get_wizard_defaults,
            commands::wizard_logic::validate_project_config,
            commands::wizard_logic::validate_describe_input,
            commands::wizard_logic::validate_launch_readiness,
            commands::wizard_logic::add_story,
            commands::wizard_logic::update_story,
            commands::wizard_logic::remove_story,
            commands::wizard_logic::reorder_stories,
            commands::wizard_logic::get_stories,
            commands::wizard_logic::save_wizard_draft,
            commands::wizard_logic::complete_describe_step,
            commands::wizard_logic::complete_atomize_step,
            commands::wizard_logic::complete_configure_step,
            commands::wizard_logic::launch_project,
            commands::wizard_logic::exit_wizard,
            commands::wizard_logic::submit_project_config,
            commands::wizard_logic::mark_wizard_stale,
            commands::notifications::add_notification,
            commands::notifications::get_notifications,
            commands::notifications::mark_notification_read,
            commands::notifications::mark_all_notifications_read,
            commands::notifications::clear_notifications
        ];
        move |invoke: tauri::ipc::Invoke<tauri::Wry>| match invoke.message.command() {
            "start_plan" => {
                commands::planning::__cmd__start_plan!(plan_commands::start_plan, invoke)
            }
            "write_to_plan" => {
                commands::planning::__cmd__write_to_plan!(plan_commands::write_to_plan, invoke)
            }
            "stop_plan" => commands::planning::__cmd__stop_plan!(plan_commands::stop_plan, invoke),
            "run_atomizer" => {
                commands::atomization::__cmd__run_atomizer!(atomizer_commands::run_atomizer, invoke)
            }
            "create_project" => commands::projects_lifecycle::__cmd__create_project!(
                project_commands::create_project,
                invoke
            ),
            "finalize_draft" => commands::projects_wizard::__cmd__finalize_draft!(
                project_commands::finalize_draft,
                invoke
            ),
            "discard_draft" => commands::projects_wizard::__cmd__discard_draft!(
                project_commands::discard_draft,
                invoke
            ),
            "list_projects" => commands::projects_lifecycle::__cmd__list_projects!(
                project_commands::list_projects,
                invoke
            ),
            "get_project_detail" => commands::projects_lifecycle::__cmd__get_project_detail!(
                project_commands::get_project_detail,
                invoke
            ),
            _ => fallback(invoke),
        }
    })
}

#[cfg(test)]
pub fn attach_app(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    services::attach_runtime_aliases(builder, {
        let fallback: fn(tauri::ipc::Invoke<tauri::Wry>) -> bool = tauri::generate_handler![
            agents::detect_agents,
            agents::refresh_agents,
            agents::check_system_readiness,
            agents::get_known_agents,
            agents::get_agent_capabilities,
            commands::display_vocabulary::get_display_vocabulary,
            crate::invoke_contract::query_plan_status,
            crate::invoke_contract::load_existing_plan,
            commands::projects_artifacts::load_existing_prd,
            commands::projects_artifacts::load_output_log,
            crate::invoke_contract::save_plan,
            commands::projects_artifacts::save_prd,
            crate::invoke_contract::save_config,
            commands::projects_artifacts::load_config,
            crate::invoke_contract::run_atomizer,
            commands::atomization::get_atomizer_activity_log,
            commands::atomization::get_atomizer_pipeline_state,
            crate::invoke_contract::create_project,
            crate::invoke_contract::finalize_draft,
            commands::projects_wizard::discard_draft,
            commands::projects_wizard::save_wizard_state,
            commands::projects_wizard::resume_wizard,
            commands::projects_wizard::hydrate_wizard,
            crate::invoke_contract::save_draft,
            crate::invoke_contract::load_draft,
            commands::projects_lifecycle::list_projects,
            commands::projects_lifecycle::pause_project,
            commands::projects_lifecycle::resume_project,
            commands::projects_lifecycle::archive_project,
            crate::invoke_contract::get_project_detail,
            commands::projects_lifecycle::get_project_stories,
            commands::projects_lifecycle::get_guardrails,
            commands::projects_lifecycle::get_project_config,
            commands::projects::get_project_snapshot,
            commands::projects_listing::list_projects_enriched,
            commands::projects_listing::list_projects_grouped,
            commands::projects_lifecycle::get_notification_prefs,
            commands::projects_lifecycle::save_notification_prefs,
            crate::invoke_contract::start_loop,
            commands::execution::stop_loop,
            commands::execution::get_activity_feed,
            commands::ask::ask_question,
            commands::ask::ask_history,
            commands::ask::stop_ask,
            commands::ask::copy_ask_message,
            commands::ask::truncate_ask_from,
            commands::ask::retry_ask,
            ephemeral_query::ephemeral_query,
            connections::list_connections,
            connections::build_connection_workspace,
            commands::wizard_logic::get_wizard_defaults,
            commands::wizard_logic::complete_describe_step,
            commands::wizard_logic::complete_atomize_step,
            commands::wizard_logic::complete_configure_step,
            commands::wizard_logic::launch_project,
            commands::wizard_logic::exit_wizard,
            commands::notifications::add_notification,
            commands::notifications::get_notifications,
            commands::notifications::mark_notification_read,
            commands::notifications::mark_all_notifications_read,
            commands::notifications::clear_notifications
        ];
        move |invoke: tauri::ipc::Invoke<tauri::Wry>| match invoke.message.command() {
            "start_plan" => {
                crate::invoke_contract::__cmd__start_plan!(plan_commands::start_plan, invoke)
            }
            "write_to_plan" => {
                commands::planning::__cmd__write_to_plan!(plan_commands::write_to_plan, invoke)
            }
            "stop_plan" => commands::planning::__cmd__stop_plan!(plan_commands::stop_plan, invoke),
            _ => fallback(invoke),
        }
    })
}

#[cfg(test)]
pub fn attach_contract<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    services::attach_runtime_aliases(builder, {
        let fallback: fn(tauri::ipc::Invoke<R>) -> bool = tauri::generate_handler![
            crate::invoke_contract::create_project,
            crate::invoke_contract::save_draft,
            crate::invoke_contract::load_draft,
            crate::invoke_contract::finalize_draft,
            crate::invoke_contract::query_plan_status,
            crate::invoke_contract::load_existing_plan,
            crate::invoke_contract::save_plan,
            crate::invoke_contract::save_config,
            crate::invoke_contract::run_atomizer,
            crate::invoke_contract::start_loop,
            crate::invoke_contract::get_project_detail,
            commands::projects_lifecycle::archive_project
        ];
        move |invoke: tauri::ipc::Invoke<R>| match invoke.message.command() {
            "start_plan" => {
                crate::invoke_contract::__cmd__start_plan!(plan_commands::start_plan, invoke)
            }
            "write_to_plan" => {
                commands::planning::__cmd__write_to_plan!(plan_commands::write_to_plan, invoke)
            }
            "stop_plan" => commands::planning::__cmd__stop_plan!(plan_commands::stop_plan, invoke),
            _ => fallback(invoke),
        }
    })
}
#[cfg(not(test))]
mod atomizer_commands {
    use super::app_core_atomizer;
    use crate::atomizer::{AtomizeArgs, AtomizeProgress, AtomizerError};
    use ralph_core::prd::Prd;
    use tauri::{AppHandle, Emitter};
    pub async fn run_atomizer(app: AppHandle, args: AtomizeArgs) -> Result<Prd, AtomizerError> {
        let normalized_args = AtomizeArgs {
            project_id: required(args.project_id, "project_id").map_err(AtomizerError::Path)?,
            project_name: required(args.project_name, "project_name")
                .map_err(AtomizerError::Path)?,
            project_dir: args.project_dir,
            agent: required(args.agent, "agent").map_err(AtomizerError::Path)?,
            model: optional(args.model),
            effort: optional(args.effort),
        };
        let run_result = app_core_atomizer::run_atomizer(
            app_core_atomizer::AtomizerRequest {
                project_id: normalized_args.project_id.clone(),
            },
            |_| crate::atomizer::run_atomizer(app.clone(), normalized_args),
            |prd| prd.stories.len(),
        )
        .await?;
        for event in &run_result.events {
            let payload = event.as_progress_payload();
            let _ = app.emit(
                "atomization-progress",
                AtomizeProgress {
                    stage: payload.stage,
                    stage_name: payload.stage_name,
                    message: payload.message,
                    project_id: payload.project_id,
                    elapsed_ms: 0,
                },
            );
        }
        Ok(run_result.output)
    }
    fn required(value: String, name: &str) -> Result<String, String> {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            return Err(format!("{name} is required"));
        }
        Ok(trimmed)
    }
    fn optional(value: Option<String>) -> Option<String> {
        value
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }
}

mod plan_commands {
    use crate::app_core_plan;
    use crate::plan_engine::{PlanEngineError, PlanSessionsState, StartPlanArgs};
    use std::time::Instant;
    use tauri::{AppHandle, Runtime, State};

    pub async fn start_plan<R: Runtime>(
        app: AppHandle<R>,
        state: State<'_, PlanSessionsState>,
        args: StartPlanArgs,
    ) -> Result<(), PlanEngineError> {
        let args = StartPlanArgs {
            project_id: required(args.project_id, "project_id").map_err(PlanEngineError::Path)?,
            project_dir: args.project_dir,
            agent: required(args.agent, "agent").map_err(PlanEngineError::Path)?,
            model: optional(args.model),
            effort: optional(args.effort),
            initial_prompt: required(args.initial_prompt, "initial_prompt")
                .map_err(PlanEngineError::Path)?,
        };
        app_core_plan::start_plan(args.project_id.clone(), |_| {
            crate::plan_engine::start_plan(app, state, args)
        })
        .await
    }

    pub async fn write_to_plan(
        state: State<'_, PlanSessionsState>,
        project_id: String,
        input: String,
    ) -> Result<(), PlanEngineError> {
        let project_id = required(project_id, "project_id").map_err(PlanEngineError::Path)?;
        let activity_at = std::sync::Arc::new(std::sync::Mutex::new(None::<Instant>));
        let touch_project_id = project_id.clone();
        app_core_plan::write_to_plan(
            project_id,
            input,
            |project_id, payload| {
                let mut sessions = state.0.lock().map_err(|_| PlanEngineError::LockPoisoned)?;
                let entry = sessions
                    .sessions
                    .get_mut(&project_id)
                    .ok_or_else(|| PlanEngineError::NoSession(project_id.clone()))?;
                entry
                    .handle
                    .write(&payload)
                    .map_err(PlanEngineError::Shell)?;
                if let Ok(mut activity) = activity_at.lock() {
                    *activity = Some(Instant::now());
                }
                Ok(())
            },
            || {
                if let Some(timestamp) = activity_at.lock().ok().and_then(|guard| *guard) {
                    if let Ok(mut sessions) = state.0.lock() {
                        if let Some(entry) = sessions.sessions.get_mut(&touch_project_id) {
                            if let Ok(mut last_activity) = entry.last_activity_at.lock() {
                                *last_activity = timestamp;
                            }
                        }
                    }
                }
            },
        )
    }

    pub async fn stop_plan(
        state: State<'_, PlanSessionsState>,
        project_id: String,
    ) -> Result<(), PlanEngineError> {
        let project_id = required(project_id, "project_id").map_err(PlanEngineError::Path)?;
        app_core_plan::stop_plan(project_id, |project_id, reason| {
            app_core_plan::cleanup_session(
                &project_id,
                reason,
                |project_id| {
                    let mut sessions = state.0.lock().map_err(|_| PlanEngineError::LockPoisoned)?;
                    Ok(sessions.sessions.remove(project_id))
                },
                |entry| entry.handle.kill().map_err(PlanEngineError::Shell),
            )
            .map(|_| ())
        })
    }

    fn required(value: String, field_name: &str) -> Result<String, String> {
        let normalized = value.trim();
        if normalized.is_empty() {
            return Err(format!("Missing required field: {field_name}"));
        }
        Ok(normalized.to_string())
    }

    fn optional(value: Option<String>) -> Option<String> {
        value
            .map(|content| content.trim().to_string())
            .filter(|content| !content.is_empty())
    }
}

#[cfg(not(test))]
mod project_commands {
    use super::app_core_projects;
    use crate::projects::artifacts::{artifact_dir, init_artifacts};
    use crate::projects::repository::{group_projects_by_status, row_to_project, PROJECT_COLUMNS};
    use crate::projects::{Project, ProjectDetail, ProjectError, ProjectsByStatus};
    use crate::storage::db::DbState;
    use ralph_core::prd::{Prd, UserStory};
    use tauri::{AppHandle, State};
    use uuid::Uuid;

    impl app_core_projects::StoryState for UserStory {
        fn passes(&self) -> bool {
            self.passes
        }
        fn blocked(&self) -> bool {
            self.blocked
        }
    }

    pub async fn create_project(
        app: AppHandle,
        db: State<'_, DbState>,
        name: String,
        description: String,
        working_directory: String,
        wizard_step: Option<String>,
    ) -> Result<Project, ProjectError> {
        let request = app_core_projects::CreateProjectRequest {
            name: required(name, "name").map_err(ProjectError::Path)?,
            description: required(description, "description").map_err(ProjectError::Path)?,
            working_directory: required(working_directory, "working_directory")
                .map_err(ProjectError::Path)?,
            wizard_step: optional(wizard_step),
        };
        app_core_projects::create_project(
            request,
            || Uuid::new_v4().to_string(),
            || chrono::Utc::now().to_rfc3339(),
            |project_id, project_name| {
                let dir = artifact_dir(&app, project_id)?;
                init_artifacts(&dir, project_name)
            },
            |record| {
                let project = Project {
                    id: record.id.clone(),
                    name: record.name.clone(),
                    description: record.description.clone(),
                    status: record.status.clone(),
                    working_directory: record.working_directory.clone(),
                    created_at: record.created_at.clone(),
                    updated_at: record.updated_at.clone(),
                    wizard_step: record.wizard_step.clone(),
                };
                let conn =
                    db.0.lock()
                        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
                conn.execute("INSERT INTO projects (id, name, description, status, working_directory, created_at, updated_at, wizard_step) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", rusqlite::params![record.id, record.name, record.description, record.status, record.working_directory, record.created_at, record.updated_at, record.wizard_step])?;
                Ok(project)
            },
        )
    }

    pub async fn finalize_draft(
        app: AppHandle,
        db: State<'_, DbState>,
        project_id: String,
    ) -> Result<(), ProjectError> {
        let project_id = required(project_id, "project_id").map_err(ProjectError::Path)?;
        app_core_projects::finalize_draft(
            project_id,
            || chrono::Utc::now().to_rfc3339(),
            |project_id, updated_at| {
                let conn =
                    db.0.lock()
                        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
                conn.execute("UPDATE projects SET wizard_step = NULL, wizard_state_json = NULL, updated_at = ?1 WHERE id = ?2", rusqlite::params![updated_at, project_id])?;
                Ok(())
            },
            |project_id| {
                let draft_path = artifact_dir(&app, project_id)?.join("draft.json");
                if draft_path.exists() {
                    let _ = std::fs::remove_file(draft_path);
                }
                Ok(())
            },
        )
    }

    pub async fn discard_draft(
        app: AppHandle,
        db: State<'_, DbState>,
        project_id: String,
    ) -> Result<(), ProjectError> {
        let project_id = required(project_id, "project_id").map_err(ProjectError::Path)?;
        app_core_projects::discard_draft(
            project_id,
            |project_id| {
                let conn =
                    db.0.lock()
                        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
                Ok(conn.execute(
                    "DELETE FROM projects WHERE id = ?1 AND status = 'draft'",
                    rusqlite::params![project_id],
                )? != 0)
            },
            |project_id| {
                let dir = artifact_dir(&app, project_id)?;
                if dir.exists() {
                    let _ = std::fs::remove_dir_all(&dir);
                }
                Ok(())
            },
            ProjectError::NotFound,
        )
    }

    pub async fn list_projects(db: State<'_, DbState>) -> Result<ProjectsByStatus, ProjectError> {
        app_core_projects::list_projects(|| {
            let conn =
                db.0.lock()
                    .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
            let query = format!("SELECT {PROJECT_COLUMNS} FROM projects ORDER BY updated_at DESC");
            let mut stmt = conn.prepare(&query)?;
            let projects: Vec<Project> = stmt
                .query_map([], row_to_project)?
                .filter_map(Result::ok)
                .collect();
            Ok(group_projects_by_status(projects))
        })
    }

    pub async fn get_project_detail(
        app: AppHandle,
        db: State<'_, DbState>,
        project_id: String,
    ) -> Result<ProjectDetail, ProjectError> {
        let project_id = required(project_id, "project_id").map_err(ProjectError::Path)?;
        let detail = app_core_projects::get_project_detail(
            project_id,
            |project_id| {
                let conn =
                    db.0.lock()
                        .map_err(|_| ProjectError::Db("Lock poisoned".to_string()))?;
                let query = format!("SELECT {PROJECT_COLUMNS} FROM projects WHERE id = ?1");
                let mut stmt = conn.prepare(&query)?;
                stmt.query_row(rusqlite::params![project_id], row_to_project)
                    .map_err(|_| ProjectError::NotFound(project_id.to_string()))
            },
            |project_id| {
                let prd_path = artifact_dir(&app, project_id)?.join("prd.json");
                Ok(if prd_path.exists() {
                    Prd::load(&prd_path)
                        .map(|prd| prd.stories)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                })
            },
        )?;
        Ok(ProjectDetail {
            project: detail.project,
            total_stories: detail.total_stories,
            passed_count: detail.passed_count,
            blocked_count: detail.blocked_count,
            pending_count: detail.pending_count,
            stories: detail.stories,
        })
    }

    fn required(value: String, name: &str) -> Result<String, String> {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            return Err(format!("{name} is required"));
        }
        Ok(trimmed)
    }

    fn optional(value: Option<String>) -> Option<String> {
        value
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }
}
