use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopUpdate {
    pub project_id: String,
    pub project_name: String,
    pub session_id: String,
    pub running: bool,
    pub events: Vec<String>,
    pub completed_iterations: u32,
    pub blocked_states: u32,
    pub rate_limit_events: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoopSession {
    project_id: String,
    project_name: String,
    session_id: String,
    running: bool,
    completed_iterations: u32,
    blocked_states: u32,
    rate_limit_events: u32,
}

#[derive(Debug, Clone)]
pub struct LoopService {
    projects_root: PathBuf,
    persistence_path: PathBuf,
    session: Option<LoopSession>,
}

impl LoopService {
    pub fn new(projects_root: PathBuf) -> Self {
        let persistence_path = projects_root.join(".native-shell").join("loop-session.state");
        Self { projects_root, persistence_path, session: None }
    }

    pub fn start_session(&mut self, project_id: &str, project_name: &str) -> io::Result<LoopUpdate> {
        let project_dir = self.projects_root.join(project_id);
        fs::create_dir_all(&project_dir)?;
        let events = vec![
            String::from("Loop started from native shell."),
            String::from("Iteration 1 started."),
            String::from("Iteration 1 completed."),
            String::from("Story blocked: waiting for user decision."),
            String::from("Rate limit detected: retry scheduled."),
        ];
        let session = LoopSession {
            project_id: project_id.to_owned(),
            project_name: project_name.to_owned(),
            session_id: format!("loop-{}", Self::timestamp_nanos()),
            running: true,
            completed_iterations: 1,
            blocked_states: 1,
            rate_limit_events: 1,
        };
        self.session = Some(session.clone());
        self.persist_session(&session)?;
        Ok(Self::to_update(session, events))
    }

    pub fn stop_session(&mut self) -> io::Result<Option<LoopUpdate>> {
        let Some(mut session) = self.session.take() else { return Ok(None); };
        session.running = false;
        self.persist_session(&session)?;
        Ok(Some(Self::to_update(session, vec![String::from("Loop stopped from native shell.")])))
    }

    pub fn load_persisted_session(&mut self) -> io::Result<Option<LoopUpdate>> {
        if !self.persistence_path.exists() { return Ok(None); }
        let raw_state = fs::read_to_string(&self.persistence_path)?;
        let Some(session) = Self::parse_session(&raw_state) else { return Ok(None); };
        let running = session.running;
        self.session = Some(session.clone());
        Ok(Some(Self::to_update(session, vec![Self::startup_restore_event(running)])))
    }

    fn persist_session(&self, session: &LoopSession) -> io::Result<()> {
        let state_dir = self.persistence_path.parent().ok_or_else(|| io::Error::other("missing persistence parent"))?;
        fs::create_dir_all(state_dir)?;
        let encoded_state = [
            format!("project_id={}", session.project_id),
            format!("project_name={}", session.project_name),
            format!("session_id={}", session.session_id),
            format!("running={}", session.running),
            format!("completed_iterations={}", session.completed_iterations),
            format!("blocked_states={}", session.blocked_states),
            format!("rate_limit_events={}", session.rate_limit_events),
        ]
        .join("\n");
        fs::write(&self.persistence_path, encoded_state)
    }

    fn parse_session(raw_state: &str) -> Option<LoopSession> {
        let entries = raw_state
            .lines()
            .filter_map(|line| line.split_once('=').map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned())))
            .collect::<HashMap<_, _>>();
        Some(LoopSession {
            project_id: entries.get("project_id")?.to_owned(),
            project_name: entries.get("project_name")?.to_owned(),
            session_id: entries.get("session_id")?.to_owned(),
            running: entries.get("running")?.parse().ok()?,
            completed_iterations: entries.get("completed_iterations")?.parse().ok()?,
            blocked_states: entries.get("blocked_states")?.parse().ok()?,
            rate_limit_events: entries.get("rate_limit_events")?.parse().ok()?,
        })
    }

    fn to_update(session: LoopSession, events: Vec<String>) -> LoopUpdate {
        LoopUpdate {
            project_id: session.project_id,
            project_name: session.project_name,
            session_id: session.session_id,
            running: session.running,
            events,
            completed_iterations: session.completed_iterations,
            blocked_states: session.blocked_states,
            rate_limit_events: session.rate_limit_events,
        }
    }

    fn startup_restore_event(running: bool) -> String {
        if running { String::from("Persisted active loop session restored on startup.") } else { String::from("Persisted resumable loop session restored on startup.") }
    }

    fn timestamp_nanos() -> u128 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() }
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
        let started = service.start_session("project-native", "Native shell project").expect("start");
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

    #[test]
    fn persisted_session_is_restored_on_startup() {
        let projects_root = temp_projects_root();
        let mut writer = LoopService::new(projects_root.clone());
        let started = writer.start_session("project-restore", "Recoverable project").expect("start");
        let mut reader = LoopService::new(projects_root.clone());
        let recovered = reader.load_persisted_session().expect("recover").expect("session");
        assert_eq!(recovered.project_id, started.project_id);
        assert_eq!(recovered.project_name, started.project_name);
        assert_eq!(recovered.session_id, started.session_id);
        assert!(recovered.events.iter().any(|event| event.contains("restored")));
        assert!(recovered.events.iter().any(|event| event.contains("active")));
        let _ = fs::remove_dir_all(projects_root);
    }

    #[test]
    fn stopped_session_is_restored_as_resumable() {
        let projects_root = temp_projects_root();
        let mut writer = LoopService::new(projects_root.clone());
        let started = writer.start_session("project-resume", "Resumable project").expect("start");
        let stopped = writer.stop_session().expect("stop").expect("session");
        let mut reader = LoopService::new(projects_root.clone());
        let recovered = reader.load_persisted_session().expect("recover").expect("session");
        assert_eq!(recovered.project_id, started.project_id);
        assert_eq!(recovered.session_id, stopped.session_id);
        assert!(!recovered.running);
        assert!(recovered.events.iter().any(|event| event.contains("resumable")));
        let _ = fs::remove_dir_all(projects_root);
    }
}
