pub mod capabilities;
pub mod damage_pump;
pub mod kwin_virtual;
pub mod wayland;
pub mod wlr_screencopy;
pub mod wlr_virtual_output;
pub mod x11;

use std::sync::Arc;

use orbiscreen_core::frame_pool::{FramePool, PooledFrameBuffer};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureBackend {
    X11,
    Wayland,
    KwinVirtual,
    WlrScreencopy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapturePreference {
    Auto,

    KwinVirtual,

    Screencopy,

    Portal,
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

pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub data: PooledFrameBuffer,
}

impl std::fmt::Debug for CapturedFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapturedFrame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("data_len", &self.data.len())
            .finish()
    }
}

impl CapturedFrame {
    pub fn size_in_bytes(width: u32, height: u32) -> usize {
        (width as usize)
            .checked_mul(height as usize)
            .and_then(|v| v.checked_mul(4))
            .unwrap_or(usize::MAX)
    }
}

#[allow(missing_debug_implementations)]
pub struct CaptureSession {
    backend_kind: CaptureBackend,
    inner: Arc<CaptureInner>,
    width: u32,
    height: u32,
}

pub(crate) fn sample_to_captured_frame(
    sample: &gstreamer::Sample,
    pool: &Arc<FramePool>,
) -> Option<CapturedFrame> {
    let Some(buffer) = sample.buffer() else {
        tracing::warn!("skipping sample with no buffer");
        return None;
    };
    let Some(caps) = sample.caps() else {
        tracing::warn!("skipping sample with no caps");
        return None;
    };
    let Some(structure) = caps.structure(0) else {
        tracing::warn!("skipping sample with empty caps");
        return None;
    };
    let (Ok(width), Ok(height)) = (
        structure.get::<i32>("width"),
        structure.get::<i32>("height"),
    ) else {
        tracing::warn!("skipping sample with missing width/height in caps");
        return None;
    };
    if width <= 0 || height <= 0 {
        tracing::warn!("skipping sample with non-positive dimensions {width}x{height}");
        return None;
    }
    let Ok(map) = buffer.map_readable() else {
        tracing::warn!("buffer not readable; skipping sample");
        return None;
    };
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|v| v.checked_mul(4));
    let Some(expected) = expected else {
        tracing::warn!("skipping sample with overflowing dimensions {width}x{height}");
        return None;
    };
    if map.size() != expected {
        tracing::warn!(
            "frame size mismatch: got {} B, expected {expected} B ({width}x{height}); dropping",
            map.size()
        );
        return None;
    }
    let mut data = pool.acquire(expected);
    data.copy_from_slice(map.as_slice());
    Some(CapturedFrame {
        width: width as u32,
        height: height as u32,
        data,
    })
}

#[allow(missing_debug_implementations)]
enum CaptureInner {
    X11(x11::X11Capture),
    Wayland(wayland::WaylandCapture),
    KwinVirtual(kwin_virtual::KwinVirtualCapture),
    WlrScreencopy(wlr_screencopy::WlrScreencopyCapture),
}

impl CaptureSession {
    fn open_x11(width: u32, height: u32) -> Result<Self, CaptureError> {
        let capture = x11::X11Capture::open(width, height)?;
        let (actual_w, actual_h) = capture.dimensions();
        Ok(Self {
            backend_kind: CaptureBackend::X11,
            inner: Arc::new(CaptureInner::X11(capture)),
            width: actual_w,
            height: actual_h,
        })
    }

    async fn open_portal(width: u32, height: u32) -> Result<Self, CaptureError> {
        tracing::info!(
            "using the screencast portal; pick the display to stream in the share dialog"
        );
        let capture =
            wayland::WaylandCapture::open(wayland::WaylandCaptureSpec { width, height }).await?;
        let (actual_w, actual_h) = capture.dimensions();
        Ok(Self {
            backend_kind: CaptureBackend::Wayland,
            inner: Arc::new(CaptureInner::Wayland(capture)),
            width: actual_w,
            height: actual_h,
        })
    }

