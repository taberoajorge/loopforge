use super::backend_adapter::{BackendAdapter, SharedProjectDetail, SharedProjects};
use app_services::{IterationStory, ProjectQueryRecord};
use loopforge_app_core::projects::{self, CreateProjectRecord, CreateProjectRequest};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub type ProjectRecord = ProjectQueryRecord;
#[derive(Debug, Clone)]
pub struct ProjectsService {
    root: PathBuf,
}
impl ProjectsService {
    pub fn new(root: Option<PathBuf>) -> Self {
        Self {
            root: root.unwrap_or_else(Self::default_root),
        }
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn create_project(&self, request: CreateProjectRequest) -> io::Result<ProjectRecord> {
        let project_name = request.name.clone();
        BackendAdapter::create_project(request, |request| {
            projects::create_project(
                request,
                || {
                    format!(
                        "proj-{}-{}",
                        Self::timestamp_nanos(),
                        Self::slug(&project_name)
                    )
                },
                Self::timestamp,
                |project_id, artifact_name| self.initialize_artifacts(project_id, artifact_name),
                |record| self.persist_record(record),
            )
        })
    }
    pub fn list_projects(&self) -> io::Result<SharedProjects> {
        BackendAdapter::list_projects(|| projects::list_projects(|| self.read_projects()))
    }
    pub fn project_detail(&self, project_id: &str) -> io::Result<SharedProjectDetail> {
        BackendAdapter::project_detail(project_id, |resolved_id| {
            let project = self.read_project(&resolved_id)?;
            let stories = self.read_stories();
            let passed_count = stories
                .iter()
                .filter(|story| story.status == "passed")
                .count();
            let blocked_count = stories
                .iter()
                .filter(|story| story.status == "blocked")
                .count();
            let total_stories = stories.len();
            Ok(SharedProjectDetail {
                project,
                total_stories,
                passed_count,
                blocked_count,
                pending_count: total_stories.saturating_sub(passed_count + blocked_count),
                stories,
            })
        })
    }
    pub fn resume_project(&self, project_id: &str) -> io::Result<ProjectRecord> {
        let projects = BackendAdapter::list_projects(|| {
            self.update_status(project_id, "active")?;
            self.read_projects()
        })?;
        Self::find_project(&projects, project_id)
    }
    pub fn archive_project(&self, project_id: &str) -> io::Result<ProjectRecord> {
        let projects = BackendAdapter::list_projects(|| {
            self.update_status(project_id, "archived")?;
            self.read_projects()
        })?;
        Self::find_project(&projects, project_id)
    }
    fn default_root() -> PathBuf {
        PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_owned()))
            .join(".config/loopforge/projects")
    }
    fn initialize_artifacts(&self, project_id: &str, project_name: &str) -> io::Result<()> {
        let project_dir = self.root.join(project_id);
        fs::create_dir_all(&project_dir)?;
        fs::write(
            project_dir.join("plan.md"),
            format!("# {project_name}\n\n- Bootstrapped from native shell wizard.\n"),
        )?;
        fs::write(project_dir.join("prd.json"), "{\"stories\":[]}\n")?;
        fs::write(project_dir.join("config.json"), "{\"maxIterations\":20}\n")?;
        fs::write(
            project_dir.join("prompt.md"),
            format!("# {project_name}\n\nContinue implementing the accepted stories.\n"),
        )?;
        fs::write(project_dir.join("guardrails.md"), "")?;
        Ok(())
    }
    fn persist_record(&self, record: CreateProjectRecord) -> io::Result<ProjectRecord> {
        let project = ProjectRecord {
            id: record.id,
            name: record.name,
            description: record.description,
            status: record.status,
            working_directory: record.working_directory,
            created_at: record.created_at,
            updated_at: record.updated_at,
            wizard_step: record.wizard_step,
        };
        let project_dir = self.root.join(&project.id);
        fs::create_dir_all(&project_dir)?;
        fs::write(
            project_dir.join("project.txt"),
            Self::encode_project(&project),
        )?;
        fs::write(
            project_dir.join("draft.json"),
            format!(
                "id={}\nname={}\nstep={}\ndescription={}\n",
                project.id,
                project.name,
                project
                    .wizard_step
                    .clone()
                    .unwrap_or_else(|| "describe".to_owned()),
                project.description
            ),
        )?;
        Ok(project)
    }
    fn read_projects(&self) -> io::Result<SharedProjects> {
        if !self.root.exists() {
            return Ok(SharedProjects::default());
        }
        let mut grouped = SharedProjects::default();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let project_path = entry.path().join("project.txt");
            if project_path.exists() {
                Self::push_project(&mut grouped, Self::read_project_file(&project_path)?);
            }
        }
        Ok(grouped)
    }
    fn read_project(&self, project_id: &str) -> io::Result<ProjectRecord> {
        Self::read_project_file(&self.root.join(project_id).join("project.txt"))
    }
    fn read_project_file(path: &Path) -> io::Result<ProjectRecord> {
        let content = fs::read_to_string(path)?;
        let mut project = ProjectRecord::default();
        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "id" => project.id = value.trim().to_owned(),
                    "name" => project.name = value.trim().to_owned(),
                    "description" => project.description = value.trim().to_owned(),
                    "status" => project.status = value.trim().to_owned(),
                    "working_directory" => project.working_directory = value.trim().to_owned(),
                    "created_at" => project.created_at = value.trim().to_owned(),
                    "updated_at" => project.updated_at = value.trim().to_owned(),
                    "wizard_step" => {
                        project.wizard_step =
                            (!value.trim().is_empty()).then(|| value.trim().to_owned())
                    }
                    _ => {}
                }
            }
        }
        if project.id.is_empty() || project.name.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid project",
            ));
        }
        Ok(project)
    }
    fn update_status(&self, project_id: &str, status: &str) -> io::Result<()> {
        let mut project = self.read_project(project_id)?;
        project.status = status.to_owned();
        project.updated_at = Self::timestamp();
        fs::write(
            self.root.join(project_id).join("project.txt"),
            Self::encode_project(&project),
        )
    }
    fn encode_project(project: &ProjectRecord) -> String {
        format!("id={}\nname={}\ndescription={}\nstatus={}\nworking_directory={}\ncreated_at={}\nupdated_at={}\nwizard_step={}\n", project.id, project.name, project.description, project.status, project.working_directory, project.created_at, project.updated_at, project.wizard_step.clone().unwrap_or_default())
    }
    fn push_project(grouped: &mut SharedProjects, project: ProjectRecord) {
        match project.status.as_str() {
            "active" => grouped.active.push(project),
            "paused" => grouped.paused.push(project),
            "completed" => grouped.completed.push(project),
            "archived" => grouped.archived.push(project),
            "blocked" => grouped.blocked.push(project),
            "failed" => grouped.failed.push(project),
            _ => grouped.draft.push(project),
        }
    }
    fn find_project(grouped: &SharedProjects, project_id: &str) -> io::Result<ProjectRecord> {
        grouped
            .active
            .iter()
            .chain(grouped.paused.iter())
            .chain(grouped.completed.iter())
            .chain(grouped.draft.iter())
            .chain(grouped.archived.iter())
            .chain(grouped.blocked.iter())
            .chain(grouped.failed.iter())
            .find(|project| project.id == project_id)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "project not found"))
    }
    fn read_stories(&self) -> Vec<IterationStory> {
        Vec::new()
    }
    fn timestamp() -> String {
        Self::timestamp_nanos().to_string()
    }
    fn timestamp_nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    }
    fn slug(value: &str) -> String {
        let compact = value
            .trim()
            .to_lowercase()
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .replace("--", "-");
        if compact.is_empty() {
            String::from("project")
        } else {
            compact
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectsService;
    use loopforge_app_core::projects::CreateProjectRequest;
    fn temp_root() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "native-shell-projects-{}",
            super::ProjectsService::timestamp_nanos()
        ))
    }
    #[test]
    fn projects_service_uses_shared_project_contracts() {
        let root = temp_root();
        let service = ProjectsService::new(Some(root.clone()));
        let project = service
            .create_project(CreateProjectRequest {
                name: String::from("LoopForge"),
                description: String::from("Backend-owned wizard"),
                working_directory: String::from("/tmp/loopforge"),
                wizard_step: Some(String::from("describe")),
            })
            .expect("create");
        let grouped = service.list_projects().expect("list");
        let archived = service.archive_project(&project.id).expect("archive");
        let detail = service.project_detail(&project.id).expect("detail");
        assert_eq!(grouped.draft[0].id, project.id);
        assert_eq!(archived.status, "archived");
        assert_eq!(detail.project.name, "LoopForge");
        let _ = std::fs::remove_dir_all(root);
    }
}
