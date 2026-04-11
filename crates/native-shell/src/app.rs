use crate::platform::{NativeUpdater, UpdateCheckOutcome, UpdateStatus};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateMenuAction {
    CheckForUpdates,
    ViewUpdateStatus,
}

#[derive(Debug)]
pub struct NativeShellApp {
    updater: NativeUpdater,
    update_status: UpdateStatus,
}

impl NativeShellApp {
    pub fn new(updater: NativeUpdater) -> Self {
        Self {
            updater,
            update_status: UpdateStatus::Idle,
        }
    }

    pub fn run_background_update_check(&mut self) {
        self.update_status = UpdateStatus::Checking;
        self.update_status = self.updater.background_check();
    }

    pub fn run_manual_update_check(&mut self) {
        self.update_status = UpdateStatus::Checking;
        self.update_status = self.updater.manual_check();
    }

    pub fn run_startup_hooks(&mut self) {
        self.run_background_update_check();
    }

    pub fn handle_update_menu_action(&mut self, action: UpdateMenuAction) -> String {
        match action {
            UpdateMenuAction::CheckForUpdates => {
                self.run_manual_update_check();
                self.update_status_label()
            }
            UpdateMenuAction::ViewUpdateStatus => self.update_status_label(),
        }
    }

    pub fn update_status(&self) -> &UpdateStatus {
        &self.update_status
    }

    pub fn update_status_label(&self) -> String {
        match &self.update_status {
            UpdateStatus::Idle => "Update status: idle".to_string(),
            UpdateStatus::Checking => "Checking for updates...".to_string(),
            UpdateStatus::NoUpdate => "You are up to date".to_string(),
            UpdateStatus::UpdateAvailable { version, .. } => {
                format!("Update available: {version}")
            }
            UpdateStatus::Failed(message) => format!("Update check failed: {message}"),
        }
    }
}

impl Default for NativeShellApp {
    fn default() -> Self {
        Self::new(NativeUpdater::new(UpdateCheckOutcome::NoUpdate))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NativeShellApp, NativeUpdater, UpdateCheckOutcome, UpdateMenuAction, UpdateStatus,
    };

    #[test]
    fn startup_hook_runs_background_check() {
        let updater = NativeUpdater::new(UpdateCheckOutcome::NoUpdate);
        let mut app = NativeShellApp::new(updater);
        app.run_startup_hooks();
        assert_eq!(app.update_status(), &UpdateStatus::NoUpdate);
        assert_eq!(app.update_status_label(), "You are up to date".to_string());
    }

    #[test]
    fn manual_menu_action_reports_update_available() {
        let updater = NativeUpdater::new(UpdateCheckOutcome::UpdateAvailable {
            version: "3.0.0".to_string(),
            notes: Some("Native shell updater migration".to_string()),
        });
        let mut app = NativeShellApp::new(updater);
        let status_label = app.handle_update_menu_action(UpdateMenuAction::CheckForUpdates);
        assert_eq!(
            app.update_status(),
            &UpdateStatus::UpdateAvailable {
                version: "3.0.0".to_string(),
                notes: Some("Native shell updater migration".to_string()),
            }
        );
        assert_eq!(
            status_label,
            "Update available: 3.0.0".to_string()
        );
    }

    #[test]
    fn menu_can_display_status_without_triggering_manual_check() {
        let updater = NativeUpdater::new(UpdateCheckOutcome::NoUpdate);
        let mut app = NativeShellApp::new(updater);
        app.run_startup_hooks();
        assert_eq!(
            app.handle_update_menu_action(UpdateMenuAction::ViewUpdateStatus),
            "You are up to date".to_string()
        );
    }
}
