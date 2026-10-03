use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use rustls::pki_types::pem::PemObject as _;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};
use wtransport::endpoint::IncomingSession;
use wtransport::tls::{Certificate, CertificateChain, PrivateKey};
use wtransport::Identity;
use wtransport::RecvStream;
use wtransport::SendStream;
use wtransport::{Endpoint, ServerConfig as WtServerConfig};
use x509_parser::prelude::FromDer as _;

use super::annexb;
use super::wt_protocol::{
    advance_datagram_seq, decode_message, encode_hello_ack, encode_pong, fragment_video_datagrams,
    video_carrier, Message, VideoCarrier, DEFAULT_DATAGRAM, MAX_FRAME,
};
use super::{token_eq, ClientGuard, DisplayCtl, H264Packet, Stats, IDR_DEBOUNCE};

pub fn default_wt_port(signaling_port: u16) -> u16 {
    signaling_port.saturating_add(2)
}

#[derive(Debug, Clone)]
pub struct WtOffer {
    pub port: u16,
    pub path: &'static str,
    pub cert_sha256: String,
    pub hosts: Vec<String>,
}

pub fn advertised_hosts() -> Vec<String> {
    let mut hosts = Vec::new();
    if let Ok(sock) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if sock.connect("1.1.1.1:80").is_ok() {
            if let Ok(addr) = sock.local_addr() {
                if let std::net::IpAddr::V4(ip) = addr.ip() {
                    if !ip.is_loopback() && !ip.is_unspecified() {
                        hosts.push(ip.to_string());
                    }
                }
            }
        }
    }
    hosts.push("127.0.0.1".into());
    hosts
}

fn host_now_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

fn idr_due(last: Instant, now: Instant) -> bool {
    now.duration_since(last) >= IDR_DEBOUNCE
}

const CERT_RENEW_SLACK: Duration = Duration::from_secs(24 * 60 * 60);

fn default_identity_paths() -> (PathBuf, PathBuf) {
    (
        orbiscreen_core::default_wt_cert_path(),
        orbiscreen_core::default_wt_key_path(),
    )
}

fn cert_usable(der: &[u8], slack: Duration) -> bool {
    let Ok((_, cert)) = x509_parser::certificate::X509Certificate::from_der(der) else {
        return false;
    };
    let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return false;
    };
    within_validity(
        cert.validity().not_before.timestamp(),
        cert.validity().not_after.timestamp(),
        i64::try_from(slack.as_secs()).unwrap_or(i64::MAX),
        now.as_secs() as i64,
    )
}

/// A certificate is usable when it is already valid and stays valid past the renew slack.
/// not_before used to be ignored, so a certificate that only becomes valid later was kept
/// on disk and served even though no client could validate it yet.
fn within_validity(not_before: i64, not_after: i64, slack_secs: i64, now: i64) -> bool {
    if not_before > now {
        return false;
    }
    not_after > now + slack_secs
}

fn load_identity(cert_path: &Path, key_path: &Path) -> Result<Identity, String> {
    let cert_pem = std::fs::read(cert_path).map_err(|e| format!("read cert: {e}"))?;
    let key_pem = std::fs::read(key_path).map_err(|e| format!("read key: {e}"))?;
    let cert_der =
        CertificateDer::from_pem_slice(&cert_pem).map_err(|e| format!("cert pem: {e}"))?;
    if !cert_usable(&cert_der, CERT_RENEW_SLACK) {
        return Err("certificate expired or expiring".into());
    }
    let cert = Certificate::from_der(cert_der.to_vec()).map_err(|e| format!("cert der: {e}"))?;
    let key_der = PrivateKeyDer::from_pem_slice(&key_pem).map_err(|e| format!("key pem: {e}"))?;
    let key = PrivateKey::from_der_pkcs8(key_der.secret_der().to_vec());
    Ok(Identity::new(CertificateChain::single(cert), key))
}

fn write_secret_file(path: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _};
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        file.write_all(contents)?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, contents)?;
    }
    Ok(())
}

fn store_identity(identity: &Identity, cert_path: &Path, key_path: &Path) -> Result<(), String> {
    let cert = identity
        .certificate_chain()
        .as_slice()
        .first()
        .ok_or_else(|| "webtransport identity has no certificate".to_string())?;
    write_secret_file(cert_path, cert.to_pem().as_bytes())
        .map_err(|e| format!("write cert: {e}"))?;
    write_secret_file(key_path, identity.private_key().to_secret_pem().as_bytes())
        .map_err(|e| format!("write key: {e}"))?;
    Ok(())
}

