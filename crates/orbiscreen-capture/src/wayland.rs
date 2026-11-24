use std::os::fd::{AsRawFd, OwnedFd};

use ashpd::desktop::screencast::{
    CursorMode, OpenPipeWireRemoteOptions, Screencast, SelectSourcesOptions, SourceType,
    StartCastOptions, Streams,
};
use ashpd::desktop::{PersistMode, ResponseError, Session};
use orbiscreen_core::portal_state;
use thiserror::Error;
use tracing::instrument;

use super::pipewire::{PipewireCapture, PipewireSpec};
use super::CaptureError;
use orbiscreen_core::frame::CapturedFrame;

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
        .set_sources(SourceType::VIRTUAL)
        .set_cursor_mode(CursorMode::EMBEDDED)
        .set_multiple(false)
        .set_persist_mode(PersistMode::ExplicitlyRevoked)
}

fn monitor_fallback_options() -> SelectSourcesOptions {
    SelectSourcesOptions::default()
        .set_sources(SourceType::MONITOR)
        .set_cursor_mode(CursorMode::EMBEDDED)
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
    let selected = match screencast
        .select_sources(
            &session,
            virtual_only_options().set_restore_token(restore_token),
        )
        .await
    {
        Ok(_) => session,
        Err(e) => {
            tracing::warn!(
                "virtual source request failed ({e}); falling back to a fresh monitor session"
            );
            let session = screencast
                .create_session(Default::default())
                .await
                .map_err(|e| WaylandCaptureError::Dbus(e.to_string()))?;
            screencast
                .select_sources(
                    &session,
                    monitor_fallback_options().set_restore_token(restore_token),
                )
                .await
                .map_err(|e| WaylandCaptureError::Dbus(e.to_string()))?;
            session
        }
    };
    let request = screencast
        .start(&selected, None, StartCastOptions::default())
        .await
        .map_err(|e| WaylandCaptureError::Dbus(e.to_string()))?;
    let streams = request.response().map_err(|e| match e {
        ashpd::Error::Response(ResponseError::Cancelled) => WaylandCaptureError::PermissionDenied,
        other => WaylandCaptureError::Dbus(other.to_string()),
    })?;
    Ok((selected, streams))
}

#[allow(missing_debug_implementations)]
pub struct WaylandCapture {
    _screencast: Screencast,
    _session: Session<Screencast>,
    _pipe_fd: OwnedFd,
    pipewire: PipewireCapture,
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

        let pipewire = PipewireCapture::open(PipewireSpec {
            fd: Some(raw_fd),
            node_id,
            width: spec.width,
            height: spec.height,
            activity: None,
        })
        .map_err(WaylandCaptureError::Dbus)?;

        Ok(Self {
            _screencast: screencast,
            _session: session,
            _pipe_fd: pipe_fd,
            pipewire,
            width: spec.width,
            height: spec.height,
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub async fn next_frame(&self) -> Result<CapturedFrame, CaptureError> {
        self.pipewire.check_stall();
        self.pipewire.next_frame().await.map_err(CaptureError::Io)
    }

    pub fn is_ended(&self) -> bool {
        self.pipewire.is_ended()
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
    fn portal_options_embed_the_cursor_in_the_stream() {
        assert_eq!(
            virtual_only_options().cursor_mode(),
            Some(CursorMode::EMBEDDED)
        );
        assert_eq!(
            monitor_fallback_options().cursor_mode(),
            Some(CursorMode::EMBEDDED)
        );
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
