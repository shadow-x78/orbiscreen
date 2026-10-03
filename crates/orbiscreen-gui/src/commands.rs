use crate::daemon_client::{DaemonClient, DaemonStatus};

#[tauri::command]
pub async fn get_status() -> DaemonStatus {
    DaemonClient::get_status().await
}

#[tauri::command]
pub async fn start_service() -> Result<String, String> {
    DaemonClient::start_service().await
}

#[tauri::command]
pub async fn stop_service() -> Result<String, String> {
    DaemonClient::stop_service().await
}

#[tauri::command]
pub async fn restart_service() -> Result<String, String> {
    DaemonClient::restart_service().await
}

#[tauri::command]
pub async fn run_doctor_check() -> Result<String, String> {
    DaemonClient::run_doctor_check().await
}

#[tauri::command]
pub async fn run_doctor_fix() -> Result<String, String> {
    DaemonClient::run_doctor_fix().await
}

#[tauri::command]
pub fn get_autostart() -> bool {
    DaemonClient::is_autostart_enabled()
}

#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    DaemonClient::set_autostart(enabled)
}

#[tauri::command]
pub fn open_browser(url: String) -> Result<(), String> {
    let rest = match url.split_once("://") {
        Some((scheme, rest))
            if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") =>
        {
            rest
        }
        _ => return Err("only http and https URLs can be opened".into()),
    };
    if rest.is_empty() || rest.contains(['\n', '\r', '\0']) {
        return Err("malformed URL".into());
    }
    std::process::Command::new("xdg-open")
        .arg(&url)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("failed to launch a browser: {e}"))
}

#[tauri::command]
pub async fn set_display_settings(width: u32, height: u32, fps: u32) -> Result<String, String> {
    DaemonClient::set_display_settings(width, height, fps).await
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
