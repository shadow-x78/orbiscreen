use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tracing::warn;

use orbiscreen_core::Config;
use orbiscreen_transport::Stats;
use zbus::interface;

#[derive(Debug)]
pub struct DaemonHandles {
    pub owner_uid: u32,
    pub is_running: Arc<AtomicBool>,
    pub stats: Arc<Stats>,
    pub config: std::sync::RwLock<Config>,
    pub encoder: &'static str,
    pub capture_backend: &'static str,
    pub shutdown_tx: tokio::sync::watch::Sender<bool>,
    pub displays: Option<orbiscreen_transport::DisplayCtl>,
    pub pairing: std::sync::Arc<orbiscreen_transport::pairing::PairingRegistry>,
}

#[derive(Clone, Debug)]
pub struct OrbiscreenDbusServer {
    handles: Arc<DaemonHandles>,
}

impl OrbiscreenDbusServer {
    pub fn new(handles: Arc<DaemonHandles>) -> Self {
        Self { handles }
    }

    async fn caller_uid(
        &self,
        connection: &zbus::Connection,
        header: &zbus::message::Header<'_>,
    ) -> Result<u32, String> {
        let sender = header
            .sender()
            .map(|s| s.to_owned())
            .ok_or_else(|| "cannot determine caller".to_string())?;
        let proxy = zbus::fdo::DBusProxy::builder(connection)
            .destination("org.freedesktop.DBus")
            .map_err(|e| e.to_string())?
            .path("/org/freedesktop/DBus")
            .map_err(|e| e.to_string())?
            .interface("org.freedesktop.DBus")
            .map_err(|e| e.to_string())?
            .build()
            .await
            .map_err(|e| e.to_string())?;
        proxy
            .get_connection_unix_user(sender.into())
            .await
            .map_err(|e| format!("cannot resolve caller credentials: {e}"))
    }

    async fn authorize(
        &self,
        connection: &zbus::Connection,
        header: &zbus::message::Header<'_>,
        method: &str,
    ) -> Result<(), String> {
        let caller = self.caller_uid(connection, header).await?;
        if self.allows(caller) {
            Ok(())
        } else {
            let denied = format!(
                "permission denied: {method} may only be called by uid {} (caller is {caller})",
                self.handles.owner_uid
            );
            warn!("{denied}");
            Err(denied)
        }
    }
}

pub fn current_uid() -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata("/proc/self")
            .map(|m| m.uid())
            .unwrap_or_else(|e| {
                warn!("cannot read /proc/self to determine the daemon uid ({e}); assuming 0");
                0
            })
    }
    #[cfg(not(unix))]
    {
        0
    }
}

#[interface(name = "com.orbiscreen.Daemon")]
impl OrbiscreenDbusServer {
    async fn get_status(&self) -> String {
        let (dw, dh, dfps, sport) = if let Ok(cfg) = self.handles.config.read() {
            (
                cfg.display.width,
                cfg.display.height,
                cfg.display.refresh_rate_hz,
                cfg.transport.signaling_port,
            )
        } else {
            (1920, 1080, 60, 8788)
        };
        serde_json::json!({
            "running": self.handles.is_running.load(Ordering::SeqCst),
            "frames_forwarded": self.handles.stats.frames_forwarded(),
            "active_clients": self.handles.stats.active_clients(),
            "total_clients": self.handles.stats.total_clients(),
            "auth_failures": self.handles.stats.auth_failures(),
            "usb_devices": self.handles.stats.usb_devices(),
            "usb_connected_devices": self.handles.stats.usb_connected_names(),
            "usb_aoa_ready": self.handles.stats.is_usb_aoa_ready(),
            "encoder": self.handles.encoder,
            "capture_backend": self.handles.capture_backend,
            "display_width": dw,
            "display_height": dh,
            "display_fps": dfps,
            "signaling_port": sport,
            "version": env!("CARGO_PKG_VERSION"),
        })
        .to_string()
    }

