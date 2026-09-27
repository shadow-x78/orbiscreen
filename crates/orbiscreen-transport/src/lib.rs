// Orbiscreen - lib.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

pub mod annexb;
pub mod aoa;
pub mod aoa_video;
pub mod display;
pub mod fec;
pub mod mdns;
pub mod pairing;
pub mod udp_crypto;
pub mod udp_stream;
pub mod wt_protocol;
pub mod wt_stream;

pub use display::{AttachedDisplay, DisplayCommand, DisplayCtl, DisplayInfo};

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, FromRequest, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Json};
use axum::routing::{get, post};
use axum::{middleware, Router};
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tower_http::services::ServeDir;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct ServiceDescriptor {
    pub instance: String,
    pub port: u16,
    pub token: Option<String>,
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("http server error: {0}")]
    Http(String),
}

use orbiscreen_input::{KeyEvent, PointerEvent, StylusEvent, TouchEvent};

#[derive(Debug, Clone, serde::Deserialize)]
pub enum IncomingInput {
    Pointer(PointerEvent),
    Key(KeyEvent),
    Stylus(StylusEvent),
    Touch(TouchEvent),
    Resize {
        width: u32,
        height: u32,
    },
    #[serde(untagged)]
    RawPointer {
        x: f64,
        y: f64,
    },
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub signaling_port: u16,
    pub client_web_dir: PathBuf,
    pub enable_usb_supervisors: bool,
    pub output_connector: Option<String>,
}

#[derive(Debug)]
pub struct Stats {
    frames_forwarded: AtomicU64,
    active_clients: AtomicUsize,
    total_clients: AtomicU64,
    auth_failures: AtomicU64,
    usb_devices: AtomicUsize,
    usb_connected_names: RwLock<Vec<String>>,
    usb_aoa_ready: AtomicBool,
    presence: tokio::sync::watch::Sender<bool>,
}

impl Default for Stats {
    fn default() -> Self {
        let (presence, _) = tokio::sync::watch::channel(false);
        Self {
            frames_forwarded: AtomicU64::new(0),
            active_clients: AtomicUsize::new(0),
            total_clients: AtomicU64::new(0),
            auth_failures: AtomicU64::new(0),
            usb_devices: AtomicUsize::new(0),
            usb_connected_names: RwLock::new(Vec::new()),
            usb_aoa_ready: AtomicBool::new(false),
            presence,
        }
    }
}

impl Stats {
    pub fn frames_forwarded(&self) -> u64 {
        self.frames_forwarded.load(Ordering::Relaxed)
    }

    pub fn active_clients(&self) -> usize {
        self.active_clients.load(Ordering::Relaxed)
    }

    pub fn total_clients(&self) -> u64 {
        self.total_clients.load(Ordering::Relaxed)
    }

    pub fn auth_failures(&self) -> u64 {
        self.auth_failures.load(Ordering::Relaxed)
    }

    pub fn usb_devices(&self) -> usize {
        self.usb_devices.load(Ordering::Relaxed)
    }

    pub fn usb_connected_names(&self) -> Vec<String> {
        self.usb_connected_names.read().unwrap().clone()
    }

    pub fn is_usb_aoa_ready(&self) -> bool {
        self.usb_aoa_ready.load(Ordering::Relaxed)
    }

    pub fn note_usb_state(&self, count: usize, names: Vec<String>, aoa_ready: bool) {
        self.usb_devices.store(count, Ordering::Relaxed);
        self.usb_aoa_ready.store(aoa_ready, Ordering::Relaxed);
        if let Ok(mut lock) = self.usb_connected_names.write() {
            *lock = names;
        }
    }

    fn note_frame(&self) {
        self.frames_forwarded.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn note_auth_failure(&self) {
        self.auth_failures.fetch_add(1, Ordering::Relaxed);
    }

    fn note_usb_devices(&self, count: usize) {
        self.usb_devices.store(count, Ordering::Relaxed);
    }

    pub fn subscribe_presence(&self) -> tokio::sync::watch::Receiver<bool> {
        self.presence.subscribe()
    }

    pub fn client_started(&self) {
        let prev = self.active_clients.fetch_add(1, Ordering::Relaxed);
        self.total_clients.fetch_add(1, Ordering::Relaxed);
        if prev == 0 {
            let _ = self.presence.send(true);
        }
    }

    pub fn client_stopped(&self) {
        loop {
            let cur = self.active_clients.load(Ordering::Relaxed);
            let next = cur.saturating_sub(1);
            if self
                .active_clients
                .compare_exchange(cur, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                if cur == 1 {
                    let _ = self.presence.send(false);
                }
                break;
            }
        }
    }
}

pub(crate) struct ClientGuard(pub(crate) Arc<Stats>);

impl Drop for ClientGuard {
    fn drop(&mut self) {
        self.0.client_stopped();
    }
}

pub fn generate_token() -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine as _;
    let mut bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rng(), &mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub(crate) fn token_eq(a: &str, b: &str) -> bool {
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    let max_len = ab.len().max(bb.len());
    let mut diff = u8::from(ab.len() != bb.len());
    for i in 0..max_len {
        let x = ab.get(i).copied().unwrap_or(0);
        let y = bb.get(i).copied().unwrap_or(0);
        diff |= x ^ y;
    }
    diff == 0
}

#[allow(missing_debug_implementations)]
pub struct Transport {
    cfg: ServerConfig,
    input_tx: mpsc::Sender<IncomingInput>,
    token: String,
    aoa_active: Arc<AtomicUsize>,
}

impl Transport {
    pub fn new(cfg: ServerConfig, input_tx: mpsc::Sender<IncomingInput>) -> Self {
        Self::with_token(cfg, input_tx, None)
    }

