// Orbiscreen - orbiscreen-transport library (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

pub mod adb;
pub mod mdns;

use std::path::PathBuf;

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Json};
use axum::routing::{get, post};
use axum::Router;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tower_http::services::ServeDir;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct ServiceDescriptor {
    pub instance: String,
    pub port: u16,
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("http server error: {0}")]
    Http(String),
}

use orbiscreen_input::{KeyEvent, PointerEvent, StylusEvent};

/// Client input payloads sent over POST `/input`.
///
/// The tagged forms (`{"Pointer": {...}}`, `{"Key": {...}}`, `{"Stylus": {...}}`)
/// are used by the Android client (field names like `wheel`, `deltaY`, `tilt_x`);
/// the untagged `{"x", "y"}` fallback is used by the web client for pointer
/// moves. `Pointer` accepts `x`/`y` for `Move` because the Kotlin `Move`
/// serializer flattens the fields.
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

#[allow(missing_debug_implementations)]
pub struct Transport {
    cfg: ServerConfig,
    input_tx: Option<mpsc::UnboundedSender<IncomingInput>>,
}

impl Transport {
    pub fn new(cfg: ServerConfig) -> Self {
        Self {
            cfg,
            input_tx: None,
        }
    }

    pub fn with_input_sender(mut self, tx: mpsc::UnboundedSender<IncomingInput>) -> Self {
        self.input_tx = Some(tx);
        self
    }

