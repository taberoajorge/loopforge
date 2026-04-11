use crate::platform::{
    NativeTray, TrayCommand, TrayCommandResult, TrayMenuState, NativeUpdater, UpdateCheckOutcome,
    UpdateStatus,
};
use crate::services::loops::LoopSessionService;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateMenuAction {
    CheckForUpdates,
    ViewUpdateStatus,
}

#[derive(Debug)]
pub struct NativeShellApp {
    updater: NativeUpdater,
    update_status: UpdateStatus,
    tray: NativeTray,
    loops: LoopSessionService,
    shell_open: bool,
    quit_requested: bool,
}

impl NativeShellApp {
    pub fn new(updater: NativeUpdater) -> Self {
        let mut app = Self {
            updater,
            update_status: UpdateStatus::Idle,
            tray: NativeTray::new(),
            loops: LoopSessionService::default(),
            shell_open: false,
            quit_requested: false,
        };
        app.refresh_tray_state();
        app
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
        self.refresh_tray_state();
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

    pub fn set_loop_session_presence(&mut self, session_id: Option<String>, paused: bool) {
        self.loops.set_session_presence(session_id, paused);
        self.refresh_tray_state();
    }

    pub fn tray_state(&self) -> TrayMenuState {
        self.tray.menu_state()
    }

    pub fn handle_tray_command(&mut self, command: TrayCommand) -> TrayCommandResult {
        let result = self.tray.execute(command, &mut self.loops);
        match result {
            TrayCommandResult::OpenShellRequested => {
                self.shell_open = true;
            }
            TrayCommandResult::QuitRequested => {
                self.quit_requested = true;
            }
            TrayCommandResult::Ignored
            | TrayCommandResult::SessionPaused
            | TrayCommandResult::SessionResumed => {}
        }
        self.refresh_tray_state();
        result
    }

    pub fn shell_open(&self) -> bool {
        self.shell_open
    }

    pub fn quit_requested(&self) -> bool {
        self.quit_requested
    }

    fn refresh_tray_state(&mut self) {
        self.tray.sync_from_loops(&self.loops);
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
        NativeShellApp, NativeUpdater, TrayCommand, TrayCommandResult, UpdateCheckOutcome,
        UpdateMenuAction, UpdateStatus,
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
    fn menu_can_display_status_without_triggering_manual_check() {
        let updater = NativeUpdater::new(UpdateCheckOutcome::NoUpdate);
        let mut app = NativeShellApp::new(updater);
        app.run_startup_hooks();
        assert_eq!(
            app.handle_update_menu_action(UpdateMenuAction::ViewUpdateStatus),
            "You are up to date".to_string()
        );
    }

    #[test]
    fn startup_initializes_tray_without_session() {
        let updater = NativeUpdater::new(UpdateCheckOutcome::NoUpdate);
        let mut app = NativeShellApp::new(updater);
        app.run_startup_hooks();
        let state = app.tray_state();
        assert!(!state.has_active_session);
        assert!(!state.pause_enabled);
        assert!(!state.resume_enabled);
    }

    #[test]
    fn tray_commands_open_pause_resume_and_quit() {
        let updater = NativeUpdater::new(UpdateCheckOutcome::NoUpdate);
        let mut app = NativeShellApp::new(updater);
        app.set_loop_session_presence(Some("session-1".to_string()), false);
        assert_eq!(
            app.handle_tray_command(TrayCommand::OpenShell),
            TrayCommandResult::OpenShellRequested
        );
        assert!(app.shell_open());
        assert_eq!(
            app.handle_tray_command(TrayCommand::PauseSession),
            TrayCommandResult::SessionPaused
        );
        assert!(app.tray_state().resume_enabled);
        assert_eq!(
            app.handle_tray_command(TrayCommand::ResumeSession),
            TrayCommandResult::SessionResumed
        );
        assert!(app.tray_state().pause_enabled);
        assert_eq!(
            app.handle_tray_command(TrayCommand::Quit),
            TrayCommandResult::QuitRequested
        );
        assert!(app.quit_requested());
    }
}
