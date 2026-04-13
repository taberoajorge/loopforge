use std::path::Path;
use tokio::process::Command;

pub async fn run_shell_command(command: &str, work_dir: Option<&Path>) -> bool {
    let mut cmd = Command::new("sh");
    cmd.args(["-c", command]);
    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }
    match cmd.output().await {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub async fn run_shell_command_background(command: &str, work_dir: Option<&Path>) {
    let nohup_cmd = format!("nohup {command} > /dev/null 2>&1 &");
    let mut cmd = Command::new("sh");
    cmd.args(["-c", &nohup_cmd]);
    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }
    let _ = cmd.spawn();
}

pub async fn kill_process_on_port(port: u16) {
    let lsof_cmd = format!("lsof -ti :{port}");
    let output = Command::new("sh").args(["-c", &lsof_cmd]).output().await;

    if let Ok(output) = output {
        let pids = String::from_utf8_lossy(&output.stdout);
        for pid_str in pids.lines() {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                let _ = Command::new("kill").arg(pid.to_string()).output().await;
            }
        }
    }
}

pub async fn graceful_kill(pid: u32) {
    use std::time::Duration;

    let pid_str = pid.to_string();

    let _ = Command::new("kill").args(["-INT", &pid_str]).output().await;
    tokio::time::sleep(Duration::from_secs(10)).await;

    if is_process_alive(pid).await {
        let _ = Command::new("kill")
            .args(["-TERM", &pid_str])
            .output()
            .await;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    if is_process_alive(pid).await {
        let _ = Command::new("kill").args(["-9", &pid_str]).output().await;
        crate::logger::log_warning(&format!("Force killed PID {pid} (SIGKILL)"));
    }
}

async fn is_process_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}