#[allow(missing_debug_implementations)]
pub struct WtHub {
    pub offer: WtOffer,
    identity: Identity,
}

impl WtHub {
    pub fn new(port: u16) -> Result<Self, String> {
        let (cert_path, key_path) = default_identity_paths();
        Self::load_or_create(port, &cert_path, &key_path)
    }

    pub fn load_or_create(port: u16, cert_path: &Path, key_path: &Path) -> Result<Self, String> {
        let hosts = advertised_hosts();
        let mut sans = vec!["localhost".to_string(), "127.0.0.1".to_string()];
        for h in &hosts {
            if !sans.contains(h) {
                sans.push(h.clone());
            }
        }
        let identity = match load_identity(cert_path, key_path) {
            Ok(identity) => {
                info!(
                    cert = %cert_path.display(),
                    "reusing persisted WebTransport certificate"
                );
                identity
            }
            Err(reason) => {
                debug!(
                    cert = %cert_path.display(),
                    reason,
                    "generating a new WebTransport certificate"
                );
                let identity = Identity::self_signed(&sans).map_err(|e| e.to_string())?;
                if let Err(e) = store_identity(&identity, cert_path, key_path) {
                    warn!(
                        cert = %cert_path.display(),
                        "could not persist WebTransport certificate: {e}"
                    );
                } else {
                    info!(
                        cert = %cert_path.display(),
                        "stored WebTransport certificate"
                    );
                }
                identity
            }
        };
        let cert = identity
            .certificate_chain()
            .as_slice()
            .first()
            .ok_or_else(|| "webtransport identity has no certificate".to_string())?;
        let hash = cert.hash();
        let cert_sha256 = B64.encode(hash.as_ref());
        Ok(Self {
            offer: WtOffer {
                port,
                path: "/orbiscreen",
                cert_sha256,
                hosts,
            },
            identity,
        })
    }

    pub fn https_pem(&self) -> Result<(Vec<u8>, Vec<u8>), String> {
        let cert = self
            .identity
            .certificate_chain()
            .as_slice()
            .first()
            .ok_or_else(|| "webtransport identity has no certificate".to_string())?;
        Ok((
            cert.to_pem().into_bytes(),
            self.identity.private_key().to_secret_pem().into_bytes(),
        ))
    }
}

enum StreamAuth {
    Shared,
    Paired {
        credential: String,
        client_id: String,
        session_id: String,
    },
}

impl StreamAuth {
    fn authenticate(
        credential: &str,
        shared_token: &str,
        registry: Option<&crate::pairing::PairingRegistry>,
        session: &str,
        has_displays: bool,
    ) -> Option<Self> {
        if !shared_token.is_empty() && token_eq(credential, shared_token) {
            // The shared token opens sessions itself and therefore carries no paired owner,
            // but it must not attach to a session that a paired client owns.
            if let Some(registry) = registry {
                if !session.is_empty() && registry.session_owner(session).is_some() {
                    return None;
                }
            }
            return Some(Self::Shared);
        }
        let registry = registry?;
        let client = registry.verify(credential)?;
        if !has_displays || session.is_empty() || !registry.owns_session(&client.client_id, session)
        {
            return None;
        }
        Some(Self::Paired {
            credential: credential.to_string(),
            client_id: client.client_id,
            session_id: session.to_string(),
        })
    }

    fn valid(&self, registry: Option<&crate::pairing::PairingRegistry>) -> bool {
        match self {
            Self::Shared => true,
            Self::Paired {
                credential,
                client_id,
                session_id,
            } => registry.is_some_and(|registry| {
                registry
                    .verify(credential)
                    .is_some_and(|client| client.client_id == *client_id)
                    && registry.owns_session(client_id, session_id)
            }),
        }
    }

    fn accepts_attachment(&self, session: &str) -> bool {
        match self {
            Self::Shared => true,
            Self::Paired { session_id, .. } => session_id == session,
        }
    }
}

async fn wait_for_revocation(
    auth: &StreamAuth,
    registry: Option<&crate::pairing::PairingRegistry>,
) {
    if matches!(auth, StreamAuth::Shared) {
        std::future::pending::<()>().await;
        return;
    }
    let mut tick = tokio::time::interval(Duration::from_millis(250));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tick.tick().await;
        if !auth.valid(registry) {
            return;
        }
    }
}

