// Orbiscreen — orbiscreen-capture library (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

pub mod wayland;
pub mod x11;

use std::sync::Arc;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureBackend {
    X11,
    Wayland,
}

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("capture backend unavailable: {0}")]
    BackendUnavailable(&'static str),
    #[error("x11 connection error: {0}")]
    X11Connect(String),
    #[error("x11 protocol error code {0}")]
    X11Protocol(u8),
    #[error("capture I/O error: {0}")]
    Io(String),
}

impl From<x11rb::xcb_ffi::ConnectError> for CaptureError {
    fn from(error: x11rb::xcb_ffi::ConnectError) -> Self {
        Self::X11Connect(format!("{error:?}"))
    }
}

impl From<x11rb::xcb_ffi::ConnectionError> for CaptureError {
    fn from(error: x11rb::xcb_ffi::ConnectionError) -> Self {
        Self::X11Connect(format!("{error:?}"))
    }
}

#[derive(Debug, Clone)]
pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl CapturedFrame {
    pub fn size_in_bytes(width: u32, height: u32) -> usize {
        (width as usize) * (height as usize) * 4
    }
}

#[allow(missing_debug_implementations)]
pub struct CaptureSession {
    backend_kind: CaptureBackend,
    inner: Arc<CaptureInner>,
}

#[allow(missing_debug_implementations)]
enum CaptureInner {
    X11(x11::X11Capture),
    Wayland(wayland::WaylandCapture),
}

impl CaptureSession {
    pub fn open(width: u32, height: u32) -> Result<Self, CaptureError> {
        match detect_backend() {
            CaptureBackend::X11 => Ok(Self {
                backend_kind: CaptureBackend::X11,
                inner: Arc::new(CaptureInner::X11(x11::X11Capture::open(width, height)?)),
            }),
            CaptureBackend::Wayland => Err(CaptureError::BackendUnavailable(
                "Wayland capture requires open_async",
            )),
        }
    }

    pub async fn open_async(width: u32, height: u32) -> Result<Self, CaptureError> {
        match detect_backend() {
            CaptureBackend::X11 => Self::open(width, height),
            CaptureBackend::Wayland => {
                let capture =
                    wayland::WaylandCapture::open(wayland::WaylandCaptureSpec { width, height })
                        .await?;
                Ok(Self {
                    backend_kind: CaptureBackend::Wayland,
                    inner: Arc::new(CaptureInner::Wayland(capture)),
                })
            }
        }
    }

    pub fn backend(&self) -> CaptureBackend {
        self.backend_kind
    }

    pub async fn next_frame(&self) -> Result<CapturedFrame, CaptureError> {
        match self.inner.as_ref() {
            CaptureInner::X11(capture) => capture.next_frame().await,
            CaptureInner::Wayland(capture) => capture.next_frame().await,
        }
    }
}

pub fn detect_backend() -> CaptureBackend {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        CaptureBackend::Wayland
    } else {
        CaptureBackend::X11
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_in_bytes_is_w_h_4() {
        assert_eq!(CapturedFrame::size_in_bytes(1920, 1080), 1920 * 1080 * 4);
        assert_eq!(CapturedFrame::size_in_bytes(0, 0), 0);
    }

    #[test]
    fn detect_prefers_wayland_when_present() {
        let prev = std::env::var_os("WAYLAND_DISPLAY");
        let prev_x = std::env::var_os("DISPLAY");
        std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
        std::env::remove_var("DISPLAY");
        assert_eq!(detect_backend(), CaptureBackend::Wayland);
        match prev {
            Some(value) => std::env::set_var("WAYLAND_DISPLAY", value),
            None => std::env::remove_var("WAYLAND_DISPLAY"),
        }
        match prev_x {
            Some(value) => std::env::set_var("DISPLAY", value),
            None => std::env::remove_var("DISPLAY"),
        }
    }

    #[test]
    fn empty_frame_is_zeroes() {
        let frame = CapturedFrame {
            width: 4,
            height: 2,
            data: vec![0; 32],
        };
        assert_eq!(frame.data.len(), 32);
        assert!(frame.data.iter().all(|b| *b == 0));
    }
}
