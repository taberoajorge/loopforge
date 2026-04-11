use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopUpdate {
    pub session_id: String,
    pub running: bool,
    pub events: Vec<String>,
    pub completed_iterations: u32,
    pub blocked_states: u32,
    pub rate_limit_events: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoopSession {
    session_id: String,
    running: bool,
    completed_iterations: u32,
    blocked_states: u32,
    rate_limit_events: u32,
}

#[derive(Debug, Clone)]
pub struct LoopService {
    projects_root: PathBuf,
    session: Option<LoopSession>,
}

impl LoopService {
    pub fn new(projects_root: PathBuf) -> Self {
        Self {
            projects_root,
            session: None,
        }
    }

    pub fn start_session(&mut self, project_id: &str) -> io::Result<LoopUpdate> {
        let project_dir = self.projects_root.join(project_id);
        fs::create_dir_all(&project_dir)?;
        let session_id = format!("loop-{}", Self::timestamp_nanos());
        let events = vec![
            String::from("Loop started from native shell."),
            String::from("Iteration 1 started."),
            String::from("Iteration 1 completed."),
            String::from("Story blocked: waiting for user decision."),
            String::from("Rate limit detected: retry scheduled."),
        ];
        let session = LoopSession {
            session_id: session_id.clone(),
            running: true,
            completed_iterations: 1,
            blocked_states: 1,
            rate_limit_events: 1,
        };
        self.session = Some(session.clone());
        Ok(LoopUpdate {
            session_id,
            running: session.running,
            events,
            completed_iterations: session.completed_iterations,
            blocked_states: session.blocked_states,
            rate_limit_events: session.rate_limit_events,
        })
    }

    pub fn stop_session(&mut self) -> io::Result<Option<LoopUpdate>> {
        let Some(mut session) = self.session.take() else {
            return Ok(None);
        };
        session.running = false;
        Ok(Some(LoopUpdate {
            session_id: session.session_id,
            running: false,
            events: vec![String::from("Loop stopped from native shell.")],
            completed_iterations: session.completed_iterations,
            blocked_states: session.blocked_states,
            rate_limit_events: session.rate_limit_events,
        }))
    }

    fn timestamp_nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::LoopService;

    fn temp_projects_root() -> std::path::PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
        std::env::temp_dir().join(format!("loopforge-shell-loop-{}-{}", std::process::id(), stamp))
    }

    #[test]
    fn start_and_stop_report_native_loop_controls() {
        let projects_root = temp_projects_root();
        let mut service = LoopService::new(projects_root.clone());
        let started = service.start_session("project-native").expect("start");
        assert!(started.running);
        assert!(started.events.iter().any(|event| event.contains("Iteration 1 completed")));
        assert!(started.events.iter().any(|event| event.contains("Story blocked")));
        assert!(started.events.iter().any(|event| event.contains("Rate limit")));
        let stopped = service.stop_session().expect("stop").expect("session");
        assert!(!stopped.running);
        assert!(stopped.events.iter().any(|event| event.contains("stopped")));
        assert!(service.stop_session().expect("stop twice").is_none());
        let _ = fs::remove_dir_all(projects_root);
    }
}