    pub fn with_token(
        cfg: ServerConfig,
        input_tx: mpsc::Sender<IncomingInput>,
        token: Option<String>,
    ) -> Self {
        Self {
            cfg,
            input_tx,
            token: token.unwrap_or_else(generate_token),
            aoa_active: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn aoa_active(&self) -> Arc<AtomicUsize> {
        self.aoa_active.clone()
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn serve(
        self,
        frames: mpsc::Receiver<H264Packet>,
        stats: Arc<Stats>,
        display_width: u32,
        display_height: u32,
        refresh_hz: u32,
        encoder_kind: &'static str,
        mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
        idr_tx: Option<mpsc::Sender<()>>,
        displays: Option<DisplayCtl>,
    ) -> Result<(), TransportError> {
        let input_tx = self.input_tx;
        let (video_tx, _video_rx) = tokio::sync::broadcast::channel::<H264Packet>(64);
        let (client_shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(16);
        let wt_port = wt_stream::default_wt_port(self.cfg.signaling_port);
        let wt_hub = match wt_stream::WtHub::new(wt_port) {
            Ok(hub) => Some(hub),
            Err(e) => {
                warn!("webtransport identity failed: {e}");
                None
            }
        };
        let state = AppState {
            config: self.cfg.clone(),
            input_tx,
            video_tx: video_tx.clone(),
            stats,
            token: self.token.clone(),
            pairing: Arc::new(
                pairing::PairingRegistry::load()
                    .map_err(|e| TransportError::Http(format!("pairing registry: {e}")))?,
            ),
            display_width,
            display_height,
            refresh_hz,
            encoder_kind,
            version: env!("CARGO_PKG_VERSION"),
            started: std::time::Instant::now(),
            idr_tx,
            client_shutdown_tx,
            displays: displays.clone(),
            wt_offer: wt_hub.as_ref().map(|h| h.offer.clone()),
            udp_keys: udp_crypto::UdpKeyRing::new(),
        };
        let https_pem = wt_hub.as_ref().and_then(|hub| match hub.https_pem() {
            Ok(pem) => Some(pem),
            Err(e) => {
                warn!("https certificate export failed: {e}");
                None
            }
        });
        let http_app = build_http_router(state.clone());
        let https_app = build_https_router(state.clone());
        let listener = TcpListener::bind(("0.0.0.0", self.cfg.signaling_port))
            .await
            .map_err(|e| TransportError::Http(e.to_string()))?;
        let local = listener
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "?".into());
        info!("orbiscreen transport listening on http://{local}");

        if let Some((cert_pem, key_pem)) = https_pem {
            let https_port = wt_port;
            let https_app = https_app;
            let https_shutdown = shutdown_rx.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    serve_https(https_port, cert_pem, key_pem, https_app, https_shutdown).await
                {
                    warn!("https server failed: {e}");
                }
            });
        }

        let udp_port = udp_stream::default_udp_port(self.cfg.signaling_port);
        let udp_video = state.video_tx.subscribe();
        let udp_idr = state.idr_tx.clone();
        let udp_stats = state.stats.clone();
        let udp_token = state.token.clone();
        let udp_shutdown = shutdown_rx.clone();
        let udp_displays = displays.clone();
        let udp_registry = Arc::clone(&state.pairing);
        let udp_keys = state.udp_keys.clone();
        tokio::spawn(async move {
            udp_stream::run_udp_hub(
                udp_port,
                udp_token,
                udp_video,
                udp_idr,
                udp_stats,
                udp_shutdown,
                udp_stream::UdpLimits::from_env(),
                udp_displays,
                Some(udp_registry),
                udp_keys,
            )
            .await;
        });

        if let Some(hub) = wt_hub {
            let wt_video = state.video_tx.clone();
            let wt_idr = state.idr_tx.clone();
            let wt_stats = state.stats.clone();
            let wt_token = state.token.clone();
            let wt_shutdown = shutdown_rx.clone();
            let wt_displays = displays.clone();
            let wt_w = display_width;
            let wt_h = display_height;
            let wt_registry = Arc::clone(&state.pairing);
            tokio::spawn(async move {
                wt_stream::run_wt_hub(
                    hub,
                    wt_token,
                    wt_video,
                    wt_idr,
                    wt_stats,
                    wt_shutdown,
                    wt_displays,
                    wt_w,
                    wt_h,
                    Some(wt_registry),
                )
                .await;
            });
        }

        let usb_stats_task = if self.cfg.enable_usb_supervisors {
            let aoa_port = self.cfg.signaling_port;
            let aoa_active = self.aoa_active.clone();
            let aoa_active_for_sup = aoa_active.clone();
            let aoa_shutdown = shutdown_rx.clone();
            let aoa_displays = displays.clone();
            tokio::spawn(async move {
                aoa::supervisor(aoa_port, aoa_active_for_sup, aoa_shutdown, aoa_displays).await;
            });

            let usb_stats = state.stats.clone();
            let mut usb_shutdown = shutdown_rx.clone();
            let aoa_active_for_stats = aoa_active.clone();
            Some(tokio::spawn(async move {
                loop {
                    let bridges = aoa_active_for_stats.load(Ordering::Relaxed);
                    let names = aoa::get_connected_candidate_names();
                    let count = if bridges > 0 { bridges } else { names.len() };
                    usb_stats.note_usb_state(count, names, bridges > 0);
                    tokio::select! {
                        _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {}
                        _ = usb_shutdown.changed() => break,
                    }
                }
                usb_stats.note_usb_state(0, Vec::new(), false);
            }))
        } else {
            None
        };

        let stats_pump = state.stats.clone();
        tokio::spawn(async move {
            let mut frames = frames;
            while let Some(pkt) = frames.recv().await {
                if stats_pump.active_clients() > 0 {
                    stats_pump.note_frame();
                }
                let _ = video_tx.send(pkt);
            }
        });

        let serve_fut = axum::serve(
            listener,
            http_app.into_make_service_with_connect_info::<SocketAddr>(),
        );
        tokio::select! {
            res = serve_fut => {
                res.map_err(|e| TransportError::Http(e.to_string()))?;
            }
            _ = shutdown_rx.changed() => {}
            _ = tokio::signal::ctrl_c() => {}
        }

        let _ = state.client_shutdown_tx.send(());
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        if let Some(task) = usb_stats_task {
            task.abort();
        }
        state.stats.note_usb_devices(0);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct H264Packet {
    pub bytes: Vec<u8>,
    pub is_keyframe: bool,
    pub pts_ns: u64,
}

#[derive(Clone)]
struct AppState {
    config: ServerConfig,
    input_tx: mpsc::Sender<IncomingInput>,
    video_tx: tokio::sync::broadcast::Sender<H264Packet>,
    stats: Arc<Stats>,
    token: String,
    pairing: Arc<pairing::PairingRegistry>,
    display_width: u32,
    display_height: u32,
    refresh_hz: u32,
    encoder_kind: &'static str,
    version: &'static str,
    started: std::time::Instant,
    idr_tx: Option<mpsc::Sender<()>>,
    client_shutdown_tx: tokio::sync::broadcast::Sender<()>,
    displays: Option<DisplayCtl>,
    wt_offer: Option<wt_stream::WtOffer>,
    udp_keys: udp_crypto::UdpKeyRing,
}

async fn alt_svc_h3(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    let client_asset = request.uri().path().starts_with("/client");
    let mut response = next.run(request).await;
    if let Some(port) = state.wt_offer.as_ref().map(|o| o.port) {
        let value = format!("h3=\":{port}\"; ma=86400");
        if let Ok(hv) = axum::http::HeaderValue::from_str(&value) {
            response.headers_mut().insert("alt-svc", hv);
        }
    }
    if client_asset {
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store"),
        );
    }
    response
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/stream", get(stream_handler).head(stream_head_handler))
        .route("/au", get(au_handler))
        .route("/idr", get(idr_handler))
        .route("/input", post(input_post))
        .route("/input/ws", get(input_ws))
        .route("/api/control", post(api_control))
        .route(
            "/api/session",
            post(api_session_open).delete(api_session_close),
        )
        .route("/api/udp-key", post(api_udp_key))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_check))
        .route("/api/pair/request", post(api_pair_request))
        .route("/api/pair/status", get(api_pair_status))
        .route("/api/pair", get(api_pair_list).post(api_pair_manage))
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/api/info", get(api_info))
        .route("/client/config.json", get(client_config))
        .nest_service("/client", ServeDir::new(&state.config.client_web_dir))
        .with_state(state)
}

fn build_http_router(state: AppState) -> Router {
    build_router(state.clone()).layer(middleware::from_fn_with_state(state, http_to_https))
}

fn build_https_router(state: AppState) -> Router {
    build_router(state.clone()).layer(middleware::from_fn_with_state(state, alt_svc_h3))
}

pub(crate) fn should_redirect_browser_to_https(path: &str) -> bool {
    matches!(path, "/" | "/client" | "/client/" | "/client/index.html")
}

pub(crate) fn hostname_from_host_header(host: &str) -> &str {
    if let Some(rest) = host.strip_prefix('[') {
        if let Some(end) = rest.find(']') {
            return &host[..=end + 1];
        }
    }
    match host.rsplit_once(':') {
        Some((name, port)) if port.bytes().all(|b| b.is_ascii_digit()) => name,
        _ => host,
    }
}

pub(crate) fn https_redirect_location(host: &str, path_and_query: &str, https_port: u16) -> String {
    format!(
        "https://{}:{}{}",
        hostname_from_host_header(host),
        https_port,
        path_and_query
    )
}

async fn http_to_https(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    let Some(port) = state.wt_offer.as_ref().map(|o| o.port) else {
        return next.run(request).await;
    };
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::HEAD {
        return next.run(request).await;
    }
    if !should_redirect_browser_to_https(request.uri().path()) {
        return next.run(request).await;
    }
    let host = request
        .headers()
        .get(axum::http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    let pq = request
        .uri()
        .path_and_query()
        .map(|p| p.as_str())
        .unwrap_or(request.uri().path());
    axum::response::Redirect::permanent(&https_redirect_location(host, pq, port)).into_response()
}

async fn serve_https(
    port: u16,
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
    app: Router,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> Result<(), String> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let config = axum_server::tls_rustls::RustlsConfig::from_pem(cert_pem, key_pem)
        .await
        .map_err(|e| e.to_string())?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("orbiscreen web client listening on https://0.0.0.0:{port}");
    let handle = axum_server::Handle::new();
    let stop = handle.clone();
    tokio::spawn(async move {
        let _ = shutdown.changed().await;
        stop.shutdown();
    });
    axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .map_err(|e| e.to_string())
}

async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": state.version,
        "encoder": state.encoder_kind,
        "frames_forwarded": state.stats.frames_forwarded(),
        "active_clients": state.stats.active_clients(),
        "auth_failures": state.stats.auth_failures(),
        "usb_devices": state.stats.usb_devices(),
        "uptime_seconds": state.started.elapsed().as_secs(),
    }))
}