struct WtCtx {
    token: String,
    registry: Option<Arc<crate::pairing::PairingRegistry>>,
    video: broadcast::Sender<H264Packet>,
    idr_tx: Option<tokio::sync::mpsc::Sender<()>>,
    stats: Arc<Stats>,
    displays: Option<DisplayCtl>,
    width: u32,
    height: u32,
}

#[allow(clippy::too_many_arguments)]
pub async fn run_wt_hub(
    hub: WtHub,
    token: String,
    video: broadcast::Sender<H264Packet>,
    idr_tx: Option<tokio::sync::mpsc::Sender<()>>,
    stats: Arc<Stats>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
    displays: Option<DisplayCtl>,
    width: u32,
    height: u32,
    registry: Option<Arc<crate::pairing::PairingRegistry>>,
) {
    let port = hub.offer.port;
    let config = WtServerConfig::builder()
        .with_bind_address(std::net::SocketAddr::from(([0, 0, 0, 0], port)))
        .with_identity(hub.identity)
        .build();
    let server = match Endpoint::server(config) {
        Ok(s) => s,
        Err(e) => {
            warn!("WebTransport bind {port} failed: {e}");
            return;
        }
    };
    info!(
        port,
        path = hub.offer.path,
        hosts = ?hub.offer.hosts,
        "WebTransport Annex-B listening"
    );
    let ctx = Arc::new(WtCtx {
        token,
        registry,
        video,
        idr_tx,
        stats,
        displays,
        width,
        height,
    });
    let in_flight = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    loop {
        tokio::select! {
            _ = shutdown.changed() => break,
            incoming = server.accept() => {
                if in_flight.load(std::sync::atomic::Ordering::Relaxed) >= MAX_INFLIGHT_SESSIONS {
                    warn!("webtransport connection refused: {} sessions already in flight", MAX_INFLIGHT_SESSIONS);
                    continue;
                }
                in_flight.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let ctx = ctx.clone();
                let in_flight = in_flight.clone();
                tokio::spawn(async move {
                    if let Err(e) = accept_session(incoming, ctx, in_flight).await {
                        debug!("webtransport session ended: {e}");
                    }
                });
            }
        }
    }
}

const MAX_INFLIGHT_SESSIONS: usize = 16;

const HELLO_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

async fn accept_session(
    incoming: IncomingSession,
    ctx: Arc<WtCtx>,
    in_flight: Arc<std::sync::atomic::AtomicUsize>,
) -> Result<(), String> {
    let guard = InFlightGuard(in_flight);
    let request = tokio::time::timeout(HELLO_TIMEOUT, incoming)
        .await
        .map_err(|_| "session handshake timed out".to_string())?
        .map_err(|e| e.to_string())?;
    if request.path() != "/orbiscreen" {
        request.not_found().await;
        return Err("wrong path".into());
    }
    let connection = tokio::time::timeout(HELLO_TIMEOUT, request.accept())
        .await
        .map_err(|_| "session accept timed out".to_string())?
        .map_err(|e| e.to_string())?;
    let (send, recv) = tokio::time::timeout(HELLO_TIMEOUT, connection.accept_bi())
        .await
        .map_err(|_| "stream accept timed out".to_string())?
        .map_err(|e| e.to_string())?;
    handle_session(connection, send, recv, ctx, Some(guard)).await
}

struct InFlightGuard(Arc<std::sync::atomic::AtomicUsize>);

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

async fn handle_session(
    connection: wtransport::Connection,
    mut send: SendStream,
    mut recv: RecvStream,
    ctx: Arc<WtCtx>,
    admission: Option<InFlightGuard>,
) -> Result<(), String> {
    let hello = match tokio::time::timeout(HELLO_TIMEOUT, read_message(&mut recv))
        .await
        .map_err(|_| "hello read timed out".to_string())??
    {
        Some(Message::Hello(h)) => h,
        Some(_) => return Err("expected hello".into()),
        None => return Err("closed before hello".into()),
    };
    let authenticated = StreamAuth::authenticate(
        &hello.token,
        &ctx.token,
        ctx.registry.as_deref(),
        &hello.session,
        ctx.displays.is_some(),
    );
    drop(admission);
    let Some(auth) = authenticated else {
        ctx.stats.note_auth_failure();
        connection.close(1u32.into(), b"unauthorized");
        return Err("unauthorized".into());
    };
    let result = tokio::select! {
        biased;
        _ = wait_for_revocation(&auth, ctx.registry.as_deref()) => Err("authorization revoked".into()),
        result = stream_session(&connection, &mut send, &mut recv, &ctx, &auth, &hello.session) => result,
    };
    connection.close(0u32.into(), b"session ended");
    result
}

