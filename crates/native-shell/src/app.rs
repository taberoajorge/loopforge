use std::fs;
use std::io;
use std::path::PathBuf;
#[path = "screens/mod.rs"]
mod screens;
#[path = "services/planning.rs"]
mod planning_service;
#[path = "services/atomization.rs"]
mod atomization_service;
#[path = "services/projects.rs"]
mod projects_service;
#[path = "services/loops.rs"]
mod loops_service;
#[path = "services/recovery.rs"]
mod recovery_service;
#[path = "theme/mod.rs"]
pub mod theme;
#[path = "view_models/home.rs"]
mod home_view_model;
use atomization_service::AtomizationService;
use home_view_model::{HomeAction, HomeProjectSummary, HomeSessionSummary, HomeViewModel, ProjectStatus, SessionState};
use loops_service::LoopService;
use planning_service::PlanningService;
use projects_service::{ProjectLifecycle, ProjectsService};
use recovery_service::{RecoveryAction, RecoveryService};
use screens::{AtomizationArtifact, AtomizationScreen, AtomizationStageUpdate, AtomizationState, HomeScreen, MonitorScreen, MonitorState, PlanningScreen, PlanningState, ProjectWizardScreen, ProjectWizardState};
use theme::{ThemeName, ThemeStore};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId { Dashboard, Wizard, Planning, Atomization, Monitor }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenView {
    Dashboard(HomeScreen),
    Wizard(ProjectWizardScreen),
    Planning(PlanningScreen),
    Atomization(AtomizationScreen),
    Monitor(MonitorScreen),
}
#[derive(Debug, Clone)]
pub struct NativeShellApp {
    active_screen: ScreenId,
    theme: ThemeName,
    theme_store: ThemeStore,
    projects: ProjectsService,
    planning_service: PlanningService,
    atomization_service: AtomizationService,
    loop_service: LoopService,
    wizard_state: ProjectWizardState,
    planning_state: PlanningState,
    atomization_state: AtomizationState,
    monitor_state: MonitorState,
    home: HomeViewModel,
}
impl NativeShellApp {
    pub fn boot(theme_path: Option<PathBuf>) -> io::Result<Self> { Self::boot_with_paths(theme_path, None) }
    pub fn boot_with_paths(theme_path: Option<PathBuf>, projects_root: Option<PathBuf>) -> io::Result<Self> {
        let theme_store = ThemeStore::new(theme_path.unwrap_or_else(ThemeStore::default_path));
        let projects = ProjectsService::new(projects_root);
        let planning_service = PlanningService::new(projects.root().to_path_buf());
        let atomization_service = AtomizationService::new(projects.root().to_path_buf());
        let loop_service = LoopService::new(projects.root().to_path_buf());
        let mut app = Self {
            active_screen: ScreenId::Dashboard,
            theme: theme_store.load()?,
            theme_store,
            projects,
            planning_service,
            atomization_service,
            loop_service,
            wizard_state: ProjectWizardState::idle(),
            planning_state: PlanningState::idle(),
            atomization_state: AtomizationState::idle(),
            monitor_state: MonitorState::idle(),
            home: Self::build_home(Vec::new()),
        };
        app.refresh_home()?;
        app.restore_persisted_loop_session()?;
        Ok(app)
    }
    pub fn set_screen(&mut self, screen: ScreenId) { self.active_screen = screen; }
    pub fn select_theme(&mut self, theme: ThemeName) -> io::Result<()> { self.theme_store.save(theme)?; self.theme = theme; Ok(()) }
    pub fn start_project_wizard(&mut self, project_name: &str, objective: &str) -> io::Result<String> {
        let record = self.projects.create_project(project_name, objective)?;
        self.wizard_state = ProjectWizardState::completed(record.id.clone(), record.name.clone(), objective.trim().to_owned());
        self.refresh_home()?;
        Ok(record.id)
    }
    pub fn start_planning_session(&mut self, project_id: &str, objective: &str) -> io::Result<()> {
        let project_name = self.home.active_projects.iter().find(|project| project.id == project_id).map(|project| project.name.clone()).unwrap_or_else(|| project_id.to_owned());
        self.planning_state = PlanningState::start(project_id.to_owned(), project_name, objective.trim().to_owned());
        match self.planning_service.start_session(project_id, objective) { Ok(activity_batch) => { for activity in activity_batch { self.planning_state = self.planning_state.clone().push_activity(activity); } self.active_screen = ScreenId::Planning; Ok(()) } Err(error) => { self.planning_state = self.planning_state.clone().fail(error.to_string()); Err(error) } }
    }
    pub fn send_planning_input(&mut self, input: &str) -> io::Result<()> {
        match self.planning_service.send_input(input) { Ok(activity_batch) => { for activity in activity_batch { self.planning_state = self.planning_state.clone().push_activity(activity); } Ok(()) } Err(error) => { self.planning_state = self.planning_state.clone().fail(error.to_string()); Err(error) } }
    }
    pub fn stop_planning_session(&mut self) -> io::Result<()> {
        let session = self.planning_service.stop_session()?;
        let plan_path = session.map(|planning_session| planning_session.plan_path.display().to_string());
        self.planning_state = self.planning_state.clone().stop(plan_path);
        Ok(())
    }
    pub fn start_atomization_session(&mut self, project_id: &str) -> io::Result<()> {
        let project_name = self.home.active_projects.iter().find(|project| project.id == project_id).map(|project| project.name.clone()).unwrap_or_else(|| project_id.to_owned());
        self.atomization_state = AtomizationState::start(project_id.to_owned(), project_name);
        match self.atomization_service.run_pipeline(project_id) { Ok(run) => { for progress in run.progress { self.atomization_state = self.atomization_state.clone().push_stage(AtomizationStageUpdate::new(progress.stage.label(), progress.detail)); } self.atomization_state = self.atomization_state.clone().complete(run.artifacts.prd_path.display().to_string(), run.artifacts.prompt_path.display().to_string(), run.artifacts.guardrails_path.display().to_string()); self.active_screen = ScreenId::Atomization; Ok(()) } Err(error) => { self.atomization_state = self.atomization_state.clone().fail(error.to_string()); Err(error) } }
    }
    pub fn open_atomization_artifact(&self, artifact: AtomizationArtifact) -> io::Result<String> {
        let Some(path) = self.atomization_state.artifact_path(artifact) else { return Err(io::Error::new(io::ErrorKind::NotFound, "atomization artifact is unavailable")); };
        fs::read_to_string(path)
    }
    pub fn start_loop_session(&mut self, project_id: &str) -> io::Result<()> {
        let project_name = self.home.active_projects.iter().find(|project| project.id == project_id).map(|project| project.name.clone()).unwrap_or_else(|| project_id.to_owned());
        self.monitor_state = MonitorState::start(project_id.to_owned(), project_name.clone());
        match self.loop_service.start_session(project_id, &project_name) {
            Ok(update) => {
                self.apply_loop_update(&update);
                self.sync_home_with_recovered_session(&update, RecoveryAction::Reattach);
                self.active_screen = ScreenId::Monitor;
                Ok(())
            }
            Err(error) => { self.monitor_state = self.monitor_state.clone().fail(error.to_string()); Err(error) }
        }
    }
    pub fn stop_loop_session(&mut self) -> io::Result<()> {
        let update = self.loop_service.stop_session()?;
        match update { Some(update) => { self.apply_loop_update(&update); self.sync_home_with_recovered_session(&update, RecoveryAction::Resume); } None => { self.monitor_state = self.monitor_state.clone().stop(); } }
        Ok(())
    }
    pub fn resume_project(&mut self, project_id: &str) -> io::Result<()> { self.projects.resume_project(project_id)?; self.refresh_home() }
    pub fn archive_project(&mut self, project_id: &str) -> io::Result<()> { self.projects.archive_project(project_id)?; self.refresh_home() }
    pub fn home(&self) -> &HomeViewModel { &self.home }
    pub fn projects_root(&self) -> PathBuf { self.projects.root().to_path_buf() }
    fn refresh_home(&mut self) -> io::Result<()> { self.home = Self::build_home(self.projects.list_projects(false)?); Ok(()) }
    fn restore_persisted_loop_session(&mut self) -> io::Result<()> {
        if let Some(recovered_session) = RecoveryService::recover(&mut self.loop_service)? {
            self.apply_loop_update(&recovered_session.update);
            self.sync_home_with_recovered_session(&recovered_session.update, recovered_session.action);
            self.active_screen = Self::screen_for_recovery_action(recovered_session.action);
        }
        Ok(())
    }
    fn screen_for_recovery_action(action: RecoveryAction) -> ScreenId { if action.opens_monitor() { ScreenId::Monitor } else { ScreenId::Dashboard } }
    fn apply_loop_update(&mut self, update: &loops_service::LoopUpdate) {
        self.monitor_state = MonitorState::start(update.project_id.clone(), update.project_name.clone()).apply_update(update.session_id.clone(), update.running, update.events.clone(), update.completed_iterations, update.blocked_states, update.rate_limit_events);
    }
    fn sync_home_with_recovered_session(&mut self, update: &loops_service::LoopUpdate, action: RecoveryAction) {
        let session_state = if update.blocked_states > 0 { SessionState::Blocked } else { SessionState::Healthy };
        let project_status = if update.running { ProjectStatus::Running } else { ProjectStatus::Idle };
        if let Some(project) = self.home.active_projects.iter_mut().find(|project| project.id == update.project_id) {
            project.status = project_status;
            project.latest_session = session_state.clone();
        } else {
            self.home.active_projects.push(HomeProjectSummary { id: update.project_id.clone(), name: update.project_name.clone(), status: project_status, latest_session: session_state.clone() });
        }
        self.home.recent_sessions.retain(|session| session.session_id != update.session_id);
        self.home.recent_sessions.insert(0, HomeSessionSummary { project_id: update.project_id.clone(), session_id: update.session_id.clone(), status: session_state });
        let action_label = action.home_action_label();
        if let Some(home_action) = self.home.primary_actions.iter_mut().find(|home_action| home_action.id == "open-monitor") { home_action.label = action_label.to_owned(); } else { self.home.primary_actions.push(HomeAction { id: String::from("open-monitor"), label: action_label.to_owned() }); }
    }
    fn build_home(projects: Vec<projects_service::ProjectRecord>) -> HomeViewModel {
        let active_projects = projects.iter().map(|project| HomeProjectSummary { id: project.id.clone(), name: project.name.clone(), status: if project.lifecycle == ProjectLifecycle::Active { ProjectStatus::Running } else { ProjectStatus::Idle }, latest_session: SessionState::Healthy }).collect::<Vec<_>>();
        let recent_sessions = projects.iter().take(2).map(|project| HomeSessionSummary { project_id: project.id.clone(), session_id: project.latest_session.clone(), status: SessionState::Healthy }).collect::<Vec<_>>();
        HomeViewModel { heading: String::from("INITIALIZE SEQUENCE"), strapline: String::from("Autonomous AI loop orchestrator."), active_projects, recent_sessions, primary_actions: vec![HomeAction { id: String::from("start-project"), label: String::from("Start new project") }, HomeAction { id: String::from("resume-project"), label: String::from("Resume project") }, HomeAction { id: String::from("archive-project"), label: String::from("Archive project") }, HomeAction { id: String::from("open-monitor"), label: String::from("Open monitor") }] }
    }
    pub fn render(&self) -> ScreenView {
        let palette = self.theme.palette();
        match self.active_screen {
            ScreenId::Dashboard => ScreenView::Dashboard(HomeScreen::themed(palette, self.home.clone())),
            ScreenId::Wizard => ScreenView::Wizard(ProjectWizardScreen::themed(palette, self.wizard_state.clone())),
            ScreenId::Planning => ScreenView::Planning(PlanningScreen::themed(palette, self.planning_state.clone())),
            ScreenId::Atomization => ScreenView::Atomization(AtomizationScreen::themed(palette, self.atomization_state.clone())),
            ScreenId::Monitor => ScreenView::Monitor(MonitorScreen::themed(palette, self.monitor_state.clone())),
        }
    }
}
