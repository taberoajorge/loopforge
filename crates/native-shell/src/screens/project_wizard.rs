use super::theme::palette::ThemePalette;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectWizardScreen {
    pub shell_background: &'static str,
    pub surface_background: &'static str,
    pub text_primary: &'static str,
    pub accent: &'static str,
}

impl ProjectWizardScreen {
    pub fn themed(palette: ThemePalette) -> Self {
        Self {
            shell_background: palette.shell_background,
            surface_background: palette.surface_background,
            text_primary: palette.text_primary,
            accent: palette.accent,
        }
    }
}
