#[derive(Clone, Debug, PartialEq, Eq)]
enum SessionPresence {
    None,
    Running(String),
    Paused(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopSessionService {
    session: SessionPresence,
}

impl LoopSessionService {
    pub fn with_session(session_id: impl Into<String>, paused: bool) -> Self {
        let session_id = session_id.into();
        let session = if paused {
            SessionPresence::Paused(session_id)
        } else {
            SessionPresence::Running(session_id)
        };
        Self { session }
    }

    pub fn set_session_presence(&mut self, session_id: Option<String>, paused: bool) {
        self.session = match session_id {
            Some(id) if paused => SessionPresence::Paused(id),
            Some(id) => SessionPresence::Running(id),
            None => SessionPresence::None,
        };
    }

    pub fn has_active_session(&self) -> bool {
        !matches!(self.session, SessionPresence::None)
    }

    pub fn can_pause(&self) -> bool {
        matches!(self.session, SessionPresence::Running(_))
    }

    pub fn can_resume(&self) -> bool {
        matches!(self.session, SessionPresence::Paused(_))
    }

    pub fn pause_session(&mut self) -> bool {
        match &self.session {
            SessionPresence::Running(session_id) => {
                self.session = SessionPresence::Paused(session_id.clone());
                true
            }
            SessionPresence::None | SessionPresence::Paused(_) => false,
        }
    }

    pub fn resume_session(&mut self) -> bool {
        match &self.session {
            SessionPresence::Paused(session_id) => {
                self.session = SessionPresence::Running(session_id.clone());
                true
            }
            SessionPresence::None | SessionPresence::Running(_) => false,
        }
    }
}

impl Default for LoopSessionService {
    fn default() -> Self {
        Self {
            session: SessionPresence::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LoopSessionService;

    #[test]
    fn supports_pause_and_resume_for_active_session() {
        let mut service = LoopSessionService::with_session("session-42", false);
        assert!(service.has_active_session());
        assert!(service.can_pause());
        assert!(service.pause_session());
        assert!(service.can_resume());
        assert!(service.resume_session());
        assert!(service.can_pause());
    }

    #[test]
    fn ignores_pause_resume_without_compatible_session_state() {
        let mut service = LoopSessionService::default();
        assert!(!service.pause_session());
        assert!(!service.resume_session());
        service.set_session_presence(Some("session-7".to_string()), true);
        assert!(!service.pause_session());
    }
}
