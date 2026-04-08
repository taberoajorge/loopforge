use crate::errors::PrdError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Critical,
    High,
    #[default]
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScopeSpec {
    #[serde(default)]
    pub files_to_modify: Vec<String>,
    #[serde(default)]
    pub files_to_create: Vec<String>,
    #[serde(default)]
    pub files_to_avoid: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VerificationSpec {
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub assertions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStory {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub scope: ScopeSpec,
    #[serde(default)]
    pub verification: VerificationSpec,
    #[serde(default)]
    pub commit_message: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub estimated_complexity: Complexity,
    #[serde(default)]
    pub estimated_minutes: u32,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub passes: bool,
    #[serde(default)]
    pub blocked: bool,
    #[serde(default)]
    pub attempts: u32,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prd {
    #[serde(alias = "project")]
    pub project_name: String,
    #[serde(default)]
    pub feature: String,
    #[serde(default)]
    pub working_directory: String,
    #[serde(default)]
    pub branch_name: Option<String>,
    #[serde(alias = "userStories", alias = "user_stories")]
    pub stories: Vec<UserStory>,
    #[serde(default)]
    pub generated_at: Option<String>,
}

impl Prd {
    pub fn load(path: &Path) -> Result<Self, PrdError> {
        let content = std::fs::read_to_string(path).map_err(|source| PrdError::ReadFailed {
            path: path.to_path_buf(),
            source,
        })?;
        let prd: Prd = serde_json::from_str(&content).map_err(|source| PrdError::ParseFailed {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(prd)
    }

    pub fn save(&self, path: &Path) -> Result<(), PrdError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, &content).map_err(|source| PrdError::WriteFailed {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(())
    }

    pub fn total_stories(&self) -> usize {
        self.stories.len()
    }

    pub fn passed_count(&self) -> usize {
        self.stories.iter().filter(|story| story.passes).count()
    }

    pub fn blocked_count(&self) -> usize {
        self.stories.iter().filter(|story| story.blocked).count()
    }

    pub fn pending_count(&self) -> usize {
        self.total_stories() - self.passed_count() - self.blocked_count()
    }

    pub fn next_actionable_story(&self) -> Option<&UserStory> {
        self.stories
            .iter()
            .find(|story| !story.passes && !story.blocked)
    }

    pub fn mark_story_passed(&mut self, story_id: &str) -> bool {
        if let Some(story) = self.stories.iter_mut().find(|story| story.id == story_id) {
            story.passes = true;
            return true;
        }
        false
    }

    pub fn total_estimated_minutes(&self) -> u32 {
        self.stories.iter().map(|story| story.estimated_minutes).sum()
    }

    pub fn validate_atomicity(&self) -> Result<(), PrdError> {
        let mut seen_ids: HashSet<&str> = HashSet::new();
        let all_ids: HashSet<&str> = self.stories.iter().map(|story| story.id.as_str()).collect();

        for story in &self.stories {
            if story.id.is_empty() {
                return Err(PrdError::ValidationFailed {
                    reason: "Story has empty id".into(),
                });
            }
            if story.title.is_empty() {
                return Err(PrdError::ValidationFailed {
                    reason: format!("Story '{}' has empty title", story.id),
                });
            }
            if story.acceptance_criteria.is_empty() {
                return Err(PrdError::ValidationFailed {
                    reason: format!("Story '{}' has no acceptance criteria", story.id),
                });
            }
            if !seen_ids.insert(story.id.as_str()) {
                return Err(PrdError::ValidationFailed {
                    reason: format!("Duplicate story id: '{}'", story.id),
                });
            }
            for dep_id in &story.depends_on {
                if !all_ids.contains(dep_id.as_str()) {
                    return Err(PrdError::ValidationFailed {
                        reason: format!(
                            "Story '{}' depends_on unknown id '{}'",
                            story.id, dep_id
                        ),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn is_valid_json(path: &Path) -> bool {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str::<Prd>(&content).ok())
            .is_some()
    }
}

#[cfg(test)]
mod tests_backward_compat {
    use super::*;

    const LEGACY_PRD: &str = r#"{
        "project": "my-app",
        "feature": "auth",
        "workingDirectory": "/tmp/my-app",
        "userStories": [
            {
                "id": "S-001",
                "title": "Add login page",
                "description": "Create a login form",
                "acceptanceCriteria": ["Login form renders"],
                "passes": false,
                "blocked": false,
                "notes": null
            }
        ]
    }"#;

    #[test]
    fn legacy_prd_deserializes() {
        let prd: Prd = serde_json::from_str(LEGACY_PRD).expect("legacy prd should deserialize");
        assert_eq!(prd.project_name, "my-app");
        assert_eq!(prd.feature, "auth");
        assert_eq!(prd.stories.len(), 1);
        assert_eq!(prd.stories[0].id, "S-001");
    }

    #[test]
    fn legacy_story_defaults_for_new_fields() {
        let prd: Prd = serde_json::from_str(LEGACY_PRD).unwrap();
        let story = &prd.stories[0];
        assert_eq!(story.priority, Priority::Medium);
        assert_eq!(story.estimated_complexity, Complexity::Medium);
        assert_eq!(story.estimated_minutes, 0);
        assert!(story.depends_on.is_empty());
        assert_eq!(story.attempts, 0);
        assert!(story.scope.files_to_modify.is_empty());
        assert!(story.verification.commands.is_empty());
        assert!(story.commit_message.is_none());
    }

    #[test]
    fn legacy_project_alias_reads_project_key() {
        let json = r#"{"project":"proj","stories":[]}"#;
        let prd: Prd = serde_json::from_str(json).unwrap();
        assert_eq!(prd.project_name, "proj");
    }

    #[test]
    fn legacy_user_stories_alias_reads_array() {
        let json = r#"{"projectName":"proj","userStories":[{"id":"S-001","title":"T","acceptanceCriteria":["ac"]}]}"#;
        let prd: Prd = serde_json::from_str(json).unwrap();
        assert_eq!(prd.stories.len(), 1);
    }
}

#[cfg(test)]
mod tests_d6_roundtrip {
    use super::*;

    fn make_full_story() -> UserStory {
        UserStory {
            id: "S-001".to_string(),
            title: "Add authentication".to_string(),
            description: Some("Implement JWT login".to_string()),
            acceptance_criteria: vec!["User can log in".to_string(), "Token is issued".to_string()],
            scope: ScopeSpec {
                files_to_modify: vec!["src/auth.rs".to_string()],
                files_to_create: vec!["src/jwt.rs".to_string()],
                files_to_avoid: vec!["migrations/*".to_string()],
            },
            verification: VerificationSpec {
                commands: vec!["cargo test".to_string()],
                assertions: vec!["grep -q 'jwt' src/auth.rs".to_string()],
            },
            commit_message: Some("feat(auth): implement JWT login".to_string()),
            priority: Priority::High,
            estimated_complexity: Complexity::Medium,
            estimated_minutes: 45,
            depends_on: vec![],
            passes: false,
            blocked: false,
            attempts: 0,
            notes: None,
        }
    }

    fn make_full_prd() -> Prd {
        Prd {
            project_name: "my-app".to_string(),
            feature: String::new(),
            working_directory: String::new(),
            branch_name: None,
            stories: vec![make_full_story()],
            generated_at: Some("2026-03-22T00:00:00Z".to_string()),
        }
    }

    #[test]
    fn full_d6_serializes_with_correct_keys() {
        let prd = make_full_prd();
        let json = serde_json::to_string(&prd).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value.get("projectName").is_some(), "must have projectName key");
        assert!(value.get("stories").is_some(), "must have stories key");
        let story = &value["stories"][0];
        assert!(story.get("scope").is_some());
        assert!(story.get("verification").is_some());
        assert!(story.get("commitMessage").is_some());
        assert_eq!(story["priority"], "high");
        assert_eq!(story["estimatedComplexity"], "medium");
        assert_eq!(story["estimatedMinutes"], 45);
    }

    #[test]
    fn full_d6_roundtrip_preserves_all_fields() {
        let original = make_full_prd();
        let json = serde_json::to_string(&original).unwrap();
        let restored: Prd = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.project_name, original.project_name);
        assert_eq!(restored.stories.len(), 1);
        let story = &restored.stories[0];
        assert_eq!(story.id, "S-001");
        assert_eq!(story.priority, Priority::High);
        assert_eq!(story.estimated_complexity, Complexity::Medium);
        assert_eq!(story.estimated_minutes, 45);
        assert_eq!(story.scope.files_to_modify, vec!["src/auth.rs"]);
        assert_eq!(story.scope.files_to_create, vec!["src/jwt.rs"]);
        assert_eq!(story.scope.files_to_avoid, vec!["migrations/*"]);
        assert_eq!(story.verification.commands, vec!["cargo test"]);
        assert_eq!(
            story.commit_message.as_deref(),
            Some("feat(auth): implement JWT login")
        );
    }

    #[test]
    fn total_estimated_minutes_sums_stories() {
        let mut prd = make_full_prd();
        prd.stories.push(UserStory {
            id: "S-002".to_string(),
            title: "Add logout".to_string(),
            acceptance_criteria: vec!["User can log out".to_string()],
            estimated_minutes: 15,
            ..make_full_story()
        });
        assert_eq!(prd.total_estimated_minutes(), 60);
    }

    #[test]
    fn validate_atomicity_passes_valid_prd() {
        let prd = make_full_prd();
        assert!(prd.validate_atomicity().is_ok());
    }

    #[test]
    fn validate_atomicity_rejects_missing_acceptance_criteria() {
        let mut prd = make_full_prd();
        prd.stories[0].acceptance_criteria.clear();
        assert!(prd.validate_atomicity().is_err());
    }

    #[test]
    fn validate_atomicity_rejects_duplicate_ids() {
        let mut prd = make_full_prd();
        prd.stories.push(make_full_story());
        assert!(prd.validate_atomicity().is_err());
    }

    #[test]
    fn validate_atomicity_rejects_unknown_dependency() {
        let mut prd = make_full_prd();
        prd.stories[0].depends_on = vec!["S-999".to_string()];
        assert!(prd.validate_atomicity().is_err());
    }

    #[test]
    fn validate_atomicity_accepts_valid_dependency() {
        let mut prd = make_full_prd();
        let second = UserStory {
            id: "S-002".to_string(),
            title: "Use auth".to_string(),
            acceptance_criteria: vec!["Auth works".to_string()],
            depends_on: vec!["S-001".to_string()],
            ..make_full_story()
        };
        prd.stories.push(second);
        assert!(prd.validate_atomicity().is_ok());
    }
}
