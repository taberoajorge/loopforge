use std::io;
use std::path::PathBuf;

#[path = "screens/home.rs"]
mod home_screen;
#[path = "screens/monitor.rs"]
mod monitor_screen;
#[path = "screens/project_wizard.rs"]
mod project_wizard_screen;
#[path = "theme/mod.rs"]
pub mod theme;

use home_screen::HomeScreen;
use monitor_screen::MonitorScreen;
use project_wizard_screen::ProjectWizardScreen;
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
}

impl NativeShellApp {
    pub fn boot(theme_path: Option<PathBuf>) -> io::Result<Self> {
        let theme_store = ThemeStore::new(theme_path.unwrap_or_else(ThemeStore::default_path));
        let theme = theme_store.load()?;
        Ok(Self {
            active_screen: ScreenId::Dashboard,
            theme,
            theme_store,
        })
    }

    pub fn set_screen(&mut self, screen: ScreenId) {
        self.active_screen = screen;
    }

    pub fn select_theme(&mut self, theme: ThemeName) -> io::Result<()> {
        self.theme_store.save(theme)?;
        self.theme = theme;
        Ok(())
    }

    pub fn theme(&self) -> ThemeName {
        self.theme
    }

    pub fn render(&self) -> ScreenView {
        let palette = self.theme.palette();
        match self.active_screen {
            ScreenId::Dashboard => ScreenView::Dashboard(HomeScreen::themed(palette)),
            ScreenId::Wizard => ScreenView::Wizard(ProjectWizardScreen::themed(palette)),
            ScreenId::Monitor => ScreenView::Monitor(MonitorScreen::themed(palette)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::theme::ThemeName;
    use super::{NativeShellApp, ScreenId, ScreenView};

    fn temp_theme_file(test_name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "loopforge-native-shell-{}-{}-{}.txt",
            test_name,
            std::process::id(),
            unique
        ))
    }

    #[test]
    fn applies_selected_theme_to_every_surface() {
        let path = temp_theme_file("surface");
        let mut app = NativeShellApp::boot(Some(path.clone())).expect("boot");
        app.select_theme(ThemeName::Dawn).expect("save");

        app.set_screen(ScreenId::Dashboard);
        let dashboard = match app.render() {
            ScreenView::Dashboard(screen) => screen,
            _ => panic!("screen mismatch"),
        };

        app.set_screen(ScreenId::Wizard);
        let wizard = match app.render() {
            ScreenView::Wizard(screen) => screen,
            _ => panic!("screen mismatch"),
        };

        app.set_screen(ScreenId::Monitor);
        let monitor = match app.render() {
            ScreenView::Monitor(screen) => screen,
            _ => panic!("screen mismatch"),
        };

        assert_eq!(dashboard.accent, wizard.accent);
        assert_eq!(wizard.accent, monitor.accent);
        assert_eq!(dashboard.shell_background, wizard.shell_background);
        assert_eq!(wizard.shell_background, monitor.shell_background);

        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn persists_selected_theme_between_restarts() {
        let path = temp_theme_file("restart");

        let mut first_boot = NativeShellApp::boot(Some(path.clone())).expect("boot");
        first_boot
            .select_theme(ThemeName::Dawn)
            .expect("persist selection");

        let second_boot = NativeShellApp::boot(Some(path.clone())).expect("boot");
        assert_eq!(second_boot.theme(), ThemeName::Dawn);

        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }
}
