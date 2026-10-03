use std::sync::atomic::{AtomicU64, Ordering};

use gstreamer::glib;
use gstreamer::prelude::*;
use gstreamer::{ClockTime, ElementFactory, Pipeline};
use gstreamer_app::{AppSink, AppSinkCallbacks, AppSrc};
use orbiscreen_core::frame_pool::PooledFrameBuffer;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{info, instrument, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderKind {
    Auto,
    Nvenc,
    Vaapi,
    X264,
}

impl EncoderKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "vaapi" => Some(Self::Vaapi),
            "nvenc" => Some(Self::Nvenc),
            "x264" => Some(Self::X264),
            _ => None,
        }
    }

    pub fn gst_element(self) -> &'static str {
        match self {
            Self::Auto => "nvh264enc",
            other => other.gst_candidates()[0],
        }
    }

    fn gst_candidates(self) -> &'static [&'static str] {
        match self {
            Self::Auto => Self::Nvenc.gst_candidates(),
            Self::Nvenc => &["nvh264enc", "nvcudah264enc", "nvautogpuh264enc"],
            Self::Vaapi => &["vah264enc", "vaapih264enc"],
            Self::X264 => &["x264enc"],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EncodeParams {
    pub kind: EncoderKind,
    pub bitrate_kbps: u32,
    pub width: u32,
    pub height: u32,
    pub framerate: u32,
    pub vbv_frames: u32,
}

impl Default for EncodeParams {
    fn default() -> Self {
        Self {
            kind: EncoderKind::Auto,
            bitrate_kbps: 8000,
            width: 1920,
            height: 1080,
            framerate: 60,
            vbv_frames: 1,
        }
    }
}

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("pipeline is flushing")]
    Flushing,
    #[error("pipeline reached end of stream")]
    Eos,
    #[error("gstreamer pipeline error: {0}")]
    Pipeline(String),
    #[error("failed to initialize gstreamer: {0}")]
    Init(String),
    #[error("frame size {got} B does not match encoder config {expected} B ({width}x{height}x4)")]
    FrameSizeMismatch {
        got: usize,
        expected: usize,
        width: u32,
        height: u32,
    },
}

pub fn init() -> Result<(), EncodeError> {
    gstreamer::init().map_err(|e| EncodeError::Init(e.to_string()))
}

fn element_available(name: &str) -> bool {
    gstreamer::Registry::get()
        .find_feature(name, gstreamer::ElementFactory::static_type())
        .is_some()
}

fn first_available_element(kind: EncoderKind) -> Option<&'static str> {
    kind.gst_candidates()
        .iter()
        .copied()
        .find(|name| element_available(name))
}

fn detect_available(preferred: EncoderKind) -> (EncoderKind, &'static str) {
    let search_order = match preferred {
        EncoderKind::Auto | EncoderKind::Nvenc => {
            [EncoderKind::Nvenc, EncoderKind::Vaapi, EncoderKind::X264]
        }
        EncoderKind::Vaapi => [EncoderKind::Vaapi, EncoderKind::Nvenc, EncoderKind::X264],
        EncoderKind::X264 => [EncoderKind::X264, EncoderKind::Nvenc, EncoderKind::Vaapi],
    };
    for kind in search_order {
        if let Some(element) = first_available_element(kind) {
            if kind == EncoderKind::X264 && preferred != EncoderKind::X264 {
                warn!(
                    "hardware H.264 encoder not found (tried nvh264enc, nvcudah264enc, vah264enc, vaapih264enc); falling back to software x264, expect high encode latency at 1440p+"
                );
            }
            return (kind, element);
        }
    }
    warn!("no H.264 encoder found; pipeline construction will fail");
    (EncoderKind::X264, "x264enc")
}

fn make_element(name: &str) -> Result<gstreamer::Element, EncodeError> {
    ElementFactory::make(name)
        .build()
        .map_err(|e| EncodeError::Pipeline(format!("{name}: {e}")))
}

