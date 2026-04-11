use std::io;
use std::path::PathBuf;
#[path = "screens/mod.rs"]
mod screens;
#[path = "screens/monitor.rs"]
mod monitor_screen;
#[path = "services/projects.rs"]
mod projects_service;
#[path = "theme/mod.rs"]
pub mod theme;
#[path = "view_models/home.rs"]
mod home_view_model;
use home_view_model::{
    HomeAction, HomeProjectSummary, HomeSessionSummary, HomeViewModel, ProjectStatus, SessionState,
};
use projects_service::{ProjectLifecycle, ProjectsService};
use screens::{HomeScreen, ProjectWizardScreen, ProjectWizardState};
use monitor_screen::MonitorScreen;
use theme::{ThemeName, ThemeStore};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId {
    Dashboard,
    Wizard,
    Monitor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenView {
    Dashboard(HomeScreen),
    Wizard(ProjectWizardScreen),
    Monitor(MonitorScreen),
}

#[derive(Debug, Clone)]
pub struct NativeShellApp {
    active_screen: ScreenId,
    theme: ThemeName,
    theme_store: ThemeStore,
    projects: ProjectsService,
    wizard_state: ProjectWizardState,
    home: HomeViewModel,
}

impl NativeShellApp {
    pub fn boot(theme_path: Option<PathBuf>) -> io::Result<Self> {
        Self::boot_with_paths(theme_path, None)
    }

    pub fn boot_with_paths(theme_path: Option<PathBuf>, projects_root: Option<PathBuf>) -> io::Result<Self> {
        let theme_store = ThemeStore::new(theme_path.unwrap_or_else(ThemeStore::default_path));
        let projects = ProjectsService::new(projects_root);
        let mut app = Self {
            active_screen: ScreenId::Dashboard,
            theme: theme_store.load()?,
            theme_store,
            projects,
            wizard_state: ProjectWizardState::idle(),
            home: Self::build_home(Vec::new()),
        };
        app.refresh_home()?;
        Ok(app)
    }

    pub fn set_screen(&mut self, screen: ScreenId) {
        self.active_screen = screen;
    }

    pub fn select_theme(&mut self, theme: ThemeName) -> io::Result<()> {
        self.theme_store.save(theme)?;
        self.theme = theme;
        Ok(())
    }

    pub fn start_project_wizard(&mut self, project_name: &str, objective: &str) -> io::Result<String> {
        let record = self.projects.create_project(project_name, objective)?;
        self.wizard_state = ProjectWizardState::completed(
            record.id.clone(),
            record.name.clone(),
            objective.trim().to_owned(),
        );
        self.refresh_home()?;
        Ok(record.id)
    }

    pub fn resume_project(&mut self, project_id: &str) -> io::Result<()> {
        self.projects.resume_project(project_id)?;
        self.refresh_home()
    }

    pub fn archive_project(&mut self, project_id: &str) -> io::Result<()> {
        self.projects.archive_project(project_id)?;
        self.refresh_home()
    }

    pub fn home(&self) -> &HomeViewModel {
        &self.home
    }

    pub fn projects_root(&self) -> PathBuf {
        self.projects.root().to_path_buf()
    }

    fn refresh_home(&mut self) -> io::Result<()> {
        self.home = Self::build_home(self.projects.list_projects(false)?);
        Ok(())
    }

    fn build_home(projects: Vec<projects_service::ProjectRecord>) -> HomeViewModel {
        let active_projects = projects
            .iter()
            .map(|project| HomeProjectSummary {
                id: project.id.clone(),
                name: project.name.clone(),
                status: match project.lifecycle {
                    ProjectLifecycle::Active => ProjectStatus::Running,
                    ProjectLifecycle::Archived => ProjectStatus::Idle,
                },
                latest_session: SessionState::Healthy,
            })
            .collect::<Vec<_>>();
        let recent_sessions = projects
            .iter()
            .take(2)
            .map(|project| HomeSessionSummary {
                project_id: project.id.clone(),
                session_id: project.latest_session.clone(),
                status: SessionState::Healthy,
            })
            .collect::<Vec<_>>();
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
            ScreenId::Monitor => ScreenView::Monitor(MonitorScreen::themed(palette)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};
    use super::{NativeShellApp, ScreenId, ScreenView};
    use super::theme::ThemeName;

    fn temp_path(label: &str, extension: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
        std::env::temp_dir().join(format!("loopforge-shell-{}-{}-{}.{}", label, std::process::id(), stamp, extension))
    }

    #[test]
    fn applies_selected_theme_to_every_surface() {
        let theme_file = temp_path("theme", "txt");
        let projects_dir = temp_path("projects", "dir");
        let mut app = NativeShellApp::boot_with_paths(Some(theme_file.clone()), Some(projects_dir.clone())).expect("boot");
        app.select_theme(ThemeName::Dawn).expect("save");
        app.set_screen(ScreenId::Dashboard);
        let dashboard = match app.render() { ScreenView::Dashboard(screen) => screen, _ => panic!("dashboard") };
        app.set_screen(ScreenId::Wizard);
        let wizard = match app.render() { ScreenView::Wizard(screen) => screen, _ => panic!("wizard") };
        assert_eq!(dashboard.accent, wizard.accent);
        let _ = fs::remove_file(theme_file);
        let _ = fs::remove_dir_all(projects_dir);
    }

    #[test]
    fn project_wizard_persists_artifacts_and_lifecycle_updates_dashboard() {
        let theme_file = temp_path("theme", "txt");
        let projects_dir = temp_path("projects", "dir");
        let mut app = NativeShellApp::boot_with_paths(Some(theme_file.clone()), Some(projects_dir.clone())).expect("boot");
        let project_id = app.start_project_wizard("Native Shell", "Complete dashboard parity").expect("create");
        let project_dir = app.projects_root().join(&project_id);
        assert!(project_dir.join("draft.json").exists());
        assert!(project_dir.join("prd.json").exists());
        assert!(project_dir.join("config.json").exists());
        app.archive_project(&project_id).expect("archive");
        assert!(app.home().active_projects.is_empty());
        app.resume_project(&project_id).expect("resume");
        assert_eq!(app.home().active_projects.len(), 1);
        assert_eq!(app.home().active_projects[0].id, project_id);
        let _ = fs::remove_file(theme_file);
        let _ = fs::remove_dir_all(projects_dir);
    }
}
