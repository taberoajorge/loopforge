use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanSessionStatus {
    Idle,
    Running,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanSessionHandle {
    pub project_id: String,
    pub status: PlanSessionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanSessionEvent {
    Started(PlanSessionHandle),
    Output { project_id: String, chunk: String },
    Stopped { project_id: String },
}

pub trait PlanSessionService {
    fn open(&self, project_id: impl Into<String>) -> PlanSessionHandle;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanCleanupReason {
    ExplicitStop,
    WindowClose,
    RestartRecovery,
    ProcessExit { exit_code: i32 },
}

type CleanupHook = Arc<dyn Fn(PlanCleanupReason) + Send + Sync + 'static>;

fn cleanup_registry() -> &'static Mutex<HashMap<String, CleanupHook>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, CleanupHook>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn register_cleanup(project_id: &str, hook: CleanupHook) {
    if let Ok(mut registry) = cleanup_registry().lock() {
        registry.insert(project_id.to_string(), hook);
    }
}

pub fn cleanup_session<SessionData, ErrorData>(
    project_id: &str,
    reason: PlanCleanupReason,
    take_session: impl FnOnce(&str) -> Result<Option<SessionData>, ErrorData>,
    stop_session: impl FnOnce(SessionData) -> Result<(), ErrorData>,
) -> Result<bool, ErrorData> {
    let session = take_session(project_id)?;
    let removed = session.is_some();
    if let Some(session) = session {
        stop_session(session)?;
    }
    let hook = cleanup_registry()
        .lock()
        .ok()
        .and_then(|mut registry| registry.remove(project_id));
    let had_hook = hook.is_some();
    if let Some(hook) = hook {
        hook(reason);
    }
    Ok(removed || had_hook)
}

pub fn cleanup_sessions<ErrorData>(
    project_ids: impl IntoIterator<Item = String>,
    reason: PlanCleanupReason,
    mut cleanup: impl FnMut(&str, PlanCleanupReason) -> Result<bool, ErrorData>,
) -> Result<(), ErrorData> {
    for project_id in project_ids {
        cleanup(&project_id, reason)?;
    }
    Ok(())
}

pub async fn start_plan<FutureData, ErrorData>(
    project_id: String,
    start: impl FnOnce(String) -> FutureData,
) -> Result<(), ErrorData>
where
    FutureData: Future<Output = Result<(), ErrorData>>,
{
    start(project_id).await
}

pub fn write_to_plan<ErrorData>(
    project_id: String,
    input: String,
    write: impl FnOnce(String, Vec<u8>) -> Result<(), ErrorData>,
    touch_activity: impl FnOnce(),
) -> Result<(), ErrorData> {
    let mut payload = input.into_bytes();
    payload.push(b'\n');
    write(project_id, payload)?;
    touch_activity();
    Ok(())
}

pub fn stop_plan<ErrorData>(
    project_id: String,
    stop: impl FnOnce(String, PlanCleanupReason) -> Result<(), ErrorData>,
) -> Result<(), ErrorData> {
    stop(project_id, PlanCleanupReason::ExplicitStop)
}