fn set_str_if_present(el: &gstreamer::Element, name: &str, value: &str) {
    if el.find_property(name).is_some() {
        el.set_property_from_str(name, value);
    }
}

fn set_u32_if_present(el: &gstreamer::Element, name: &str, value: u32) {
    let Some(spec) = el.find_property(name) else {
        return;
    };
    if spec.downcast_ref::<glib::ParamSpecUInt>().is_some() {
        el.set_property(name, value);
    } else if spec.downcast_ref::<glib::ParamSpecInt>().is_some() {
        el.set_property(name, value.min(i32::MAX as u32) as i32);
    } else {
        el.set_property_from_str(name, &value.to_string());
    }
}

pub fn suggested_bitrate_kbps(width: u32, height: u32, framerate: u32) -> u32 {
    let pps = (width as u64) * (height as u64) * (framerate.max(1) as u64);
    let kbps = (pps * 8 / 100 / 1000) as u32;
    kbps.clamp(8_000, 50_000)
}

pub fn one_frame_vbv_ms(framerate: u32) -> u32 {
    let fps = framerate.max(1);
    1000u32.div_ceil(fps)
}

pub fn one_frame_vbv_kb(bitrate_kbps: u32, framerate: u32) -> u32 {
    let fps = framerate.max(1);
    bitrate_kbps.max(1).div_ceil(fps).max(1)
}

pub fn scaled_vbv_ms(frames: u32, framerate: u32) -> u32 {
    let fps = framerate.max(1);
    let frames = frames.clamp(1, 8);
    if frames == 1 {
        return one_frame_vbv_ms(framerate);
    }
    (1000u32 / fps * frames).clamp(50, 200)
}

pub fn scaled_vbv_kb(frames: u32, bitrate_kbps: u32, framerate: u32) -> u32 {
    let fps = framerate.max(1);
    let frames = frames.clamp(1, 8);
    if frames == 1 {
        return one_frame_vbv_kb(bitrate_kbps, framerate);
    }
    let one_frame = bitrate_kbps.max(1).div_ceil(fps).max(1);
    (one_frame * frames).max(200)
}

fn configure_one_frame_vbv(
    encoder: &gstreamer::Element,
    bitrate_kbps: u32,
    framerate: u32,
    frames: u32,
) {
    let frames = frames.clamp(1, 8);
    let vbv_ms = scaled_vbv_ms(frames, framerate);
    let vbv_kb = scaled_vbv_kb(frames, bitrate_kbps, framerate);
    set_u32_if_present(encoder, "vbv-buf-capacity", vbv_ms);
    set_u32_if_present(encoder, "cpb-size", vbv_kb);
    set_u32_if_present(encoder, "vbv-buffer-size", vbv_kb);
    set_u32_if_present(encoder, "rc-lookahead", 0);
    info!(
        vbv_ms,
        vbv_kb, bitrate_kbps, framerate, "configured low-latency VBV/CPB buffer"
    );
}

fn max_u32_property(el: &gstreamer::Element, name: &str) -> Option<u32> {
    let spec = el.find_property(name)?;
    spec.downcast_ref::<glib::ParamSpecUInt>()
        .map(|p| p.maximum())
        .or_else(|| {
            spec.downcast_ref::<glib::ParamSpecInt>()
                .map(|p| p.maximum() as u32)
        })
}

fn configure_infinite_gop(encoder: &gstreamer::Element) {
    set_str_if_present(encoder, "intra-refresh", "true");
    if let Some(max) = max_u32_property(encoder, "key-int-max") {
        let value = if max == 0 { 120 } else { max.min(120) };
        encoder.set_property_from_str("key-int-max", &value.to_string());
    }
    if encoder.find_property("gop-size").is_some() {
        if let Some(spec) = encoder.find_property("gop-size") {
            if spec.downcast_ref::<glib::ParamSpecInt>().is_some() {
                encoder.set_property_from_str("gop-size", "120");
            } else if let Some(max) = max_u32_property(encoder, "gop-size") {
                let value = if max == 0 { 120 } else { max.min(120) };
                encoder.set_property_from_str("gop-size", &value.to_string());
            }
        }
    }
    if let Some(max) = max_u32_property(encoder, "keyframe-period") {
        let value = if max == 0 { 120 } else { max.min(120) };
        encoder.set_property_from_str("keyframe-period", &value.to_string());
    }
    if encoder
        .find_property("min-force-key-unit-interval")
        .is_some()
    {
        encoder.set_property("min-force-key-unit-interval", 250_000_000u64);
    }
}

