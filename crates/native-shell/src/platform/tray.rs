use crate::services::loops::LoopSessionService;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayCommand {
    OpenShell,
    PauseSession,
    ResumeSession,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayCommandResult {
    OpenShellRequested,
    SessionPaused,
    SessionResumed,
    QuitRequested,
    Ignored,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrayMenuState {
    pub has_active_session: bool,
    pub pause_enabled: bool,
    pub resume_enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct NativeTray {
    menu_state: TrayMenuState,
}

impl NativeTray {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn menu_state(&self) -> TrayMenuState {
        self.menu_state
    }

    pub fn sync_from_loops(&mut self, loops: &LoopSessionService) {
        self.menu_state = TrayMenuState {
            has_active_session: loops.has_active_session(),
            pause_enabled: loops.can_pause(),
            resume_enabled: loops.can_resume(),
        };
    }

    pub fn execute(
        &mut self,
        command: TrayCommand,
        loops: &mut LoopSessionService,
    ) -> TrayCommandResult {
        let result = match command {
            TrayCommand::OpenShell => TrayCommandResult::OpenShellRequested,
            TrayCommand::PauseSession => {
                if loops.pause_session() {
                    TrayCommandResult::SessionPaused
                } else {
                    TrayCommandResult::Ignored
                }
            }
            TrayCommand::ResumeSession => {
                if loops.resume_session() {
                    TrayCommandResult::SessionResumed
                } else {
                    TrayCommandResult::Ignored
                }
            }
            TrayCommand::Quit => TrayCommandResult::QuitRequested,
        };
        self.sync_from_loops(loops);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{NativeTray, TrayCommand, TrayCommandResult};
    use crate::services::loops::LoopSessionService;

    #[test]
    fn reflects_session_presence_and_pause_resume_capability() {
        let mut tray = NativeTray::new();
        let mut loops = LoopSessionService::with_session("session-42", false);
        tray.sync_from_loops(&loops);
        assert!(tray.menu_state().has_active_session);
        assert!(tray.menu_state().pause_enabled);
        assert!(!tray.menu_state().resume_enabled);
        assert_eq!(
            tray.execute(TrayCommand::PauseSession, &mut loops),
            TrayCommandResult::SessionPaused
        );
        assert!(!tray.menu_state().pause_enabled);
        assert!(tray.menu_state().resume_enabled);
    }

    #[test]
    fn open_and_quit_commands_do_not_depend_on_tauri() {
        let mut tray = NativeTray::new();
        let mut loops = LoopSessionService::default();
        assert_eq!(
            tray.execute(TrayCommand::OpenShell, &mut loops),
            TrayCommandResult::OpenShellRequested
        );
        assert_eq!(
            tray.execute(TrayCommand::Quit, &mut loops),
            TrayCommandResult::QuitRequested
        );
    }
}
