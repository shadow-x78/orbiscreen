// Orbiscreen - daemon_client.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub frames_forwarded: u64,
    pub active_clients: u32,
    pub total_clients: u64,
    pub auth_failures: u64,
    pub usb_devices: u32,
    #[serde(default)]
    pub usb_connected_devices: Vec<String>,
    #[serde(default)]
    pub usb_aoa_ready: bool,
    pub encoder: String,
    pub capture_backend: String,
    pub display_width: u32,
    pub display_height: u32,
    pub display_fps: u32,
    pub signaling_port: u16,
    pub udp_port: u16,
    pub local_ips: Vec<String>,
    pub session_token: Option<String>,
}

impl Default for DaemonStatus {
    fn default() -> Self {
        Self {
            running: false,
            frames_forwarded: 0,
            active_clients: 0,
            total_clients: 0,
            auth_failures: 0,
            usb_devices: 0,
            usb_connected_devices: Vec::new(),
            usb_aoa_ready: false,
            encoder: "Unknown".to_string(),
            capture_backend: "Unknown".to_string(),
            display_width: 1920,
            display_height: 1080,
            display_fps: 60,
            signaling_port: 8788,
            udp_port: 8789,
            local_ips: Vec::new(),
            session_token: None,
        }
    }
}

pub struct DaemonClient;

impl DaemonClient {
    pub async fn get_status() -> DaemonStatus {
        let dbus_result = tokio::time::timeout(std::time::Duration::from_millis(500), async {
            let conn = zbus::Connection::session().await?;
            let proxy = zbus::Proxy::new(
                &conn,
                "org.shadow-x78.Orbiscreen",
                "/com/orbiscreen/Daemon",
                "com.orbiscreen.Daemon",
            )
            .await?;
            let json_str = proxy.call::<_, _, String>("GetStatus", &()).await?;
            let mut status = serde_json::from_str::<DaemonStatus>(&json_str)
                .map_err(|e| zbus::Error::Failure(e.to_string()))?;
            status.udp_port = status.signaling_port.saturating_add(1);
            status.local_ips = Self::detect_local_ips();
            Ok::<DaemonStatus, zbus::Error>(status)
        })
        .await;

        if let Ok(Ok(status)) = dbus_result {
            return status;
        }

        let is_running = Self::check_process_running().await;
        DaemonStatus {
            running: is_running,
            local_ips: Self::detect_local_ips(),
            ..Default::default()
        }
    }

    pub async fn start_service() -> Result<String, String> {
        let systemd_res = Command::new("systemctl")
            .args(["--user", "start", "orbiscreen"])
            .output()
            .await;

        if let Ok(out) = systemd_res {
            if out.status.success() {
                return Ok("Started orbiscreen systemd service".to_string());
            }
        }

        let spawn_res = Command::new("orbiscreen")
            .args(["start", "--daemon"])
            .spawn();

        match spawn_res {
            Ok(_) => Ok("Spawned background orbiscreen daemon".to_string()),
            Err(e) => Err(format!("Failed to start daemon: {e}")),
        }
    }

    pub async fn stop_service() -> Result<String, String> {
        if let Ok(conn) = zbus::Connection::session().await {
            if let Ok(proxy) = zbus::Proxy::new(
                &conn,
                "org.shadow-x78.Orbiscreen",
                "/com/orbiscreen/Daemon",
                "com.orbiscreen.Daemon",
            )
            .await
            {
                if let Ok(reply) = proxy.call::<_, _, String>("Stop", &()).await {
                    return Ok(reply);
                }
            }
        }

        let _ = Command::new("systemctl")
            .args(["--user", "stop", "orbiscreen"])
            .output()
            .await;

        let _ = Command::new("orbiscreen").arg("stop").output().await;

        Ok("Stop command sent".to_string())
    }

    pub async fn restart_service() -> Result<String, String> {
        let _ = Self::stop_service().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
        Self::start_service().await
    }

    pub async fn run_doctor_check() -> Result<String, String> {
        let out = Command::new("orbiscreen")
            .args(["doctor", "--json"])
            .output()
            .await
            .map_err(|e| format!("Failed to run doctor: {e}"))?;

        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    pub async fn run_doctor_fix() -> Result<String, String> {
        let out = Command::new("orbiscreen")
            .args(["doctor", "--fix", "--yes"])
            .output()
            .await
            .map_err(|e| format!("Failed to run doctor fix: {e}"))?;

        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    pub fn is_autostart_enabled() -> bool {
        let autostart_file = Self::autostart_file_path();
        autostart_file.exists()
    }

    pub fn set_autostart(enabled: bool) -> Result<(), String> {
        let autostart_file = Self::autostart_file_path();
        if enabled {
            if let Some(parent) = autostart_file.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let desktop_content = "[Desktop Entry]\nType=Application\nName=Orbiscreen\nExec=orbiscreen start --daemon\nHidden=false\nNoDisplay=false\nX-GNOME-Autostart-enabled=true\n";
            std::fs::write(&autostart_file, desktop_content).map_err(|e| e.to_string())?;
        } else if autostart_file.exists() {
            std::fs::remove_file(&autostart_file).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn autostart_file_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".config/autostart/orbiscreen.desktop")
    }

    fn detect_local_ips() -> Vec<String> {
        let mut ips = Vec::new();
        if let Ok(addrs) = std::net::UdpSocket::bind("0.0.0.0:0") {
            if addrs.connect("8.8.8.8:80").is_ok() {
                if let Ok(local_addr) = addrs.local_addr() {
                    ips.push(local_addr.ip().to_string());
                }
            }
        }
        if ips.is_empty() {
            ips.push("127.0.0.1".to_string());
        }
        ips
    }

    async fn check_process_running() -> bool {
        let out = Command::new("pgrep")
            .args(["-x", "orbiscreen"])
            .output()
            .await;
        if let Ok(res) = out {
            res.status.success()
        } else {
            false
        }
    }
}
