use crate::monitor_state::MonitorState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SidebarEntryKind {
    Session,
    Agent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SidebarEntry {
    pub row_index: usize,
    pub kind: SidebarEntryKind,
    pub label: String,
    pub detail: String,
    pub is_selected: bool,
}

pub fn build_sidebar_entries(state: &MonitorState, active_row: usize) -> Vec<SidebarEntry> {
    let session_entries = state
        .sessions
        .iter()
        .enumerate()
        .map(|(session_index, session)| SidebarEntry {
            row_index: session_index,
            kind: SidebarEntryKind::Session,
            label: session.title.clone(),
            detail: session.status.clone(),
            is_selected: session_index == active_row,
        });

    let session_count = state.sessions.len();
    let agent_entries = state
        .agents
        .iter()
        .enumerate()
        .map(|(agent_index, agent)| {
            let row_index = session_count + agent_index;
            SidebarEntry {
                row_index,
                kind: SidebarEntryKind::Agent,
                label: agent.name.clone(),
                detail: agent.model.clone(),
                is_selected: row_index == active_row,
            }
        });

    session_entries.chain(agent_entries).collect()
}

pub fn row_count(state: &MonitorState) -> usize {
    state.sessions.len() + state.agents.len()
}
