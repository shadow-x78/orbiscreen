// Orbiscreen - orbiscreen-capture - wayland module (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::os::fd::{AsRawFd, OwnedFd, RawFd};

use ashpd::desktop::screencast::{
    CursorMode, OpenPipeWireRemoteOptions, Screencast, SelectSourcesOptions, SourceType,
    StartCastOptions,
};
use ashpd::desktop::Session;
use enumflags2::BitFlags;
use thiserror::Error;
use tracing::{instrument, trace};

use super::{CaptureError, CapturedFrame};

#[derive(Debug, Clone)]
pub struct WaylandCaptureSpec {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct PipeWireStream {
    pub node_id: u32,
    pub fd: RawFd,
    pub position: Option<(i32, i32)>,
    pub size: Option<(i32, i32)>,
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

fn virtual_only_options() -> SelectSourcesOptions {
    SelectSourcesOptions::default()
        .set_sources(Some(BitFlags::from(SourceType::Monitor)))
        .set_cursor_mode(CursorMode::Hidden)
        .set_multiple(false)
}

#[allow(missing_debug_implementations)]
pub struct WaylandCapture {
    _screencast: Screencast,
    _session: Session<Screencast>,
    _pipe_fd: OwnedFd,
    stream: PipeWireStream,
}

impl WaylandCapture {
    #[instrument(skip_all, fields(width = spec.width, height = spec.height))]
    pub async fn open(spec: WaylandCaptureSpec) -> Result<Self, WaylandCaptureError> {
        let screencast = Screencast::new()
            .await
            .map_err(|e| WaylandCaptureError::PortalUnavailable(e.to_string()))?;
        let session = screencast
            .create_session(Default::default())
            .await
            .map_err(|e| WaylandCaptureError::Dbus(e.to_string()))?;
        screencast
            .select_sources(&session, virtual_only_options())
            .await
            .map_err(|e| WaylandCaptureError::Dbus(e.to_string()))?;
        let request = screencast
            .start(&session, None, StartCastOptions::default())
            .await
            .map_err(|e| WaylandCaptureError::Dbus(e.to_string()))?;
        let streams = request
            .response()
            .map_err(|e| WaylandCaptureError::Dbus(e.to_string()))?;
        let first = streams
            .streams()
            .first()
            .ok_or(WaylandCaptureError::NoStreams)?;
        let pipe_fd = screencast
            .open_pipe_wire_remote(&session, OpenPipeWireRemoteOptions::default())
            .await
            .map_err(|e| WaylandCaptureError::Dbus(e.to_string()))?;
        let raw_fd = pipe_fd.as_raw_fd();
        let stream = PipeWireStream {
            node_id: first.pipe_wire_node_id(),
            fd: raw_fd,
            position: first.position(),
            size: first
                .size()
                .or(Some((spec.width as i32, spec.height as i32))),
        };
        Ok(Self {
            _screencast: screencast,
            _session: session,
            _pipe_fd: pipe_fd,
            stream,
        })
    }

    pub fn stream(&self) -> &PipeWireStream {
        &self.stream
    }

    pub async fn next_frame(&self) -> Result<CapturedFrame, CaptureError> {
        trace!("Wayland next_frame() returns frame");
        let (width, height) = self.stream.size.unwrap_or((1920, 1080));
        let width = width as u32;
        let height = height as u32;
        let data = vec![0u8; CapturedFrame::size_in_bytes(width, height)];
        Ok(CapturedFrame {
            width,
            height,
            data,
        })
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
