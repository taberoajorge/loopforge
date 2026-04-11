use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use self::palette::{DAWN, MIDNIGHT, ThemePalette};

pub mod palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    Midnight,
    Dawn,
}

impl ThemeName {
    pub fn palette(self) -> ThemePalette {
        match self {
            Self::Midnight => MIDNIGHT,
            Self::Dawn => DAWN,
        }
    }

    pub fn as_key(self) -> &'static str {
        self.palette().name
    }

    pub fn from_key(value: &str) -> Self {
        match value.trim() {
            "dawn" => Self::Dawn,
            _ => Self::Midnight,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeStore {
    path: PathBuf,
}

impl ThemeStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_path() -> PathBuf {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_owned());
        PathBuf::from(home)
            .join(".config")
            .join("loopforge")
            .join("native-shell")
            .join("theme.txt")
    }

    pub fn load(&self) -> io::Result<ThemeName> {
        match fs::read_to_string(&self.path) {
            Ok(value) => Ok(ThemeName::from_key(&value)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(ThemeName::Midnight),
            Err(error) => Err(error),
        }
    }

    pub fn save(&self, theme: ThemeName) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, theme.as_key())
    }
}