    async fn list_clients(&self) -> Vec<String> {
        let active = self.handles.stats.active_clients();
        let total = self.handles.stats.total_clients();
        vec![format!(
            "HTTP MPEG-TS /stream: {active} active client(s), {total} total connection(s)"
        )]
    }

    async fn set_resolution(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: zbus::message::Header<'_>,
        width: u32,
        height: u32,
        fps: u32,
    ) -> String {
        if let Err(denied) = self.authorize(connection, &header, "SetResolution").await {
            return denied;
        }
        self.apply_resolution(width, height, fps).await
    }

    async fn stop(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) -> String {
        if let Err(denied) = self.authorize(connection, &header, "Stop").await {
            return denied;
        }
        self.stop_daemon()
    }

    async fn get_config(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) -> String {
        if let Err(denied) = self.authorize(connection, &header, "GetConfig").await {
            return denied;
        }
        self.read_config()
    }

    async fn list_pairings(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) -> String {
        if let Err(denied) = self.authorize(connection, &header, "ListPairings").await {
            return denied;
        }
        self.pair_list_json()
    }

    async fn approve_pairing(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: zbus::message::Header<'_>,
        request_id: String,
    ) -> String {
        if let Err(denied) = self.authorize(connection, &header, "ApprovePairing").await {
            return denied;
        }
        self.pair_approve_json(&request_id)
    }

    async fn deny_pairing(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: zbus::message::Header<'_>,
        request_id: String,
    ) -> String {
        if let Err(denied) = self.authorize(connection, &header, "DenyPairing").await {
            return denied;
        }
        self.pair_deny_json(&request_id)
    }

    async fn revoke_pairing(
        &self,
        #[zbus(connection)] connection: &zbus::Connection,
        #[zbus(header)] header: zbus::message::Header<'_>,
        client_id: String,
    ) -> String {
        if let Err(denied) = self.authorize(connection, &header, "RevokePairing").await {
            return denied;
        }
        self.pair_revoke_json(&client_id)
    }
}

impl OrbiscreenDbusServer {
    fn allows(&self, caller_uid: u32) -> bool {
        caller_uid == self.handles.owner_uid
    }

    async fn apply_resolution(&self, width: u32, height: u32, fps: u32) -> String {
        let width = width.clamp(320, 7680);
        let height = height.clamp(240, 4320);
        let fps = fps.clamp(30, 240);

        if let Ok(mut cfg) = self.handles.config.write() {
            cfg.display.width = width;
            cfg.display.height = height;
            cfg.display.refresh_rate_hz = fps;
            cfg.display.pinned = true;
            let config_path = orbiscreen_core::default_config_path();
            if let Some(parent) = config_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(toml) = orbiscreen_core::dump_config(&cfg) {
                let _ = std::fs::write(&config_path, toml);
            }
        }

        if let Some(ctl) = &self.handles.displays {
            ctl.set_defaults(width, height, fps).await;
            if let Some(info) = ctl.lookup(None).await {
                let _ = ctl.resize(&info.id, width, height, Some(fps)).await;
            }
            return format!("Resolution updated to {width}x{height}@{fps}Hz");
        }

        let target_output = "Virtual-ORBISCREEN";
        let mode_str = format!("output.{target_output}.mode.{width}x{height}@{fps}");
        let res = tokio::process::Command::new("kscreen-doctor")
            .arg(&mode_str)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await;

        match res {
            Ok(status) if status.success() => {
                format!("Resolution updated to {width}x{height}@{fps}Hz via kscreen-doctor")
            }
            _ => {
                format!("Resolution saved in configuration: {width}x{height}@{fps}Hz")
            }
        }
    }

    fn stop_daemon(&self) -> String {
        if self.handles.is_running.swap(false, Ordering::SeqCst) {
            let _ = self.handles.shutdown_tx.send(true);
            "Orbiscreen daemon shutting down".to_string()
        } else {
            "Orbiscreen is not running".to_string()
        }
    }

    fn read_config(&self) -> String {
        if let Ok(cfg) = self.handles.config.read() {
            match orbiscreen_core::dump_config(&cfg) {
                Ok(toml) => toml,
                Err(e) => format!("config serialize error: {e}"),
            }
        } else {
            "config read error".to_string()
        }
    }

