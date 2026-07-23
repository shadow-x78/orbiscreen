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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum IncomingInput {
    Pointer(PointerEvent),
    Key(KeyEvent),
    Stylus(StylusEvent),
    RawPointer { x: f64, y: f64 },
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
    ) -> Result<(), TransportError> {
        let (fallback_tx, _fallback_rx) = mpsc::unbounded_channel();
        let input_tx = self.input_tx.unwrap_or(fallback_tx);
        let app = build_router(self.cfg.clone(), input_tx);
        let listener = TcpListener::bind(("0.0.0.0", self.cfg.signaling_port))
            .await
            .map_err(|e| TransportError::Http(e.to_string()))?;
        let local = listener
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "?".into());
        info!("orbiscreen transport listening on http://{local}");

        tokio::spawn(async move {
            let mut frames = frames;
            while frames.recv().await.is_some() {}
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
}

fn build_router(cfg: ServerConfig, input_tx: mpsc::UnboundedSender<IncomingInput>) -> Router {
    let state = AppState {
        config: cfg,
        input_tx,
    };
    Router::new()
        .route("/", get(root_handler))
        .route("/health", get(|| async { "ok" }))
        .route("/ws", get(ws_handler))
        .route("/sdp", post(sdp_post))
        .route("/input", post(input_post))
        .nest_service("/client", ServeDir::new(&state.config.client_web_dir))
        .with_state(state)
}

async fn root_handler() -> Html<&'static str> {
    Html(
        r#"<!doctype html><html><head><meta charset=utf-8><title>Orbiscreen</title>
<meta http-equiv="refresh" content="0; url=/client/index.html"></head>
<body><a href="/client/index.html">Open the client</a></body></html>"#,
    )
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    info!("WebSocket signaling upgrade requested");
    ws.on_upgrade(move |socket| handle_signaling_ws(socket, state))
}

async fn handle_signaling_ws(mut socket: axum::extract::ws::WebSocket, state: AppState) {
    use axum::extract::ws::Message;
    while let Some(Ok(message)) = socket.recv().await {
        let text = match message {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            _ => continue,
        };
        debug!("WS message: {text}");
        if let Ok(input) = serde_json::from_str::<IncomingInput>(&text) {
            let _ = state.input_tx.send(input);
        } else if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
            if let (Some(x), Some(y)) = (
                val.get("x").and_then(|v| v.as_f64()),
                val.get("y").and_then(|v| v.as_f64()),
            ) {
                let _ = state
                    .input_tx
                    .send(IncomingInput::Pointer(PointerEvent::Move { x, y }));
            }
        }
        let reply = serde_json::json!({
            "type": "ready",
            "webrtc": { "available": true },
        });
        if socket
            .send(Message::Text(reply.to_string().into()))
            .await
            .is_err()
        {
            break;
        }
    }
    warn!("signaling websocket closed");
}

#[derive(serde::Deserialize)]
struct SdpPayload {
    sdp: String,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    sdp_type: Option<String>,
}

async fn sdp_post(
    State(_state): State<AppState>,
    Json(payload): Json<SdpPayload>,
) -> impl IntoResponse {
    info!("Received SDP offer: length {}", payload.sdp.len());
    let _answer_sdp = "v=0\r\no=- 0 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=video 9 UDP/TLS/RTP/SAVPF 96\r\nc=IN IP4 0.0.0.0\r\na=setup:passive\r\na=mid:0\r\na=sendonly\r\na=rtpmap:96 H264/90000\r\n".to_string();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "type": "answer",
            "sdp": payload.sdp,
        })),
    )
}

async fn input_post(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    debug!("received /input payload: {payload}");
    if let Ok(event) = serde_json::from_value::<IncomingInput>(payload.clone()) {
        let _ = state.input_tx.send(event);
    } else if let (Some(x), Some(y)) = (
        payload.get("x").and_then(|v| v.as_f64()),
        payload.get("y").and_then(|v| v.as_f64()),
    ) {
        let _ = state
            .input_tx
            .send(IncomingInput::Pointer(PointerEvent::Move { x, y }));
    }
    StatusCode::ACCEPTED
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
