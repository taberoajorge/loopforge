use serde::{Deserialize, Serialize};

fn default_snapshot_version() -> u32 {
    1
}

fn default_wizard_step() -> String {
    "describe".to_string()
}

fn default_highest_step() -> u32 {
    1
}

fn default_plan_agent() -> String {
    "claude".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WizardDescribeSnapshot {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub working_directory: String,
    #[serde(default = "default_plan_agent")]
    pub plan_agent: String,
    #[serde(default)]
    pub plan_model: Option<String>,
    #[serde(default)]
    pub plan_effort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WizardPlanSnapshot {
    #[serde(default)]
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WizardAtomizeSnapshot {
    #[serde(default)]
    pub stories_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WizardConfigureSnapshot {
    #[serde(default)]
    pub execute_agent: String,
    #[serde(default)]
    pub execute_model: Option<String>,
    #[serde(default)]
    pub execute_effort: Option<String>,
    #[serde(default)]
    pub fallback_chain: Vec<String>,
    #[serde(default)]
    pub gutter_threshold: u32,
    #[serde(default)]
    pub max_iterations: u32,
    #[serde(default)]
    pub cooldown_seconds: u32,
    #[serde(default)]
    pub test_command: String,
    #[serde(default)]
    pub max_verification_retries: u32,
    #[serde(default)]
    pub scm_provider: String,
    #[serde(default)]
    pub review_polling_interval: u32,
    #[serde(default)]
    pub review_timeout: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WizardSnapshot {
    #[serde(default = "default_snapshot_version")]
    pub version: u32,
    #[serde(default)]
    pub project_id: String,
    #[serde(default = "default_wizard_step")]
    pub current_step: String,
    #[serde(default = "default_highest_step")]
    pub highest_step: u32,
    #[serde(default)]
    pub describe: WizardDescribeSnapshot,
    #[serde(default)]
    pub plan: WizardPlanSnapshot,
    #[serde(default)]
    pub atomize: WizardAtomizeSnapshot,
    #[serde(default)]
    pub configure: WizardConfigureSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SaveWizardStateCommand {
    #[serde(default)]
    pub project_id: String,
    #[serde(default = "default_wizard_step")]
    pub wizard_step: String,
    #[serde(default)]
    pub wizard_state_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SaveDraftCommand {
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub draft_json: String,
}

#[cfg(test)]
mod tests {
    use super::{
        SaveWizardStateCommand, WizardConfigureSnapshot, WizardDescribeSnapshot, WizardSnapshot,
    };

    #[test]
    fn wizard_snapshot_roundtrips_serialization() {
        let snapshot = WizardSnapshot {
            version: 1,
            project_id: "project-1".to_string(),
            current_step: "configure".to_string(),
            highest_step: 4,
            describe: WizardDescribeSnapshot {
                name: "LoopForge".to_string(),
                description: "Shared contracts".to_string(),
                working_directory: "/tmp/loopforge".to_string(),
                plan_agent: "codex".to_string(),
                plan_model: Some("gpt-5.4".to_string()),
                plan_effort: Some("high".to_string()),
            },
            plan: Default::default(),
            atomize: Default::default(),
            configure: WizardConfigureSnapshot {
                execute_agent: "codex".to_string(),
                fallback_chain: vec!["claude".to_string()],
                max_iterations: 12,
                ..Default::default()
            },
        };

        let json = serde_json::to_string(&snapshot).expect("serialize wizard snapshot");
        let restored: WizardSnapshot =
            serde_json::from_str(&json).expect("deserialize wizard snapshot");

        assert_eq!(restored, snapshot);
        assert!(json.contains("\"currentStep\":\"configure\""));
    }

    #[test]
    fn legacy_wizard_snapshot_uses_backward_compatible_defaults() {
        let legacy = r#"{
            "projectId":"project-1",
            "describe":{
                "name":"LoopForge",
                "description":"Shared contracts",
                "workingDirectory":"/tmp/loopforge"
            },
            "plan":{"completed":true},
            "atomize":{"storiesCount":2}
        }"#;

        let snapshot: WizardSnapshot =
            serde_json::from_str(legacy).expect("deserialize legacy snapshot");

        assert_eq!(snapshot.version, 1);
        assert_eq!(snapshot.current_step, "describe");
        assert_eq!(snapshot.highest_step, 1);
        assert_eq!(snapshot.describe.plan_agent, "claude");
        assert_eq!(snapshot.configure.execute_agent, "");
    }

    #[test]
    fn save_wizard_state_command_uses_camel_case() {
        let command = SaveWizardStateCommand {
            project_id: "project-1".to_string(),
            wizard_step: "plan".to_string(),
            wizard_state_json: "{}".to_string(),
        };

        let value = serde_json::to_value(command).expect("serialize wizard state command");

        assert_eq!(value["projectId"], "project-1");
        assert_eq!(value["wizardStep"], "plan");
        assert_eq!(value["wizardStateJson"], "{}");
    }
}
