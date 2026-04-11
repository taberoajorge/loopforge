use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningSession {
    pub project_id: String,
    pub active: bool,
    pub activity_lines: Vec<String>,
    pub plan_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PlanningService {
    projects_root: PathBuf,
    session: Option<PlanningSession>,
}

impl PlanningService {
    pub fn new(projects_root: PathBuf) -> Self {
        Self {
            projects_root,
            session: None,
        }
    }

    pub fn start_session(&mut self, project_id: &str, objective: &str) -> io::Result<Vec<String>> {
        let project_dir = self.projects_root.join(project_id);
        fs::create_dir_all(&project_dir)?;
        let plan_path = project_dir.join("plan.md");
        let activity_lines = Self::build_start_activity(project_id, objective);
        fs::write(&plan_path, Self::render_plan(&activity_lines))?;
        self.session = Some(PlanningSession {
            project_id: project_id.to_owned(),
            active: true,
            activity_lines: activity_lines.clone(),
            plan_path,
        });
        Ok(activity_lines)
    }

    pub fn send_input(&mut self, input: &str) -> io::Result<Vec<String>> {
        let response_lines = Self::build_follow_up_activity(input);
        let Some(session) = self.session.as_mut() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "planning session is not active",
            ));
        };
        session.activity_lines.extend(response_lines.clone());
        fs::write(&session.plan_path, Self::render_plan(&session.activity_lines))?;
        Ok(response_lines)
    }

    pub fn stop_session(&mut self) -> io::Result<Option<PlanningSession>> {
        let Some(mut session) = self.session.take() else {
            return Ok(None);
        };
        session.active = false;
        session
            .activity_lines
            .push(String::from("Planning session stopped from native shell."));
        fs::write(&session.plan_path, Self::render_plan(&session.activity_lines))?;
        Ok(Some(session))
    }

    fn build_start_activity(project_id: &str, objective: &str) -> Vec<String> {
        let objective_line = if objective.trim().is_empty() {
            String::from("Objective: Clarify project goals before execution.")
        } else {
            format!("Objective: {}", objective.trim())
        };
        vec![
            format!("Planning session started for {}.", project_id),
            objective_line,
            String::from("Drafting initial implementation sequence."),
        ]
    }

    fn build_follow_up_activity(input: &str) -> Vec<String> {
        let follow_up = if input.trim().is_empty() {
            String::from("Follow-up request received with empty body.")
        } else {
            format!("Follow-up request: {}", input.trim())
        };
        vec![follow_up, String::from("Plan outline updated with follow-up input.")]
    }

    fn render_plan(activity_lines: &[String]) -> String {
        let mut plan = String::from("# Native Shell Plan\n\n");
        for activity_line in activity_lines {
            plan.push_str("- ");
            plan.push_str(activity_line);
            plan.push('\n');
        }
        plan
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::PlanningService;

    fn temp_projects_root() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("loopforge-shell-planning-{}-{}", std::process::id(), stamp))
    }

    #[test]
    fn start_send_and_stop_persist_plan_output() {
        let projects_root = temp_projects_root();
        let mut service = PlanningService::new(projects_root.clone());
        let started = service.start_session("project-native", "Ship planning controls").expect("start");
        assert!(!started.is_empty());
        let follow_up = service.send_input("Include stop flow in acceptance").expect("send");
        assert!(!follow_up.is_empty());
        let stopped = service.stop_session().expect("stop").expect("session");
        assert!(!stopped.active);
        assert!(service.stop_session().expect("stop twice").is_none());
        let plan_content = fs::read_to_string(projects_root.join("project-native").join("plan.md")).expect("plan");
        assert!(plan_content.contains("Follow-up request: Include stop flow in acceptance"));
        assert!(plan_content.contains("Planning session stopped from native shell."));
        let _ = fs::remove_dir_all(projects_root);
    }
}
