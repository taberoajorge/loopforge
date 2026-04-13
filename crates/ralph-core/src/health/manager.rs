use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
use std::path::Path;
use std::process::Stdio;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System};

pub async fn run_shell_command(command: &str, work_dir: Option<&Path>) -> bool {
    let mut process = crate::platform::shell_command(command);
    if let Some(directory) = work_dir {
        process.current_dir(directory);
    }
    match process.output().await {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub async fn run_shell_command_background(command: &str, work_dir: Option<&Path>) {
    let mut process = crate::platform::shell_command(command);
    process
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(directory) = work_dir {
        process.current_dir(directory);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        process.creation_flags(0x08000000);
    }
    let _ = process.spawn();
}

pub async fn kill_process_on_port(port: u16) {
    let process_ids = tokio::task::spawn_blocking(move || collect_process_ids_on_port(port))
        .await
        .unwrap_or_default();
    for process_id in process_ids {
        let _ = send_signal(process_id, Signal::Term).await;
    }
}

fn collect_process_ids_on_port(port: u16) -> Vec<u32> {
    let family_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let protocol_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let mut collected_ids = Vec::new();
    let socket_info_list = match netstat2::get_sockets_info(family_flags, protocol_flags) {
        Ok(items) => items,
        Err(_) => return collected_ids,
    };
    for socket_info in socket_info_list {
        let local_port = match socket_info.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_socket_info) => tcp_socket_info.local_port,
            ProtocolSocketInfo::Udp(udp_socket_info) => udp_socket_info.local_port,
        };
        if local_port != port {
            continue;
        }
        for process_id in socket_info.associated_pids {
            collected_ids.push(process_id);
        }
    }
    collected_ids.sort_unstable();
    collected_ids.dedup();
    collected_ids
}

fn process_exists(process_id: u32) -> bool {
    let mut system = System::new();
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    system.process(Pid::from_u32(process_id)).is_some()
}

fn signal_process(process_id: u32, signal: Signal) -> bool {
    let mut system = System::new();
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    match system.process(Pid::from_u32(process_id)) {
        Some(process) => process.kill_with(signal).unwrap_or(false),
        None => false,
    }
}

pub async fn graceful_kill(pid: u32) {
    use std::time::Duration;
    let _ = send_signal(pid, Signal::Interrupt).await;
    tokio::time::sleep(Duration::from_secs(10)).await;

    if is_process_alive(pid).await {
        let _ = send_signal(pid, Signal::Term).await;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    if is_process_alive(pid).await {
        let _ = send_signal(pid, Signal::Kill).await;
        crate::logger::log_warning(&format!("Force killed PID {pid} (SIGKILL)"));
    }
}

async fn is_process_alive(pid: u32) -> bool {
    tokio::task::spawn_blocking(move || process_exists(pid))
        .await
        .unwrap_or(false)
}

async fn send_signal(process_id: u32, signal: Signal) -> bool {
    tokio::task::spawn_blocking(move || signal_process(process_id, signal))
        .await
        .unwrap_or(false)
}