    pub async fn serve(
        self,
        frames: mpsc::UnboundedReceiver<H264Packet>,
        display_width: u32,
        display_height: u32,
        refresh_hz: u32,
        encoder_kind: &'static str,
    ) -> Result<(), TransportError> {
        let (fallback_tx, _fallback_rx) = mpsc::unbounded_channel();
        let input_tx = self.input_tx.unwrap_or(fallback_tx);
        // Large enough that a slow consumer only loses a second of video,
        // not the whole backlog leading to Lagged.
        let (video_tx, _video_rx) = tokio::sync::broadcast::channel::<H264Packet>(360);
        let app = build_router(
            self.cfg.clone(),
            input_tx,
            video_tx.clone(),
            display_width,
            display_height,
            refresh_hz,
            encoder_kind,
        );
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

        tokio::spawn(async move {
            let mut frames = frames;
            while let Some(pkt) = frames.recv().await {
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
    display_width: u32,
    display_height: u32,
    refresh_hz: u32,
    encoder_kind: &'static str,
    version: &'static str,
}

fn build_router(
    cfg: ServerConfig,
    input_tx: mpsc::UnboundedSender<IncomingInput>,
    video_tx: tokio::sync::broadcast::Sender<H264Packet>,
    display_width: u32,
    display_height: u32,
    refresh_hz: u32,
    encoder_kind: &'static str,
) -> Router {
    let state = AppState {
        config: cfg,
        input_tx,
        video_tx,
        display_width,
        display_height,
        refresh_hz,
        encoder_kind,
        version: env!("CARGO_PKG_VERSION"),
    };
    Router::new()
        .route("/", get(root_handler))
        .route("/health", get(|| async { "ok" }))
        .route("/ws", get(ws_handler))
        .route("/sdp", post(sdp_post))
        .route("/input", post(input_post))
        .route("/stream", get(stream_handler))
        .route("/api/info", get(api_info))
        .route("/api/control", post(api_control))
        .nest_service("/client", ServeDir::new(&state.config.client_web_dir))
        .with_state(state)
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

async fn api_control(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    match payload.get("action").and_then(|v| v.as_str()) {
        Some("lock") => {
            info!("host control: lock");
            StatusCode::OK
        }
        Some("blank") | Some("unblank") => {
            info!("host control: blank toggle");
            StatusCode::OK
        }
        Some("ctrl_alt_del") => {
            info!("host control: ctrl_alt_del");
            StatusCode::OK
        }
        Some("open") => {
            info!("host control: open {:?}", payload.get("target"));
            StatusCode::OK
        }
        _ => StatusCode::BAD_REQUEST,
    }
}

async fn root_handler() -> Html<&'static str> {
    Html(
        r#"<!doctype html><html><head><meta charset=utf-8><title>Orbiscreen</title>
<meta http-equiv="refresh" content="0; url=/client/index.html"></head>
<body><a href="/client/index.html">Open the client</a></body></html>"#,
    )
}

async fn ws_handler(ws: WebSocketUpgrade, State(_state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: axum::extract::ws::WebSocket) {
    info!("signaling websocket connected");
    while let Some(Ok(msg)) = socket.recv().await {
        debug!("ws message: {msg:?}");
        let reply = serde_json::json!({
            "type": "ready",
            "webrtc": { "available": true },
        });
        if socket
            .send(axum::extract::ws::Message::Text(reply.to_string().into()))
            .await
            .is_err()
        {
            break;
        }
    }
}

async fn sdp_post(State(_state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "WebRTC not yet implemented, use /stream"
        })),
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

/// Builds a per-client GStreamer MPEG-TS pipeline (appsrc → mpegtsmux → appsink)
/// and streams it as the HTTP response body. All failures are handled in-band:
/// a broken pipeline yields 503 instead of panicking the axum task.
async fn stream_handler(State(state): State<AppState>) -> axum::response::Response {
    use gstreamer::prelude::*;
    use gstreamer_app::{AppSink, AppSinkCallbacks, AppSrc};
    use tokio_stream::StreamExt;

    gstreamer::init().ok();

    // Stream pipeline: h264parse converts byte-stream NAL units into a
    // properly framed AU stream, then mpegtsmux writes PES+TS. Setting
    // config-interval=1 requests SPS/PPS per keyframe, which lets clients
    // joining mid-stream decode immediately.
    // The encoder already strips avc headers via h264parse inside the encode
    // crate. On the read side we just need to feed TS packets. Skip h264parse
    // here so we don't double-process or crash on convoluted state.
    // Reset the transport to the most standard mpegts-writer pipeline:
    // appsrc accepts already-encoded byte-stream AU H.264 from x264enc. We
    // explicitly anchor caps, hand PTS ourselves (encoder emits valid PTS),
    // and ask h264parse to add SPS/PPS to every IDR via config-interval=1.
    // Then mpegtsmux writes proper TS packets with PAT/PMT rebuilt per
    // keyframe, so any client that connects mid-stream can sync.
    let pipeline_str = "appsrc name=src format=time is-live=false \
                        ! video/x-h264,stream-format=byte-stream,alignment=au,framerate=0/1 \
                        ! h264parse config-interval=1 \
                        ! mpegtsmux alignment=7 \
                        ! appsink name=sink drop=false sync=false max-buffers=0";
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

    // Telling appsrc what we're pushing is REQUIRED - without video/x-h264
    // caps, downstream parsers see an unrecognized stream and produce no
    // output samples for the HTTP response body.
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

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    let pushed = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let pushed_cb = pushed.clone();

    appsink.set_callbacks(
        AppSinkCallbacks::builder()
            .new_sample(move |sink| match sink.pull_sample() {
                Ok(sample) => {
                    if let Some(buffer) = sample.buffer() {
                        if let Ok(map) = buffer.map_readable() {
                            let n =
                                pushed_cb.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                            if n <= 5 || n % 30 == 0 {
                                debug!("appsink sample #{n}: {} B", map.size());
                            }
                            let _ = tx.send(map.to_vec());
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

    if let Err(e) = pipeline.set_state(gstreamer::State::Playing) {
        warn!("stream pipeline failed to reach playing state: {e}");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    let mut video_rx = state.video_tx.subscribe();

    let appsrc_clone = appsrc.clone();
    tokio::spawn(async move {
        while let Ok(pkt) = video_rx.recv().await {
            // Sanity check: H.264 NAL units start with 00 00 00 01 (or 00 00 01).
            // Filter out anything that doesn't so mpegtsmux never sees garbage
            // and crashes in gst_base_parse_handle_buffer.
            let valid = pkt.bytes.len() >= 4
                && (pkt.bytes[0] == 0
                    && pkt.bytes[1] == 0
                    && (pkt.bytes[2] == 1 || pkt.bytes[2..4] == [0, 1]));
            if !valid {
                debug!(
                    "skipping non-NAL packet: {} B (header={:02x?})",
                    pkt.bytes.len(),
                    &pkt.bytes[..4]
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

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
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
}
