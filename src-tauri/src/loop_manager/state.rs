use super::StartLoopArgs;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

pub(crate) struct LoopHandle {
    pub(crate) shutdown_flag: Arc<AtomicBool>,
    pub(crate) args: StartLoopArgs,
    pub(crate) _join_handle: JoinHandle<()>,
}

impl LoopHandle {
    pub(crate) fn placeholder(args: StartLoopArgs) -> Self {
        Self {
            shutdown_flag: Arc::new(AtomicBool::new(true)),
            args,
            _join_handle: tokio::spawn(async {}),
        }
    }
}

#[derive(Default)]
pub struct LoopManagerState(pub Mutex<HashMap<String, LoopHandle>>);

impl LoopManagerState {
    pub fn shutdown_all(&self) {
        if let Ok(handles) = self.0.lock() {
            for handle in handles.values() {
                handle.shutdown_flag.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn active_count(&self) -> usize {
        self.0.lock().map(|handles| handles.len()).unwrap_or(0)
    }
}
