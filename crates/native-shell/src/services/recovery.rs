use std::io;

use super::loops::{LoopService, LoopUpdate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    Reattach,
    Resume,
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
        let action = if update.running { RecoveryAction::Reattach } else { RecoveryAction::Resume };
        let event_message = if action == RecoveryAction::Reattach {
            "Recovered active session and reattached monitor."
        } else {
            "Recovered paused session ready to resume."
        };
        update.events.push(event_message.to_owned());
        Ok(Some(RecoveredSession { action, update }))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{RecoveryAction, RecoveryService};
    use crate::loops_service::LoopService;

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
        let _ = fs::remove_dir_all(projects_root);
    }
}