    fn pair_list_json(&self) -> String {
        let clients: Vec<_> = self
            .handles
            .pairing
            .clients()
            .into_iter()
            .filter(|c| c.status == orbiscreen_transport::pairing::ClientStatus::Approved)
            .collect();
        serde_json::json!({
            "requests": self.handles.pairing.pending_requests(),
            "clients": clients,
        })
        .to_string()
    }

    fn pair_resolve_request_id(&self, request_id: &str) -> Option<String> {
        if !request_id.is_empty() {
            return Some(request_id.to_string());
        }
        self.handles
            .pairing
            .pending_requests()
            .last()
            .map(|r| r.request_id.clone())
    }

    fn pair_approve_json(&self, request_id: &str) -> String {
        let Some(id) = self.pair_resolve_request_id(request_id) else {
            return serde_json::json!({
                "ok": false,
                "error": "no pending pairing request",
            })
            .to_string();
        };
        match self.handles.pairing.approve(&id) {
            Ok(Some(client)) => serde_json::json!({
                "ok": true,
                "client_id": client.client_id,
                "label": client.label,
            })
            .to_string(),
            Ok(None) => serde_json::json!({
                "ok": false,
                "error": "unknown request",
            })
            .to_string(),
            Err(_) => serde_json::json!({
                "ok": false,
                "error": "pairing storage unavailable",
            })
            .to_string(),
        }
    }

    fn pair_deny_json(&self, request_id: &str) -> String {
        let Some(id) = self.pair_resolve_request_id(request_id) else {
            return serde_json::json!({
                "ok": false,
                "error": "no pending pairing request",
            })
            .to_string();
        };
        match self.handles.pairing.deny(&id) {
            Ok(removed) => serde_json::json!({ "ok": removed }).to_string(),
            Err(_) => serde_json::json!({
                "ok": false,
                "error": "pairing storage unavailable",
            })
            .to_string(),
        }
    }

    fn pair_revoke_json(&self, client_id: &str) -> String {
        match self.handles.pairing.revoke(client_id) {
            Ok(removed) => serde_json::json!({ "ok": removed }).to_string(),
            Err(_) => serde_json::json!({
                "ok": false,
                "error": "pairing storage unavailable",
            })
            .to_string(),
        }
    }
}

pub async fn call_stop(conn: &zbus::Connection) -> zbus::Result<String> {
    let proxy = zbus::Proxy::new(
        conn,
        "org.shadow-x78.Orbiscreen",
        "/com/orbiscreen/Daemon",
        "com.orbiscreen.Daemon",
    )
    .await?;
    match tokio::time::timeout(
        std::time::Duration::from_millis(1000),
        proxy.call::<_, _, String>("Stop", &()),
    )
    .await
    {
        Ok(Ok(res)) => Ok(res),
        _ => {
            let fallback = zbus::Proxy::new(
                conn,
                "com.orbiscreen.Daemon",
                "/com/orbiscreen/Daemon",
                "com.orbiscreen.Daemon",
            )
            .await?;
            match tokio::time::timeout(
                std::time::Duration::from_millis(1000),
                fallback.call::<_, _, String>("Stop", &()),
            )
            .await
            {
                Ok(res) => res,
                Err(_) => Err(zbus::Error::Failure("D-Bus request timed out".to_string())),
            }
        }
    }
}

pub async fn request_stop() -> zbus::Result<String> {
    let conn = zbus::connection::Builder::session()?.build().await?;
    call_stop(&conn).await
}

