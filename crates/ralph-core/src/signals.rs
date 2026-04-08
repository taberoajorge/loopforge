use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;

pub fn setup_shutdown_flag() -> Arc<AtomicBool> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    tokio::spawn(async move {
        let _ = signal::ctrl_c().await;
        crate::logger::log_warning("Ctrl+C received, shutting down gracefully...");
        shutdown_clone.store(true, Ordering::SeqCst);

        let _ = signal::ctrl_c().await;
        crate::logger::log_warning("Second Ctrl+C, force exiting...");
        std::process::exit(130);
    });

    shutdown
}

pub fn is_shutdown_requested(flag: &AtomicBool) -> bool {
    flag.load(Ordering::SeqCst)
}
