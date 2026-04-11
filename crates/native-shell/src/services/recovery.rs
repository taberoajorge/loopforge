use std::io;

use super::loops_service::{LoopService, LoopUpdate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    Reattach,
    Resume,
}

impl RecoveryAction {
    pub fn from_running(running: bool) -> Self {
        if running { Self::Reattach } else { Self::Resume }
    }

    pub fn startup_event(self) -> &'static str {
        if self == Self::Reattach {
            "Recovered active session and reattached monitor."
        } else {
            "Recovered paused session ready to resume."
        }
    }

    pub fn home_action_label(self) -> &'static str {
        if self == Self::Reattach { "Reattach active loop" } else { "Resume loop session" }
    }

    pub fn opens_monitor(self) -> bool { self == Self::Reattach }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveredSession {
    pub action: RecoveryAction,
    pub update: LoopUpdate,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RecoveryService;

impl RecoveryService {
    pub fn recover(loop_service: &mut LoopService) -> io::Result<Option<RecoveredSession>> {
        let Some(mut update) = loop_service.load_persisted_session()? else { return Ok(None); };
        let action = RecoveryAction::from_running(update.running);
        if !update.events.iter().any(|event| event == action.startup_event()) {
            update.events.push(action.startup_event().to_owned());
        }
        Ok(Some(RecoveredSession { action, update }))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{RecoveryAction, RecoveryService};
    use super::super::loops_service::LoopService;

    fn temp_projects_root() -> std::path::PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
        std::env::temp_dir().join(format!("loopforge-shell-recovery-{}-{}", std::process::id(), stamp))
    }

    #[test]
    fn recover_reattaches_running_session() {
        let projects_root = temp_projects_root();
        let mut writer = LoopService::new(projects_root.clone());
        writer.start_session("project-active", "Active project").expect("start");
        let mut reader = LoopService::new(projects_root.clone());
        let recovered = RecoveryService::recover(&mut reader).expect("recover").expect("session");
        assert_eq!(recovered.action, RecoveryAction::Reattach);
        assert!(recovered.update.running);
        assert!(recovered.update.events.iter().any(|event| event.contains("reattached")));
        let _ = fs::remove_dir_all(projects_root);
    }

    #[test]
    fn recover_surfaces_resumable_session() {
        let projects_root = temp_projects_root();
        let mut writer = LoopService::new(projects_root.clone());
        writer.start_session("project-resume", "Resumable project").expect("start");
        writer.stop_session().expect("stop").expect("session");
        let mut reader = LoopService::new(projects_root.clone());
        let recovered = RecoveryService::recover(&mut reader).expect("recover").expect("session");
        assert_eq!(recovered.action, RecoveryAction::Resume);
        assert!(!recovered.update.running);
        assert!(recovered.update.events.iter().any(|event| event.contains("ready to resume")));
        let _ = fs::remove_dir_all(projects_root);
    }

    #[test]
    fn recover_returns_none_without_persisted_session() {
        let projects_root = temp_projects_root();
        let mut service = LoopService::new(projects_root.clone());
        let recovered = RecoveryService::recover(&mut service).expect("recover");
        assert!(recovered.is_none());
        let _ = fs::remove_dir_all(projects_root);
    }

    #[test]
    fn recovery_action_surfaces_expected_home_labels() {
        assert!(RecoveryAction::Reattach.opens_monitor());
        assert!(!RecoveryAction::Resume.opens_monitor());
        assert_eq!(RecoveryAction::Reattach.home_action_label(), "Reattach active loop");
        assert_eq!(RecoveryAction::Resume.home_action_label(), "Resume loop session");
    }
}
