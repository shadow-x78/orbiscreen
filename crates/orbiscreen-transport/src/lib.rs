// Orbiscreen - orbiscreen-transport library (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
pub mod adb;
pub mod mdns;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

    fn note_frame(&self) {
        self.frames_forwarded.fetch_add(1, Ordering::Relaxed);
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
    if ab.len() != bb.len() {
        return false;
    }
    ab.iter().zip(bb).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[allow(missing_debug_implementations)]
pub struct Transport {
    cfg: ServerConfig,
    input_tx: mpsc::UnboundedSender<IncomingInput>,
    token: String,
}

impl Transport {
    pub fn new(cfg: ServerConfig, input_tx: mpsc::UnboundedSender<IncomingInput>) -> Self {
        Self {
            cfg,
            input_tx,
            token: generate_token(),
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub async fn serve(
        self,
        frames: mpsc::Receiver<H264Packet>,
        stats: Arc<Stats>,
        display_width: u32,
        display_height: u32,
        refresh_hz: u32,
        encoder_kind: &'static str,
    ) -> Result<(), TransportError> {
        let input_tx = self.input_tx;
        let (video_tx, _video_rx) = tokio::sync::broadcast::channel::<H264Packet>(360);
        let state = AppState {
            config: self.cfg.clone(),
            input_tx,
            video_tx: video_tx.clone(),
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

        match adb::setup_reverse_for_all(adb::default_adb_path(), self.cfg.signaling_port) {
            Ok(devices) => info!("ADB reverse port forwarding configured for devices: {devices:?}"),
            Err(adb::AdbError::NoDevice | adb::AdbError::NotInstalled) => {}
            Err(e) => debug!("ADB reverse port forwarding inactive: {e}"),
        }

        let stats_pump = state.stats.clone();
        tokio::spawn(async move {
            let mut frames = frames;
            while let Some(pkt) = frames.recv().await {
                stats_pump.note_frame();
                let _ = video_tx.send(pkt);
            }
        });

        axum::serve(listener, app)
            .await
            .map_err(|e| TransportError::Http(e.to_string()))?;
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
    input_tx: mpsc::UnboundedSender<IncomingInput>,
    video_tx: tokio::sync::broadcast::Sender<H264Packet>,
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
        .route("/stream", get(stream_handler))
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
        "total_clients": state.stats.total_clients(),
        "uptime_seconds": state.started.elapsed().as_secs(),
    }))
}

async fn auth_check(
    State(state): State<AppState>,
    headers: HeaderMap,
    query: Query<std::collections::HashMap<String, String>>,
    request: axum::extract::Request,
    next: middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    let header_ok = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|t| token_eq(t, &state.token));
    let query_ok = query
        .0
        .get("token")
        .is_some_and(|t| token_eq(t, &state.token));
    if header_ok || query_ok {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
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

async fn client_config(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "token": state.token,
        "display_width": state.display_width,
        "display_height": state.display_height,
    }))
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

fn inject_ctrl_alt_del(tx: &mpsc::UnboundedSender<IncomingInput>) {
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
        let _ = tx.send(IncomingInput::Key(KeyEvent { code, pressed }));
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
    debug!("received /input payload: {payload}");
    let event = serde_json::from_value::<IncomingInput>(payload.clone()).ok();
    match event {
        Some(ev) => {
            let _ = state.input_tx.send(ev);
        }
        None => {
            if let (Some(x), Some(y)) = (
                payload.get("x").and_then(|v| v.as_f64()),
                payload.get("y").and_then(|v| v.as_f64()),
            ) {
                let _ = state
                    .input_tx
                    .send(IncomingInput::Pointer(PointerEvent::Move { x, y }));
            } else {
                return StatusCode::BAD_REQUEST;
            }
        }
    }
    StatusCode::ACCEPTED
}

async fn stream_handler(State(state): State<AppState>) -> axum::response::Response {
    use gstreamer::prelude::*;
    use gstreamer_app::{AppSink, AppSinkCallbacks, AppSrc};
    use tokio_stream::StreamExt;

    gstreamer::init().ok();

    let pipeline_str = "appsrc name=src format=time is-live=false \
                        ! video/x-h264,stream-format=byte-stream,alignment=au,framerate=0/1 \
                        ! h264parse config-interval=1 \
                        ! mpegtsmux alignment=7 \
                        ! appsink name=sink drop=true sync=false max-buffers=2";
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

    let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(256);
    appsink.set_callbacks(
        AppSinkCallbacks::builder()
            .new_sample(move |sink| match sink.pull_sample() {
                Ok(sample) => {
                    if let Some(buffer) = sample.buffer() {
                        if let Ok(map) = buffer.map_readable() {
                            if tx.try_send(map.to_vec()).is_err() {
                                debug!("per-client TS channel full; dropping chunk");
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

    if let Err(e) = pipeline.set_state(gstreamer::State::Playing) {
        warn!("stream pipeline failed to reach playing state: {e}");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    state.stats.client_started();
    let mut video_rx = state.video_tx.subscribe();
    let stats = state.stats.clone();

    struct PipelineGuard(gstreamer::Pipeline);
    impl Drop for PipelineGuard {
        fn drop(&mut self) {
            let _ = self.0.set_state(gstreamer::State::Null);
        }
    }
    let pipeline_for_task = pipeline.clone();
    let appsrc_clone = appsrc.clone();
    tokio::spawn(async move {
        let _pipeline_guard = PipelineGuard(pipeline_for_task);
        let _guard = ClientGuard(stats);
        loop {
            let pkt = match video_rx.recv().await {
                Ok(pkt) => pkt,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    debug!("stream client lagged {n} packets; fast-forwarding");
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            };

            let valid = pkt.bytes.len() >= 4
                && (pkt.bytes[0] == 0
                    && pkt.bytes[1] == 0
                    && (pkt.bytes[2] == 1 || pkt.bytes[2..4] == [0, 1]));
            if !valid {
                let header_len = pkt.bytes.len().min(4);
                debug!(
                    "skipping non-NAL packet: {} B (header={:02x?})",
                    pkt.bytes.len(),
                    &pkt.bytes[..header_len]
                );
                continue;
            }

            let Ok(mut buffer) = gstreamer::Buffer::with_size(pkt.bytes.len()) else {
                warn!("failed to allocate gstreamer buffer for packet");
                break;
            };
            {
                let Some(buffer_mut) = buffer.get_mut() else {
                    warn!("gstreamer buffer not writable");
                    break;
                };
                if buffer_mut.copy_from_slice(0, &pkt.bytes).is_err() {
                    warn!("packet larger than allocated gstreamer buffer");
                    break;
                }
                if pkt.is_keyframe {
                    buffer_mut.set_flags(gstreamer::BufferFlags::MARKER);
                }
                buffer_mut.set_pts(gstreamer::ClockTime::from_nseconds(pkt.pts_ns));
            }
            if let Err(e) = appsrc_clone.push_buffer(buffer) {
                debug!("push_buffer (client gone or EOS): {e}");
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
}
