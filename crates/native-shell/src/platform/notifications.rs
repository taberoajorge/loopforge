use std::io::{Error, Result};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationEvent {
    PlanningCompleted,
    LoopCompleted,
    StoryCompleted,
    StoryBlocked,
    RateLimited,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationPreferences {
    pub planning: bool,
    pub completions: bool,
    pub blocked: bool,
    pub rate_limits: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            planning: true,
            completions: true,
            blocked: true,
            rate_limits: true,
        }
    }
}

impl NotificationPreferences {
    pub fn allows(&self, event: NotificationEvent) -> bool {
        match event {
            NotificationEvent::PlanningCompleted => self.planning,
            NotificationEvent::LoopCompleted | NotificationEvent::StoryCompleted => self.completions,
            NotificationEvent::StoryBlocked => self.blocked,
            NotificationEvent::RateLimited => self.rate_limits,
        }
    }
}

pub trait NotificationTransport {
    fn send(&self, title: &str, body: &str) -> Result<()>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ShellNotificationTransport;

impl NotificationTransport for ShellNotificationTransport {
    fn send(&self, title: &str, body: &str) -> Result<()> {
        send_native_notification(title, body)
    }
}

#[derive(Debug, Clone)]
pub struct NotificationCenter<Transport = ShellNotificationTransport> {
    transport: Transport,
}

impl Default for NotificationCenter<ShellNotificationTransport> {
    fn default() -> Self {
        Self {
            transport: ShellNotificationTransport,
        }
    }
}

impl<Transport: NotificationTransport> NotificationCenter<Transport> {
    pub fn new(transport: Transport) -> Self {
        Self { transport }
    }

    pub fn notify(
        &self,
        event: NotificationEvent,
        preferences: &NotificationPreferences,
        title: &str,
        body: &str,
    ) -> Result<bool> {
        if !preferences.allows(event) {
            return Ok(false);
        }
        self.transport.send(title, body)?;
        Ok(true)
    }
}

#[cfg(target_os = "macos")]
fn send_native_notification(title: &str, body: &str) -> Result<()> {
    let escaped_title = escape_apple_script_text(title);
    let escaped_body = escape_apple_script_text(body);
    let script = format!(
        "display notification \"{}\" with title \"{}\"",
        escaped_body, escaped_title
    );
    let status = Command::new("osascript").arg("-e").arg(script).status()?;
    ensure_success(status.success(), "osascript")
}

#[cfg(target_os = "linux")]
fn send_native_notification(title: &str, body: &str) -> Result<()> {
    let status = Command::new("notify-send").arg(title).arg(body).status()?;
    ensure_success(status.success(), "notify-send")
}

#[cfg(target_os = "windows")]
fn send_native_notification(title: &str, body: &str) -> Result<()> {
    let escaped_title = title.replace('\'', "''");
    let escaped_body = body.replace('\'', "''");
    let script = format!(
        "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null;\
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] > $null;\
$xml = New-Object Windows.Data.Xml.Dom.XmlDocument;\
$xml.LoadXml('<toast><visual><binding template=\"ToastGeneric\"><text>{}</text><text>{}</text></binding></visual></toast>');\
$toast = [Windows.UI.Notifications.ToastNotification]::new($xml);\
$notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('LoopForge');\
$notifier.Show($toast);",
        escaped_title, escaped_body
    );
    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .status()?;
    ensure_success(status.success(), "powershell")
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn send_native_notification(_title: &str, _body: &str) -> Result<()> {
    Ok(())
}

fn ensure_success(success: bool, command: &str) -> Result<()> {
    if success {
        return Ok(());
    }
    Err(Error::other(format!("native notification command failed: {command}")))
}

#[cfg(target_os = "macos")]
fn escape_apple_script_text(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
