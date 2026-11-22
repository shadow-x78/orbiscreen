pub mod damage_pump;
pub mod kwin_virtual;
pub mod pipewire;
pub mod wayland;
pub mod wlr_screencopy;
pub mod wlr_virtual_output;
pub mod x11;

use std::sync::Arc;

use orbiscreen_core::frame::frame_len;
pub use orbiscreen_core::frame::CapturedFrame;
use orbiscreen_core::frame_pool::FramePool;
use orbiscreen_core::session::CaptureStep;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureBackend {
    X11,
    Wayland,
    KwinVirtual,
    WlrScreencopy,
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
    #[error("virtual output parked")]
    Parked,
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

#[allow(missing_debug_implementations)]
pub struct CaptureSession {
    backend_kind: CaptureBackend,
    inner: Arc<CaptureInner>,
    width: u32,
    height: u32,
}

/// Outcome of turning one GStreamer sample into a capture frame.
#[derive(Debug)]
pub(crate) enum FrameSample {
    Captured(CapturedFrame),
    /// The pixels are identical to the previous frame, so nothing downstream
    /// needs to run. Mirrors what the X11 backend already does.
    Unchanged,
    /// The sample was unusable and has been logged.
    Dropped,
}

/// Remembers the previous frame's digest. Returns true when the digest matches.
pub(crate) fn frame_is_unchanged(
    last_hash: &std::sync::Mutex<Option<[u64; 2]>>,
    hash: [u64; 2],
) -> bool {
    let mut guard = last_hash.lock().unwrap_or_else(|e| e.into_inner());
    let unchanged = guard.as_ref() == Some(&hash);
    *guard = Some(hash);
    unchanged
}

pub(crate) fn sample_to_captured_frame(
    sample: &gstreamer::Sample,
    pool: &Arc<FramePool>,
    last_hash: &std::sync::Mutex<Option<[u64; 2]>>,
) -> FrameSample {
    let Some(buffer) = sample.buffer() else {
        tracing::warn!("skipping sample with no buffer");
        return FrameSample::Dropped;
    };
    let Some(caps) = sample.caps() else {
        tracing::warn!("skipping sample with no caps");
        return FrameSample::Dropped;
    };
    let Some(structure) = caps.structure(0) else {
        tracing::warn!("skipping sample with empty caps");
        return FrameSample::Dropped;
    };
    let (Ok(width), Ok(height)) = (
        structure.get::<i32>("width"),
        structure.get::<i32>("height"),
    ) else {
        tracing::warn!("skipping sample with missing width/height in caps");
        return FrameSample::Dropped;
    };
    if width <= 0 || height <= 0 {
        tracing::warn!("skipping sample with non-positive dimensions {width}x{height}");
        return FrameSample::Dropped;
    }
    let Ok(map) = buffer.map_readable() else {
        tracing::warn!("buffer not readable; skipping sample");
        return FrameSample::Dropped;
    };
    let Some(expected) = frame_len(width as u32, height as u32) else {
        tracing::warn!("skipping sample with overflowing dimensions {width}x{height}");
        return FrameSample::Dropped;
    };
    if map.size() != expected {
        tracing::warn!(
            "frame size mismatch: got {} B, expected {expected} B ({width}x{height}); dropping",
            map.size()
        );
        return FrameSample::Dropped;
    }

    let hash = x11::fast_frame_hash(&map.as_slice()[..expected]);
    if frame_is_unchanged(last_hash, hash) {
        return FrameSample::Unchanged;
    }

    let mut data = pool.acquire(expected);
    data.copy_from_slice(map.as_slice());
    FrameSample::Captured(CapturedFrame {
        width: width as u32,
        height: height as u32,
        data,
    })
}

struct KwinSlot {
    capture: Option<std::sync::Arc<kwin_virtual::KwinVirtualCapture>>,
    width: u32,
    height: u32,
    refresh_hz: u32,
    connector: Option<String>,
}

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct VirtualOutputLease {
    kwin: Option<std::sync::Arc<tokio::sync::Mutex<KwinSlot>>>,
}

impl VirtualOutputLease {
    pub fn none() -> Self {
        Self { kwin: None }
    }

    pub async fn park(&self) {
        let Some(slot) = &self.kwin else {
            return;
        };
        let mut guard = slot.lock().await;
        if guard.capture.take().is_some() {
            tracing::info!("parked KWin virtual output");
        }
    }

    pub async fn unpark(&self) -> Result<bool, CaptureError> {
        let Some(slot) = &self.kwin else {
            return Ok(false);
        };
        let (width, height, refresh_hz) = {
            let guard = slot.lock().await;
            if guard.capture.is_some() {
                return Ok(false);
            }
            (guard.width, guard.height, guard.refresh_hz)
        };
        let capture = tokio::task::spawn_blocking(move || {
            kwin_virtual::KwinVirtualCapture::open(kwin_virtual::KwinVirtualSpec::unnamed(
                width,
                height,
                refresh_hz,
            ))
        })
        .await
        .map_err(|e| CaptureError::Io(format!("kwin-virtual unpark task: {e}")))??;
        let connector = capture.connector_name().to_string();
        {
            let mut guard = slot.lock().await;
            guard.connector = Some(connector.clone());
            guard.capture = Some(std::sync::Arc::new(capture));
        }
        tracing::info!(connector, "restored KWin virtual output");
        Ok(true)
    }

    pub fn is_kwin(&self) -> bool {
        self.kwin.is_some()
    }

    pub async fn connector_name(&self) -> Option<String> {
        let slot = self.kwin.as_ref()?;
        slot.lock().await.connector.clone()
    }
}

#[allow(missing_debug_implementations)]
enum CaptureInner {
    X11(x11::X11Capture),
    Wayland(wayland::WaylandCapture),
    KwinVirtual(std::sync::Arc<tokio::sync::Mutex<KwinSlot>>),
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

    async fn open_kwin(
        width: u32,
        height: u32,
        output_name: Option<String>,
        refresh_hz: u32,
    ) -> Result<Self, CaptureError> {
        let capture = tokio::task::spawn_blocking(move || {
            let spec = match output_name {
                Some(name) => kwin_virtual::KwinVirtualSpec {
                    width,
                    height,
                    names: vec![name],
                    description: "Orbiscreen Virtual Display".into(),
                    refresh_hz,
                },
                None => kwin_virtual::KwinVirtualSpec::unnamed(width, height, refresh_hz),
            };
            kwin_virtual::KwinVirtualCapture::open(spec)
        })
        .await
        .map_err(|e| CaptureError::Io(format!("kwin-virtual open task: {e}")))??;
        let (actual_w, actual_h) = capture.dimensions();
        tracing::info!(
            "KWin virtual display created via zkde-screencast: no root, no share dialog"
        );
        let connector = capture.connector_name().to_string();
        let slot = std::sync::Arc::new(tokio::sync::Mutex::new(KwinSlot {
            capture: Some(std::sync::Arc::new(capture)),
            width: actual_w,
            height: actual_h,
            refresh_hz,
            connector: Some(connector),
        }));
        Ok(Self {
            backend_kind: CaptureBackend::KwinVirtual,
            inner: Arc::new(CaptureInner::KwinVirtual(slot)),
            width: actual_w,
            height: actual_h,
        })
    }

    async fn open_screencopy(output_name: Option<String>) -> Result<Self, CaptureError> {
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

    pub async fn open_step(
        step: CaptureStep,
        width: u32,
        height: u32,
        refresh_hz: u32,
        output_name: Option<&str>,
    ) -> Result<Self, CaptureError> {
        match step {
            CaptureStep::X11Root => Self::open_x11(width, height),
            CaptureStep::KwinVirtual => {
                Self::open_kwin(width, height, output_name.map(str::to_string), refresh_hz).await
            }
            CaptureStep::Portal => Self::open_portal(width, height).await,
            CaptureStep::WlrScreencopy => {
                Self::open_screencopy(output_name.map(str::to_string)).await
            }
            CaptureStep::Evdi => Err(CaptureError::BackendUnavailable(
                "evdi capture is opened by the display layer",
            )),
            CaptureStep::WlrootsVirtual => Err(CaptureError::BackendUnavailable(
                "wlroots virtual output is opened by the daemon",
            )),
        }
    }

    pub fn backend(&self) -> CaptureBackend {
        self.backend_kind
    }

    pub fn is_ended(&self) -> bool {
        match self.inner.as_ref() {
            CaptureInner::KwinVirtual(slot) => slot
                .try_lock()
                .ok()
                .and_then(|g| g.capture.as_ref().map(|c| c.is_ended()))
                .unwrap_or(false),
            CaptureInner::WlrScreencopy(capture) => capture.is_ended(),
            CaptureInner::Wayland(capture) => capture.is_ended(),
            CaptureInner::X11(_) => false,
        }
    }

    pub fn virtual_output_lease(&self) -> VirtualOutputLease {
        match self.inner.as_ref() {
            CaptureInner::KwinVirtual(slot) => VirtualOutputLease {
                kwin: Some(slot.clone()),
            },
            _ => VirtualOutputLease { kwin: None },
        }
    }

    pub async fn virtual_output_name(&self) -> Option<String> {
        match self.inner.as_ref() {
            CaptureInner::KwinVirtual(slot) => slot.lock().await.connector.clone(),
            _ => None,
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
            CaptureInner::KwinVirtual(slot) => {
                let cap = slot.lock().await.capture.clone();
                match cap {
                    Some(capture) => capture.next_frame().await,
                    None => Err(CaptureError::Parked),
                }
            }
            CaptureInner::WlrScreencopy(capture) => capture.next_frame().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_frames_are_detected_once() {
        let last = std::sync::Mutex::new(None);
        assert!(!frame_is_unchanged(&last, [1, 2]));
        assert!(frame_is_unchanged(&last, [1, 2]));
        assert!(!frame_is_unchanged(&last, [3, 4]));
        assert!(frame_is_unchanged(&last, [3, 4]));
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
