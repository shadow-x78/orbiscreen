// Orbiscreen - lib.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

pub mod adb;
pub mod mdns;

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use axum::extract::State;
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

use orbiscreen_input::{KeyEvent, PointerEvent, StylusEvent};

#[derive(Debug, Clone, serde::Deserialize)]
pub enum IncomingInput {
    Pointer(PointerEvent),
    Key(KeyEvent),
    Stylus(StylusEvent),
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
}

#[derive(Debug, Default)]
pub struct Stats {
    frames_forwarded: AtomicU64,
    active_clients: AtomicUsize,
    total_clients: AtomicU64,
    auth_failures: AtomicU64,
    usb_devices: AtomicUsize,
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

    fn note_frame(&self) {
        self.frames_forwarded.fetch_add(1, Ordering::Relaxed);
    }

    fn note_auth_failure(&self) {
        self.auth_failures.fetch_add(1, Ordering::Relaxed);
    }

    fn note_usb_devices(&self, count: usize) {
        self.usb_devices.store(count, Ordering::Relaxed);
    }

    pub fn client_started(&self) {
        self.active_clients.fetch_add(1, Ordering::Relaxed);
        self.total_clients.fetch_add(1, Ordering::Relaxed);
    }

    pub fn client_stopped(&self) {
        self.active_clients.fetch_sub(1, Ordering::Relaxed);
    }
}

struct ClientGuard(Arc<Stats>);

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

fn token_eq(a: &str, b: &str) -> bool {
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
        }
    }

    pub fn token(&self) -> &str {
        &self.token
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
    ) -> Result<(), TransportError> {
        let input_tx = self.input_tx;
        let (video_tx, _video_rx) = tokio::sync::broadcast::channel::<SeqPacket>(16);
        let join_buffer: Arc<Mutex<VecDeque<SeqPacket>>> = Arc::new(Mutex::new(VecDeque::new()));
        let state = AppState {
            config: self.cfg.clone(),
            input_tx,
            video_tx: video_tx.clone(),
            join_buffer: Arc::clone(&join_buffer),
            stats,
            token: self.token.clone(),
            display_width,
            display_height,
            refresh_hz,
            encoder_kind,
            version: env!("CARGO_PKG_VERSION"),
            started: std::time::Instant::now(),
        };
        let app = build_router(state.clone());
        let listener = TcpListener::bind(("0.0.0.0", self.cfg.signaling_port))
            .await
            .map_err(|e| TransportError::Http(e.to_string()))?;
        let local = listener
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "?".into());
        info!("orbiscreen transport listening on http://{local}");

        let adb_port = self.cfg.signaling_port;
        let adb_stats = state.stats.clone();
        let adb_stats_for_exit = state.stats.clone();
        let mut adb_shutdown = shutdown_rx.clone();
        let adb_task = tokio::spawn(async move {
            let mut known: Vec<String> = Vec::new();
            loop {
                let port_now = adb_port;
                let joined = tokio::task::spawn_blocking(move || {
                    adb::setup_reverse_for_all(adb::default_adb_path(), port_now)
                })
                .await;
                match joined {
                    Ok(Ok(devices)) => {
                        for serial in &devices {
                            if !known.contains(serial) {
                                info!("ADB reverse tunnel established on USB device {serial}");
                            }
                        }
                        for serial in &known {
                            if !devices.contains(serial) {
                                info!("USB device {serial} disconnected (tunnel closed by adb)");
                            }
                        }
                        known = devices;
                    }
                    Ok(Err(adb::AdbError::NoDevice | adb::AdbError::NotInstalled)) => {
                        if !known.is_empty() {
                            for serial in &known {
                                info!("USB device {serial} disconnected (tunnel closed by adb)");
                            }
                        }
                        known.clear();
                    }
                    Ok(Err(e)) => debug!("ADB reverse port forwarding inactive: {e}"),
                    Err(_) => break,
                }
                adb_stats.note_usb_devices(known.len());
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {}
                    _ = adb_shutdown.changed() => break,
                }
            }
            adb_stats.note_usb_devices(0);
        });

        let stats_pump = state.stats.clone();
        tokio::spawn(async move {
            let mut frames = frames;
            let next_seq = std::sync::atomic::AtomicU64::new(0);
            while let Some(pkt) = frames.recv().await {
                let seq = next_seq.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let sp = SeqPacket { seq, pkt };
                if stats_pump.active_clients() > 0 {
                    stats_pump.note_frame();
                }
                let mut jb = join_buffer.lock().unwrap_or_else(|e| e.into_inner());
                if sp.pkt.is_keyframe {
                    jb.clear();
                    jb.push_back(sp.clone());
                }
                let _ = video_tx.send(sp);
            }
        });

        let serve_fut = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        );
        tokio::select! {
            res = serve_fut => {
                res.map_err(|e| TransportError::Http(e.to_string()))?;
            }
            _ = shutdown_rx.changed() => {}
            _ = tokio::signal::ctrl_c() => {}
        }

        adb_task.abort();
        adb_stats_for_exit.note_usb_devices(0);
        let teardown = tokio::task::spawn_blocking(move || {
            adb::teardown_reverse_for_all(adb::default_adb_path(), adb_port)
        })
        .await;
        match teardown {
            Ok(Ok(devices)) => {
                info!("ADB reverse tunnels removed for devices: {devices:?}")
            }
            Ok(Err(adb::AdbError::NoDevice | adb::AdbError::NotInstalled)) => {}
            Ok(Err(e)) => debug!("ADB reverse teardown inactive: {e}"),
            Err(_) => debug!("ADB reverse teardown task aborted"),
        }
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
struct SeqPacket {
    seq: u64,
    pkt: H264Packet,
}

