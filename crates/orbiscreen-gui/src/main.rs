// Orbiscreen - main.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

mod commands;
mod daemon_client;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

fn cleanup_legacy_desktop_file() {
    if let Some(home) = std::env::var_os("HOME") {
        let legacy_path =
            std::path::PathBuf::from(home).join(".local/share/applications/orbiscreen.desktop");
        if legacy_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&legacy_path) {
                if content.contains("Exec=orbiscreen start") {
                    let _ = std::fs::remove_file(&legacy_path);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    if std::env::var_os("__NV_DISABLE_EXPLICIT_SYNC").is_none() {
        std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
    }
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
    cleanup_legacy_desktop_file();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "orbiscreen_gui=info".into()),
        )
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let toggle = MenuItem::with_id(app, "toggle", "Open Dashboard", true, None::<&str>)?;
            let start = MenuItem::with_id(app, "start", "Start Service", true, None::<&str>)?;
            let stop = MenuItem::with_id(app, "stop", "Stop Service", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&toggle, &start, &stop, &quit])?;

            let icon = app.default_window_icon().cloned();
            if let Some(window) = app.get_webview_window("main") {
                if let Some(ref ic) = icon {
                    let _ = window.set_icon(ic.clone());
                }
            }
            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Orbiscreen Host Control Center");

            if let Some(icon) = icon {
                tray_builder = tray_builder.icon(icon);
            }

            let _tray = tray_builder
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "start" => {
                        tauri::async_runtime::spawn(async {
                            let _ = daemon_client::DaemonClient::start_service().await;
                        });
                    }
                    "stop" => {
                        tauri::async_runtime::spawn(async {
                            let _ = daemon_client::DaemonClient::stop_service().await;
                        });
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::start_service,
            commands::stop_service,
            commands::restart_service,
            commands::run_doctor_check,
            commands::run_doctor_fix,
            commands::get_autostart,
            commands::set_autostart,
            commands::open_browser
        ])
        .run(tauri::generate_context!())
        .expect("error while running orbiscreen-gui");
}
