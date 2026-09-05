// Orbiscreen - lib.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

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
}

impl Default for EncodeParams {
    fn default() -> Self {
        Self {
            kind: EncoderKind::Auto,
            bitrate_kbps: 8000,
            width: 1920,
            height: 1080,
            framerate: 60,
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
                    "hardware H.264 encoder not found (tried nvh264enc, nvcudah264enc, vah264enc, vaapih264enc); falling back to software x264 — expect high encode latency at 1440p+"
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
    kind: EncoderKind,
    width: u32,
    height: u32,
    rx: Option<mpsc::Receiver<EncodedChunk>>,
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
        info!(?kind, element = encoder_el, "Using GStreamer encoder");
        let encoder = make_element(encoder_el)?;

        let appsrc = ElementFactory::make("appsrc")
            .build()
            .map_err(|e| EncodeError::Pipeline(format!("appsrc: {e}")))?
            .downcast::<AppSrc>()
            .map_err(|_| EncodeError::Pipeline("appsrc downcast".into()))?;

        let videoconvert = make_element("videoconvert")?;

        let appsink = ElementFactory::make("appsink")
            .build()
            .map_err(|e| EncodeError::Pipeline(format!("appsink: {e}")))?
            .downcast::<AppSink>()
            .map_err(|_| EncodeError::Pipeline("appsink downcast".into()))?;
        appsink.set_sync(false);
        appsink.set_drop(true);
        appsink.set_max_buffers(2);
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
        appsrc.set_max_bytes((params.width as u64) * params.height as u64 * 4 * 4);

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
            if encoder.find_property("gop-size").is_some() {
                encoder.set_property_from_str("gop-size", "60");
            }
            if encoder.find_property("b-frames").is_some() {
                encoder.set_property_from_str("b-frames", "0");
            }
        }
        if kind == EncoderKind::Vaapi {
            if encoder.find_property("rate-control").is_some() {
                encoder.set_property_from_str("rate-control", "cbr");
            }
            if encoder.find_property("keyframe-period").is_some() {
                encoder.set_property_from_str("keyframe-period", "60");
            }
            if encoder.find_property("key-int-max").is_some() {
                encoder.set_property_from_str("key-int-max", "60");
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
            if encoder.find_property("key-int-max").is_some() {
                encoder.set_property_from_str("key-int-max", "60");
            }
            if encoder.find_property("b-adapt").is_some() {
                encoder.set_property_from_str("b-adapt", "0");
            }
            if encoder.find_property("bframes").is_some() {
                encoder.set_property_from_str("bframes", "0");
            }
        }

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
                        tracing::debug!("encoded chunk dropped: consumer channel full");
                    }
                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .eos(move |_| {
                    info!("encoder EOS");
                })
                .build(),
        );

        pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| EncodeError::Pipeline(format!("set_state Playing: {e}")))?;

        Ok(Self {
            pipeline,
            appsrc,
            kind,
            width: params.width,
            height: params.height,
            rx: Some(rx),
        })
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
}