fn query_token(uri_query: Option<&str>) -> Option<&str> {
    uri_query?
        .split('&')
        .find_map(|pair| pair.strip_prefix("token="))
        .filter(|t| !t.is_empty())
}

fn request_credential(request: &axum::extract::Request) -> Option<String> {
    let header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            if v.get(..7)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("bearer "))
            {
                Some(v[7..].to_string())
            } else {
                None
            }
        });
    header.or_else(|| {
        query_token(request.uri().query())
            .filter(|t| !t.is_empty())
            .map(str::to_string)
    })
}

fn request_has_token(request: &axum::extract::Request, token: &str) -> bool {
    request_credential(request).is_some_and(|t| token_eq(&t, token))
}

async fn auth_check(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    if request_has_token(&request, &state.token) {
        return next.run(request).await;
    }
    if let Some(credential) = request_credential(&request) {
        if state.pairing.verify(&credential).is_some() {
            return next.run(request).await;
        }
    }
    state.stats.note_auth_failure();
    let peer = request
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|c| c.0.to_string())
        .unwrap_or_else(|| "?".into());
    warn!(
        "unauthorized request rejected (peer={}, {} {}, authorization_present={}, query_token_present={})",
        peer,
        request.method(),
        request.uri().path(),
        headers.contains_key(axum::http::header::AUTHORIZATION),
        query_token(request.uri().query()).is_some()
    );
    (
        StatusCode::UNAUTHORIZED,
        [(
            axum::http::header::WWW_AUTHENTICATE,
            HeaderValue::from_static("Bearer"),
        )],
        "unauthorized",
    )
        .into_response()
}

async fn api_info(State(state): State<AppState>) -> impl IntoResponse {
    let mut width = state.display_width;
    let mut height = state.display_height;
    let mut encoder = state.encoder_kind.to_string();
    if let Some(ctl) = &state.displays {
        if let Some(info) = ctl.lookup(None).await {
            width = info.width;
            height = info.height;
            encoder = info.encoder;
        }
    }
    let envelope = serde_json::json!({
        "display_width": width,
        "display_height": height,
        "refresh_hz": state.refresh_hz,
        "encoder": encoder,
        "version": state.version,
        "udp_port": udp_stream::default_udp_port(state.config.signaling_port),
        "wt_port": state.wt_offer.as_ref().map(|o| o.port),
        "wt_path": state.wt_offer.as_ref().map(|o| o.path),
        "cert_sha256": state.wt_offer.as_ref().map(|o| o.cert_sha256.clone()),
        "wt_hosts": state.wt_offer.as_ref().map(|o| o.hosts.clone()),
        "transport": ["http-mpegts", "udp-annexb", "webtransport-annexb"],
        "idr_path": "/idr",
    });
    Json(envelope)
}

fn query_value<'a>(uri_query: Option<&'a str>, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}=");
    uri_query?
        .split('&')
        .find_map(|pair| pair.strip_prefix(prefix.as_str()))
        .filter(|t| !t.is_empty())
}

async fn api_session_open(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    let _peer_ip = request
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip().to_string());
    let supplied_credential = request_credential(&request);
    let payload: serde_json::Value = match axum::Json::from_request(request, &()).await {
        Ok(axum::Json(value)) => value,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"ok": false, "error": "invalid json"})),
            )
                .into_response()
        }
    };
    let Some(ctl) = &state.displays else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(
                serde_json::json!({"ok": false, "error": "per-client displays are not available"}),
            ),
        )
            .into_response();
    };
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("client")
        .to_string();
    let key = payload
        .get("key")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let width = payload
        .get("width")
        .and_then(|v| v.as_u64())
        .unwrap_or(u64::from(state.display_width))
        .clamp(320, 7680) as u32;
    let height = payload
        .get("height")
        .and_then(|v| v.as_u64())
        .unwrap_or(u64::from(state.display_height))
        .clamp(240, 4320) as u32;

    let bitrate_kbps = payload
        .get("bitrate_kbps")
        .and_then(|v| v.as_u64())
        .map(|v| v.clamp(1_000, 120_000) as u32);
    let owner_id = supplied_credential
        .as_deref()
        .and_then(|c| state.pairing.verify(c))
        .map(|client| client.client_id);
    match ctl
        .acquire_with_bitrate(name, key, width, height, bitrate_kbps)
        .await
    {
        Ok(info) => {
            if let Some(owner_id) = &owner_id {
                if !state.pairing.bind_session(owner_id, &info.id) {
                    let _ = ctl.release(&info.id).await;
                    return (
                        StatusCode::FORBIDDEN,
                        Json(serde_json::json!({"ok": false, "error": "session bind failed"})),
                    )
                        .into_response();
                }
            }
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "ok": true,
                    "id": info.id,
                    "name": info.name,
                    "connector": info.connector,
                    "width": info.width,
                    "height": info.height,
                    "encoder": info.encoder,
                    "udp_port": udp_stream::default_udp_port(state.config.signaling_port),
                    "refresh_hz": state.refresh_hz,
                    "bitrate_kbps": bitrate_kbps,
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"ok": false, "error": e})),
        )
            .into_response(),
    }
}

async fn api_session_close(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    let Some(ctl) = &state.displays else {
        return StatusCode::NOT_IMPLEMENTED;
    };
    let id = query_value(request.uri().query(), "id")
        .or_else(|| query_value(request.uri().query(), "session"))
        .unwrap_or("")
        .to_string();
    if id.is_empty() {
        return StatusCode::BAD_REQUEST;
    }
    if let Some(credential) = request_credential(&request) {
        if let Some(client) = state.pairing.verify(&credential) {
            if !state.pairing.owns_session(&client.client_id, &id) {
                state.stats.note_auth_failure();
                return StatusCode::FORBIDDEN;
            }
            state.pairing.forget_session(&id);
        }
    }
    state.udp_keys.revoke_session(&id);
    ctl.release(&id).await;
    StatusCode::OK
}

