// Orbiscreen - D-Bus Session Service Interface (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use zbus::interface;

#[derive(Debug)]
pub struct OrbiscreenDbusServer {
    is_running: Arc<AtomicBool>,
}

impl OrbiscreenDbusServer {
    pub fn new(is_running: Arc<AtomicBool>) -> Self {
        Self { is_running }
    }
}

#[interface(name = "com.orbiscreen.Daemon")]
impl OrbiscreenDbusServer {
    async fn get_status(&self) -> String {
        if self.is_running.load(Ordering::SeqCst) {
            "Running".to_string()
        } else {
            "Stopped".to_string()
        }
    }

    async fn start(&self) -> String {
        self.is_running.store(true, Ordering::SeqCst);
        "Orbiscreen daemon started via D-Bus".to_string()
    }

    async fn stop(&self) -> String {
        self.is_running.store(false, Ordering::SeqCst);
        "Orbiscreen daemon stopped via D-Bus".to_string()
    }

    async fn list_clients(&self) -> Vec<String> {
        vec![
            "HTTP Direct /stream".to_string(),
            "WebRTC Signaling Active".to_string(),
        ]
    }

    async fn get_config(&self) -> String {
        r#"{"width":1920,"height":1080,"refresh_rate":60,"encoder":"auto"}"#.to_string()
    }
}

pub async fn run_dbus_server(is_running: Arc<AtomicBool>) -> zbus::Result<()> {
    let server = OrbiscreenDbusServer::new(is_running);
    let _conn = zbus::connection::Builder::session()?
        .name("com.orbiscreen.Daemon")?
        .serve_at("/com/orbiscreen/Daemon", server)?
        .build()
        .await?;

    // Keep connection alive
    tokio::task::id();
    Ok(())
}
