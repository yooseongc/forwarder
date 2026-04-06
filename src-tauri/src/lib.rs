mod commands;
mod config;
mod credential;
pub mod error;
mod ssh;
mod state;

/// Shared test lock for tests that mutate FORWARDER_CONFIG_DIR env var.
#[cfg(test)]
pub(crate) static ENV_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

use state::AppState;
use tauri::{
    AppHandle, Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Another instance tried to launch — show existing window
            show_main_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .setup(|app| {
            setup_tray(app)?;

            // Hide window if launched with --minimized (e.g. autostart)
            let minimized = std::env::args().any(|a| a == "--minimized");
            if minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = commands::auto_connect_on_startup(app_handle).await {
                    log::error!("Auto-connect failed: {}", e);
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_profiles,
            commands::save_profile,
            commands::delete_profile,
            commands::connect_profile,
            commands::disconnect_profile,
            commands::get_all_status,
            commands::save_credential,
            commands::delete_credential,
            commands::has_credential,
            commands::reconnect_profile,
            commands::enable_tunnel,
            commands::get_autostart_enabled,
            commands::set_autostart_enabled,
            commands::open_key_file_dialog,
            commands::ping_host,
            commands::export_config,
            commands::import_config,
            commands::reset_host_key,
            commands::reset_all_host_keys,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItemBuilder::with_id("show", "창 열기").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "종료").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .tooltip("SSH Forwarder")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "quit" => {
                disconnect_all(app);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Gracefully disconnect all active SSH sessions before shutdown.
fn disconnect_all(app: &AppHandle) {
    let state = app.state::<AppState>();
    // Use blocking lock since we're in a sync context (tray event handler)
    let connections = state.connections.clone();
    if let Ok(mut guard) = connections.try_lock() {
        for (id, conn) in guard.iter_mut() {
            log::info!("Disconnecting profile on exit: {}", id);
            conn.set_disconnected();
        }
        guard.clear();
    };
}