    async fn open_kwin(width: u32, height: u32) -> Result<Self, CaptureError> {
        let capture = tokio::task::spawn_blocking(move || {
            kwin_virtual::KwinVirtualCapture::open(kwin_virtual::KwinVirtualSpec { width, height })
        })
        .await
        .map_err(|e| CaptureError::Io(format!("kwin-virtual open task: {e}")))??;
        let (actual_w, actual_h) = capture.dimensions();
        tracing::info!(
            "KWin virtual display created via zkde-screencast: no root, no share dialog"
        );
        Ok(Self {
            backend_kind: CaptureBackend::KwinVirtual,
            inner: Arc::new(CaptureInner::KwinVirtual(capture)),
            width: actual_w,
            height: actual_h,
        })
    }

    pub async fn open_screencopy(output_name: Option<String>) -> Result<Self, CaptureError> {
        let capture = tokio::task::spawn_blocking(move || {
            wlr_screencopy::WlrScreencopyCapture::open(wlr_screencopy::WlrScreencopySpec {
                output_name,
            })
        })
        .await
        .map_err(|e| CaptureError::Io(format!("wlr-screencopy open task: {e}")))??;
        let (actual_w, actual_h) = capture.dimensions();
        tracing::info!("wlroots screencopy capture open: no portal, no share dialog");
        Ok(Self {
            backend_kind: CaptureBackend::WlrScreencopy,
            inner: Arc::new(CaptureInner::WlrScreencopy(capture)),
            width: actual_w,
            height: actual_h,
        })
    }

    pub async fn open_with_preference(
        width: u32,
        height: u32,
        preference: CapturePreference,
    ) -> Result<Self, CaptureError> {
        let on_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
        if !on_wayland {
            return match preference {
                CapturePreference::Auto => Self::open_x11(width, height),
                CapturePreference::KwinVirtual => Err(CaptureError::BackendUnavailable(
                    "kwin-virtual capture requires a Wayland session",
                )),
                CapturePreference::Screencopy => Err(CaptureError::BackendUnavailable(
                    "screencopy capture requires a Wayland session",
                )),
                CapturePreference::Portal => Err(CaptureError::BackendUnavailable(
                    "portal capture requires a Wayland session",
                )),
            };
        }
        match preference {
            CapturePreference::Auto => match Self::open_kwin(width, height).await {
                Ok(session) => Ok(session),
                Err(e) => {
                    tracing::info!(
                        "KWin virtual display unavailable ({e}); falling back to the portal"
                    );
                    Self::open_portal(width, height).await
                }
            },
            CapturePreference::KwinVirtual => Self::open_kwin(width, height).await,
            CapturePreference::Screencopy => Self::open_screencopy(None).await,
            CapturePreference::Portal => Self::open_portal(width, height).await,
        }
    }

    pub fn backend(&self) -> CaptureBackend {
        self.backend_kind
    }

    pub fn is_ended(&self) -> bool {
        match self.inner.as_ref() {
            CaptureInner::KwinVirtual(capture) => capture.is_ended(),
            CaptureInner::WlrScreencopy(capture) => capture.is_ended(),
            CaptureInner::X11(_) | CaptureInner::Wayland(_) => false,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub async fn next_frame(&self) -> Result<CapturedFrame, CaptureError> {
        match self.inner.as_ref() {
            CaptureInner::X11(capture) => capture.next_frame().await,
            CaptureInner::Wayland(capture) => capture.next_frame().await,
            CaptureInner::KwinVirtual(capture) => capture.next_frame().await,
            CaptureInner::WlrScreencopy(capture) => capture.next_frame().await,
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
        std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
        assert_eq!(detect_backend(), CaptureBackend::Wayland);
        match prev {
            Some(value) => std::env::set_var("WAYLAND_DISPLAY", value),
            None => std::env::remove_var("WAYLAND_DISPLAY"),
        }
    }

    #[test]
    fn empty_frame_is_zeroes() {
        let pool = FramePool::new();
        let frame = CapturedFrame {
            width: 4,
            height: 2,
            data: pool.acquire(32),
        };
        assert_eq!(frame.data.len(), 32);
        assert!(frame.data.iter().all(|b| *b == 0));
    }
}