async fn stream_session(
    connection: &wtransport::Connection,
    send: &mut SendStream,
    recv: &mut RecvStream,
    ctx: &WtCtx,
    auth: &StreamAuth,
    session: &str,
) -> Result<(), String> {
    let attached = if let Some(ctl) = &ctx.displays {
        let id = if session.is_empty() {
            None
        } else {
            Some(session.to_string())
        };
        match ctl.attach(id, None).await {
            Ok(att) => Some(att),
            Err(e) => {
                warn!("webtransport attach failed: {e}");
                return Err(e);
            }
        }
    } else {
        None
    };
    let session_id = attached.as_ref().map(|a| a.info.id.clone());
    let mut video_rx = if let Some(att) = attached {
        att.video
    } else {
        ctx.video.subscribe()
    };

    ctx.stats.client_started();
    let _guard = ClientGuard(ctx.stats.clone());
    struct DetachGuard(Option<(DisplayCtl, String)>);
    impl Drop for DetachGuard {
        fn drop(&mut self) {
            if let Some((ctl, id)) = self.0.take() {
                tokio::spawn(async move { ctl.detach(&id).await });
            }
        }
    }
    let _detach = DetachGuard(ctx.displays.clone().zip(session_id.clone()));
    if !auth.valid(ctx.registry.as_deref())
        || session_id
            .as_deref()
            .is_some_and(|id| !auth.accepts_attachment(id))
    {
        return Err("unauthorized attachment".into());
    }
    let ack = encode_hello_ack(
        ctx.width.min(u32::from(u16::MAX)) as u16,
        ctx.height.min(u32::from(u16::MAX)) as u16,
    )
    .map_err(|e| format!("{e:?}"))?;
    write_all(send, &ack).await?;

    let mut last_idr = Instant::now()
        .checked_sub(IDR_DEBOUNCE)
        .unwrap_or_else(Instant::now);
    let mut request_idr = || {
        let now = Instant::now();
        if !idr_due(last_idr, now) {
            return;
        }
        last_idr = now;
        if let (Some(ctl), Some(id)) = (ctx.displays.as_ref(), session_id.as_ref()) {
            let ctl = ctl.clone();
            let id = id.clone();
            tokio::spawn(async move { ctl.idr(&id).await });
        } else if let Some(tx) = &ctx.idr_tx {
            let _ = tx.try_send(());
        }
    };
    request_idr();

    let mut wait_key = true;
    let mut seq: u16 = 0;
    let mut cached_sps_pps: Option<super::annexb::SpsPps> = None;
    let mut max_datagram = connection
        .max_datagram_size()
        .unwrap_or(0)
        .clamp(0, DEFAULT_DATAGRAM);
    let mut use_datagrams = max_datagram >= super::wt_protocol::DATAGRAM_HEADER + 64;
    if !use_datagrams {
        info!(
            max_datagram,
            "webtransport peer has no datagrams; sending video on the control stream"
        );
    } else {
        info!(max_datagram, "webtransport video via QUIC datagrams");
    }
    let mut buf = Vec::new();
    loop {
        tokio::select! {
            incoming = read_into(recv, &mut buf) => {
                incoming?;
                while let Some(msg) = pop_message(&mut buf)? {
                    match msg {
                        Message::Idr => request_idr(),
                        Message::Ping(t0) => {
                            let pong = encode_pong(t0, host_now_ns()).map_err(|e| format!("{e:?}"))?;
                            write_all(send, &pong).await?;
                        }
                        Message::Bye => return Ok(()),
                        Message::Hello(_) | Message::HelloAck(_) | Message::Video(_) | Message::Pong { .. } => {}
                    }
                }
            }
            pkt = video_rx.recv() => {
                let pkt = match pkt {
                    Ok(p) => p,
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        wait_key = true;
                        request_idr();
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => return Ok(()),
                };
                if wait_key {
                    if !pkt.is_keyframe {
                        request_idr();
                        continue;
                    }
                    wait_key = false;
                }
                if !annexb::is_annexb(&pkt.bytes) {
                    continue;
                }
                let mut pkt = pkt;
                if pkt.is_keyframe {
                    let found = annexb::extract_sps_pps(&pkt.bytes);
                    if !found.sps.is_empty() && !found.pps.is_empty() {
                        cached_sps_pps = Some(found);
                    }
                    pkt.bytes = annexb::with_parameter_sets(&pkt.bytes, cached_sps_pps.as_ref());
                }
                match video_carrier(pkt.is_keyframe, use_datagrams) {
                    VideoCarrier::Reliable => {
                        let frame = match super::wt_protocol::encode_video(&pkt, host_now_ns()) {
                            Ok(f) => f,
                            Err(super::wt_protocol::CodecError::TooLarge(_)) => {
                                wait_key = true;
                                request_idr();
                                continue;
                            }
                            Err(e) => return Err(format!("{e:?}")),
                        };
                        if write_all(send, &frame).await.is_err() {
                            return Ok(());
                        }
                    }
                    VideoCarrier::Datagram => {
                        if let Some(sz) = connection.max_datagram_size() {
                            max_datagram = sz.clamp(256, DEFAULT_DATAGRAM);
                        }
                        let dgrams =
                            fragment_video_datagrams(seq, &pkt, host_now_ns(), max_datagram);
                        if advance_datagram_seq(pkt.is_keyframe) {
                            seq = seq.wrapping_add(1);
                        }
                        let mut drop_rest = false;
                        for dgram in dgrams {
                            match connection.send_datagram(&dgram) {
                                Ok(()) => {}
                                Err(wtransport::error::SendDatagramError::TooLarge) => {
                                    max_datagram = max_datagram.saturating_sub(64).max(256);
                                    drop_rest = true;
                                    wait_key = true;
                                    request_idr();
                                    break;
                                }
                                Err(wtransport::error::SendDatagramError::NotConnected) => {
                                    return Ok(());
                                }
                                Err(wtransport::error::SendDatagramError::UnsupportedByPeer) => {
                                    use_datagrams = false;
                                    drop_rest = true;
                                    break;
                                }
                            }
                        }
                        if drop_rest && use_datagrams {
                            continue;
                        }
                        if !use_datagrams {
                            let frame = super::wt_protocol::encode_video(&pkt, host_now_ns())
                                .map_err(|e| format!("{e:?}"))?;
                            if write_all(send, &frame).await.is_err() {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn write_all(send: &mut SendStream, bytes: &[u8]) -> Result<(), String> {
    send.write_all(bytes).await.map_err(|e| e.to_string())
}

async fn read_into(recv: &mut RecvStream, buf: &mut Vec<u8>) -> Result<(), String> {
    let mut tmp = [0u8; 8192];
    match recv.read(&mut tmp).await {
        Ok(Some(n)) => {
            buf.extend_from_slice(&tmp[..n]);
            Ok(())
        }
        Ok(None) => Err("stream closed".into()),
        Err(e) => Err(e.to_string()),
    }
}

async fn read_message(recv: &mut RecvStream) -> Result<Option<Message>, String> {
    let mut buf = Vec::new();
    loop {
        read_into(recv, &mut buf).await?;
        if let Some(msg) = pop_message(&mut buf)? {
            return Ok(Some(msg));
        }
        if buf.len() > MAX_FRAME + 4 {
            return Err("hello frame too large".into());
        }
    }
}

fn pop_message(buf: &mut Vec<u8>) -> Result<Option<Message>, String> {
    match super::wt_protocol::split_frame(buf) {
        Ok(Some((body, n))) => {
            let msg = decode_message(body).map_err(|e| format!("{e:?}"))?;
            buf.drain(..n);
            Ok(Some(msg))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("{e:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paired_registry() -> (Arc<crate::pairing::PairingRegistry>, String, String) {
        let registry = Arc::new(crate::pairing::PairingRegistry::load_from(None).unwrap());
        let request = registry.request_pairing("tablet", "192.0.2.1").unwrap();
        let client = registry.approve(&request.request_id).unwrap().unwrap();
        let credential = registry.claim(&request.request_id, "192.0.2.1").unwrap();
        assert!(registry.bind_session(&client.client_id, "owned-session"));
        (registry, credential, client.client_id)
    }

    #[test]
    fn paired_auth_requires_explicit_owned_display_session() {
        let (registry, credential, _) = paired_registry();
        for (session, displays) in [
            ("", true),
            ("other-session", true),
            ("owned-session", false),
        ] {
            assert!(StreamAuth::authenticate(
                &credential,
                "shared",
                Some(&registry),
                session,
                displays,
            )
            .is_none());
        }
        assert!(
            StreamAuth::authenticate(&credential, "shared", None, "owned-session", true).is_none()
        );
        assert!(StreamAuth::authenticate(
            "invalid",
            "shared",
            Some(&registry),
            "owned-session",
            true
        )
        .is_none());
        let auth = StreamAuth::authenticate(
            &credential,
            "shared",
            Some(&registry),
            "owned-session",
            true,
        )
        .unwrap();
        assert!(auth.valid(Some(&registry)));
        assert!(auth.accepts_attachment("owned-session"));
        assert!(!auth.accepts_attachment("other-session"));
        assert!(!auth.valid(None));
        registry.forget_session("owned-session");
        assert!(!auth.valid(Some(&registry)));
    }

    #[test]
    fn paired_auth_does_not_accept_another_clients_session() {
        let (registry, _, _) = paired_registry();
        let request = registry.request_pairing("other", "192.0.2.2").unwrap();
        registry.approve(&request.request_id).unwrap().unwrap();
        let credential = registry.claim(&request.request_id, "192.0.2.2").unwrap();
        assert!(StreamAuth::authenticate(
            &credential,
            "shared",
            Some(&registry),
            "owned-session",
            true,
        )
        .is_none());
    }

    #[test]
    fn shared_token_cannot_attach_to_a_paired_clients_session() {
        let (registry, _, _) = paired_registry();
        // The shared token owns no session of its own, so it may not attach to one that a
        // paired client owns.
        assert!(
            StreamAuth::authenticate("shared", "shared", Some(&registry), "owned-session", true)
                .is_none(),
            "shared token reached a session owned by a paired client"
        );
        // Unowned sessions still work, so legacy clients are unaffected.
        assert!(matches!(
            StreamAuth::authenticate("shared", "shared", Some(&registry), "", true),
            Some(StreamAuth::Shared)
        ));
        assert!(matches!(
            StreamAuth::authenticate("shared", "shared", Some(&registry), "unowned", true),
            Some(StreamAuth::Shared)
        ));
    }

    #[test]
    fn validity_window_rejects_not_yet_valid_and_expired_certificates() {
        const DAY: i64 = 86_400;
        let now = 1_700_000_000;
        let slack = 30 * DAY;
        // Not yet valid: must not be served.
        assert!(!within_validity(now + DAY, now + 400 * DAY, slack, now));
        // Valid but inside the renew slack: replaced.
        assert!(!within_validity(now - 400 * DAY, now + DAY, slack, now));
        // Already expired.
        assert!(!within_validity(now - 400 * DAY, now - DAY, slack, now));
        // Comfortably valid.
        assert!(within_validity(now - DAY, now + 400 * DAY, slack, now));
        // Exactly at not_before is valid.
        assert!(within_validity(now, now + 400 * DAY, slack, now));
    }

    #[test]
    fn shared_auth_preserves_legacy_default_stream() {
        let auth = StreamAuth::authenticate("shared", "shared", None, "", false).unwrap();
        assert!(matches!(auth, StreamAuth::Shared));
        assert!(auth.valid(None));
        assert!(auth.accepts_attachment("legacy"));
        assert!(StreamAuth::authenticate("", "", None, "", false).is_none());
    }

    #[tokio::test]
    async fn paired_revocation_interrupts_idle_stream() {
        let (registry, credential, client_id) = paired_registry();
        let identity = Identity::self_signed(["localhost", "127.0.0.1"]).unwrap();
        let digest = identity.certificate_chain().as_slice()[0].hash();
        let server = Endpoint::server(
            WtServerConfig::builder()
                .with_bind_address("127.0.0.1:0".parse().unwrap())
                .with_identity(identity)
                .build(),
        )
        .unwrap();
        let port = server.local_addr().unwrap().port();
        let (video, _) = broadcast::channel(8);
        let (commands, mut commands_rx) = tokio::sync::mpsc::channel(8);
        let stats = Arc::new(Stats::default());
        let ctx = Arc::new(WtCtx {
            token: "shared".into(),
            registry: Some(registry.clone()),
            video: video.clone(),
            idr_tx: None,
            stats,
            displays: Some(DisplayCtl::new(commands)),
            width: 640,
            height: 480,
        });
        let accept = tokio::spawn(async move {
            let incoming = server.accept().await;
            let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let result = accept_session(incoming, ctx, counter).await;
            server.wait_idle().await;
            result
        });
        let client = Endpoint::client(
            wtransport::ClientConfig::builder()
                .with_bind_default()
                .with_server_certificate_hashes([digest])
                .build(),
        )
        .unwrap();
        let connection = client
            .connect(format!("https://127.0.0.1:{port}/orbiscreen"))
            .await
            .unwrap();
        let (mut send, mut recv) = connection.open_bi().await.unwrap().await.unwrap();
        send.write_all(
            &super::super::wt_protocol::encode_hello(&credential, "owned-session").unwrap(),
        )
        .await
        .unwrap();
        let command = tokio::time::timeout(Duration::from_secs(2), commands_rx.recv())
            .await
            .unwrap()
            .unwrap();
        match command {
            crate::DisplayCommand::Attach { id, key: _, reply } => {
                assert_eq!(id.as_deref(), Some("owned-session"));
                assert!(reply
                    .send(Ok(crate::AttachedDisplay {
                        info: crate::DisplayInfo {
                            id: "owned-session".into(),
                            name: "tablet".into(),
                            connector: "test".into(),
                            width: 640,
                            height: 480,
                            encoder: "test".into(),
                        },
                        video: video.subscribe(),
                    }))
                    .is_ok());
            }
            _ => panic!("expected attach"),
        }
        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(2), read_message(&mut recv))
                .await
                .unwrap()
                .unwrap(),
            Some(Message::HelloAck(_))
        ));
        video
            .send(H264Packet {
                bytes: vec![0, 0, 0, 1, 0x65, 9],
                is_keyframe: true,
                pts_ns: 1,
            })
            .unwrap();
        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(2), read_message(&mut recv))
                .await
                .unwrap()
                .unwrap(),
            Some(Message::Video(_))
        ));
        assert!(registry.revoke(&client_id).unwrap());
        tokio::time::timeout(Duration::from_secs(2), connection.closed())
            .await
            .unwrap();
        let detached = tokio::time::timeout(Duration::from_secs(2), async {
            while let Some(command) = commands_rx.recv().await {
                if let crate::DisplayCommand::Detach { id } = command {
                    return id;
                }
            }
            panic!("missing detach")
        })
        .await
        .unwrap();
        assert_eq!(detached, "owned-session");
        assert!(tokio::time::timeout(Duration::from_secs(3), accept)
            .await
            .unwrap()
            .unwrap()
            .is_err());
    }

    #[test]
    fn default_port_is_signaling_plus_two() {
        assert_eq!(default_wt_port(8788), 8790);
        assert_eq!(default_wt_port(u16::MAX), u16::MAX);
    }

    #[test]
    fn advertised_hosts_includes_loopback() {
        let hosts = advertised_hosts();
        assert!(hosts.iter().any(|h| h == "127.0.0.1"));
    }

    fn temp_identity_paths() -> (std::path::PathBuf, std::path::PathBuf) {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("orbiscreen-wt-{}-{}", std::process::id(), nonce));
        std::fs::create_dir_all(&dir).expect("temp identity dir");
        (dir.join("wt-cert.pem"), dir.join("wt-key.pem"))
    }

    #[test]
    fn self_signed_offer_has_sha256() {
        let (cert, key) = temp_identity_paths();
        let hub = WtHub::load_or_create(0, &cert, &key).expect("identity");
        let raw = B64.decode(&hub.offer.cert_sha256).expect("b64");
        assert_eq!(raw.len(), 32);
        assert_eq!(hub.offer.path, "/orbiscreen");
        assert!(cert.is_file());
        assert!(key.is_file());
    }

    #[test]
    fn persisted_identity_reuses_sha256() {
        let (cert, key) = temp_identity_paths();
        let first = WtHub::load_or_create(0, &cert, &key).expect("first");
        let second = WtHub::load_or_create(0, &cert, &key).expect("second");
        assert_eq!(first.offer.cert_sha256, second.offer.cert_sha256);
    }

    #[test]
    fn missing_key_regenerates_identity() {
        let (cert, key) = temp_identity_paths();
        let first = WtHub::load_or_create(0, &cert, &key).expect("first");
        std::fs::remove_file(&key).expect("drop key");
        let second = WtHub::load_or_create(0, &cert, &key).expect("second");
        assert_ne!(first.offer.cert_sha256, second.offer.cert_sha256);
    }

    #[test]
    fn expired_identity_is_replaced() {
        let (cert, key) = temp_identity_paths();
        let first = WtHub::load_or_create(0, &cert, &key).expect("first");
        let expired = Identity::self_signed_builder()
            .subject_alt_names(["localhost"])
            .from_now_utc()
            .validity_days(0)
            .build()
            .expect("expired");
        store_identity(&expired, &cert, &key).expect("store expired");
        assert!(!cert_usable(
            CertificateDer::from_pem_slice(&std::fs::read(&cert).unwrap())
                .unwrap()
                .as_ref(),
            CERT_RENEW_SLACK
        ));
        let second = WtHub::load_or_create(0, &cert, &key).expect("renewed");
        assert_ne!(first.offer.cert_sha256, second.offer.cert_sha256);
        assert!(cert_usable(
            CertificateDer::from_pem_slice(&std::fs::read(&cert).unwrap())
                .unwrap()
                .as_ref(),
            Duration::ZERO
        ));
    }

    #[test]
    fn freshly_generated_cert_is_usable() {
        let identity = Identity::self_signed(["localhost"]).expect("identity");
        let der = identity.certificate_chain().as_slice()[0].der();
        assert!(cert_usable(der, CERT_RENEW_SLACK));
    }

    #[tokio::test]
    async fn hello_and_video_over_webtransport() {
        use crate::wt_protocol::{decode_message, encode_hello, split_frame, Message};
        use crate::H264Packet;
        use std::sync::Arc;
        use tokio::sync::{broadcast, watch};
        use wtransport::tls::Sha256Digest;
        use wtransport::ClientConfig;
        use wtransport::Endpoint;

        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("ephemeral port")
            .local_addr()
            .expect("local addr")
            .port();
        let (cert, key) = temp_identity_paths();
        let hub = WtHub::load_or_create(port, &cert, &key).expect("identity");
        let mut digest = [0u8; 32];
        digest.copy_from_slice(&B64.decode(&hub.offer.cert_sha256).unwrap());
        let token = "wt-test-token".to_string();
        let (video_tx, _) = broadcast::channel::<H264Packet>(8);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let stats = Arc::new(crate::Stats::default());
        let pump = video_tx.clone();
        let server = tokio::spawn(run_wt_hub(
            hub,
            token.clone(),
            video_tx,
            None,
            stats,
            shutdown_rx,
            None,
            640,
            480,
            None,
        ));
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;

        let client_cfg = ClientConfig::builder()
            .with_bind_default()
            .with_server_certificate_hashes([Sha256Digest::new(digest)])
            .build();
        let url = format!("https://127.0.0.1:{port}/orbiscreen");
        let connection = Endpoint::client(client_cfg)
            .expect("client endpoint")
            .connect(&url)
            .await
            .expect("connect");
        let (mut send, mut recv) = connection.open_bi().await.expect("open").await.expect("bi");
        send.write_all(&encode_hello(&token, "").unwrap())
            .await
            .expect("hello");

        let mut buf = Vec::new();
        let mut tmp = [0u8; 2048];
        let ack = loop {
            let n = recv.read(&mut tmp).await.expect("read").expect("eof");
            buf.extend_from_slice(&tmp[..n]);
            if let Some((body, used)) = split_frame(&buf).unwrap() {
                let msg = decode_message(body).unwrap();
                buf.drain(..used);
                break msg;
            }
        };
        match ack {
            Message::HelloAck(a) => {
                assert_eq!(a.width, 640);
                assert_eq!(a.height, 480);
            }
            other => panic!("{other:?}"),
        }

        pump.send(H264Packet {
            bytes: vec![0, 0, 0, 1, 0x65, 9],
            is_keyframe: true,
            pts_ns: 1,
        })
        .unwrap();

        let key = loop {
            let n = recv.read(&mut tmp).await.expect("read").expect("eof");
            buf.extend_from_slice(&tmp[..n]);
            if let Some((body, used)) = split_frame(&buf).unwrap() {
                let msg = decode_message(body).unwrap();
                buf.drain(..used);
                break msg;
            }
        };
        match key {
            Message::Video(v) => {
                assert!(v.is_keyframe);
                assert_eq!(v.au, vec![0, 0, 0, 1, 0x65, 9]);
            }
            other => panic!("expected reliable IDR, got {other:?}"),
        }

        pump.send(H264Packet {
            bytes: vec![0, 0, 0, 1, 0x41, 1],
            is_keyframe: false,
            pts_ns: 2,
        })
        .unwrap();
        let dgram = connection.receive_datagram().await.expect("datagram");
        let frag = crate::wt_protocol::parse_video_datagram(dgram.payload().as_ref())
            .expect("video datagram");
        assert!(!frag.is_keyframe);
        assert_eq!(frag.payload, vec![0, 0, 0, 1, 0x41, 1]);

        let _ = shutdown_tx.send(true);
        let _ = server.await;
    }
}
