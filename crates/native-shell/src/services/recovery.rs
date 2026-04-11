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
