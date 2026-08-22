// Orbiscreen - orbiscreen-display library (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use evdi::device_node::DeviceNodeStatus;
use evdi::prelude::*;
use thiserror::Error;
use tracing::{info, instrument, warn};

pub const OPEN_MODE_TIMEOUT: Duration = Duration::from_secs(3);
pub const UPDATE_BUFFER_TIMEOUT: Duration = Duration::from_millis(50);

#[derive(Debug, Error)]
pub enum DisplayError {
    #[error("evdi kernel module is not installed")]
    KernelModuleMissing,
    #[error("evdi kernel module is older than the linked libevdi requires")]
    KernelModuleOutdated,
    #[error("no evdi device node is available (try running with root to call evdi_device_add)")]
    NoDeviceNode,
    #[error("failed to open evdi device node: {0}")]
    OpenDevice(String),
    #[error("timed out waiting for the compositor to publish a mode")]
    ModeTimeout,
    #[error("evdi event channel closed")]
    ChannelClosed,
    #[error("no evdi buffer registered: {0}")]
    NoBuffer(String),
    #[error("unsupported evdi pixel format: {0}")]
    UnsupportedFormat(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualDisplaySpec {
    pub width: u32,
    pub height: u32,
    pub refresh_rate_hz: u32,
}

impl VirtualDisplaySpec {
    pub const FULL_HD_60: Self = Self {
        width: 1920,
        height: 1080,
        refresh_rate_hz: 60,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayStatus {
    Compatible,
    Outdated,
    KernelModuleMissing,
    NoDeviceNode,
}

#[instrument]
pub fn probe() -> DisplayStatus {
    match evdi::check_kernel_mod() {
        KernelModStatus::Compatible => match DeviceNode::list_available() {
            Ok(nodes) if !nodes.is_empty() => DisplayStatus::Compatible,
            _ => DisplayStatus::NoDeviceNode,
        },
        KernelModStatus::Outdated => DisplayStatus::Outdated,
        KernelModStatus::NotInstalled => DisplayStatus::KernelModuleMissing,
    }
}

pub fn device_config_for(spec: VirtualDisplaySpec) -> DeviceConfig {
    if spec == VirtualDisplaySpec::FULL_HD_60 {
        DeviceConfig::sample()
    } else {
        DeviceConfig::new(build_edid(spec.width, spec.height), spec.width, spec.height)
    }
}

#[derive(Debug, Clone)]
pub struct OwnedFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl OwnedFrame {
    pub fn size_in_bytes(width: u32, height: u32) -> usize {
        (width as usize) * (height as usize) * 4
    }
}

fn pick_node(index: Option<u32>) -> Result<(DeviceNode, u32), DisplayError> {
    match index {
        Some(i) => Ok((DeviceNode::new(i as i32), i)),
        None => {
            let mut ids: Vec<i32> = std::fs::read_dir("/dev/dri")
                .map_err(|e| DisplayError::OpenDevice(format!("/dev/dri: {e:?}")))?
                .flatten()
                .filter_map(|entry| {
                    let name = entry.file_name().into_string().ok()?;
                    let id = name.strip_prefix("card")?.parse::<i32>().ok()?;
                    (DeviceNode::new(id).status() == DeviceNodeStatus::Available).then_some(id)
                })
                .collect();
            ids.sort_unstable();
            let id = ids
                .into_iter()
                .next_back()
                .ok_or(DisplayError::NoDeviceNode)?;
            Ok((DeviceNode::new(id), id as u32))
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct VirtualDisplay {
    spec: VirtualDisplaySpec,
    handle: Handle,
    buffer_id: BufferId,
    device_index: u32,
    buffer_mode: Mode,
}

impl VirtualDisplay {
    #[instrument(skip_all, fields(width = spec.width, height = spec.height))]
    pub async fn open(spec: VirtualDisplaySpec) -> Result<Self, DisplayError> {
        Self::open_at(spec, None).await
    }

    #[instrument(skip_all, fields(width = spec.width, height = spec.height, index))]
    #[allow(unsafe_code)]
    pub async fn open_at(
        spec: VirtualDisplaySpec,
        index: Option<u32>,
    ) -> Result<Self, DisplayError> {
        match probe() {
            DisplayStatus::KernelModuleMissing => return Err(DisplayError::KernelModuleMissing),
            DisplayStatus::Outdated => return Err(DisplayError::KernelModuleOutdated),
            _ => {}
        }

        let (node, device_index) = pick_node(index)?;
        info!(node = ?node, card = device_index, "Opening evdi device node");
        #[allow(unsafe_code)]
        let unconnected =
            unsafe { node.open() }.map_err(|e| DisplayError::OpenDevice(format!("{e:?}")))?;

        let cfg = device_config_for(spec);
        let mut handle = unconnected.connect(&cfg);

        let mode = handle
            .events
            .await_mode(OPEN_MODE_TIMEOUT)
            .await
            .map_err(|e| match e {
                evdi::events::AwaitEventError::Timeout => DisplayError::ModeTimeout,
                evdi::events::AwaitEventError::ChannelClosed => DisplayError::ChannelClosed,
            })?;
        info!(?mode, "evdi mode established");

        let buffer_id = handle.new_buffer(&mode);
        Ok(Self {
            spec,
            handle,
            buffer_id,
            device_index,
            buffer_mode: mode,
        })
    }

    pub fn spec(&self) -> VirtualDisplaySpec {
        self.spec
    }

    pub fn device_index(&self) -> u32 {
        self.device_index
    }

    pub fn current_mode(&self) -> Option<Mode> {
        self.handle.events.current_mode()
    }

    pub fn actual_dimensions(&self) -> (u32, u32) {
        (self.buffer_mode.width, self.buffer_mode.height)
    }

    pub async fn next_frame(&mut self) -> Result<Option<OwnedFrame>, DisplayError> {
        let Some(current) = self.current_mode() else {
            return Ok(None);
        };
        if current.width != self.buffer_mode.width || current.height != self.buffer_mode.height {
            info!(
                from = ?self.buffer_mode,
                to = ?current,
                "evdi mode change: reallocating buffer"
            );
            self.handle.unregister_buffer(self.buffer_id);
            self.buffer_id = self.handle.new_buffer(&current);
            self.buffer_mode = current;
        }

        match self
            .handle
            .request_update(self.buffer_id, UPDATE_BUFFER_TIMEOUT)
            .await
        {
            Ok(()) => {}
            Err(evdi::handle::RequestUpdateError::AwaitUpdate(
                evdi::events::AwaitEventError::Timeout,
            )) => return Ok(None),
            Err(evdi::handle::RequestUpdateError::AwaitUpdate(
                evdi::events::AwaitEventError::ChannelClosed,
            )) => return Err(DisplayError::ChannelClosed),
            Err(evdi::handle::RequestUpdateError::UnregisteredBuffer) => {
                return Err(DisplayError::NoBuffer(
                    "registered buffer was unregistered by the kernel".into(),
                ))
            }
        }

        let buffer = self
            .handle
            .get_buffer(self.buffer_id)
            .ok_or(DisplayError::NoBuffer("registered buffer vanished".into()))?;
        let width = buffer.width as u32;
        let height = buffer.height as u32;
        to_tight_bgra(
            buffer.bytes(),
            buffer.stride as u32,
            width,
            height,
            self.buffer_mode.pixel_format,
        )
        .map(Some)
    }

    pub fn drm_connector_name(&self) -> Option<String> {
        Some(format!("DVI-I-{}", self.device_index + 1))
    }
}

fn to_tight_bgra(
    bytes: &[u8],
    stride: u32,
    width: u32,
    height: u32,
    pixel_format: Result<drm_fourcc::DrmFourcc, evdi::UnrecognizedFourcc>,
) -> Result<OwnedFrame, DisplayError> {
    let expected = OwnedFrame::size_in_bytes(width, height);
    match pixel_format {
        Ok(drm_fourcc::DrmFourcc::Xrgb8888) | Ok(drm_fourcc::DrmFourcc::Argb8888) => {
            let row_bytes = (width * 4) as usize;
            let mut data = Vec::with_capacity(expected);
            for row in 0..height as usize {
                let start = row * stride as usize;
                if start + row_bytes > bytes.len() {
                    return Err(DisplayError::NoBuffer(format!(
                        "framebuffer truncated: need {} B, have {}",
                        start + row_bytes,
                        bytes.len()
                    )));
                }
                data.extend_from_slice(&bytes[start..start + row_bytes]);
            }
            Ok(OwnedFrame {
                width,
                height,
                data,
            })
        }
        Ok(drm_fourcc::DrmFourcc::Rgb565) => {
            let mut data = Vec::with_capacity(expected);
            for row in 0..height as usize {
                let start = row * stride as usize;
                for col in 0..width as usize {
                    let off = start + col * 2;
                    if off + 2 > bytes.len() {
                        return Err(DisplayError::NoBuffer("framebuffer truncated".into()));
                    }
                    let pixel = u16::from_le_bytes([bytes[off], bytes[off + 1]]);
                    let r = ((pixel >> 11) & 0x1F) as u8;
                    let g = ((pixel >> 5) & 0x3F) as u8;
                    let b = (pixel & 0x1F) as u8;
                    let r8 = (r << 3) | (r >> 2);
                    let g8 = (g << 2) | (g >> 4);
                    let b8 = (b << 3) | (b >> 2);
                    data.extend_from_slice(&[b8, g8, r8, 0xFF]);
                }
            }
            Ok(OwnedFrame {
                width,
                height,
                data,
            })
        }
        Ok(other) => Err(DisplayError::UnsupportedFormat(other.to_string())),
        Err(e) => Err(DisplayError::UnsupportedFormat(format!(
            "unknown fourcc: {e:?}"
        ))),
    }
}

#[derive(Debug, Clone)]
pub struct EvdiPumpInfo {
    pub width: u32,
    pub height: u32,
    pub connector: Option<String>,
    pub device_index: u32,
}

#[allow(missing_debug_implementations)]
pub struct EvdiFramePump {
    rx: tokio::sync::mpsc::UnboundedReceiver<OwnedFrame>,
    stop: Arc<AtomicBool>,
    info: EvdiPumpInfo,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl EvdiFramePump {
    pub fn spawn(spec: VirtualDisplaySpec) -> Result<Self, DisplayError> {
        let (done_tx, done_rx) = std::sync::mpsc::channel::<Result<EvdiPumpInfo, DisplayError>>();
        let (frames_tx, frames_rx) = tokio::sync::mpsc::unbounded_channel::<OwnedFrame>();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_thread = Arc::clone(&stop);

        let thread = std::thread::Builder::new()
            .name("orbiscreen-evdi-pump".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("evdi pump current-thread runtime");
                runtime.block_on(async move {
                    let mut display = match VirtualDisplay::open(spec).await {
                        Ok(display) => display,
                        Err(e) => {
                            let _ = done_tx.send(Err(e));
                            return;
                        }
                    };
                    let (width, height) = display.actual_dimensions();
                    let info = EvdiPumpInfo {
                        width,
                        height,
                        connector: display.drm_connector_name(),
                        device_index: display.device_index(),
                    };
                    let _ = done_tx.send(Ok(info));

                    loop {
                        if stop_for_thread.load(std::sync::atomic::Ordering::Relaxed) {
                            break;
                        }
                        match display.next_frame().await {
                            Ok(Some(frame)) => {
                                if frames_tx.send(frame).is_err() {
                                    break;
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                warn!("evdi frame error: {e}");
                                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            }
                        }
                    }
                });
            })
            .map_err(|e| DisplayError::OpenDevice(format!("spawn pump thread: {e}")))?;

        let info = done_rx
            .recv_timeout(Duration::from_secs(10))
            .map_err(|_| DisplayError::ModeTimeout)??;

        Ok(Self {
            rx: frames_rx,
            stop,
            info,
            thread: Some(thread),
        })
    }

    pub async fn next_frame(&mut self) -> Option<OwnedFrame> {
        self.rx.recv().await
    }

    pub fn info(&self) -> &EvdiPumpInfo {
        &self.info
    }

    pub fn actual_dimensions(&self) -> (u32, u32) {
        (self.info.width, self.info.height)
    }
}

impl Drop for EvdiFramePump {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

fn build_edid(width: u32, height: u32) -> [u8; 128] {
    let mut edid = [0u8; 128];

    edid[0..8].copy_from_slice(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
    edid[8..10].copy_from_slice(b"OB");
    edid[10..12].copy_from_slice(&[0x53, 0x01]);
    edid[12..16].copy_from_slice(&[1, 2, 3, 4]);
    edid[16] = 1;
    edid[17] = 36;
    edid[18..20].copy_from_slice(&[1, 4]);

    edid[20] = 0x80;
    let cm_w = (width as f32 / 3.0).round() as u8;
    let cm_h = (height as f32 / 3.0).round() as u8;
    edid[21] = cm_w.max(1);
    edid[22] = cm_h.max(1);
    edid[23] = 0x78;
    edid[24] = 0x0A;

    let pixels_h = (width / 8).saturating_sub(1) as u16;
    let pixels_v = (height / 8).saturating_sub(1) as u16;
    let h_blank = ((pixels_h as f32) * 0.18) as u16;
    let v_blank = ((pixels_v as f32) * 0.05) as u16;
    edid[54] = (pixels_h & 0xFF) as u8;
    edid[55] = ((pixels_h >> 8) & 0x03) as u8;
    edid[56] = (pixels_v & 0xFF) as u8;
    edid[57] = ((pixels_v >> 8) & 0x03) as u8;
    edid[58] = (h_blank & 0xFF) as u8;
    edid[59] = ((h_blank >> 8) & 0x03) as u8;
    edid[60] = (v_blank & 0xFF) as u8;
    edid[61] = ((v_blank >> 8) & 0x03) as u8;
    edid[62] = 0x1A;
    edid[63] = 0x00;
    edid[64] = (width & 0xFF) as u8;
    edid[65] = ((width >> 8) & 0x0F) as u8;
    edid[66] = (height & 0xFF) as u8;
    edid[67] = ((height >> 8) & 0x0F) as u8;
    edid[68] = 0x00;
    edid[69] = 0x00;
    edid[70] = 0x1E;
    edid[71] = 0x00;

    edid[72] = 0xFD;
    edid[77] = 30;
    edid[78] = 75;
    edid[79] = 30;
    edid[80] = 110;
    edid[81] = 0x10;
    edid[82] = 0x0A;

    edid[93] = 0xFC;
    let name = b"Orbiscreen";
    for (i, byte) in name.iter().enumerate().take(13) {
        edid[95 + i] = *byte;
    }

    let mut sum: u8 = 0;
    for byte in edid.iter().take(127) {
        sum = sum.wrapping_add(*byte);
    }
    edid[127] = (256u16 - sum as u16) as u8;
    edid
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xrgb_frame(width: u32, height: u32, stride: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; height as usize * stride];
        for row in 0..height as usize {
            for col in 0..width as usize {
                let off = row * stride + col * 4;
                bytes[off] = 10;
                bytes[off + 1] = 20;
                bytes[off + 2] = 30;
            }
        }
        bytes
    }

    #[test]
    fn spec_full_hd_60_has_expected_dimensions() {
        let s = VirtualDisplaySpec::FULL_HD_60;
        assert_eq!(s.width, 1920);
        assert_eq!(s.height, 1080);
        assert_eq!(s.refresh_rate_hz, 60);
    }

    #[test]
    fn edid_1080p_has_valid_checksum() {
        let edid = build_edid(1920, 1080);
        let sum: u8 = edid.iter().fold(0u8, |a, b| a.wrapping_add(*b));
        assert_eq!(sum, 0, "EDID checksum must be zero");
    }

    #[test]
    fn edid_4k_has_valid_checksum() {
        let edid = build_edid(3840, 2160);
        let sum: u8 = edid.iter().fold(0u8, |a, b| a.wrapping_add(*b));
        assert_eq!(sum, 0, "EDID checksum must be zero");
    }

    #[test]
    fn edid_header_magic_is_present() {
        let edid = build_edid(1280, 720);
        assert_eq!(
            &edid[..8],
            &[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]
        );
    }

    #[test]
    fn device_config_for_1080p_uses_sample() {
        let cfg = device_config_for(VirtualDisplaySpec::FULL_HD_60);
        assert_eq!(cfg.width_pixels, 1920);
        assert_eq!(cfg.height_pixels, 1080);
    }

    #[test]
    fn device_config_for_other_resolution_synthesizes() {
        let cfg = device_config_for(VirtualDisplaySpec {
            width: 2560,
            height: 1440,
            refresh_rate_hz: 60,
        });
        assert_eq!(cfg.width_pixels, 2560);
        assert_eq!(cfg.height_pixels, 1440);
        assert_eq!(cfg.edid().len(), 128);
    }

    #[test]
    fn probe_is_safe_without_evdi_loaded() {
        let _ = probe();
    }

    #[test]
    fn to_tight_bgra_removes_stride_padding() {
        let bytes = xrgb_frame(2, 2, 12);
        let frame =
            to_tight_bgra(&bytes, 12, 2, 2, Ok(drm_fourcc::DrmFourcc::Xrgb8888)).expect("convert");
        assert_eq!(frame.width, 2);
        assert_eq!(frame.height, 2);
        assert_eq!(frame.data.len(), 16);
        assert_eq!(&frame.data[0..4], &[10, 20, 30, 0]);
    }

    #[test]
    fn to_tight_bgra_tolerates_unpadded_stride() {
        let bytes = xrgb_frame(4, 3, 16);
        let frame =
            to_tight_bgra(&bytes, 16, 4, 3, Ok(drm_fourcc::DrmFourcc::Xrgb8888)).expect("convert");
        assert_eq!(frame.data.len(), OwnedFrame::size_in_bytes(4, 3));
    }

    #[test]
    fn to_tight_bgra_rejects_truncated_buffer() {
        let bytes = vec![0u8; 8];
        let result = to_tight_bgra(&bytes, 8, 4, 2, Ok(drm_fourcc::DrmFourcc::Xrgb8888));
        assert!(result.is_err());
    }

    #[test]
    fn to_tight_bgra_converts_rgb565() {
        let bytes = [0xFF, 0xFF];
        let frame =
            to_tight_bgra(&bytes, 2, 1, 1, Ok(drm_fourcc::DrmFourcc::Rgb565)).expect("convert");
        assert_eq!(frame.data, vec![0xFF, 0xFF, 0xFF, 0xFF]);
    }
}
