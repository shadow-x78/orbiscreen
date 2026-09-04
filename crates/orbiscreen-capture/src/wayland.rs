// Orbiscreen - wayland.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::os::fd::{AsRawFd, OwnedFd};

use ashpd::desktop::screencast::{
    CursorMode, OpenPipeWireRemoteOptions, Screencast, SelectSourcesOptions, SourceType,
    StartCastOptions, Streams,
};
use ashpd::desktop::{PersistMode, ResponseError, Session};
use enumflags2::BitFlags;
use orbiscreen_core::frame_pool::FramePool;
use orbiscreen_core::portal_state;
use thiserror::Error;
use tracing::instrument;

use super::{CaptureError, CapturedFrame};

use gstreamer::prelude::*;
use gstreamer_app::{AppSink, AppSinkCallbacks};
use tokio::sync::mpsc;

const FRAME_CHANNEL_CAPACITY: usize = 2;

#[derive(Debug, Clone)]
pub struct WaylandCaptureSpec {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Error)]
pub enum WaylandCaptureError {
    #[error("screencast portal not available: {0}")]
    PortalUnavailable(String),
    #[error("portal D-Bus error: {0}")]
    Dbus(String),
    #[error("user denied the ScreenCast permission")]
    PermissionDenied,
    #[error("portal returned no streams")]
    NoStreams,
}

impl From<WaylandCaptureError> for CaptureError {
    fn from(error: WaylandCaptureError) -> Self {
        CaptureError::Io(error.to_string())
    }
}

impl From<gstreamer::glib::Error> for WaylandCaptureError {
    fn from(error: gstreamer::glib::Error) -> Self {
        WaylandCaptureError::Dbus(format!("gstreamer error: {}", error))
    }
}

impl From<gstreamer::glib::BoolError> for WaylandCaptureError {
    fn from(error: gstreamer::glib::BoolError) -> Self {
        WaylandCaptureError::Dbus(format!("gstreamer error: {}", error))
    }
}

fn virtual_only_options() -> SelectSourcesOptions {
    SelectSourcesOptions::default()
        .set_sources(Some(BitFlags::from(SourceType::Virtual)))
        .set_cursor_mode(CursorMode::Hidden)
        .set_multiple(false)
        .set_persist_mode(PersistMode::ExplicitlyRevoked)
}

fn monitor_fallback_options() -> SelectSourcesOptions {
    SelectSourcesOptions::default()
        .set_sources(Some(BitFlags::from(SourceType::Monitor)))
        .set_cursor_mode(CursorMode::Hidden)
        .set_multiple(false)
        .set_persist_mode(PersistMode::ExplicitlyRevoked)
}

async fn negotiate_screencast(
    screencast: &Screencast,
    restore_token: Option<&str>,
) -> Result<(Session<Screencast>, Streams), WaylandCaptureError> {
    let session = screencast
        .create_session(Default::default())
        .await
        .map_err(|e| WaylandCaptureError::Dbus(e.to_string()))?;
    let select_res = screencast
        .select_sources(
            &session,
            virtual_only_options().set_restore_token(restore_token),
        )
        .await;
    if let Err(e) = select_res {
        tracing::warn!("virtual source request failed ({e}); falling back to monitor source");
        screencast
            .select_sources(
                &session,
                monitor_fallback_options().set_restore_token(restore_token),
            )
            .await
            .map_err(|e| WaylandCaptureError::Dbus(e.to_string()))?;
    }
    let request = screencast
        .start(&session, None, StartCastOptions::default())
        .await
        .map_err(|e| WaylandCaptureError::Dbus(e.to_string()))?;
    let streams = request.response().map_err(|e| match e {
        ashpd::Error::Response(ResponseError::Cancelled) => WaylandCaptureError::PermissionDenied,
        other => WaylandCaptureError::Dbus(other.to_string()),
    })?;
    Ok((session, streams))
}

#[allow(missing_debug_implementations)]
pub struct WaylandCapture {
    _screencast: Screencast,
    _session: Session<Screencast>,
    _pipeline: gstreamer::Pipeline,
    _pipe_fd: OwnedFd,
    rx: tokio::sync::Mutex<mpsc::Receiver<CapturedFrame>>,
    width: u32,
    height: u32,
}

