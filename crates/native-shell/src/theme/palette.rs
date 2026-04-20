#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePalette {
    pub name: &'static str,
    pub shell_background: &'static str,
    pub surface_background: &'static str,
    pub text_primary: &'static str,
    pub accent: &'static str,
}

pub const MIDNIGHT: ThemePalette = ThemePalette {
    name: "midnight",
    shell_background: "#090b16",
    surface_background: "#14182b",
    text_primary: "#edf2ff",
    accent: "#6b87ff",
};

pub const DAWN: ThemePalette = ThemePalette {
    name: "dawn",
    shell_background: "#f6f3eb",
    surface_background: "#fffdf8",
    text_primary: "#312a20",
    accent: "#9a5b29",
};