fn force_key_unit_event() -> gstreamer::Event {
    let structure = gstreamer::Structure::builder("GstForceKeyUnit")
        .field("all-headers", true)
        .field("count", 0u32)
        .build();
    gstreamer::event::CustomUpstream::new(structure)
}

#[derive(Debug, Clone)]
pub struct EncodedChunk {
    pub bytes: Vec<u8>,
    pub is_keyframe: bool,
    pub pts_ns: u64,
}

#[allow(missing_debug_implementations)]
pub struct Encoder {
    pipeline: Pipeline,
    appsrc: AppSrc,
    encoder: gstreamer::Element,
    kind: EncoderKind,
    width: u32,
    height: u32,
    rx: Option<mpsc::Receiver<EncodedChunk>>,
    last_key_request_ns: AtomicU64,
}

impl Drop for Encoder {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gstreamer::State::Null);
    }
}

impl Encoder {
    #[instrument(skip_all, fields(width = params.width, height = params.height))]
    pub fn new(params: EncodeParams) -> Result<Self, EncodeError> {
        init()?;
        let (kind, encoder_el) = detect_available(params.kind);
        match Self::build_pipeline(&params, kind, encoder_el) {
            Ok(enc) => Ok(enc),
            Err(e) if kind != EncoderKind::X264 => {
                warn!(
                    "Hardware encoder {encoder_el} failed to initialize ({e}); falling back to software x264"
                );
                Self::build_pipeline(&params, EncoderKind::X264, EncoderKind::X264.gst_element())
            }
            Err(e) => Err(e),
        }
    }

