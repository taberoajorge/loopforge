use super::super::home_view_model::{
    HomeAction, HomeProjectSummary, HomeSessionSummary, HomeViewModel,
};
use super::super::theme::palette::ThemePalette;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeScreen {
    pub shell_background: &'static str,
    pub surface_background: &'static str,
    pub text_primary: &'static str,
    pub accent: &'static str,
    pub heading: String,
    pub strapline: String,
    pub is_empty: bool,
    pub active_projects: Vec<HomeProjectSummary>,
    pub recent_sessions: Vec<HomeSessionSummary>,
    pub primary_actions: Vec<HomeAction>,
}

impl HomeScreen {
    pub fn themed(palette: ThemePalette, view_model: HomeViewModel) -> Self {
        let is_empty = view_model.active_projects.is_empty();
        Self {
            shell_background: palette.shell_background,
            surface_background: palette.surface_background,
            text_primary: palette.text_primary,
            accent: palette.accent,
            heading: view_model.heading,
            strapline: view_model.strapline,
            is_empty,
            active_projects: view_model.active_projects,
            recent_sessions: view_model.recent_sessions,
            primary_actions: view_model.primary_actions,
        }
    }
}
