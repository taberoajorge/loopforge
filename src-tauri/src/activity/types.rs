use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PlanEventKind {
    Search,
    DocsLookup,
    McpCall,
    PlanContent,
    Thinking,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanEvent {
    pub kind: PlanEventKind,
    pub content: String,
    pub timestamp: String,
}