    fn build_pipeline(
        params: &EncodeParams,
        kind: EncoderKind,
        encoder_el: &'static str,
    ) -> Result<Self, EncodeError> {
        info!(?kind, element = encoder_el, "Using GStreamer encoder");
        let encoder = make_element(encoder_el)?;

        let appsrc = ElementFactory::make("appsrc")
            .build()
            .map_err(|e| EncodeError::Pipeline(format!("appsrc: {e}")))?
            .downcast::<AppSrc>()
            .map_err(|_| EncodeError::Pipeline("appsrc downcast".into()))?;

        let videoconvert = make_element("videoconvert")?;
        set_str_if_present(&videoconvert, "n-threads", "4");

        let appsink = ElementFactory::make("appsink")
            .build()
            .map_err(|e| EncodeError::Pipeline(format!("appsink: {e}")))?
            .downcast::<AppSink>()
            .map_err(|_| EncodeError::Pipeline("appsink downcast".into()))?;
        appsink.set_sync(false);
        appsink.set_drop(true);
        appsink.set_max_buffers(1);
        appsink.set_caps(Some(
            &gstreamer::Caps::builder("video/x-h264")
                .field("stream-format", "byte-stream")
                .field("alignment", "au")
                .build(),
        ));

        let pipeline = Pipeline::new();
        pipeline
            .add_many([
                appsrc.upcast_ref(),
                &videoconvert,
                &encoder,
                appsink.upcast_ref(),
            ])
            .map_err(|e| EncodeError::Pipeline(format!("add_many: {e}")))?;

        let caps = gstreamer::Caps::builder("video/x-raw")
            .field("format", "BGRA")
            .field("width", params.width as i32)
            .field("height", params.height as i32)
            .field(
                "framerate",
                gstreamer::Fraction::new(params.framerate as i32, 1),
            )
            .build();
        appsrc.set_caps(Some(&caps));
        appsrc.set_format(gstreamer::Format::Time);
        appsrc.set_is_live(true);
        appsrc.set_do_timestamp(false);
        appsrc.set_max_bytes((params.width as u64) * params.height as u64 * 4);

        if encoder.find_property("bitrate").is_some() {
            encoder.set_property_from_str("bitrate", &params.bitrate_kbps.to_string());
        }
        if encoder.find_property("byte-stream").is_some() {
            encoder.set_property_from_str("byte-stream", "true");
        }
        if kind == EncoderKind::Nvenc {
            if encoder.find_property("tune").is_some() {
                encoder.set_property_from_str("tune", "ultra-low-latency");
            }
            if encoder.find_property("zerolatency").is_some() {
                encoder.set_property_from_str("zerolatency", "true");
            }
            if encoder.find_property("preset").is_some() {
                encoder.set_property_from_str("preset", "p1");
            }
            if encoder.find_property("rc-mode").is_some() {
                encoder.set_property_from_str("rc-mode", "cbr");
            }
            if encoder.find_property("repeat-sequence-header").is_some() {
                encoder.set_property_from_str("repeat-sequence-header", "true");
            }
            if encoder.find_property("b-frames").is_some() {
                encoder.set_property_from_str("b-frames", "0");
            }
            set_u32_if_present(&encoder, "qp-max-i", 35);
            set_u32_if_present(&encoder, "qp-max-p", 38);
        }
        if kind == EncoderKind::Vaapi {
            if encoder.find_property("rate-control").is_some() {
                encoder.set_property_from_str("rate-control", "cbr");
            }
            if encoder.find_property("b-frames").is_some() {
                encoder.set_property_from_str("b-frames", "0");
            }
            if encoder.find_property("target-usage").is_some() {
                encoder.set_property_from_str("target-usage", "7");
            }
            if encoder.find_property("ref-frames").is_some() {
                encoder.set_property_from_str("ref-frames", "1");
            }
        }
        if kind == EncoderKind::X264 {
            encoder.set_property_from_str("tune", "zerolatency");
            encoder.set_property_from_str("speed-preset", "ultrafast");
            if encoder.find_property("sliced-threads").is_some() {
                encoder.set_property_from_str("sliced-threads", "true");
            }
            if encoder.find_property("threads").is_some() {
                encoder.set_property_from_str("threads", "4");
            }
            if encoder.find_property("repeat-headers").is_some() {
                encoder.set_property_from_str("repeat-headers", "true");
            }
            if encoder.find_property("b-adapt").is_some() {
                encoder.set_property_from_str("b-adapt", "0");
            }
            if encoder.find_property("bframes").is_some() {
                encoder.set_property_from_str("bframes", "0");
            }
        }
        configure_one_frame_vbv(
            &encoder,
            params.bitrate_kbps,
            params.framerate,
            params.vbv_frames,
        );
        configure_infinite_gop(&encoder);

        let h264parse = make_element("h264parse")?;
        h264parse.set_property_from_str("config-interval", "1");

        let nv12_caps = ElementFactory::make("capsfilter")
            .build()
            .map_err(|e| EncodeError::Pipeline(format!("capsfilter: {e}")))?;
        nv12_caps.set_property(
            "caps",
            gstreamer::Caps::builder("video/x-raw")
                .field("format", "NV12")
                .build(),
        );

        pipeline
            .add(&h264parse)
            .map_err(|e| EncodeError::Pipeline(format!("add parse: {e}")))?;
        pipeline
            .add(&nv12_caps)
            .map_err(|e| EncodeError::Pipeline(format!("add capsfilter: {e}")))?;
        gstreamer::Element::link_many([
            appsrc.upcast_ref(),
            &videoconvert,
            &nv12_caps,
            &encoder,
            &h264parse,
            appsink.upcast_ref(),
        ])
        .map_err(|e| EncodeError::Pipeline(format!("link parse: {e}")))?;

        let (tx, rx) = mpsc::channel::<EncodedChunk>(64);
        appsink.set_callbacks(
            AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    let sample = match sink.pull_sample() {
                        Ok(s) => s,
                        Err(e) => {
                            warn!("encoder pull_sample error: {e}");
                            return Err(gstreamer::FlowError::Eos);
                        }
                    };
                    let buffer = sample.buffer().ok_or_else(|| {
                        warn!("sample had no buffer");
                        gstreamer::FlowError::Eos
                    })?;
                    let map = buffer
                        .map_readable()
                        .map_err(|_| gstreamer::FlowError::Eos)?;
                    let bytes = map.to_vec();
                    let is_keyframe = !buffer.flags().contains(gstreamer::BufferFlags::DELTA_UNIT);
                    let Some(pts_ns) = buffer.pts().map(|t| t.nseconds()) else {
                        tracing::debug!("encoded chunk without PTS dropped");
                        return Ok(gstreamer::FlowSuccess::Ok);
                    };
                    if tx
                        .try_send(EncodedChunk {
                            bytes,
                            is_keyframe,
                            pts_ns,
                        })
                        .is_err()
                    {
                        tracing::warn!("encoded chunk dropped: consumer channel full");
                    }
                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .eos(move |_| {
                    info!("encoder EOS");
                })
                .build(),
        );

        if let Some(bus) = pipeline.bus() {
            bus.set_sync_handler(|_bus, msg| {
                match msg.view() {
                    gstreamer::MessageView::Error(err) => {
                        tracing::error!(
                            target: "orbiscreen_encode",
                            "GStreamer encoder error: {} (debug: {})",
                            err.error(),
                            err.debug().unwrap_or_default()
                        );
                    }
                    gstreamer::MessageView::Warning(warn) => {
                        tracing::warn!(
                            target: "orbiscreen_encode",
                            "GStreamer encoder warning: {} (debug: {})",
                            warn.error(),
                            warn.debug().unwrap_or_default()
                        );
                    }
                    _ => {}
                }
                gstreamer::BusSyncReply::Drop
            });
        }

        pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| EncodeError::Pipeline(format!("set_state Playing: {e}")))?;

