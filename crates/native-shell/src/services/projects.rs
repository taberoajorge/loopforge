use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectLifecycle {
    Active,
    Archived,
}

impl ProjectLifecycle {
    fn as_key(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }

    fn from_key(value: &str) -> Self {
        match value.trim() {
            "archived" => Self::Archived,
            _ => Self::Active,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub lifecycle: ProjectLifecycle,
    pub latest_session: String,
}

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
    pub fn create_project(&self, name: &str, summary: &str) -> io::Result<ProjectRecord> {
        let project_id = format!("proj-{}-{}", Self::timestamp_nanos(), Self::slug(name));
        let record = ProjectRecord {
            id: project_id,
            name: name.trim().to_owned(),
            lifecycle: ProjectLifecycle::Active,
            latest_session: format!("sess-{}", Self::timestamp_nanos()),
        };
        self.write_draft_artifact(&record, summary)?;
        self.persist_record(&record)?;
        Ok(record)
    }
    pub fn list_projects(&self, include_archived: bool) -> io::Result<Vec<ProjectRecord>> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut projects = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let metadata_path = entry.path().join("project.txt");
            if !metadata_path.exists() {
                continue;
            }
            let record = Self::read_record(&metadata_path)?;
            if include_archived || record.lifecycle != ProjectLifecycle::Archived {
                projects.push(record);
            }
        }
        projects.sort_by(|left, right| right.id.cmp(&left.id));
        Ok(projects)
    }
    pub fn resume_project(&self, project_id: &str) -> io::Result<ProjectRecord> {
        self.update_lifecycle(project_id, ProjectLifecycle::Active)
    }
    pub fn archive_project(&self, project_id: &str) -> io::Result<ProjectRecord> {
        self.update_lifecycle(project_id, ProjectLifecycle::Archived)
    }
    fn default_root() -> PathBuf {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_owned());
        PathBuf::from(home).join(".config").join("loopforge").join("projects")
    }
    fn update_lifecycle(&self, project_id: &str, lifecycle: ProjectLifecycle) -> io::Result<ProjectRecord> {
        let mut record = Self::read_record(&self.root.join(project_id).join("project.txt"))?;
        record.lifecycle = lifecycle;
        self.persist_record(&record)?;
        Ok(record)
    }
    fn persist_record(&self, record: &ProjectRecord) -> io::Result<()> {
        let project_dir = self.root.join(&record.id);
        fs::create_dir_all(&project_dir)?;
        fs::write(project_dir.join("project.txt"), Self::encode_record(record))?;
        fs::write(
            project_dir.join("plan.md"),
            format!("# {}\n\n- Bootstrapped from native shell wizard.\n", record.name),
        )?;
        fs::write(project_dir.join("prd.json"), "{\"stories\":[]}\n")?;
        fs::write(project_dir.join("config.json"), "{\"maxIterations\":20}\n")?;
        fs::write(
            project_dir.join("prompt.md"),
            format!("# {}\n\nContinue implementing the accepted stories.\n", record.name),
        )?;
        fs::write(project_dir.join("guardrails.md"), "")?;
        Ok(())
    }
    fn write_draft_artifact(&self, record: &ProjectRecord, summary: &str) -> io::Result<()> {
        let project_dir = self.root.join(&record.id);
        fs::create_dir_all(&project_dir)?;
        fs::write(
            project_dir.join("draft.json"),
            format!(
                "{{\"id\":\"{}\",\"name\":\"{}\",\"step\":\"configure\",\"summary\":\"{}\"}}\n",
                record.id,
                record.name,
                summary.trim()
            ),
        )
    }
    fn read_record(path: &Path) -> io::Result<ProjectRecord> {
        let content = fs::read_to_string(path)?;
        let mut id = String::new();
        let mut name = String::new();
        let mut lifecycle = ProjectLifecycle::Active;
        let mut latest_session = String::new();
        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "id" => id = value.trim().to_owned(),
                    "name" => name = value.trim().to_owned(),
                    "lifecycle" => lifecycle = ProjectLifecycle::from_key(value),
                    "latest_session" => latest_session = value.trim().to_owned(),
                    _ => {}
                }
            }
        }
        if id.is_empty() || name.is_empty() || latest_session.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid project"));
        }
        Ok(ProjectRecord {
            id,
            name,
            lifecycle,
            latest_session,
        })
    }
    fn encode_record(record: &ProjectRecord) -> String {
        format!(
            "id={}\nname={}\nlifecycle={}\nlatest_session={}\n",
            record.id,
            record.name,
            record.lifecycle.as_key(),
            record.latest_session
        )
    }
    fn timestamp_nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    }
    fn slug(value: &str) -> String {
        let normalized = value
            .trim()
            .to_lowercase()
            .chars()
            .map(|character| if character.is_ascii_alphanumeric() { character } else { '-' })
            .collect::<String>();
        let compact = normalized.trim_matches('-').replace("--", "-");
        if compact.is_empty() {
            String::from("project")
        } else {
            compact
        }
    }
}
