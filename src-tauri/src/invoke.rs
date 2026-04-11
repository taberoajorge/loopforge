use crate::{agents, commands, connections, ephemeral_query};
#[path = "services/mod.rs"]
mod services;

fn attach_with_session_aliases<R: tauri::Runtime, F>(
    builder: tauri::Builder<R>,
    fallback: F,
) -> tauri::Builder<R>
where
    F: Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static,
{
    builder.invoke_handler(move |invoke| match invoke.message.command() {
        "session_stats" => services::session_adapter::__cmd__session_stats_command!(
            services::session_adapter::session_stats_command,
            invoke
        ),
        "get_iteration_history" => {
            services::session_adapter::__cmd__get_iteration_history_command!(
                services::session_adapter::get_iteration_history_command,
                invoke
            )
        }
        _ => fallback(invoke),
    })
}

#[cfg(not(test))]
pub fn attach_app(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    attach_with_session_aliases(
        builder,
        tauri::generate_handler![
            agents::detect_agents,
            agents::refresh_agents,
            agents::get_agent_capabilities,
            commands::planning::start_plan,
            commands::planning::write_to_plan,
            commands::planning::stop_plan,
            commands::planning::query_plan_status,
            commands::projects_artifacts::load_existing_plan,
            commands::projects_artifacts::load_existing_prd,
            commands::projects_artifacts::load_output_log,
            commands::projects_artifacts::save_plan,
            commands::projects_artifacts::save_prd,
            commands::projects_artifacts::save_config,
            commands::projects_artifacts::load_config,
            commands::atomization::run_atomizer,
            commands::projects_lifecycle::create_project,
            commands::projects_wizard::finalize_draft,
            commands::projects_wizard::discard_draft,
            commands::projects_wizard::save_wizard_state,
            commands::projects_wizard::resume_wizard,
            commands::projects_wizard::save_draft,
            commands::projects_wizard::load_draft,
            commands::projects_lifecycle::list_projects,
            commands::projects_lifecycle::pause_project,
            commands::projects_lifecycle::resume_project,
            commands::projects_lifecycle::archive_project,
            commands::projects_lifecycle::get_project_detail,
            commands::projects_lifecycle::get_project_stories,
            commands::projects_lifecycle::get_guardrails,
            commands::projects_lifecycle::get_project_config,
            commands::projects::get_project_snapshot,
            commands::projects_listing::list_projects_enriched,
            commands::projects_lifecycle::get_notification_prefs,
            commands::projects_lifecycle::save_notification_prefs,
            commands::execution::start_loop,
            commands::execution::stop_loop,
            commands::ask::ask_question,
            commands::ask::ask_history,
            commands::ask::stop_ask,
            commands::ask::copy_ask_message,
            commands::ask::truncate_ask_from,
            commands::ask::retry_ask,
            ephemeral_query::ephemeral_query,
            connections::list_connections,
            connections::build_connection_workspace,
        ],
    )
}

#[cfg(test)]
pub fn attach_app(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    attach_with_session_aliases(
        builder,
        tauri::generate_handler![
            agents::detect_agents,
            agents::refresh_agents,
            agents::get_agent_capabilities,
            crate::invoke_contract::start_plan,
            commands::planning::write_to_plan,
            commands::planning::stop_plan,
            crate::invoke_contract::query_plan_status,
            crate::invoke_contract::load_existing_plan,
            commands::projects_artifacts::load_existing_prd,
            commands::projects_artifacts::load_output_log,
            crate::invoke_contract::save_plan,
            commands::projects_artifacts::save_prd,
            crate::invoke_contract::save_config,
            commands::projects_artifacts::load_config,
            crate::invoke_contract::run_atomizer,
            crate::invoke_contract::create_project,
            crate::invoke_contract::finalize_draft,
            commands::projects_wizard::discard_draft,
            commands::projects_wizard::save_wizard_state,
            commands::projects_wizard::resume_wizard,
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
            commands::projects_lifecycle::get_notification_prefs,
            commands::projects_lifecycle::save_notification_prefs,
            crate::invoke_contract::start_loop,
            commands::execution::stop_loop,
            commands::ask::ask_question,
            commands::ask::ask_history,
            commands::ask::stop_ask,
            commands::ask::copy_ask_message,
            commands::ask::truncate_ask_from,
            commands::ask::retry_ask,
            ephemeral_query::ephemeral_query,
            connections::list_connections,
            connections::build_connection_workspace,
        ],
    )
}

#[cfg(test)]
pub fn attach_contract<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    attach_with_session_aliases(
        builder,
        tauri::generate_handler![
            crate::invoke_contract::create_project,
            crate::invoke_contract::save_draft,
            crate::invoke_contract::load_draft,
            crate::invoke_contract::finalize_draft,
            crate::invoke_contract::start_plan,
            commands::planning::write_to_plan,
            commands::planning::stop_plan,
            crate::invoke_contract::query_plan_status,
            crate::invoke_contract::load_existing_plan,
            crate::invoke_contract::save_plan,
            crate::invoke_contract::save_config,
            crate::invoke_contract::run_atomizer,
            crate::invoke_contract::start_loop,
            crate::invoke_contract::get_project_detail,
            commands::projects_lifecycle::archive_project,
        ],
    )
}
