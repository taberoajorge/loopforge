use crate::loop_manager::LoopManagerState;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

const TRAY_ID: &str = "loopforge-tray";

pub fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let show_item = MenuItemBuilder::with_id("show_window", "Show Window").build(app)?;
    let pause_all_item = MenuItemBuilder::with_id("pause_all", "Pause All Loops").build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .item(&pause_all_item)
        .item(&separator)
        .item(&quit_item)
        .build()?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(
            tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("tray icon must be valid"),
        )
        .menu(&menu)
        .tooltip("LoopForge")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show_window" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "pause_all" => {
                let state = app.state::<LoopManagerState>();
                state.shutdown_all();
                update_tooltip(app, 0);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

pub fn update_tooltip(app: &AppHandle, active_loops: usize) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let tooltip = if active_loops == 0 {
            "LoopForge".to_string()
        } else if active_loops == 1 {
            "LoopForge \u{2014} 1 active loop".to_string()
        } else {
            format!("LoopForge \u{2014} {active_loops} active loops")
        };
        let _ = tray.set_tooltip(Some(tooltip.as_str()));
    }
}
