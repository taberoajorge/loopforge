use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectQueryRecord {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub working_directory: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub wizard_step: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsByStatus<T = ProjectQueryRecord> {
    #[serde(default)]
    pub active: Vec<T>,
    #[serde(default)]
    pub paused: Vec<T>,
    #[serde(default)]
    pub completed: Vec<T>,
    #[serde(default)]
    pub draft: Vec<T>,
    #[serde(default)]
    pub archived: Vec<T>,
    #[serde(default)]
    pub blocked: Vec<T>,
    #[serde(default)]
    pub failed: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct IterationStory {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub duration_secs: Option<i64>,
    #[serde(default)]
    pub duration_label: Option<String>,
    #[serde(default)]
    pub attempts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail<ProjectData = ProjectQueryRecord, StoryData = IterationStory> {
    #[serde(default)]
    pub project: ProjectData,
    #[serde(default)]
    pub total_stories: usize,
    #[serde(default)]
    pub passed_count: usize,
    #[serde(default)]
    pub blocked_count: usize,
    #[serde(default)]
    pub pending_count: usize,
    #[serde(default)]
    pub stories: Vec<StoryData>,
}

#[cfg(test)]
mod tests {
    use super::{IterationStory, ProjectDetail, ProjectQueryRecord, ProjectsByStatus};

    #[test]
    fn projects_by_status_roundtrips_serialization() {
        let grouped = ProjectsByStatus {
            active: vec![ProjectQueryRecord {
                id: "project-1".to_string(),
                name: "LoopForge".to_string(),
                status: "active".to_string(),
                ..Default::default()
            }],
            blocked: vec![ProjectQueryRecord {
                id: "project-2".to_string(),
                name: "Blocked".to_string(),
                status: "blocked".to_string(),
                wizard_step: Some("configure".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let json = serde_json::to_string(&grouped).expect("serialize projects by status");
        let restored: ProjectsByStatus =
            serde_json::from_str(&json).expect("deserialize projects by status");

        assert_eq!(restored, grouped);
        assert!(json.contains("\"wizardStep\":\"configure\""));
    }

    #[test]
    fn project_detail_uses_project_query_contract_shape() {
        let detail = ProjectDetail {
            project: ProjectQueryRecord {
                id: "project-1".to_string(),
                name: "LoopForge".to_string(),
                status: "draft".to_string(),
                ..Default::default()
            },
            total_stories: 3,
            passed_count: 1,
            blocked_count: 1,
            pending_count: 1,
            stories: vec![IterationStory {
                id: "story-1".to_string(),
                title: "Extract contracts".to_string(),
                status: "passed".to_string(),
                attempts: 2,
                ..Default::default()
            }],
        };

        let value = serde_json::to_value(detail).expect("serialize project detail");

        assert_eq!(value["project"]["id"], "project-1");
        assert_eq!(value["totalStories"], 3);
        assert_eq!(value["stories"][0]["attempts"], 2);
    }
}
