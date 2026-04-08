use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub slot: PluginSlot,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginSlot {
    Runtime,
    Agent,
    Workspace,
    Tracker,
    Scm,
    Notifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub key: String,
    pub label: String,
    pub field_type: ConfigFieldType,
    pub required: bool,
    pub default_value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigFieldType {
    Text,
    Number,
    Boolean,
    Select { options: Vec<String> },
}

pub trait RuntimePlugin: Send + Sync {
    fn info(&self) -> PluginInfo;
    fn api_version(&self) -> u32 { 1 }
    fn config_schema(&self) -> Vec<ConfigField> { vec![] }
    fn health_check(&self) -> impl std::future::Future<Output = Result<bool>> + Send;
    fn pre_iteration(&self, iteration: u32) -> impl std::future::Future<Output = Result<()>> + Send;
    fn post_iteration(&self, iteration: u32, success: bool) -> impl std::future::Future<Output = Result<()>> + Send;
}

pub trait AgentPlugin: Send + Sync {
    fn info(&self) -> PluginInfo;
    fn api_version(&self) -> u32 { 1 }
    fn config_schema(&self) -> Vec<ConfigField> { vec![] }
    fn health_check(&self) -> impl std::future::Future<Output = Result<bool>> + Send;
    fn supported_agents(&self) -> Vec<String>;
    fn invoke(&self, prompt: &str, work_dir: &str) -> impl std::future::Future<Output = Result<String>> + Send;
}

pub trait WorkspacePlugin: Send + Sync {
    fn info(&self) -> PluginInfo;
    fn api_version(&self) -> u32 { 1 }
    fn config_schema(&self) -> Vec<ConfigField> { vec![] }
    fn health_check(&self) -> impl std::future::Future<Output = Result<bool>> + Send;
    fn setup_workspace(&self, config: &HashMap<String, Value>) -> impl std::future::Future<Output = Result<String>> + Send;
    fn cleanup_workspace(&self, workspace_path: &str) -> impl std::future::Future<Output = Result<()>> + Send;
}

pub trait TrackerPlugin: Send + Sync {
    fn info(&self) -> PluginInfo;
    fn api_version(&self) -> u32 { 1 }
    fn config_schema(&self) -> Vec<ConfigField> { vec![] }
    fn health_check(&self) -> impl std::future::Future<Output = Result<bool>> + Send;
    fn report_progress(&self, story_id: &str, status: &str) -> impl std::future::Future<Output = Result<()>> + Send;
    fn sync_stories(&self) -> impl std::future::Future<Output = Result<Vec<Value>>> + Send;
}

pub trait ScmPlugin: Send + Sync {
    fn info(&self) -> PluginInfo;
    fn api_version(&self) -> u32 { 1 }
    fn config_schema(&self) -> Vec<ConfigField> { vec![] }
    fn health_check(&self) -> impl std::future::Future<Output = Result<bool>> + Send;
    fn create_pr(&self, title: &str, body: &str, branch: &str) -> impl std::future::Future<Output = Result<String>> + Send;
    fn fetch_review_comments(&self, pr_id: &str) -> impl std::future::Future<Output = Result<Vec<Value>>> + Send;
    fn post_comment(&self, pr_id: &str, body: &str) -> impl std::future::Future<Output = Result<()>> + Send;
}

pub trait NotifierPlugin: Send + Sync {
    fn info(&self) -> PluginInfo;
    fn api_version(&self) -> u32 { 1 }
    fn config_schema(&self) -> Vec<ConfigField> { vec![] }
    fn health_check(&self) -> impl std::future::Future<Output = Result<bool>> + Send;
    fn notify(&self, title: &str, message: &str, level: &str) -> impl std::future::Future<Output = Result<()>> + Send;
}

pub struct BuiltinRuntime;

impl RuntimePlugin for BuiltinRuntime {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "builtin-runtime".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            slot: PluginSlot::Runtime,
            description: "Default runtime lifecycle hooks".into(),
        }
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn pre_iteration(&self, _iteration: u32) -> Result<()> {
        Ok(())
    }

    async fn post_iteration(&self, _iteration: u32, _success: bool) -> Result<()> {
        Ok(())
    }
}

pub struct BuiltinNotifier;

impl NotifierPlugin for BuiltinNotifier {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "builtin-notifier".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            slot: PluginSlot::Notifier,
            description: "Default stdout notifier".into(),
        }
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn notify(&self, title: &str, message: &str, level: &str) -> Result<()> {
        tracing::info!(level, title, message, "notification");
        Ok(())
    }
}
