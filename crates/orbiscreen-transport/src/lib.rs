// Orbiscreen — orbiscreen-transport library (GPL-3.0-or-later)
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

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub signaling_port: u16,
    pub client_web_dir: PathBuf,
}

#[allow(missing_debug_implementations)]
pub struct Transport {
    cfg: ServerConfig,
}

impl Transport {
    pub fn new(cfg: ServerConfig) -> Self {
        Self { cfg }
    }

    pub async fn serve(
        self,
        frames: mpsc::UnboundedReceiver<H264Packet>,
    ) -> Result<(), TransportError> {
        let app = build_router(self.cfg.clone());
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
}

fn build_router(cfg: ServerConfig) -> Router {
    let state = AppState { config: cfg };
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

async fn ws_handler(ws: WebSocketUpgrade, State(_state): State<AppState>) -> impl IntoResponse {
    info!("WebSocket signaling upgrade requested");
    ws.on_upgrade(handle_signaling_ws)
}

async fn handle_signaling_ws(mut socket: axum::extract::ws::WebSocket) {
    use axum::extract::ws::Message;
    while let Some(Ok(message)) = socket.recv().await {
        let text = match message {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            _ => continue,
        };
        info!("WS message: {text}");
        let reply = serde_json::json!({
            "type": "ready",
            "webrtc": { "available": false },
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

async fn sdp_post() -> impl IntoResponse {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "webrtc signaling not yet implemented",
            "see": "CHANGELOG.md",
        })),
    )
}

async fn input_post(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    debug!("received /input payload: {payload}");
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
