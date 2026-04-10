#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectCommand {
    Create { id: String, name: String },
    Archive { id: String },
}

pub trait ProjectService {
    fn apply(&self, command: ProjectCommand) -> ProjectSummary;
}
