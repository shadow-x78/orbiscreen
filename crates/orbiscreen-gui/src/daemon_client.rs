use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use orbiscreen_core::ports;
use orbiscreen_core::report::{DoctorReport, StatusReport};
use tokio::process::Command;

#[derive(Debug, Clone, Default)]
pub struct DaemonStatus {
    pub report: StatusReport,
    pub udp_port: u16,
    pub local_ips: Vec<String>,
}

impl Deref for DaemonStatus {
    type Target = StatusReport;

    fn deref(&self) -> &Self::Target {
        &self.report
    }
}

impl DerefMut for DaemonStatus {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.report
    }
}

impl DaemonStatus {
    pub fn new(report: StatusReport) -> Self {
        Self {
            udp_port: ports::udp_port(report.signaling_port),
            report,
            local_ips: Vec::new(),
        }
    }

    pub fn token(&self) -> Option<String> {
        orbiscreen_core::paths::read_trimmed(&orbiscreen_core::default_token_path())
            .ok()
            .filter(|token| !token.is_empty())
    }
}

pub struct DaemonClient;

impl DaemonClient {
    pub async fn get_status() -> DaemonStatus {
        let dbus_result = tokio::time::timeout(std::time::Duration::from_millis(500), async {
            let conn = zbus::Connection::session().await?;
            let proxy = zbus::Proxy::new(
                &conn,
                "com.orbiscreen.Daemon",
                "/com/orbiscreen/Daemon",
                "com.orbiscreen.Daemon",
            )
            .await?;
            let json_str = proxy.call::<_, _, String>("GetStatus", &()).await?;
            let report = serde_json::from_str::<StatusReport>(&json_str)
                .map_err(|e| zbus::Error::Failure(e.to_string()))?;
            let mut status = DaemonStatus::new(report);
            status.local_ips = Self::detect_local_ips();
            Ok::<DaemonStatus, zbus::Error>(status)
        })
        .await;

        if let Ok(Ok(status)) = dbus_result {
            return status;
        }

        let is_running = Self::check_process_running().await;
        let cfg_path = orbiscreen_core::default_config_path();
        let (cfg_w, cfg_h, cfg_fps) = if let Ok(s) = std::fs::read_to_string(&cfg_path) {
            if let Ok(cfg) = orbiscreen_core::load_config(&s) {
                (
                    cfg.display.width,
                    cfg.display.height,
                    cfg.display.refresh_rate_hz,
                )
            } else {
                (1920, 1080, 60)
            }
        } else {
            (1920, 1080, 60)
        };
        let mut status = DaemonStatus::new(StatusReport {
            running: is_running,
            display_width: cfg_w,
            display_height: cfg_h,
            display_fps: cfg_fps,
            ..StatusReport::new()
        });
        status.local_ips = Self::detect_local_ips();
        status
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
                "com.orbiscreen.Daemon",
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

        let systemctl = Command::new("systemctl")
            .args(["--user", "stop", "orbiscreen"])
            .output()
            .await;
        if matches!(&systemctl, Ok(out) if out.status.success()) {
            return Ok("Stopped orbiscreen systemd service".to_string());
        }

        let cli = Command::new("orbiscreen").arg("stop").output().await;
        match cli {
            Ok(out) if out.status.success() => Ok("Stopped orbiscreen daemon".to_string()),
            Ok(out) => Err(format!(
                "stop failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            Err(e) => Err(format!("failed to stop daemon: {e}")),
        }
    }

    pub async fn restart_service() -> Result<String, String> {
        Self::stop_service().await?;
        Self::start_service().await
    }

    pub async fn run_doctor_check() -> Result<DoctorReport, String> {
        let out = Command::new("orbiscreen")
            .args(["doctor", "--json"])
            .output()
            .await
            .map_err(|e| format!("Failed to run doctor: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "doctor failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        serde_json::from_slice::<DoctorReport>(&out.stdout)
            .map_err(|e| format!("invalid doctor report: {e}"))
    }

    pub async fn run_doctor_fix() -> Result<String, String> {
        let out = Command::new("orbiscreen")
            .args(["doctor", "--fix", "--yes"])
            .output()
            .await
            .map_err(|e| format!("Failed to run doctor fix: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "doctor fix failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }

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

    pub async fn set_display_settings(width: u32, height: u32, fps: u32) -> Result<String, String> {
        let width = width.clamp(
            orbiscreen_core::DisplayConfig::MIN_WIDTH,
            orbiscreen_core::DisplayConfig::MAX_WIDTH,
        );
        let height = height.clamp(
            orbiscreen_core::DisplayConfig::MIN_HEIGHT,
            orbiscreen_core::DisplayConfig::MAX_HEIGHT,
        );
        let fps = fps.clamp(
            orbiscreen_core::DisplayConfig::MIN_REFRESH_RATE_HZ,
            orbiscreen_core::DisplayConfig::MAX_REFRESH_RATE_HZ,
        );

        let conn = zbus::Connection::session()
            .await
            .map_err(|e| format!("D-Bus unavailable: {e}"))?;
        let proxy = zbus::Proxy::new(
            &conn,
            "com.orbiscreen.Daemon",
            "/com/orbiscreen/Daemon",
            "com.orbiscreen.Daemon",
        )
        .await
        .map_err(|e| format!("daemon proxy: {e}"))?;
        proxy
            .call::<_, _, String>("SetResolution", &(width, height, fps))
            .await
            .map_err(|e| format!("SetResolution failed: {e}"))
    }

    fn autostart_file_path() -> PathBuf {
        gtk::glib::user_config_dir()
            .join("autostart")
            .join("orbiscreen.desktop")
    }

    pub fn open_browser(url: &str) -> Result<(), String> {
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
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("failed to launch a browser: {e}"))
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
