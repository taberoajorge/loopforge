use std::io;
use std::path::PathBuf;
#[path = "screens/mod.rs"]
mod screens;
#[path = "screens/monitor.rs"]
mod monitor_screen;
#[path = "services/planning.rs"]
mod planning_service;
#[path = "services/projects.rs"]
mod projects_service;
#[path = "theme/mod.rs"]
pub mod theme;
#[path = "view_models/home.rs"]
mod home_view_model;
use home_view_model::{HomeAction, HomeProjectSummary, HomeSessionSummary, HomeViewModel, ProjectStatus, SessionState};
use monitor_screen::MonitorScreen;
use planning_service::PlanningService;
use projects_service::{ProjectLifecycle, ProjectsService};
use screens::{HomeScreen, PlanningScreen, PlanningState, ProjectWizardScreen, ProjectWizardState};
use theme::{ThemeName, ThemeStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId { Dashboard, Wizard, Planning, Monitor }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenView {
    Dashboard(HomeScreen),
    Wizard(ProjectWizardScreen),
    Planning(PlanningScreen),
    Monitor(MonitorScreen),
}

#[derive(Debug, Clone)]
pub struct NativeShellApp {
    active_screen: ScreenId,
    theme: ThemeName,
    theme_store: ThemeStore,
    projects: ProjectsService,
    planning_service: PlanningService,
    wizard_state: ProjectWizardState,
    planning_state: PlanningState,
    home: HomeViewModel,
}

impl NativeShellApp {
    pub fn boot(theme_path: Option<PathBuf>) -> io::Result<Self> { Self::boot_with_paths(theme_path, None) }

    pub fn boot_with_paths(theme_path: Option<PathBuf>, projects_root: Option<PathBuf>) -> io::Result<Self> {
        let theme_store = ThemeStore::new(theme_path.unwrap_or_else(ThemeStore::default_path));
        let projects = ProjectsService::new(projects_root);
        let planning_service = PlanningService::new(projects.root().to_path_buf());
        let mut app = Self {
            active_screen: ScreenId::Dashboard,
            theme: theme_store.load()?,
            theme_store,
            projects,
            planning_service,
            wizard_state: ProjectWizardState::idle(),
            planning_state: PlanningState::idle(),
            home: Self::build_home(Vec::new()),
        };
        app.refresh_home()?;
        Ok(app)
    }

    pub fn set_screen(&mut self, screen: ScreenId) { self.active_screen = screen; }

    pub fn select_theme(&mut self, theme: ThemeName) -> io::Result<()> {
        self.theme_store.save(theme)?;
        self.theme = theme;
        Ok(())
    }

    pub fn start_project_wizard(&mut self, project_name: &str, objective: &str) -> io::Result<String> {
        let record = self.projects.create_project(project_name, objective)?;
        self.wizard_state = ProjectWizardState::completed(record.id.clone(), record.name.clone(), objective.trim().to_owned());
        self.refresh_home()?;
        Ok(record.id)
    }

    pub fn start_planning_session(&mut self, project_id: &str, objective: &str) -> io::Result<()> {
        let project_name = self.home.active_projects.iter().find(|project| project.id == project_id).map(|project| project.name.clone()).unwrap_or_else(|| project_id.to_owned());
        self.planning_state = PlanningState::start(project_id.to_owned(), project_name, objective.trim().to_owned());
        match self.planning_service.start_session(project_id, objective) {
            Ok(activity_batch) => {
                for activity in activity_batch { self.planning_state = self.planning_state.clone().push_activity(activity); }
                self.active_screen = ScreenId::Planning;
                Ok(())
            }
            Err(error) => { self.planning_state = self.planning_state.clone().fail(error.to_string()); Err(error) }
        }
    }

    pub fn send_planning_input(&mut self, input: &str) -> io::Result<()> {
        match self.planning_service.send_input(input) {
            Ok(activity_batch) => { for activity in activity_batch { self.planning_state = self.planning_state.clone().push_activity(activity); } Ok(()) }
            Err(error) => { self.planning_state = self.planning_state.clone().fail(error.to_string()); Err(error) }
        }
    }

    pub fn stop_planning_session(&mut self) -> io::Result<()> {
        let session = self.planning_service.stop_session()?;
        let plan_path = session.map(|planning_session| planning_session.plan_path.display().to_string());
        self.planning_state = self.planning_state.clone().stop(plan_path);
        Ok(())
    }

    pub fn resume_project(&mut self, project_id: &str) -> io::Result<()> { self.projects.resume_project(project_id)?; self.refresh_home() }

    pub fn archive_project(&mut self, project_id: &str) -> io::Result<()> { self.projects.archive_project(project_id)?; self.refresh_home() }

    pub fn home(&self) -> &HomeViewModel { &self.home }

    pub fn projects_root(&self) -> PathBuf { self.projects.root().to_path_buf() }

    fn refresh_home(&mut self) -> io::Result<()> { self.home = Self::build_home(self.projects.list_projects(false)?); Ok(()) }

    fn build_home(projects: Vec<projects_service::ProjectRecord>) -> HomeViewModel {
        let active_projects = projects.iter().map(|project| HomeProjectSummary {
            id: project.id.clone(),
            name: project.name.clone(),
            status: if project.lifecycle == ProjectLifecycle::Active { ProjectStatus::Running } else { ProjectStatus::Idle },
            latest_session: SessionState::Healthy,
        }).collect::<Vec<_>>();
        let recent_sessions = projects.iter().take(2).map(|project| HomeSessionSummary {
            project_id: project.id.clone(),
            session_id: project.latest_session.clone(),
            status: SessionState::Healthy,
        }).collect::<Vec<_>>();
        HomeViewModel {
            heading: String::from("INITIALIZE SEQUENCE"),
            strapline: String::from("Autonomous AI loop orchestrator."),
            active_projects,
            recent_sessions,
            primary_actions: vec![
                HomeAction { id: String::from("start-project"), label: String::from("Start new project") },
                HomeAction { id: String::from("resume-project"), label: String::from("Resume project") },
                HomeAction { id: String::from("archive-project"), label: String::from("Archive project") },
                HomeAction { id: String::from("open-monitor"), label: String::from("Open monitor") },
            ],
        }
    }

    pub fn render(&self) -> ScreenView {
        let palette = self.theme.palette();
        match self.active_screen {
            ScreenId::Dashboard => ScreenView::Dashboard(HomeScreen::themed(palette, self.home.clone())),
            ScreenId::Wizard => ScreenView::Wizard(ProjectWizardScreen::themed(palette, self.wizard_state.clone())),
            ScreenId::Planning => ScreenView::Planning(PlanningScreen::themed(palette, self.planning_state.clone())),
            ScreenId::Monitor => ScreenView::Monitor(MonitorScreen::themed(palette)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};
    use super::{NativeShellApp, ScreenId, ScreenView};

    fn temp_path(label: &str, extension: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
        std::env::temp_dir().join(format!("loopforge-shell-{}-{}-{}.{}", label, std::process::id(), stamp, extension))
    }

    #[test]
    fn planning_session_streams_activity_and_persists_plan_output() {
        let theme_file = temp_path("theme", "txt");
        let projects_dir = temp_path("projects", "dir");
        let mut app = NativeShellApp::boot_with_paths(Some(theme_file.clone()), Some(projects_dir.clone())).expect("boot");
        let project_id = app.start_project_wizard("Native Shell", "Plan inside shell").expect("create");
        app.start_planning_session(&project_id, "Build native planning controls").expect("start");
        app.send_planning_input("Add a stop flow and persist plan output").expect("input");
        app.stop_planning_session().expect("stop");
        app.set_screen(ScreenId::Planning);
        let planning_screen = match app.render() { ScreenView::Planning(screen) => screen, _ => panic!("planning") };
        assert!(!planning_screen.activity_lines.is_empty());
        assert!(!planning_screen.session_active);
        let plan_content = fs::read_to_string(app.projects_root().join(project_id).join("plan.md")).expect("plan");
        assert!(plan_content.contains("Follow-up request: Add a stop flow and persist plan output"));
        assert!(plan_content.contains("Planning session stopped from native shell."));
        let _ = fs::remove_file(theme_file);
        let _ = fs::remove_dir_all(projects_dir);
    }
}
