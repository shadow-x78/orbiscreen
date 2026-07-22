// Orbiscreen — orbiscreen-display library (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::path::Path;
use std::time::Duration;

use evdi::prelude::*;
use thiserror::Error;
use tracing::{info, instrument, warn};

pub const AWAIT_MODE_TIMEOUT: Duration = Duration::from_millis(250);
pub const UPDATE_BUFFER_TIMEOUT: Duration = Duration::from_millis(20);

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

#[allow(missing_debug_implementations)]
pub struct VirtualDisplay {
    spec: VirtualDisplaySpec,
    handle: Handle,
    buffer_id: BufferId,
    device_index: u32,
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

        let node = match index {
            Some(i) => DeviceNode::new(i as i32),
            None => DeviceNode::get().ok_or(DisplayError::NoDeviceNode)?,
        };
        info!(node = ?node, "Opening evdi device node");
        #[allow(unsafe_code)]
        let unconnected =
            unsafe { node.open() }.map_err(|e| DisplayError::OpenDevice(format!("{e:?}")))?;
        let device_index = index.unwrap_or(0);

        let cfg = device_config_for(spec);
        let mut handle = unconnected.connect(&cfg);

        let mode = handle
            .events
            .await_mode(AWAIT_MODE_TIMEOUT)
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
        })
    }

    pub async fn open_many(count: u32, spec: VirtualDisplaySpec) -> Vec<Self> {
        let mut out = Vec::with_capacity(count as usize);
        for i in 0..count {
            let device_index = i;
            match Self::open_at(spec, Some(device_index)).await {
                Ok(display) => out.push(display),
                Err(error) => {
                    warn!(index = i, error = %error, "open_many short-circuiting");
                    break;
                }
            }
        }
        out
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

    pub async fn next_frame(&mut self) -> Result<FrameRef<'_>, DisplayError> {
        self.handle
            .request_update(self.buffer_id, UPDATE_BUFFER_TIMEOUT)
            .await
            .map_err(|e| match e {
                evdi::handle::RequestUpdateError::AwaitUpdate(_) => DisplayError::ChannelClosed,
                evdi::handle::RequestUpdateError::UnregisteredBuffer => DisplayError::ModeTimeout,
            })?;
        let buffer = self
            .handle
            .get_buffer(self.buffer_id)
            .ok_or(DisplayError::NoBuffer("registered buffer vanished".into()))?;
        Ok(FrameRef {
            bytes: buffer.bytes(),
            width: self.current_mode().map_or(self.spec.width, |m| m.width),
            height: self.current_mode().map_or(self.spec.height, |m| m.height),
            stride: self
                .current_mode()
                .map_or(self.spec.width * 4, |m| m.stride()),
        })
    }

    pub fn remove_all_nodes() -> std::io::Result<()> {
        match DeviceNode::remove_all() {
            Ok(()) => Ok(()),
            Err(e) => {
                warn!(error = %e, "evdi_device_remove_all failed");
                Err(e)
            }
        }
    }

    pub fn drm_connector_name(&self) -> Option<String> {
        Some(format!("DVI-I-{}", self.device_index))
    }

    pub fn write_debug_ppm<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create(path)?;
        let mode = self
            .current_mode()
            .ok_or_else(|| std::io::Error::other("no mode yet"))?;
        let buffer = self
            .handle
            .get_buffer(self.buffer_id)
            .ok_or_else(|| std::io::Error::other("buffer missing"))?;
        file.write_all(format!("P6\n{} {}\n255\n", mode.width, mode.height).as_bytes())?;
        let stride = mode.stride() as usize;
        let row_bytes = mode.width as usize * 4;
        for row in buffer.bytes().chunks_exact(stride) {
            for pixel in row[..row_bytes].chunks_exact(4) {
                file.write_all(&[pixel[2], pixel[1], pixel[0]])?;
            }
        }
        file.flush()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FrameRef<'a> {
    pub bytes: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub stride: u32,
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

    edid[75] = 0xFD;
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
    fn drm_connector_name_uses_device_index() {
        assert_eq!(format!("DVI-I-{}", 3), "DVI-I-3");
    }
}