pub async fn call_status(conn: &zbus::Connection) -> zbus::Result<String> {
    let proxy = zbus::Proxy::new(
        conn,
        "org.shadow-x78.Orbiscreen",
        "/com/orbiscreen/Daemon",
        "com.orbiscreen.Daemon",
    )
    .await?;
    match tokio::time::timeout(
        std::time::Duration::from_millis(800),
        proxy.call::<_, _, String>("GetStatus", &()),
    )
    .await
    {
        Ok(Ok(res)) => Ok(res),
        _ => {
            let fallback = zbus::Proxy::new(
                conn,
                "com.orbiscreen.Daemon",
                "/com/orbiscreen/Daemon",
                "com.orbiscreen.Daemon",
            )
            .await?;
            match tokio::time::timeout(
                std::time::Duration::from_millis(800),
                fallback.call::<_, _, String>("GetStatus", &()),
            )
            .await
            {
                Ok(res) => res,
                Err(_) => Err(zbus::Error::Failure("D-Bus request timed out".to_string())),
            }
        }
    }
}

pub async fn request_status() -> zbus::Result<String> {
    let conn = zbus::connection::Builder::session()?.build().await?;
    call_status(&conn).await
}

pub async fn call_set_resolution(
    conn: &zbus::Connection,
    width: u32,
    height: u32,
    fps: u32,
) -> zbus::Result<String> {
    let proxy = zbus::Proxy::new(
        conn,
        "org.shadow-x78.Orbiscreen",
        "/com/orbiscreen/Daemon",
        "com.orbiscreen.Daemon",
    )
    .await?;
    match tokio::time::timeout(
        std::time::Duration::from_millis(1500),
        proxy.call::<_, _, String>("SetResolution", &(width, height, fps)),
    )
    .await
    {
        Ok(Ok(res)) => Ok(res),
        _ => {
            let fallback = zbus::Proxy::new(
                conn,
                "com.orbiscreen.Daemon",
                "/com/orbiscreen/Daemon",
                "com.orbiscreen.Daemon",
            )
            .await?;
            match tokio::time::timeout(
                std::time::Duration::from_millis(1500),
                fallback.call::<_, _, String>("SetResolution", &(width, height, fps)),
            )
            .await
            {
                Ok(res) => res,
                Err(_) => Err(zbus::Error::Failure("D-Bus request timed out".to_string())),
            }
        }
    }
}

pub async fn run_dbus_server(handles: Arc<DaemonHandles>) -> zbus::Result<()> {
    let server = OrbiscreenDbusServer::new(handles);
    let _conn = zbus::connection::Builder::session()?
        .name("org.shadow-x78.Orbiscreen")?
        .name("com.orbiscreen.Daemon")?
        .serve_at("/com/orbiscreen/Daemon", server)?
        .build()
        .await?;

    std::future::pending::<()>().await;
    Ok(())
}

async fn call_owned_method<B>(
    conn: &zbus::Connection,
    member: &str,
    body: &B,
) -> zbus::Result<String>
where
    B: zbus::export::serde::Serialize + zbus::zvariant::DynamicType,
{
    let proxy = zbus::Proxy::new(
        conn,
        "org.shadow-x78.Orbiscreen",
        "/com/orbiscreen/Daemon",
        "com.orbiscreen.Daemon",
    )
    .await?;
    match tokio::time::timeout(
        std::time::Duration::from_millis(1500),
        proxy.call::<_, B, String>(member, body),
    )
    .await
    {
        Ok(Ok(res)) => Ok(res),
        _ => {
            let fallback = zbus::Proxy::new(
                conn,
                "com.orbiscreen.Daemon",
                "/com/orbiscreen/Daemon",
                "com.orbiscreen.Daemon",
            )
            .await?;
            match tokio::time::timeout(
                std::time::Duration::from_millis(1500),
                fallback.call::<_, B, String>(member, body),
            )
            .await
            {
                Ok(res) => res,
                Err(_) => Err(zbus::Error::Failure("D-Bus request timed out".to_string())),
            }
        }
    }
}

pub async fn call_pair_list(conn: &zbus::Connection) -> zbus::Result<String> {
    call_owned_method::<()>(conn, "ListPairings", &()).await
}

pub async fn call_pair_approve(conn: &zbus::Connection, request_id: &str) -> zbus::Result<String> {
    call_owned_method(conn, "ApprovePairing", &request_id.to_string()).await
}

pub async fn call_pair_deny(conn: &zbus::Connection, request_id: &str) -> zbus::Result<String> {
    call_owned_method(conn, "DenyPairing", &request_id.to_string()).await
}

