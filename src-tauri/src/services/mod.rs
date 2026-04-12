pub mod monitor_adapter;
pub mod session_adapter;

pub fn attach_runtime_aliases<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
    handler: impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static,
) -> tauri::Builder<R> {
    builder.invoke_handler(handler)
}