#[derive(Clone)]
struct AppState {
    config: ServerConfig,
    input_tx: mpsc::Sender<IncomingInput>,
    video_tx: tokio::sync::broadcast::Sender<SeqPacket>,
    join_buffer: Arc<Mutex<VecDeque<SeqPacket>>>,
    stats: Arc<Stats>,
    token: String,
    display_width: u32,
    display_height: u32,
    refresh_hz: u32,
    encoder_kind: &'static str,
    version: &'static str,
    started: std::time::Instant,
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/stream", get(stream_handler).head(stream_head_handler))
        .route("/input", post(input_post))
        .route("/api/control", post(api_control))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_check))
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/api/info", get(api_info))
        .route("/client/config.json", get(client_config))
        .nest_service("/client", ServeDir::new(&state.config.client_web_dir))
        .with_state(state)
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

async fn auth_check(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    let header_ok = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            if v.len() > 7 && v[..7].eq_ignore_ascii_case("bearer ") {
                Some(&v[7..])
            } else {
                None
            }
        })
        .is_some_and(|t| token_eq(t, &state.token));
    let query_ok = query_token(request.uri().query()).is_some_and(|t| token_eq(t, &state.token));
    if header_ok || query_ok {
        next.run(request).await
    } else {
        state.stats.note_auth_failure();
        let peer = request
            .extensions()
            .get::<axum::extract::ConnectInfo<SocketAddr>>()
            .map(|c| c.0.to_string())
            .unwrap_or_else(|| "?".into());
        let auth_desc = match headers.get(axum::http::header::AUTHORIZATION) {
            None => "missing".to_string(),
            Some(v) => match v.to_str() {
                Err(_) => "non-utf8".to_string(),
                Ok(s) if s.len() > 7 && s[..7].eq_ignore_ascii_case("bearer ") => {
                    let t = &s[7..];
                    format!("bearer(len={} prefix={})", t.len(), t.get(..4).unwrap_or(t))
                }
                Ok(s) => format!(
                    "unexpected(scheme={})",
                    s.split_whitespace().next().unwrap_or("?")
                ),
            },
        };
        warn!(
            "unauthorized request rejected (peer={}, {} {}, auth={}, query_token={}, expected prefix={} len={})",
            peer,
            request.method(),
            request.uri().path(),
            auth_desc,
            query_token(request.uri().query()).is_some(),
            state.token.get(..4).unwrap_or(&state.token),
            state.token.len()
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
}

async fn api_info(State(state): State<AppState>) -> impl IntoResponse {
    let envelope = serde_json::json!({
        "display_width": state.display_width,
        "display_height": state.display_height,
        "refresh_hz": state.refresh_hz,
        "encoder": state.encoder_kind,
        "version": state.version,
    });
    Json(envelope)
}

async fn client_config(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    debug!(
        "client config served with live token (peer={})",
        request
            .extensions()
            .get::<axum::extract::ConnectInfo<SocketAddr>>()
            .map(|c| c.0.to_string())
            .unwrap_or_else(|| "?".into())
    );
    (
        [
            ("content-type", "application/json"),
            ("cache-control", "no-cache, no-store, must-revalidate"),
        ],
        Json(serde_json::json!({
            "token": state.token,
            "display_width": state.display_width,
            "display_height": state.display_height,
        })),
    )
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

async fn api_control(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    match payload.get("action").and_then(|v| v.as_str()) {
        Some("lock") => {
            let ok = run_command("loginctl", &["lock-session"]).await
                || run_command("xdg-screensaver", &["lock"]).await;
            if ok {
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
        Some("set_resolution") => {
            let width = payload
                .get("width")
                .and_then(|v| v.as_u64())
                .unwrap_or(1920) as u32;
            let height = payload
                .get("height")
                .and_then(|v| v.as_u64())
                .unwrap_or(1080) as u32;
            info!("host control: requested resolution change to {width}x{height}");
            // If running on KDE Plasma, invoke kscreen-doctor to switch virtual output mode
            let mode_str = format!("output.Virtual-ORBISCREEN.mode.{width}x{height}@60");
            let _ = tokio::process::Command::new("kscreen-doctor")
                .arg(&mode_str)
                .status()
                .await;
            (
                StatusCode::OK,
                Json(serde_json::json!({"ok": true, "width": width, "height": height})),
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

async fn input_post(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    match serde_json::from_value::<IncomingInput>(payload) {
        Ok(ev) => {
            if state.input_tx.try_send(ev).is_err() {
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

async fn stream_head_handler() -> impl IntoResponse {
    ([("content-type", "video/mp2t")], StatusCode::OK)
}

async fn stream_handler(State(state): State<AppState>) -> axum::response::Response {
    use gstreamer::prelude::*;
    use gstreamer_app::{AppSink, AppSinkCallbacks, AppSrc};
    use tokio_stream::StreamExt;

    if state.stats.active_clients() >= MAX_STREAM_CLIENTS {
        warn!("stream client limit reached ({MAX_STREAM_CLIENTS}); rejecting connection");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    gstreamer::init().ok();

    let pipeline_str =
        "appsrc name=src format=time is-live=true do-timestamp=false min-latency=0 max-latency=0 \
                        ! video/x-h264,stream-format=byte-stream,alignment=au,framerate=0/1 \
                        ! h264parse config-interval=1 \
                        ! mpegtsmux alignment=7 \
                        ! appsink name=sink drop=false sync=false max-buffers=512 emit-signals=false";
    let pipeline = match gstreamer::parse::launch(pipeline_str) {
        Ok(p) => match p.downcast::<gstreamer::Pipeline>() {
            Ok(pipeline) => pipeline,
            Err(_) => {
                warn!("stream pipeline did not downcast to Pipeline");
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            }
        },
        Err(e) => {
            warn!("failed to build stream pipeline: {e}");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    };

    let appsrc = match pipeline
        .by_name("src")
        .and_then(|e| e.downcast::<AppSrc>().ok())
    {
        Some(s) => s,
        None => {
            warn!("stream pipeline missing appsrc element");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    };

    let caps = gstreamer::Caps::builder("video/x-h264")
        .field("stream-format", "byte-stream")
        .field("alignment", "au")
        .field("framerate", gstreamer::Fraction::new(0, 1))
        .build();
    appsrc.set_caps(Some(&caps));
    appsrc.set_format(gstreamer::Format::Time);

    let appsink = match pipeline
        .by_name("sink")
        .and_then(|e| e.downcast::<AppSink>().ok())
    {
        Some(s) => s,
        None => {
            warn!("stream pipeline missing appsink element");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    };

    let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(512);
    let tx_alive = tx.clone();
    appsink.set_callbacks(
        AppSinkCallbacks::builder()
            .new_sample(move |sink| match sink.pull_sample() {
                Ok(sample) => {
                    if let Some(buffer) = sample.buffer() {
                        if let Ok(map) = buffer.map_readable() {
                            let _ = tx.try_send(map.to_vec());
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

    if let Err(e) = pipeline.set_state(gstreamer::State::Playing) {
        warn!("stream pipeline failed to reach playing state: {e}");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    state.stats.client_started();
    let appsrc_clone = appsrc.clone();
    let stats = state.stats.clone();

    struct PipelineGuard(gstreamer::Pipeline);
    impl Drop for PipelineGuard {
        fn drop(&mut self) {
            let _ = self.0.set_state(gstreamer::State::Null);
        }
    }
    let pipeline_for_task = pipeline.clone();

    let (keyframe_packet, mut video_rx) = {
        let jb = state.join_buffer.lock().unwrap_or_else(|e| e.into_inner());
        let kf = jb.front().cloned();
        let rx = state.video_tx.subscribe();
        (kf, rx)
    };
    let has_keyframe = keyframe_packet
        .as_ref()
        .is_some_and(|sp| sp.pkt.is_keyframe);
    let keyframe_seq = keyframe_packet.as_ref().map(|sp| sp.seq);
    let mut pts_base: Option<u64> = if has_keyframe {
        keyframe_packet.as_ref().map(|sp| sp.pkt.pts_ns)
    } else {
        None
    };

    if let (Some(base), Some(sp)) = (pts_base, &keyframe_packet) {
        let mut normalized = sp.pkt.clone();
        normalized.pts_ns = sp.pkt.pts_ns.saturating_sub(base);
        let _ = push_h264_packet(&appsrc_clone, &normalized);
    }

    tokio::spawn(async move {
        let _pipeline_guard = PipelineGuard(pipeline_for_task);
        let _guard = ClientGuard(stats);

        let mut wait_keyframe = pts_base.is_none();
        loop {
            if tx_alive.is_closed() {
                debug!("stream client disconnected");
                break;
            }
            let sp = match video_rx.recv().await {
                Ok(sp) => sp,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    debug!("stream client lagged {n} packets; waiting for keyframe");
                    wait_keyframe = true;
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            };
            if has_keyframe {
                if let Some(last) = keyframe_seq {
                    if sp.seq <= last {
                        continue;
                    }
                }
            }
            if wait_keyframe {
                if !sp.pkt.is_keyframe {
                    continue;
                }
                wait_keyframe = false;
                if pts_base.is_none() {
                    pts_base = Some(sp.pkt.pts_ns);
                }
            }

            let base = pts_base.unwrap_or(0);
            let mut normalized = sp.pkt.clone();
            normalized.pts_ns = sp.pkt.pts_ns.saturating_sub(base);

            if push_h264_packet(&appsrc_clone, &normalized).is_err() {
                break;
            }
        }
        let _ = pipeline.set_state(gstreamer::State::Null);
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
    }

    #[test]
    fn stats_track_auth_failures() {
        let stats = Arc::new(Stats::default());
        assert_eq!(stats.auth_failures(), 0);
        stats.note_auth_failure();
        stats.note_auth_failure();
        assert_eq!(stats.auth_failures(), 2);
    }
}