async fn api_udp_key(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    use base64::Engine as _;
    let credential = request_credential(&request).unwrap_or_default();
    let payload: serde_json::Value = match axum::Json::from_request(request, &()).await {
        Ok(axum::Json(value)) => value,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"ok": false, "error": "invalid json"})),
            )
                .into_response();
        }
    };
    let session = payload
        .get("session")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim();
    let kind = if !state.token.is_empty()
        && token_eq(&credential, &state.token)
        && state.pairing.verify(&credential).is_none()
    {
        udp_crypto::MintKind::SharedToken
    } else if let Some(client) = state.pairing.verify(&credential) {
        udp_crypto::MintKind::Paired {
            owns_session: state.pairing.owns_session(&client.client_id, session),
        }
    } else {
        udp_crypto::MintKind::Anonymous
    };
    if !udp_crypto::may_mint_udp_key(kind, session) {
        state.stats.note_auth_failure();
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"ok": false, "error": "udp key denied"})),
        )
            .into_response();
    }
    let key = state
        .udp_keys
        .issue(Some(session.to_string()), udp_crypto::UDP_KEY_TTL);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ok": true,
            "key_id": udp_crypto::key_id_hex(&key.id),
            "key": base64::engine::general_purpose::STANDARD.encode(key.secret),
            "udp_port": udp_stream::default_udp_port(state.config.signaling_port),
            "expires_in": udp_crypto::UDP_KEY_TTL.as_secs(),
        })),
    )
        .into_response()
}

async fn client_config(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    let local = request
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .is_some_and(|peer| peer.0.ip().is_loopback());
    let supplied = request_credential(&request);
    let paired = supplied
        .as_deref()
        .is_some_and(|c| state.pairing.verify(c).is_some());
    let authenticated = request_has_token(&request, &state.token) || paired;
    if !local && !authenticated {
        state.stats.note_auth_failure();
        return (
            StatusCode::UNAUTHORIZED,
            [("cache-control", "no-store")],
            "authentication required",
        )
            .into_response();
    }
    (
        [
            ("content-type", "application/json"),
            ("cache-control", "no-cache, no-store, must-revalidate"),
        ],
        Json(serde_json::json!({
            "token": if paired { supplied.as_deref() } else { Some(state.token.as_str()) },
            "display_width": state.display_width,
            "display_height": state.display_height,
            "wt_port": state.wt_offer.as_ref().map(|o| o.port),
            "wt_path": state.wt_offer.as_ref().map(|o| o.path),
            "cert_sha256": state.wt_offer.as_ref().map(|o| o.cert_sha256.clone()),
            "wt_hosts": state.wt_offer.as_ref().map(|o| o.hosts.clone()),
        })),
    )
        .into_response()
}

async fn api_pair_request(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let label = payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("device");
    match state.pairing.request_pairing(label, &peer.ip().to_string()) {
        Some(request) => {
            info!(peer = %peer, "pairing request submitted; awaiting host approval");
            (
                StatusCode::OK,
                [("cache-control", "no-store")],
                Json(serde_json::json!({
                    "request_id": request.request_id,
                    "status": "pending",
                })),
            )
                .into_response()
        }
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "pairing unavailable"})),
        )
            .into_response(),
    }
}

async fn api_pair_status(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    let Some(request_id) = query_value(request.uri().query(), "id").map(str::to_string) else {
        return (
            StatusCode::BAD_REQUEST,
            [("cache-control", "no-store")],
            "missing request id",
        )
            .into_response();
    };
    let peer_str = peer.ip().to_string();
    let Some(request) = state
        .pairing
        .pending_requests()
        .into_iter()
        .find(|r| r.request_id == request_id)
    else {
        return (
            StatusCode::OK,
            [("cache-control", "no-store")],
            Json(serde_json::json!({"status": "unknown"})),
        )
            .into_response();
    };
    if request.peer != peer_str {
        state.stats.note_auth_failure();
        return (
            StatusCode::FORBIDDEN,
            [("cache-control", "no-store")],
            "request belongs to another peer",
        )
            .into_response();
    }
    let credential = state.pairing.claim(&request.request_id, &peer_str);
    match credential {
        Some(credential) => (
            StatusCode::OK,
            [("cache-control", "no-store")],
            Json(serde_json::json!({
                "status": "approved",
                "credential": credential,
            })),
        )
            .into_response(),
        None => (
            StatusCode::OK,
            [("cache-control", "no-store")],
            Json(serde_json::json!({"status": "pending"})),
        )
            .into_response(),
    }
}

async fn api_pair_list(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    if !is_host_request(&state, &request) {
        return (StatusCode::UNAUTHORIZED, "host authorization required").into_response();
    }
    (
        [("cache-control", "no-store")],
        Json(serde_json::json!({
            "requests": state.pairing.pending_requests(),
            "clients": state.pairing.clients(),
        })),
    )
        .into_response()
}

async fn api_pair_manage(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> axum::response::Response {
    if !is_host_request(&state, &request) {
        return (StatusCode::UNAUTHORIZED, "host authorization required").into_response();
    }
    let payload: serde_json::Value = match axum::Json::from_request(request, &()).await {
        Ok(axum::Json(value)) => value,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"ok": false, "error": "invalid json"})),
            )
                .into_response()
        }
    };
    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");
    let id = payload
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let outcome = match action {
        "approve" => state
            .pairing
            .approve(&id)
            .map(|client| {
                client
                    .map(|c| serde_json::json!({"ok": true, "client_id": c.client_id}))
                    .unwrap_or_else(|| serde_json::json!({"ok": false, "error": "unknown request"}))
            })
            .map_err(|e| e.to_string()),
        "deny" => state
            .pairing
            .deny(&id)
            .map(|removed| serde_json::json!({"ok": removed}))
            .map_err(|e| e.to_string()),
        "revoke" => state
            .pairing
            .revoke(&id)
            .map(|removed| serde_json::json!({"ok": removed}))
            .map_err(|e| e.to_string()),
        _ => Err(unavailable_pairing().to_string()),
    };
    match outcome {
        Ok(json) => ([("cache-control", "no-store")], Json(json)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"ok": false, "error": "pairing storage unavailable"})),
        )
            .into_response(),
    }
}

fn unavailable_pairing() -> std::io::Error {
    std::io::Error::other("pairing registry unavailable")
}

fn is_host_request(state: &AppState, request: &axum::extract::Request) -> bool {
    request_has_token(request, &state.token)
}

async fn run_command(program: &str, args: &[&str]) -> bool {
    use std::process::Stdio;
    match tokio::process::Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

fn is_wayland() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some()
}

async fn dpms_force(on: bool) -> bool {
    let state = if on { "on" } else { "off" };
    if is_wayland() {
        if run_command("swaymsg", &["output", "*", "dpms", state]).await {
            return true;
        }
        if run_command("hyprctl", &["dispatch", "dpms", state]).await {
            return true;
        }
        let active = (!on).to_string();
        if run_command(
            "gdbus",
            &[
                "call",
                "--session",
                "--dest",
                "org.gnome.ScreenSaver",
                "--object-path",
                "/org/gnome/ScreenSaver",
                "--method",
                "org.gnome.ScreenSaver.SetActive",
                &active,
            ],
        )
        .await
        {
            return true;
        }
    }
    run_command("xset", &["dpms", "force", state]).await
}

fn inject_ctrl_alt_del(tx: &mpsc::Sender<IncomingInput>) {
    const KEY_LEFTCTRL: u32 = 29;
    const KEY_LEFTALT: u32 = 56;
    const KEY_DELETE: u32 = 111;
    for (code, pressed) in [
        (KEY_LEFTCTRL, true),
        (KEY_LEFTALT, true),
        (KEY_DELETE, true),
        (KEY_LEFTCTRL, false),
        (KEY_LEFTALT, false),
        (KEY_DELETE, false),
    ] {
        let _ = tx.try_send(IncomingInput::Key(KeyEvent { code, pressed }));
    }
}