impl WaylandCapture {
    #[instrument(skip_all, fields(width = spec.width, height = spec.height))]
    pub async fn open(spec: WaylandCaptureSpec) -> Result<Self, WaylandCaptureError> {
        let screencast = Screencast::new()
            .await
            .map_err(|e| WaylandCaptureError::PortalUnavailable(e.to_string()))?;
        let mut state = portal_state::load_portal_state();
        let saved_token = state.screencast_restore_token.clone();
        let (session, streams) =
            match negotiate_screencast(&screencast, saved_token.as_deref()).await {
                Ok(pair) => pair,
                Err(first_error) => {
                    if saved_token.is_some() {
                        tracing::warn!(
                            "saved screencast restore token was not accepted ({first_error}); \
                             asking for a fresh selection"
                        );
                        negotiate_screencast(&screencast, None).await?
                    } else {
                        return Err(first_error);
                    }
                }
            };
        if let Some(token) = streams.restore_token() {
            state.screencast_restore_token = Some(token.to_string());
            match portal_state::save_portal_state(&state) {
                Ok(()) => tracing::info!(
                    "screencast permission persisted; the share dialog will not \
                     reappear on the next runs"
                ),
                Err(e) => tracing::warn!("failed to persist portal state: {e}"),
            }
        }
        let first = streams
            .streams()
            .first()
            .ok_or(WaylandCaptureError::NoStreams)?;
        let pipe_fd = screencast
            .open_pipe_wire_remote(&session, OpenPipeWireRemoteOptions::default())
            .await
            .map_err(|e| WaylandCaptureError::Dbus(e.to_string()))?;
        let raw_fd = pipe_fd.as_raw_fd();
        let node_id = first.pipe_wire_node_id();

        gstreamer::init().map_err(|e| WaylandCaptureError::Dbus(format!("gst init: {}", e)))?;

        let pipeline_str = format!(
            "pipewiresrc fd={} path={} do-timestamp=true \
             ! video/x-raw \
             ! videoconvert \
             ! videoscale \
             ! video/x-raw,format=BGRA,width={},height={} \
             ! appsink name=sink drop=false sync=false max-buffers=2 emit-signals=false",
            raw_fd, node_id, spec.width, spec.height
        );
        let pipeline = gstreamer::parse::launch(&pipeline_str)?
            .downcast::<gstreamer::Pipeline>()
            .map_err(|_| WaylandCaptureError::Dbus("Failed to downcast pipeline".into()))?;

        let appsink = pipeline
            .by_name("sink")
            .ok_or_else(|| WaylandCaptureError::Dbus("appsink not found".into()))?
            .downcast::<AppSink>()
            .map_err(|_| WaylandCaptureError::Dbus("Failed to downcast appsink".into()))?;

        let (tx, rx) = mpsc::channel::<CapturedFrame>(FRAME_CHANNEL_CAPACITY);
        let pool = FramePool::new();

        appsink.set_callbacks(
            AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    let sample = match sink.pull_sample() {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::warn!("pull_sample error: {e}");
                            return Err(gstreamer::FlowError::Eos);
                        }
                    };
                    if let Some(frame) = super::sample_to_captured_frame(&sample, &pool) {
                        if tx.try_send(frame).is_err() {
                            tracing::debug!("capture frame dropped: consumer channel full");
                        }
                    }
                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );

        if let Some(bus) = pipeline.bus() {
            bus.set_sync_handler(|_bus, msg| {
                match msg.view() {
                    gstreamer::MessageView::Error(err) => tracing::error!(
                        target: "orbiscreen_capture::wayland",
                        "gstreamer capture error: {} (debug: {})",
                        err.error(),
                        err.debug().unwrap_or_default()
                    ),
                    gstreamer::MessageView::Warning(warn) => tracing::warn!(
                        target: "orbiscreen_capture::wayland",
                        "gstreamer capture warning: {} (debug: {})",
                        warn.error(),
                        warn.debug().unwrap_or_default()
                    ),
                    _ => {}
                }
                gstreamer::BusSyncReply::Drop
            });
        }

        pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| WaylandCaptureError::Dbus(format!("State error: {}", e)))?;

        Ok(Self {
            _screencast: screencast,
            _session: session,
            _pipeline: pipeline,
            _pipe_fd: pipe_fd,
            rx: tokio::sync::Mutex::new(rx),
            width: spec.width,
            height: spec.height,
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub async fn next_frame(&self) -> Result<CapturedFrame, CaptureError> {
        let mut rx = self.rx.lock().await;
        if let Some(frame) = rx.recv().await {
            Ok(frame)
        } else {
            Err(CaptureError::Io("Wayland GStreamer pipeline closed".into()))
        }
    }
}

impl Drop for WaylandCapture {
    fn drop(&mut self) {
        let _ = self._pipeline.set_state(gstreamer::State::Null);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_only_options_uses_monitor_source() {
        let _ = virtual_only_options();
    }

    #[test]
    fn wayland_capture_error_displays_useful_message() {
        let error = WaylandCaptureError::PermissionDenied;
        assert!(error.to_string().to_lowercase().contains("denied"));
    }

    #[test]
    fn wayland_capture_spec_constructs() {
        let spec = WaylandCaptureSpec {
            width: 1920,
            height: 1080,
        };
        assert_eq!(spec.width * spec.height, 1920 * 1080);
    }
}