        if kind != EncoderKind::X264 {
            if let Some(bus) = pipeline.bus() {
                if let Some(msg) = bus.timed_pop_filtered(
                    gstreamer::ClockTime::from_mseconds(50),
                    &[gstreamer::MessageType::Error],
                ) {
                    if let gstreamer::MessageView::Error(err) = msg.view() {
                        let _ = pipeline.set_state(gstreamer::State::Null);
                        return Err(EncodeError::Pipeline(format!(
                            "{}: {}",
                            err.error(),
                            err.debug().unwrap_or_default()
                        )));
                    }
                }
            }
        }

        Ok(Self {
            pipeline,
            appsrc,
            encoder,
            kind,
            width: params.width,
            height: params.height,
            rx: Some(rx),
            last_key_request_ns: AtomicU64::new(0),
        })
    }

    pub fn request_keyframe(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let prev = self.last_key_request_ns.load(Ordering::Relaxed);
        if now.saturating_sub(prev) < 200_000_000 {
            return;
        }
        self.last_key_request_ns.store(now, Ordering::Relaxed);
        let Some(pad) = self.encoder.static_pad("src") else {
            warn!("encoder has no src pad; cannot request IDR");
            return;
        };
        if pad.send_event(force_key_unit_event()) {
            info!(kind = ?self.kind, "requested IDR from encoder");
        } else {
            warn!(kind = ?self.kind, "encoder rejected force-key-unit");
        }
    }

    pub fn subscribe(&mut self) -> Option<mpsc::Receiver<EncodedChunk>> {
        self.rx.take()
    }

    pub fn push_frame(
        &self,
        frame: &[u8],
        width: u32,
        height: u32,
        pts_ns: u64,
    ) -> Result<(), EncodeError> {
        let expected = self.width as usize * self.height as usize * 4;
        if frame.len() != expected || width != self.width || height != self.height {
            return Err(EncodeError::FrameSizeMismatch {
                got: frame.len(),
                expected,
                width: self.width,
                height: self.height,
            });
        }
        let mut buffer = gstreamer::Buffer::with_size(frame.len())
            .map_err(|e| EncodeError::Pipeline(format!("alloc buffer: {e}")))?;
        {
            let buffer_mut = buffer.get_mut().ok_or_else(|| {
                EncodeError::Pipeline("buffer not uniquely owned after allocation".into())
            })?;
            buffer_mut
                .copy_from_slice(0, frame)
                .map_err(|e| EncodeError::Pipeline(format!("copy_from_slice: {e}")))?;
            buffer_mut.set_pts(ClockTime::from_nseconds(pts_ns));
        }
        self.appsrc.push_buffer(buffer).map_err(|e| match e {
            gstreamer::FlowError::Flushing => EncodeError::Flushing,
            gstreamer::FlowError::Eos => EncodeError::Eos,
            other => EncodeError::Pipeline(format!("push_buffer: {other}")),
        })?;
        Ok(())
    }

    pub fn push_frame_owned(
        &self,
        frame: PooledFrameBuffer,
        width: u32,
        height: u32,
        pts_ns: u64,
    ) -> Result<(), EncodeError> {
        let expected = self.width as usize * self.height as usize * 4;
        if frame.len() != expected || width != self.width || height != self.height {
            return Err(EncodeError::FrameSizeMismatch {
                got: frame.len(),
                expected,
                width: self.width,
                height: self.height,
            });
        }
        let mut buffer = gstreamer::Buffer::from_mut_slice(frame);
        {
            let buffer_mut = buffer.get_mut().ok_or_else(|| {
                EncodeError::Pipeline("buffer not uniquely owned after wrapping".into())
            })?;
            buffer_mut.set_pts(ClockTime::from_nseconds(pts_ns));
        }
        self.appsrc.push_buffer(buffer).map_err(|e| match e {
            gstreamer::FlowError::Flushing => EncodeError::Flushing,
            gstreamer::FlowError::Eos => EncodeError::Eos,
            other => EncodeError::Pipeline(format!("push_buffer: {other}")),
        })?;
        Ok(())
    }

    pub fn stop(&self) {
        if let Err(e) = self.appsrc.end_of_stream() {
            warn!("failed to signal EOS on stop: {e}");
        }
        let _ = self.pipeline.set_state(gstreamer::State::Null);
    }

    pub fn frame_duration_ns(framerate: u32) -> u64 {
        1_000_000_000 / u64::from(framerate).max(1)
    }

    pub fn kind(&self) -> EncoderKind {
        self.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_encoders() {
        assert_eq!(EncoderKind::parse("auto"), Some(EncoderKind::Auto));
        assert_eq!(EncoderKind::parse("x264"), Some(EncoderKind::X264));
        assert_eq!(EncoderKind::parse("NVENC"), Some(EncoderKind::Nvenc));
        assert_eq!(EncoderKind::parse("Vaapi"), Some(EncoderKind::Vaapi));
    }

    #[test]
    fn rejects_unknown_encoders() {
        assert_eq!(EncoderKind::parse("vp9"), None);
        assert_eq!(EncoderKind::parse(""), None);
    }

    #[test]
    fn gst_element_names_are_stable() {
        assert_eq!(EncoderKind::X264.gst_element(), "x264enc");
        assert_eq!(EncoderKind::Nvenc.gst_element(), "nvh264enc");
        assert_eq!(EncoderKind::Vaapi.gst_element(), "vah264enc");
        assert!(EncoderKind::Vaapi
            .gst_candidates()
            .contains(&"vaapih264enc"));
        assert!(EncoderKind::Nvenc.gst_candidates().contains(&"nvh264enc"));
    }

    #[test]
    fn default_params_target_full_hd() {
        let params = EncodeParams::default();
        assert_eq!(params.width, 1920);
        assert_eq!(params.height, 1080);
        assert_eq!(params.framerate, 60);
        assert_eq!(params.kind, EncoderKind::Auto);
    }

    #[test]
    fn init_is_idempotent() {
        init().unwrap();
        init().unwrap();
    }

    #[test]
    fn detect_available_returns_a_known_kind() {
        init().unwrap();
        let (kind, element) = detect_available(EncoderKind::X264);
        assert!(matches!(
            kind,
            EncoderKind::X264 | EncoderKind::Vaapi | EncoderKind::Nvenc,
        ));
        assert!(!element.is_empty());
    }

    #[test]
    fn auto_prefers_hardware_when_va_or_nvenc_is_registered() {
        init().unwrap();
        let (kind, element) = detect_available(EncoderKind::Auto);
        if element_available("vah264enc")
            || element_available("vaapih264enc")
            || element_available("nvh264enc")
        {
            assert_ne!(kind, EncoderKind::X264, "auto selected software {element}");
            assert_ne!(element, "x264enc");
        }
    }

    #[test]
    fn frame_duration_ns_matches_framerate() {
        assert_eq!(Encoder::frame_duration_ns(60), 16_666_666);
        assert_eq!(Encoder::frame_duration_ns(30), 33_333_333);
    }

    #[test]
    fn intra_refresh_exists_only_on_x264() {
        init().unwrap();
        let Ok(x264) = make_element("x264enc") else {
            return;
        };
        assert!(
            x264.find_property("intra-refresh").is_some(),
            "x264enc must expose intra-refresh for 1-3 datagram healing"
        );
        if element_available("vah264enc") {
            let va = make_element("vah264enc").unwrap();
            assert!(
                va.find_property("intra-refresh").is_none(),
                "vah264enc unexpectedly grew intra-refresh; update FEC docs"
            );
        }
    }

    #[test]
    fn infinite_gop_uses_property_maximum() {
        init().unwrap();
        let encoder = match make_element("x264enc") {
            Ok(enc) => enc,
            Err(_) => return,
        };
        configure_infinite_gop(&encoder);
        assert!(encoder.property::<bool>("intra-refresh"));
        assert!(encoder.property::<u32>("key-int-max") > 60);
    }

    fn try_live_encoder(kind: EncoderKind) -> Option<Encoder> {
        match Encoder::new(EncodeParams {
            kind,
            bitrate_kbps: 8000,
            width: 64,
            height: 64,
            framerate: 60,
            vbv_frames: 1,
        }) {
            Ok(enc) => Some(enc),
            Err(e) => {
                eprintln!("Skipping: no H.264 encoder available: {e}");
                None
            }
        }
    }

    fn assert_live_one_frame_vbv(enc: &Encoder) {
        let want_ms = one_frame_vbv_ms(60);
        let want_kb = one_frame_vbv_kb(8000, 60);
        if enc.encoder.find_property("vbv-buf-capacity").is_some() {
            let got = enc.encoder.property::<u32>("vbv-buf-capacity");
            assert_ne!(
                got, 600,
                "Encoder::new left x264enc on the 600 ms default VBV"
            );
            assert_eq!(got, want_ms);
        }
        if enc.encoder.find_property("cpb-size").is_some() {
            let got = enc.encoder.property::<u32>("cpb-size");
            assert_ne!(got, 0, "Encoder::new left vah264enc cpb-size at auto (0)");

            if got != want_kb {
                assert!(
                    got >= want_kb,
                    "vah264enc cpb-size {got} is below the one-frame window {want_kb}"
                );
            }
        }
        if enc.encoder.find_property("vbv-buffer-size").is_some() {
            let got = enc.encoder.property::<u32>("vbv-buffer-size");
            assert_ne!(
                got, 0,
                "Encoder::new left nvh264enc vbv-buffer-size at NVENC default (0)"
            );
            assert_eq!(got, want_kb);
        }
    }

    #[test]
    fn suggested_bitrate_scales_with_resolution_and_fps() {
        assert_eq!(suggested_bitrate_kbps(1920, 1080, 60), 9953);
        assert_eq!(suggested_bitrate_kbps(2560, 1600, 90), 29491);
        assert_eq!(suggested_bitrate_kbps(1280, 720, 30), 8000);
        assert_eq!(suggested_bitrate_kbps(3840, 2160, 60), 39813);
    }

    #[test]
    fn one_frame_vbv_is_bitrate_over_fps() {
        assert_eq!(one_frame_vbv_ms(60), 17);
        assert_eq!(one_frame_vbv_ms(30), 34);
        assert_eq!(one_frame_vbv_ms(1), 1000);
        assert_eq!(one_frame_vbv_ms(0), 1000);
        assert_eq!(one_frame_vbv_kb(8000, 60), 134);
        assert_eq!(one_frame_vbv_kb(8000, 30), 267);
        assert_eq!(one_frame_vbv_kb(1000, 60), 17);
        assert_eq!(one_frame_vbv_kb(0, 60), 1);
    }

    #[test]
    fn configure_one_frame_vbv_writes_cpb_before_playing() {
        init().unwrap();
        let Ok(enc) = make_element("vah264enc") else {
            return;
        };
        configure_one_frame_vbv(&enc, 8000, 60, 1);
        assert_eq!(enc.property::<u32>("cpb-size"), one_frame_vbv_kb(8000, 60));
        assert_ne!(
            enc.property::<u32>("cpb-size"),
            scaled_vbv_kb(4, 8000, 60),
            "production is still on the four-frame window"
        );
    }

    #[test]
    fn x264_encoder_new_caps_vbv_at_one_frame() {
        init().unwrap();
        let Some(enc) = try_live_encoder(EncoderKind::X264) else {
            return;
        };
        assert_eq!(enc.kind(), EncoderKind::X264);
        assert_live_one_frame_vbv(&enc);
        enc.stop();
    }

    #[test]
    fn vaapi_encoder_new_caps_cpb_at_one_frame() {
        init().unwrap();
        if !element_available("vah264enc") && !element_available("vaapih264enc") {
            return;
        }
        let Some(enc) = try_live_encoder(EncoderKind::Vaapi) else {
            return;
        };
        assert_eq!(enc.kind(), EncoderKind::Vaapi);
        assert_live_one_frame_vbv(&enc);
        enc.stop();
    }

    #[test]
    fn auto_encoder_new_caps_vbv_at_one_frame() {
        init().unwrap();
        let Some(enc) = try_live_encoder(EncoderKind::Auto) else {
            return;
        };
        assert_live_one_frame_vbv(&enc);
        enc.stop();
    }

    #[test]
    fn request_keyframe_is_safe_before_playing() {
        init().unwrap();
        let encoder = Encoder::new(EncodeParams {
            kind: EncoderKind::X264,
            bitrate_kbps: 1000,
            width: 64,
            height: 64,
            framerate: 30,
            vbv_frames: 1,
        });
        if let Ok(encoder) = encoder {
            encoder.request_keyframe();
            encoder.stop();
        }
    }

    #[test]
    fn encodes_frame_and_emits_keyframe() {
        init().unwrap();
        let mut encoder = match Encoder::new(EncodeParams {
            kind: EncoderKind::X264,
            bitrate_kbps: 1000,
            width: 64,
            height: 64,
            framerate: 30,
            vbv_frames: 1,
        }) {
            Ok(e) => e,
            Err(_) => return,
        };
        let mut rx = encoder.subscribe().unwrap();
        let dummy_frame = vec![128u8; 64 * 64 * 4];
        for i in 0..10 {
            let pts = i * 33_333_333;
            if i == 4 {
                std::thread::sleep(std::time::Duration::from_millis(250));
                encoder.request_keyframe();
            }
            if encoder.push_frame(&dummy_frame, 64, 64, pts).is_err() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let mut received = 0;
        let mut keyframes = 0;
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_millis(1000) {
            if let Ok(chunk) = rx.try_recv() {
                received += 1;
                if chunk.is_keyframe {
                    keyframes += 1;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(received > 0);
        assert!(keyframes > 0);
    }
}