pub(crate) const IDR_DEBOUNCE: Duration = Duration::from_millis(250);

fn idr_due(last: Instant, now: Instant) -> bool {
    now.duration_since(last) >= IDR_DEBOUNCE
}

fn request_idr(state: &AppState, session: Option<&str>) {
    if let Some(ctl) = &state.displays {
        let ctl = ctl.clone();
        let id = session.unwrap_or("").to_string();
        tokio::spawn(async move {
            ctl.idr(&id).await;
        });
        return;
    }
    if let Some(tx) = &state.idr_tx {
        let _ = tx.try_send(());
    }
}

async fn api_control(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    match payload.get("action").and_then(|v| v.as_str()) {
        Some("lock") => {
            let ok = run_command("loginctl", &["lock-session"]).await
                || run_command("xdg-screensaver", &["lock"]).await;
            if ok {
                let _ = state.client_shutdown_tx.send(());
                info!("host control: session locked");
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            } else {
                warn!("host control: no usable screen-lock tool found");
                (
                    StatusCode::NOT_IMPLEMENTED,
                    Json(serde_json::json!({"ok": false, "error": "no lock tool available"})),
                )
            }
        }
        Some("blank") => {
            if dpms_force(false).await {
                info!("host control: display blanked");
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            } else {
                (
                    StatusCode::NOT_IMPLEMENTED,
                    Json(serde_json::json!({"ok": false, "error": "DPMS off not available"})),
                )
            }
        }
        Some("unblank") => {
            if dpms_force(true).await {
                info!("host control: display unblanked");
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            } else {
                (
                    StatusCode::NOT_IMPLEMENTED,
                    Json(serde_json::json!({"ok": false, "error": "DPMS on not available"})),
                )
            }
        }
        Some("ctrl_alt_del") => {
            inject_ctrl_alt_del(&state.input_tx);
            info!("host control: Ctrl+Alt+Del injected");
            (StatusCode::OK, Json(serde_json::json!({"ok": true})))
        }
        Some("idr") | Some("keyframe") => {
            let session = payload
                .get("session")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .or_else(|| {
                    headers
                        .get("x-orbiscreen-session")
                        .and_then(|v| v.to_str().ok())
                        .map(str::to_string)
                });
            request_idr(&state, session.as_deref());
            info!(session = ?session, "host control: IDR requested");
            (StatusCode::OK, Json(serde_json::json!({"ok": true})))
        }
        Some("set_resolution") => {
            let width = payload
                .get("width")
                .and_then(|v| v.as_u64())
                .unwrap_or(1920)
                .clamp(320, 7680) as u32;
            let height = payload
                .get("height")
                .and_then(|v| v.as_u64())
                .unwrap_or(1080)
                .clamp(240, 4320) as u32;
            let fps = payload
                .get("fps")
                .and_then(|v| v.as_u64())
                .unwrap_or(60)
                .clamp(30, 240) as u32;
            info!("host control: requested resolution change to {width}x{height}@{fps}Hz");
            if let Some(ctl) = &state.displays {
                let mut session = payload
                    .get("session")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if session.is_empty() {
                    session = ctl
                        .lookup(None)
                        .await
                        .map(|info| info.id)
                        .unwrap_or_default();
                }
                match ctl.resize(&session, width, height).await {
                    Ok(info) => {
                        return (
                            StatusCode::OK,
                            Json(serde_json::json!({
                                "ok": true,
                                "width": info.width,
                                "height": info.height,
                                "connector": info.connector,
                                "fps": fps,
                            })),
                        );
                    }
                    Err(e) => {
                        return (
                            StatusCode::BAD_GATEWAY,
                            Json(serde_json::json!({"ok": false, "error": e})),
                        );
                    }
                }
            }
            let fallback = if state.config.signaling_port == 8790 {
                "Virtual-ORBISCREEN-2"
            } else {
                "Virtual-ORBISCREEN"
            };
            let target_output = state.config.output_connector.as_deref().unwrap_or(fallback);
            info!("host control: requested resolution change on {target_output} to {width}x{height}@{fps}Hz");
            let mode_str = format!("output.{target_output}.mode.{width}x{height}@{fps}");
            let res = tokio::process::Command::new("kscreen-doctor")
                .arg(&mode_str)
                .status()
                .await;
            let ok = match res {
                Ok(s) if s.success() => true,
                _ => {
                    let fallback_str = format!("output.{target_output}.mode.{width}x{height}@60");
                    tokio::process::Command::new("kscreen-doctor")
                        .arg(&fallback_str)
                        .status()
                        .await
                        .map(|s| s.success())
                        .unwrap_or(false)
                }
            };
            if ok {
                let _ = state
                    .input_tx
                    .try_send(IncomingInput::Resize { width, height });
            }
            (
                StatusCode::OK,
                Json(serde_json::json!({"ok": ok, "width": width, "height": height, "fps": fps})),
            )
        }
        Some("open") => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"ok": false,
                "error": "opening arbitrary URLs from remote clients is not permitted"})),
        ),
        _ => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"ok": false, "error": "unknown action"})),
        ),
    }
}

async fn root_handler() -> Html<&'static str> {
    Html(
        r#"<!doctype html><html><head><meta charset=utf-8><title>Orbiscreen</title>
<meta http-equiv="refresh" content="0; url=/client/index.html"></head>
<body><a href="/client/index.html">Open the client</a></body></html>"#,
    )
}

async fn input_ws(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> axum::response::Response {
    let session = headers
        .get("x-orbiscreen-session")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    ws.on_upgrade(move |socket| handle_input_ws(socket, state.displays, state.input_tx, session))
}

async fn handle_input_ws(
    mut socket: axum::extract::ws::WebSocket,
    displays: Option<DisplayCtl>,
    input_tx: tokio::sync::mpsc::Sender<IncomingInput>,
    session: Option<String>,
) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let axum::extract::ws::Message::Text(text) = msg {
            if let Ok(input) = serde_json::from_str::<IncomingInput>(&text) {
                if let Some(ctl) = &displays {
                    ctl.input(session.clone(), input).await;
                } else if input_tx.try_send(input).is_err() {
                    debug!("input ws queue full; dropping event");
                }
            }
        }
    }
}

async fn input_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    match serde_json::from_value::<IncomingInput>(payload) {
        Ok(ev) => {
            if let Some(ctl) = &state.displays {
                let session = headers
                    .get("x-orbiscreen-session")
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string);
                ctl.input(session, ev).await;
            } else if state.input_tx.try_send(ev).is_err() {
                debug!("input queue full; dropping event");
            }
        }
        Err(_) => return StatusCode::BAD_REQUEST,
    }
    StatusCode::ACCEPTED
}

fn push_h264_packet(
    appsrc: &gstreamer_app::AppSrc,
    pkt: &H264Packet,
) -> Result<(), gstreamer::FlowError> {
    let valid = pkt.bytes.len() >= 3
        && pkt.bytes[0] == 0
        && pkt.bytes[1] == 0
        && (pkt.bytes[2] == 1 || pkt.bytes.len() >= 4 && pkt.bytes[2] == 0 && pkt.bytes[3] == 1);
    if !valid {
        let header_len = pkt.bytes.len().min(4);
        debug!(
            "skipping non-NAL packet: {} B (header={:02x?})",
            pkt.bytes.len(),
            &pkt.bytes[..header_len]
        );
        return Ok(());
    }

    let mut buffer =
        gstreamer::Buffer::with_size(pkt.bytes.len()).map_err(|_| gstreamer::FlowError::Error)?;
    {
        let buffer_mut = buffer.get_mut().ok_or_else(|| {
            warn!("gstreamer buffer not writable");
            gstreamer::FlowError::Error
        })?;
        if buffer_mut.copy_from_slice(0, &pkt.bytes).is_err() {
            warn!("packet larger than allocated gstreamer buffer");
            return Err(gstreamer::FlowError::Error);
        }
        if pkt.is_keyframe {
            buffer_mut.set_flags(gstreamer::BufferFlags::MARKER);
        }
        buffer_mut.set_pts(gstreamer::ClockTime::from_nseconds(pkt.pts_ns));
    }
    appsrc.push_buffer(buffer).map(|_| ())
}

