use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use orbiscreen_core::Config;
use orbiscreen_transport::Stats;
use zbus::interface;

#[derive(Debug)]
pub struct DaemonHandles {
    pub is_running: Arc<AtomicBool>,
    pub stats: Arc<Stats>,
    pub config: Config,
    pub encoder: &'static str,
    pub capture_backend: &'static str,
    pub shutdown_tx: tokio::sync::watch::Sender<bool>,
}

#[derive(Clone, Debug)]
pub struct OrbiscreenDbusServer {
    handles: Arc<DaemonHandles>,
}

impl OrbiscreenDbusServer {
    pub fn new(handles: Arc<DaemonHandles>) -> Self {
        Self { handles }
    }
}

#[interface(name = "com.orbiscreen.Daemon")]
impl OrbiscreenDbusServer {
    async fn get_status(&self) -> String {
        serde_json::json!({
            "running": self.handles.is_running.load(Ordering::SeqCst),
            "frames_forwarded": self.handles.stats.frames_forwarded(),
            "active_clients": self.handles.stats.active_clients(),
            "total_clients": self.handles.stats.total_clients(),
            "auth_failures": self.handles.stats.auth_failures(),
            "usb_devices": self.handles.stats.usb_devices(),
            "encoder": self.handles.encoder,
            "capture_backend": self.handles.capture_backend,
        })
        .to_string()
    }

    async fn stop(&self) -> String {
        if self.handles.is_running.swap(false, Ordering::SeqCst) {
            let _ = self.handles.shutdown_tx.send(true);
            "Orbiscreen daemon shutting down".to_string()
        } else {
            "Orbiscreen is not running".to_string()
        }
    }

    async fn list_clients(&self) -> Vec<String> {
        let active = self.handles.stats.active_clients();
        let total = self.handles.stats.total_clients();
        vec![format!(
            "HTTP MPEG-TS /stream: {active} active client(s), {total} total connection(s)"
        )]
    }

    async fn get_config(&self) -> String {
        match orbiscreen_core::dump_config(&self.handles.config) {
            Ok(toml) => toml,
            Err(e) => format!("config serialize error: {e}"),
        }
    }
}

pub async fn call_stop(conn: &zbus::Connection) -> zbus::Result<String> {
    let proxy = zbus::Proxy::new(
        conn,
        "com.orbiscreen.Daemon",
        "/com/orbiscreen/Daemon",
        "com.orbiscreen.Daemon",
    )
    .await?;
    proxy.call("Stop", &()).await
}

pub async fn request_stop() -> zbus::Result<String> {
    let conn = zbus::connection::Builder::session()?.build().await?;
    call_stop(&conn).await
}

pub async fn run_dbus_server(handles: Arc<DaemonHandles>) -> zbus::Result<()> {
    let server = OrbiscreenDbusServer::new(handles);
    let _conn = zbus::connection::Builder::session()?
        .name("com.orbiscreen.Daemon")?
        .serve_at("/com/orbiscreen/Daemon", server)?
        .build()
        .await?;

    std::future::pending::<()>().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_handles() -> Arc<DaemonHandles> {
        let (shutdown_tx, _shutdown_rx) = tokio::sync::watch::channel(false);
        Arc::new(DaemonHandles {
            is_running: Arc::new(AtomicBool::new(true)),
            stats: Arc::new(Stats::default()),
            config: Config::default(),
            encoder: "x264",
            capture_backend: "Wayland",
            shutdown_tx,
        })
    }

    #[tokio::test]
    async fn status_contains_live_stats() {
        let server = OrbiscreenDbusServer::new(test_handles());
        let status = server.get_status().await;
        assert!(status.contains("\"running\":true"));
        assert!(status.contains("\"frames_forwarded\":0"));
        assert!(status.contains("\"auth_failures\":0"));
        assert!(status.contains("\"usb_devices\":0"));
        let value: serde_json::Value = serde_json::from_str(&status).unwrap();
        assert_eq!(value["encoder"], "x264");
    }

    #[tokio::test]
    async fn stop_flips_running_flag_and_signals() {
        let handles = test_handles();
        let mut shutdown_rx = handles.shutdown_tx.subscribe();
        let server = OrbiscreenDbusServer::new(handles.clone());
        let reply = server.stop().await;
        assert!(reply.contains("shutting down"));
        assert!(!handles.is_running.load(Ordering::SeqCst));
        assert!(*shutdown_rx.borrow_and_update());
        assert!(server.stop().await.contains("not running"));
    }

    #[tokio::test]
    async fn list_clients_reports_counts() {
        let handles = test_handles();
        handles.stats.client_started();
        let server = OrbiscreenDbusServer::new(handles);
        let clients = server.list_clients().await;
        assert_eq!(clients.len(), 1);
        assert!(clients[0].contains("1 active"));
    }

    #[tokio::test]
    async fn get_config_returns_current_toml() {
        let server = OrbiscreenDbusServer::new(test_handles());
        let cfg = server.get_config().await;
        assert!(cfg.contains("[display]"));
        assert!(cfg.contains("width = 1920"));
    }
}
