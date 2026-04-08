pub mod manager;
pub mod service;

use crate::config::{RalphConfig, ServiceConfig};
use crate::logger;

pub async fn check_all_services_parallel(config: &RalphConfig) -> bool {
    let services = match &config.services {
        Some(svc) => svc,
        None => return true,
    };

    let legacy_fut = async {
        match &services.legacy {
            Some(svc) => ensure_service_running(svc).await,
            None => true,
        }
    };

    let new_svc_fut = async {
        match &services.new_service {
            Some(svc) => ensure_service_running(svc).await,
            None => true,
        }
    };

    let (legacy_ok, new_ok) = tokio::join!(legacy_fut, new_svc_fut);
    legacy_ok && new_ok
}

async fn ensure_service_running(svc_config: &ServiceConfig) -> bool {
    if let Some(health_url) = &svc_config.health_url {
        if service::check_health(health_url).await {
            return true;
        }
        logger::log_warning(&format!("{} is DOWN, attempting auto-start...", svc_config.name));
        if let Some(start_cmd) = &svc_config.start_command {
            let work_dir = svc_config.working_directory.as_deref();
            if let Some(stop_cmd) = &svc_config.stop_command {
                manager::run_shell_command(stop_cmd, work_dir).await;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            if svc_config.service_type.as_deref() == Some("docker") {
                if let Some(build_cmd) = &svc_config.build_command {
                    if !manager::run_shell_command(build_cmd, work_dir).await {
                        logger::log_error(
                            &format!("Docker build FAILED for {}", svc_config.name),
                            None,
                        );
                        return svc_config.optional;
                    }
                }
            }
            manager::run_shell_command_background(start_cmd, work_dir).await;
            if service::wait_for_health(
                &svc_config.name,
                health_url,
                svc_config.max_start_wait_secs,
            )
            .await
            {
                logger::log_success(&format!("{} started successfully", svc_config.name));
                return true;
            }
        }
        if svc_config.optional {
            logger::log_warning(&format!("{} is optional, continuing", svc_config.name));
            return true;
        }
        return false;
    }
    true
}