const MAX_STREAM_CLIENTS: usize = 8;

async fn au_handler(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> axum::response::Response {
    annexb_byte_stream(state, request, false).await
}

async fn idr_handler(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> axum::response::Response {
    annexb_byte_stream(state, request, true).await
}

async fn annexb_byte_stream(
    state: AppState,
    request: axum::extract::Request,
    keys_only: bool,
) -> axum::response::Response {
    use tokio_stream::StreamExt;
    if state.stats.active_clients() >= MAX_STREAM_CLIENTS {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let session_q = query_value(request.uri().query(), "session").map(str::to_string);
    let key_q = query_value(request.uri().query(), "key").map(str::to_string);
    let attached = if let Some(ctl) = &state.displays {
        match ctl.attach(session_q.clone(), key_q).await {
            Ok(att) => Some(att),
            Err(e) => {
                warn!("au attach failed: {e}");
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            }
        }
    } else {
        None
    };
    let session_id = attached.as_ref().map(|a| a.info.id.clone());
    let mut video_rx = if let Some(att) = attached {
        att.video
    } else {
        state.video_tx.subscribe()
    };
    state.stats.client_started();
    let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(8);
    let stats = state.stats.clone();
    let idr_tx = state.idr_tx.clone();
    let displays = state.displays.clone();
    let lag_ctl = displays.clone();
    let lag_session = session_id.clone();
    tokio::spawn(async move {
        let _guard = ClientGuard(stats);
        struct DetachGuard(Option<(DisplayCtl, String)>);
        impl Drop for DetachGuard {
            fn drop(&mut self) {
                if let Some((ctl, id)) = self.0.take() {
                    tokio::spawn(async move { ctl.detach(&id).await });
                }
            }
        }
        let _detach = DetachGuard(displays.zip(session_id));
        let mut wait_key = true;
        let mut last_idr = Instant::now()
            .checked_sub(IDR_DEBOUNCE)
            .unwrap_or_else(Instant::now);
        let mut request_idr = || {
            let now = Instant::now();
            if now.duration_since(last_idr) < IDR_DEBOUNCE {
                return;
            }
            last_idr = now;
            if let (Some(ctl), Some(id)) = (lag_ctl.as_ref(), lag_session.as_ref()) {
                let ctl = ctl.clone();
                let id = id.clone();
                tokio::spawn(async move { ctl.idr(&id).await });
            } else if let Some(tx) = &idr_tx {
                let _ = tx.try_send(());
            }
        };
        request_idr();
        let mut cached_sps_pps: Option<annexb::SpsPps> = None;
        loop {
            if tx.is_closed() {
                break;
            }
            let pkt = match video_rx.recv().await {
                Ok(p) => p,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    wait_key = true;
                    request_idr();
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            };
            if keys_only && !pkt.is_keyframe {
                continue;
            }
            if wait_key {
                if !pkt.is_keyframe {
                    request_idr();
                    continue;
                }
                wait_key = false;
            }
            let mut pkt = pkt;
            if pkt.is_keyframe {
                let found = annexb::extract_sps_pps(&pkt.bytes);
                if !found.sps.is_empty() && !found.pps.is_empty() {
                    cached_sps_pps = Some(found);
                }
                pkt.bytes = annexb::with_parameter_sets(&pkt.bytes, cached_sps_pps.as_ref());
            }
            let Ok(frame) = wt_protocol::encode_video(&pkt, {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0)
            }) else {
                continue;
            };
            if tx.send(frame).await.is_err() {
                break;
            }
        }
    });
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx)
        .map(|chunk| Ok::<_, std::convert::Infallible>(axum::body::Bytes::from(chunk)));
    (
        [
            ("content-type", "application/octet-stream"),
            ("cache-control", "no-cache, no-store, must-revalidate"),
        ],
        axum::body::Body::from_stream(stream),
    )
        .into_response()
}

async fn stream_head_handler() -> impl IntoResponse {
    ([("content-type", "video/mp2t")], StatusCode::OK)
}

#[derive(serde::Deserialize, Debug, Default)]
struct StreamQuery {
    session: Option<String>,
    key: Option<String>,
}

fn build_video_pipeline() -> Result<
    (
        gstreamer::Pipeline,
        gstreamer_app::AppSrc,
        gstreamer_app::AppSink,
    ),
    (),
> {
    use gstreamer::prelude::*;
    use gstreamer_app::{AppSink, AppSrc};
    let pipeline_str = "appsrc name=src format=time is-live=true block=true do-timestamp=false \
                        ! video/x-h264,stream-format=byte-stream,alignment=au \
                        ! h264parse config-interval=1 \
                        ! mpegtsmux alignment=7 \
                        ! appsink name=sink drop=false sync=false max-buffers=8 emit-signals=false";
    let p = gstreamer::parse::launch(pipeline_str).map_err(|_| ())?;
    let pipeline = p.downcast::<gstreamer::Pipeline>().map_err(|_| ())?;
    let appsrc = pipeline
        .by_name("src")
        .and_then(|e| e.downcast::<AppSrc>().ok())
        .ok_or(())?;
    let appsink = pipeline
        .by_name("sink")
        .and_then(|e| e.downcast::<AppSink>().ok())
        .ok_or(())?;
    Ok((pipeline, appsrc, appsink))
}

async fn stream_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<StreamQuery>,
) -> axum::response::Response {
    use gstreamer::prelude::*;
    use gstreamer_app::{AppSink, AppSinkCallbacks, AppSrc};
    use tokio_stream::StreamExt;

    if state.stats.active_clients() >= MAX_STREAM_CLIENTS {
        warn!("stream client limit reached ({MAX_STREAM_CLIENTS}); rejecting connection");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    gstreamer::init().ok();

    let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(2048);
    let tx_alive = tx.clone();

    let setup_pipeline = |pipeline: &gstreamer::Pipeline,
                          appsrc: &AppSrc,
                          appsink: &AppSink,
                          tx: tokio::sync::mpsc::Sender<Vec<u8>>| {
        let caps = gstreamer::Caps::builder("video/x-h264")
            .field("stream-format", "byte-stream")
            .field("alignment", "au")
            .build();
        appsrc.set_caps(Some(&caps));
        appsrc.set_format(gstreamer::Format::Time);
        appsrc.set_do_timestamp(false);
        appsrc.set_max_bytes(4 * 1024 * 1024);
        appsrc.set_block(true);

        appsink.set_sync(false);
        appsink.set_drop(false);
        appsink.set_max_buffers(0);

        appsink.set_callbacks(
            AppSinkCallbacks::builder()
                .new_sample(move |sink| match sink.pull_sample() {
                    Ok(sample) => {
                        if let Some(buffer) = sample.buffer() {
                            if let Ok(map) = buffer.map_readable() {
                                if tx.blocking_send(map.to_vec()).is_err() {
                                    return Err(gstreamer::FlowError::Eos);
                                }
                            }
                        }
                        Ok(gstreamer::FlowSuccess::Ok)
                    }
                    Err(e) => {
                        debug!("pull_sample EOS/err: {e}");
                        Err(gstreamer::FlowError::Eos)
                    }
                })
                .build(),
        );

        if let Some(bus) = pipeline.bus() {
            bus.set_sync_handler(|_bus, msg| {
                match msg.view() {
                    gstreamer::MessageView::Error(err) => tracing::error!(
                        target: "orbiscreen_transport",
                        "stream pipeline error: {} (debug: {})",
                        err.error(),
                        err.debug().unwrap_or_default()
                    ),
                    gstreamer::MessageView::Warning(warn) => tracing::warn!(
                        target: "orbiscreen_transport",
                        "stream pipeline warning: {} (debug: {})",
                        warn.error(),
                        warn.debug().unwrap_or_default()
                    ),
                    _ => {}
                }
                gstreamer::BusSyncReply::Drop
            });
        }
    };

    struct PipelineGuard(Option<gstreamer::Pipeline>);
    impl Drop for PipelineGuard {
        fn drop(&mut self) {
            if let Some(p) = self.0.take() {
                let _ = p.set_state(gstreamer::State::Null);
            }
        }
    }

    let session_q = query.session.clone().or_else(|| {
        headers
            .get("x-orbiscreen-session")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    });
    let key_q = query.key.clone().or_else(|| {
        headers
            .get("x-orbiscreen-client-key")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    });
    let attached = if let Some(ctl) = &state.displays {
        match ctl.attach(session_q.clone(), key_q).await {
            Ok(att) => Some(att),
            Err(e) => {
                warn!("stream attach failed: {e}");
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            }
        }
    } else {
        None
    };
    let session_id = attached.as_ref().map(|a| a.info.id.clone());

    let (p, src, sink) = match build_video_pipeline() {
        Ok(res) => res,
        Err(_) => {
            warn!("failed to build video pipeline");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    };
    let mut pipeline_guard = PipelineGuard(Some(p.clone()));
    setup_pipeline(&p, &src, &sink, tx.clone());
    if let Err(e) = p.set_state(gstreamer::State::Playing) {
        warn!("stream pipeline failed to reach playing state: {e}");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let (_pipeline, appsrc, _appsink) = (p, src, sink);

    state.stats.client_started();
    let appsrc_clone = appsrc.clone();
    let stats = state.stats.clone();
    let idr_tx = state.idr_tx.clone();
    let displays = state.displays.clone();
    let lag_ctl = displays.clone();
    let lag_session = session_id.clone();
    let Some(pipeline_for_task) = pipeline_guard.0.take() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let refresh_hz = state.refresh_hz.max(1);
    let nominal_frame_ns = 1_000_000_000u64 / u64::from(refresh_hz);
    let mut video_rx = if let Some(att) = attached {
        att.video
    } else {
        state.video_tx.subscribe()
    };
    let mut client_shutdown_rx = state.client_shutdown_tx.subscribe();

    tokio::spawn(async move {
        let mut _pipeline_guard = PipelineGuard(Some(pipeline_for_task));
        let _guard = ClientGuard(stats);
        struct DetachGuard(Option<(DisplayCtl, String)>);
        impl Drop for DetachGuard {
            fn drop(&mut self) {
                if let Some((ctl, id)) = self.0.take() {
                    tokio::spawn(async move { ctl.detach(&id).await });
                }
            }
        }
        let _detach = DetachGuard(displays.zip(session_id));

        let mut wait_keyframe = true;
        let mut stream_pts_ns: u64 = 0;
        let mut last_pkt_pts_ns: Option<u64> = None;
        let mut last_idr_at = Instant::now()
            .checked_sub(IDR_DEBOUNCE)
            .unwrap_or_else(Instant::now);
        let mut request_idr = || {
            let now = Instant::now();
            if !idr_due(last_idr_at, now) {
                return;
            }
            last_idr_at = now;
            if let (Some(ctl), Some(id)) = (lag_ctl.as_ref(), lag_session.as_ref()) {
                let ctl = ctl.clone();
                let id = id.clone();
                tokio::spawn(async move { ctl.idr(&id).await });
            } else if let Some(tx) = &idr_tx {
                let _ = tx.try_send(());
            }
        };

        request_idr();
        loop {
            if tx_alive.is_closed() {
                debug!("stream client disconnected");
                break;
            }
            let pkt = tokio::select! {
                _ = client_shutdown_rx.recv() => {
                    debug!("stream client shutting down due to session lock or daemon shutdown");
                    break;
                }
                _ = tx_alive.closed() => {
                    debug!("stream client connection closed");
                    break;
                }
                res = video_rx.recv() => match res {
                    Ok(pkt) => pkt,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        debug!("stream client lagged {n} packets; waiting for keyframe");
                        wait_keyframe = true;
                        last_pkt_pts_ns = None;
                        request_idr();
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                },
            };
            if wait_keyframe {
                if !pkt.is_keyframe {
                    request_idr();
                    continue;
                }
                wait_keyframe = false;
            }

            let delta_ns = match last_pkt_pts_ns {
                Some(last) => {
                    let diff = pkt.pts_ns.saturating_sub(last);

                    if diff > 250_000_000 {
                        debug!(
                            diff_ms = diff / 1_000_000,
                            "large timestamp gap detected; clamping stream PTS delta to prevent player desync"
                        );
                        nominal_frame_ns
                    } else {
                        diff
                    }
                }
                None => 0,
            };
            last_pkt_pts_ns = Some(pkt.pts_ns);
            stream_pts_ns = stream_pts_ns.saturating_add(delta_ns);

            let mut normalized = pkt;
            normalized.pts_ns = stream_pts_ns;

            if let Err(e) = push_h264_packet(&appsrc_clone, &normalized) {
                match e {
                    gstreamer::FlowError::Flushing | gstreamer::FlowError::Eos => break,
                    _ => {
                        wait_keyframe = true;
                        request_idr();
                    }
                }
            }
        }
        if let Some(p) = _pipeline_guard.0.take() {
            let _ = p.set_state(gstreamer::State::Null);
        }
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx)
        .map(|chunk| Ok::<_, std::convert::Infallible>(axum::body::Bytes::from(chunk)));

    (
        [
            ("content-type", "video/mp2t"),
            ("cache-control", "no-cache, no-store, must-revalidate"),
        ],
        axum::body::Body::from_stream(stream),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_test_state() -> AppState {
        let (input_tx, _) = mpsc::channel(1);
        let (video_tx, _) = tokio::sync::broadcast::channel(1);
        let (client_shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        AppState {
            config: ServerConfig {
                signaling_port: 8788,
                client_web_dir: PathBuf::from("clients/web"),
                enable_usb_supervisors: false,
                output_connector: None,
            },
            input_tx,
            video_tx,
            stats: Arc::new(Stats::default()),
            token: "test-credential".into(),
            pairing: Arc::new(pairing::PairingRegistry::load_from(None).unwrap()),
            display_width: 1920,
            display_height: 1080,
            refresh_hz: 60,
            encoder_kind: "auto",
            version: "test",
            started: Instant::now(),
            idr_tx: None,
            client_shutdown_tx,
            displays: None,
            wt_offer: None,
            udp_keys: udp_crypto::UdpKeyRing::new(),
        }
    }

    #[tokio::test]
    async fn config_bootstrap_requires_loopback_or_authentication() {
        for (peer, credential, expected) in [
            (Some("127.0.0.1:1234"), None, StatusCode::OK),
            (Some("[::1]:1234"), None, StatusCode::OK),
            (Some("192.0.2.1:1234"), None, StatusCode::UNAUTHORIZED),
            (None, None, StatusCode::UNAUTHORIZED),
            (
                Some("192.0.2.1:1234"),
                Some("wrong"),
                StatusCode::UNAUTHORIZED,
            ),
            (
                Some("192.0.2.1:1234"),
                Some("test-credential"),
                StatusCode::OK,
            ),
        ] {
            let state = config_test_state();
            let stats = state.stats.clone();
            let mut builder = axum::http::Request::builder().uri("/client/config.json");
            if let Some(credential) = credential {
                builder = builder.header("authorization", format!("Bearer {credential}"));
            }
            let mut request = builder.body(axum::body::Body::empty()).unwrap();
            if let Some(peer) = peer {
                request.extensions_mut().insert(axum::extract::ConnectInfo(
                    peer.parse::<SocketAddr>().unwrap(),
                ));
            }
            let response = client_config(State(state), request).await.into_response();
            assert_eq!(response.status(), expected);
            assert!(response.headers()["cache-control"]
                .to_str()
                .unwrap()
                .contains("no-store"));
            let body = axum::body::to_bytes(response.into_body(), 4096)
                .await
                .unwrap();
            if expected == StatusCode::OK {
                let config: serde_json::Value = serde_json::from_slice(&body).unwrap();
                assert_eq!(config["token"], "test-credential");
            } else {
                assert_eq!(stats.auth_failures(), 1);
                assert!(!String::from_utf8_lossy(&body).contains("test-credential"));
            }
        }
    }

    #[test]
    fn token_authentication_accepts_only_matching_credentials() {
        for (authorization, query, accepted) in [
            (Some("bEaReR test-credential"), "", true),
            (Some("Bearer wrong"), "", false),
            (Some("Basic test-credential"), "", false),
            (Some("Bearer "), "", false),
            (None, "?token=test-credential", true),
            (None, "?token=wrong", false),
            (None, "", false),
        ] {
            let mut builder = axum::http::Request::builder().uri(format!("/input{query}"));
            if let Some(authorization) = authorization {
                builder = builder.header("authorization", authorization);
            }
            let request = builder.body(axum::body::Body::empty()).unwrap();
            assert_eq!(request_has_token(&request, "test-credential"), accepted);
        }
    }

    #[test]
    fn browser_paths_redirect_to_https() {
        assert!(should_redirect_browser_to_https("/"));
        assert!(should_redirect_browser_to_https("/client/index.html"));
        assert!(!should_redirect_browser_to_https("/health"));
        assert!(!should_redirect_browser_to_https("/client/config.json"));
        assert!(!should_redirect_browser_to_https("/api/info"));
        assert!(!should_redirect_browser_to_https("/stream"));
        assert!(!should_redirect_browser_to_https("/idr"));
        assert!(!should_redirect_browser_to_https("/au"));
    }

    #[test]
    fn https_redirect_rewrites_host_port() {
        assert_eq!(
            https_redirect_location("192.168.39.191:8788", "/client/index.html?token=a", 8790),
            "https://192.168.39.191:8790/client/index.html?token=a"
        );
        assert_eq!(hostname_from_host_header("[fe80::1]:8788"), "[fe80::1]");
        assert_eq!(hostname_from_host_header("localhost"), "localhost");
    }

    #[test]
    fn service_descriptor_carries_port() {
        let svc = ServiceDescriptor {
            instance: "my-laptop".into(),
            port: 8788,
            token: None,
        };
        assert_eq!(svc.port, 8788);
        assert_eq!(svc.instance, "my-laptop");
    }

    #[test]
    fn h264_packet_roundtrips_debug() {
        let pkt = H264Packet {
            bytes: vec![0, 1, 2],
            is_keyframe: true,
            pts_ns: 16_666_667,
        };
        let s = format!("{pkt:?}");
        assert!(s.contains("is_keyframe"));
        assert!(s.contains("true"));
    }

    #[test]
    fn generated_token_has_expected_entropy() {
        let a = generate_token();
        let b = generate_token();
        assert_ne!(a, b);
        assert!(a.len() >= 40);
        assert!(a
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn token_eq_is_constant_time_shape() {
        assert!(token_eq("abc", "abc"));
        assert!(!token_eq("abc", "abd"));
        assert!(!token_eq("abc", "abcd"));
    }

    #[test]
    fn idr_debounce_is_250ms() {
        assert_eq!(IDR_DEBOUNCE, Duration::from_millis(250));
        let t0 = Instant::now();
        assert!(!idr_due(t0, t0 + Duration::from_millis(249)));
        assert!(idr_due(t0, t0 + Duration::from_millis(250)));
    }

    fn run_node(args: &[&str]) -> Option<std::process::Output> {
        match std::process::Command::new("node").args(args).output() {
            Ok(output) => Some(output),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => panic!("node: {e}"),
        }
    }

    #[test]
    fn js_hello_frame_decodes_in_rust() {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let script = manifest.join("../../clients/web/annexb.js");
        let Some(output) = run_node(&[
            "-e",
            &format!(
                "const a=require({}); process.stdout.write(Buffer.from(a.encodeHello('tok','sess')));",
                serde_json::to_string(&script.to_string_lossy()).unwrap()
            ),
        ]) else {
            eprintln!("skipping js_hello_frame_decodes_in_rust: node not installed");
            return;
        };
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let (body, used) = wt_protocol::split_frame(&output.stdout)
            .unwrap()
            .expect("complete frame");
        assert_eq!(used, output.stdout.len());
        match wt_protocol::decode_message(body).unwrap() {
            wt_protocol::Message::Hello(h) => {
                assert_eq!(h.token, "tok");
                assert_eq!(h.session, "sess");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn annexb_js_unit_tests() {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let script = manifest.join("../../clients/web/annexb.test.js");
        let Some(output) = run_node(&["--test", script.to_str().expect("utf-8 path")]) else {
            eprintln!("skipping annexb_js_unit_tests: node not installed");
            return;
        };
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn stats_js_unit_tests() {
        let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let script = manifest.join("../../clients/web/stats.test.js");
        let Some(output) = run_node(&["--test", script.to_str().expect("utf-8 path")]) else {
            eprintln!("skipping stats_js_unit_tests: node not installed");
            return;
        };
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn stats_track_clients_and_frames() {
        let stats = Arc::new(Stats::default());
        stats.note_frame();
        stats.client_started();
        stats.client_started();
        stats.client_stopped();
        assert_eq!(stats.frames_forwarded(), 1);
        assert_eq!(stats.active_clients(), 1);
        assert_eq!(stats.total_clients(), 2);
        drop(ClientGuard(Arc::clone(&stats)));
        assert_eq!(stats.active_clients(), 0);
        let presence = stats.subscribe_presence();
        assert!(!*presence.borrow());
        stats.client_started();
        assert!(*stats.subscribe_presence().borrow());
        stats.client_stopped();
        assert!(!*stats.subscribe_presence().borrow());
        stats.client_stopped();
        assert_eq!(stats.active_clients(), 0);
    }

    #[test]
    fn stats_track_auth_failures() {
        let stats = Arc::new(Stats::default());
        assert_eq!(stats.auth_failures(), 0);
        stats.note_auth_failure();
        stats.note_auth_failure();
        assert_eq!(stats.auth_failures(), 2);
    }

    #[test]
    fn incoming_input_parses_touch_payload() {
        let json = serde_json::json!({
            "Touch": {"slot": 2, "id": 7, "x": 100.0, "y": 200.0, "pressed": true}
        });
        let ev: IncomingInput = serde_json::from_value(json).expect("touch payload");
        match ev {
            IncomingInput::Touch(t) => {
                assert_eq!(t.slot, 2);
                assert_eq!(t.id, 7);
                assert_eq!(t.x, 100.0);
                assert_eq!(t.y, 200.0);
                assert!(t.pressed);
            }
            other => panic!("expected Touch, got {other:?}"),
        }
    }
}
