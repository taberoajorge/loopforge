use super::backend_adapter::BackendAdapter;
use loopforge_app_core::plan::{PlanSessionEvent, PlanSessionHandle, PlanSessionStatus};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PlanningService {
    projects_root: PathBuf,
    plan_path: Option<PathBuf>,
    session: Option<PlanSessionHandle>,
    events: Vec<PlanSessionEvent>,
}

impl PlanningService {
    pub fn new(projects_root: PathBuf) -> Self {
        Self {
            projects_root,
            plan_path: None,
            session: None,
            events: Vec::new(),
        }
    }

    pub fn start_session(
        &mut self,
        project_id: &str,
        objective: &str,
    ) -> io::Result<Vec<PlanSessionEvent>> {
        let project_id = project_id.to_owned();
        let plan_path = self.projects_root.join(&project_id).join("plan.md");
        let handle = BackendAdapter::open_plan_session(project_id.clone(), |resolved_id| {
            fs::create_dir_all(self.projects_root.join(&resolved_id))?;
            Ok::<_, io::Error>(PlanSessionHandle {
                project_id: resolved_id,
                status: PlanSessionStatus::Running,
            })
        })?;
        let mut events = vec![PlanSessionEvent::Started(handle.clone())];
        for activity in Self::build_start_activity(objective) {
            let event = BackendAdapter::write_plan_input(project_id.clone(), activity, |_, _| {
                Ok::<_, io::Error>(())
            })?;
            events.push(event);
        }
        self.plan_path = Some(plan_path);
        self.session = Some(handle);
        self.events = events.clone();
        self.persist_plan()?;
        Ok(events)
    }

    pub fn send_input(&mut self, input: &str) -> io::Result<PlanSessionEvent> {
        let project_id = self
            .session
            .as_ref()
            .map(|handle| handle.project_id.clone())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "planning session is not active")
            })?;
        let event = BackendAdapter::write_plan_input(
            project_id,
            Self::build_follow_up_activity(input),
            |_, _| Ok::<_, io::Error>(()),
        )?;
        self.events.push(event.clone());
        self.persist_plan()?;
        Ok(event)
    }

    pub fn stop_session(&mut self) -> io::Result<Option<PlanSessionHandle>> {
        let Some(handle) = self.session.take() else {
            return Ok(None);
        };
        let stopped = BackendAdapter::stop_plan_session(handle.project_id.clone(), |_| {
            Ok::<_, io::Error>(())
        })?;
        self.events.push(stopped);
        self.persist_plan()?;
        let stopped_handle = PlanSessionHandle {
            project_id: handle.project_id,
            status: PlanSessionStatus::Stopped,
        };
        self.session = None;
        Ok(Some(stopped_handle))
    }

    fn build_start_activity(objective: &str) -> Vec<String> {
        let objective_line = if objective.trim().is_empty() {
            String::from("Objective: Clarify project goals before execution.")
        } else {
            format!("Objective: {}", objective.trim())
        };
        vec![
            objective_line,
            String::from("Drafting initial implementation sequence."),
        ]
    }

    fn build_follow_up_activity(input: &str) -> String {
        if input.trim().is_empty() {
            String::from("Follow-up request received with empty body.")
        } else {
            format!("Follow-up request: {}", input.trim())
        }
    }

    fn persist_plan(&self) -> io::Result<()> {
        let Some(plan_path) = self.plan_path.as_ref() else {
            return Ok(());
        };
        fs::write(plan_path, Self::render_plan(&self.events))
    }

    fn render_plan(events: &[PlanSessionEvent]) -> String {
        let mut plan = String::from("# Native Shell Plan\n\n");
        for event in events {
            match event {
                PlanSessionEvent::Started(handle) => {
                    plan.push_str("- Planning session started for ");
                    plan.push_str(&handle.project_id);
                    plan.push_str(".\n");
                }
                PlanSessionEvent::Output { chunk, .. } => {
                    plan.push_str("- ");
                    plan.push_str(chunk);
                    plan.push('\n');
                }
                PlanSessionEvent::Stopped { .. } => {
                    plan.push_str("- Planning session stopped from native shell.\n");
                }
            }
        }
        plan
    }
}

#[cfg(test)]
mod tests {
    use super::PlanningService;
    use loopforge_app_core::plan::{PlanSessionEvent, PlanSessionStatus};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_projects_root() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "loopforge-shell-planning-{}-{}",
            std::process::id(),
            stamp
        ))
    }

    #[test]
    fn planning_service_uses_shared_plan_contracts() {
        let projects_root = temp_projects_root();
        let mut service = PlanningService::new(projects_root.clone());
        let started = service
            .start_session("project-native", "Ship planning controls")
            .expect("start");
        let follow_up = service
            .send_input("Include stop flow in acceptance")
            .expect("send");
        let stopped = service.stop_session().expect("stop").expect("session");

        assert!(matches!(
            started.first(),
            Some(PlanSessionEvent::Started(handle))
            if handle.project_id == "project-native" && handle.status == PlanSessionStatus::Running
        ));
        assert!(matches!(
            follow_up,
            PlanSessionEvent::Output { project_id, chunk }
            if project_id == "project-native" && chunk == "Follow-up request: Include stop flow in acceptance"
        ));
        assert_eq!(stopped.status, PlanSessionStatus::Stopped);
        assert!(service.stop_session().expect("stop twice").is_none());

        let plan_content =
            fs::read_to_string(projects_root.join("project-native").join("plan.md")).expect("plan");
        assert!(plan_content.contains("Objective: Ship planning controls"));
        assert!(plan_content.contains("Follow-up request: Include stop flow in acceptance"));
        assert!(plan_content.contains("Planning session stopped from native shell."));
        let _ = fs::remove_dir_all(projects_root);
    }
}