pub async fn call_pair_revoke(conn: &zbus::Connection, client_id: &str) -> zbus::Result<String> {
    call_owned_method(conn, "RevokePairing", &client_id.to_string()).await
}

pub async fn request_pair_list() -> zbus::Result<String> {
    let conn = zbus::connection::Builder::session()?.build().await?;
    call_pair_list(&conn).await
}

pub async fn request_pair_approve(request_id: &str) -> zbus::Result<String> {
    let conn = zbus::connection::Builder::session()?.build().await?;
    call_pair_approve(&conn, request_id).await
}

pub async fn request_pair_deny(request_id: &str) -> zbus::Result<String> {
    let conn = zbus::connection::Builder::session()?.build().await?;
    call_pair_deny(&conn, request_id).await
}

pub async fn request_pair_revoke(client_id: &str) -> zbus::Result<String> {
    let conn = zbus::connection::Builder::session()?.build().await?;
    call_pair_revoke(&conn, client_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_handles() -> Arc<DaemonHandles> {
        let (shutdown_tx, _shutdown_rx) = tokio::sync::watch::channel(false);
        Arc::new(DaemonHandles {
            owner_uid: current_uid(),
            is_running: Arc::new(AtomicBool::new(true)),
            stats: Arc::new(Stats::default()),
            config: std::sync::RwLock::new(Config::default()),
            encoder: "x264",
            capture_backend: "Wayland",
            shutdown_tx,
            displays: None,
            pairing: std::sync::Arc::new(
                orbiscreen_transport::pairing::PairingRegistry::load_from(None).expect("registry"),
            ),
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
        let reply = server.stop_daemon();
        assert!(reply.contains("shutting down"));
        assert!(!handles.is_running.load(Ordering::SeqCst));
        assert!(*shutdown_rx.borrow_and_update());
        assert!(server.stop_daemon().contains("not running"));
    }

    #[test]
    fn only_the_daemons_own_uid_is_authorized() {
        let handles = test_handles();
        let owner = handles.owner_uid;
        let server = OrbiscreenDbusServer::new(handles);
        assert!(server.allows(owner));
        assert!(!server.allows(owner + 1));
        assert!(!server.allows(owner.wrapping_add(u32::MAX)));
        assert_eq!(current_uid(), owner, "test handles must use the real uid");
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
        let cfg = server.read_config();
        assert!(cfg.contains("[display]"));
        assert!(cfg.contains("width = 1920"));
    }

    #[tokio::test]
    async fn set_resolution_updates_config() {
        let server = OrbiscreenDbusServer::new(test_handles());
        let reply = server.apply_resolution(2560, 1600, 90).await;
        assert!(reply.contains("2560x1600@90Hz"));
        let status = server.get_status().await;
        assert!(status.contains("\"display_width\":2560"));
        assert!(status.contains("\"display_height\":1600"));
        assert!(status.contains("\"display_fps\":90"));
    }

    fn pair_server_with_request(label: &str, peer: &str) -> (OrbiscreenDbusServer, String) {
        let handles = test_handles();
        let server = OrbiscreenDbusServer::new(handles.clone());
        let id = handles
            .pairing
            .request_pairing(label, peer)
            .expect("request accepted")
            .request_id;
        (server, id)
    }

    #[tokio::test]
    async fn pair_list_reports_pending_and_paired() {
        let (server, id) = pair_server_with_request("Tab S5e", "192.0.2.30");
        let list = server.pair_list_json();
        let value: serde_json::Value = serde_json::from_str(&list).unwrap();
        assert_eq!(
            value["requests"][0]["request_id"].as_str().unwrap(),
            id,
            "the pending request must be listed"
        );
        assert_eq!(value["requests"][0]["label"], "Tab S5e");
        assert_eq!(
            value["clients"].as_array().unwrap().len(),
            0,
            "nothing is paired before approval"
        );
    }

    #[tokio::test]
    async fn approve_pairing_issues_a_claimable_credential_slot() {
        let (server, id) = pair_server_with_request("Tab S5e", "192.0.2.30");
        let reply: serde_json::Value =
            serde_json::from_str(&server.pair_approve_json(&id)).unwrap();
        assert_eq!(reply["ok"], true, "approval must succeed: {reply}");
        assert_eq!(server.handles.pairing.clients().len(), 1);
        let pending = server.handles.pairing.pending_requests();
        assert_eq!(
            pending.len(),
            1,
            "the request stays queued until the device claims it"
        );
        assert!(
            pending[0].approved,
            "approval must mark the request as approved"
        );
        let credential = server
            .handles
            .pairing
            .claim(&id, "192.0.2.30")
            .expect("the device must be able to claim its credential after approval");
        assert!(!credential.is_empty());
        assert!(
            server.handles.pairing.pending_requests().is_empty(),
            "claim consumes the request"
        );
        assert!(
            server.handles.pairing.claim(&id, "192.0.2.30").is_none(),
            "a request must yield its credential exactly once"
        );
    }

    #[tokio::test]
    async fn approve_without_an_id_targets_the_newest_request() {
        let handles = test_handles();
        let server = OrbiscreenDbusServer::new(handles.clone());
        let first = handles
            .pairing
            .request_pairing("first", "192.0.2.30")
            .unwrap()
            .request_id;
        let second = handles
            .pairing
            .request_pairing("second", "192.0.2.31")
            .unwrap()
            .request_id;
        let reply: serde_json::Value = serde_json::from_str(&server.pair_approve_json("")).unwrap();
        assert_eq!(reply["ok"], true);
        let pending = server.handles.pairing.pending_requests();
        assert_eq!(pending.len(), 2, "both requests remain until claimed");
        assert!(
            pending.iter().any(|r| r.request_id == second && r.approved),
            "the newest request must be the approved one"
        );
        assert!(
            pending.iter().any(|r| r.request_id == first && !r.approved),
            "the oldest request must remain unapproved"
        );
    }

    #[test]
    fn approve_and_deny_unknown_ids_fail_cleanly() {
        let server = OrbiscreenDbusServer::new(test_handles());
        let approve: serde_json::Value =
            serde_json::from_str(&server.pair_approve_json("")).unwrap();
        assert_eq!(approve["ok"], false);
        assert_eq!(approve["error"], "no pending pairing request");
        let deny: serde_json::Value =
            serde_json::from_str(&server.pair_deny_json("no-such-id")).unwrap();
        assert_eq!(deny["ok"], false);
    }

    #[tokio::test]
    async fn deny_pairing_removes_the_request() {
        let (server, id) = pair_server_with_request("Tab S5e", "192.0.2.30");
        let reply: serde_json::Value = serde_json::from_str(&server.pair_deny_json(&id)).unwrap();
        assert_eq!(reply["ok"], true);
        assert!(server.handles.pairing.pending_requests().is_empty());
        assert!(server.handles.pairing.clients().is_empty());
    }

    #[tokio::test]
    async fn revoke_pairing_removes_a_paired_client() {
        let (server, id) = pair_server_with_request("Tab S5e", "192.0.2.30");
        let approved: serde_json::Value =
            serde_json::from_str(&server.pair_approve_json(&id)).unwrap();
        let client_id = approved["client_id"].as_str().unwrap().to_string();
        let reply: serde_json::Value =
            serde_json::from_str(&server.pair_revoke_json(&client_id)).unwrap();
        assert_eq!(reply["ok"], true);
        let clients = server.handles.pairing.clients();
        assert_eq!(
            clients.len(),
            1,
            "the stored entry stays for the audit trail"
        );
        assert_eq!(
            clients[0].status,
            orbiscreen_transport::pairing::ClientStatus::Revoked,
            "the paired device must be revoked"
        );
        assert!(
            server.handles.pairing.claim(&id, "192.0.2.30").is_none(),
            "a revoked request must never yield a credential"
        );
        assert!(
            server.handles.pairing.pending_requests().is_empty(),
            "claim of a revoked request consumes it"
        );
    }
}
