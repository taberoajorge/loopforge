use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime};

pub struct TauriPathResolver<R: Runtime> {
    app: Arc<AppHandle<R>>,
}

impl<R: Runtime> TauriPathResolver<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app: Arc::new(app) }
    }

    pub fn project_artifact_dir(&self, project_id: &str) -> Result<PathBuf, String> {
        self.app
            .path()
            .app_data_dir()
            .map(|data_dir| data_dir.join("projects").join(project_id))
            .map_err(|err| err.to_string())
    }

    pub fn app_data_dir(&self) -> Result<PathBuf, String> {
        self.app
            .path()
            .app_data_dir()
            .map_err(|err| err.to_string())
    }
}
