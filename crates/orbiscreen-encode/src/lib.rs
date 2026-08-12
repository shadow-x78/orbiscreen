// Orbiscreen - orbiscreen-encode library (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use gstreamer::prelude::*;
use gstreamer::{ClockTime, ElementFactory, Pipeline};
use gstreamer_app::{AppSink, AppSinkCallbacks, AppSrc};
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{info, instrument, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderKind {
    Vaapi,
    Nvenc,
    X264,
}

impl EncoderKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "vaapi" => Some(Self::Vaapi),
            "nvenc" => Some(Self::Nvenc),
            "x264" => Some(Self::X264),
            _ => None,
        }
    }

    pub fn gst_element(self) -> &'static str {
        match self {
            Self::Vaapi => "vaapih264enc",
            Self::Nvenc => "nvh264enc",
            Self::X264 => "x264enc",
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
            kind: EncoderKind::X264,
            bitrate_kbps: 8000,
            width: 1920,
            height: 1080,
            framerate: 60,
        }
    }
}

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("encoder not available: {0}")]
    EncoderUnavailable(&'static str),
    #[error("gstreamer pipeline error: {0}")]
    Pipeline(String),
    #[error("failed to initialize gstreamer: {0}")]
    Init(String),
}

pub fn init() -> Result<(), EncodeError> {
    gstreamer::init().map_err(|e| EncodeError::Init(e.to_string()))
}

pub fn detect_available(preferred: EncoderKind) -> EncoderKind {
    for kind in [
        preferred,
        EncoderKind::X264,
        EncoderKind::Vaapi,
        EncoderKind::Nvenc,
    ] {
        if ElementFactory::make(kind.gst_element()).build().is_ok() {
            return kind;
        }
    }
    warn!("no H.264 encoder found; pipeline construction will fail");
    EncoderKind::X264
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
    rx: Option<mpsc::UnboundedReceiver<EncodedChunk>>,
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
        let kind = detect_available(params.kind);
        let encoder_el = kind.gst_element();
        info!(?kind, "Using GStreamer encoder");
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
        appsrc.set_do_timestamp(true);
        appsrc.set_max_bytes((params.width as u64) * params.height as u64 * 4 * 60);

        if encoder.find_property("bitrate").is_some() {
            encoder.set_property_from_str("bitrate", &params.bitrate_kbps.to_string());
        }
        if kind == EncoderKind::X264 {
            encoder.set_property_from_str("tune", "zerolatency");
            encoder.set_property_from_str("speed-preset", "ultrafast");
            // Zero-latency tuning already sets repeat-headers=1. Setting it again
            // here crashes on newer GStreamer builds where the property was
            // removed from GstX264Enc, so only touch it when present.
            if encoder.find_property("repeat-headers").is_some() {
                encoder.set_property_from_str("repeat-headers", "true");
            }
            if encoder.find_property("key-int-max").is_some() {
                encoder.set_property_from_str("key-int-max", "30");
            }
        }

        // h264parse puts SPS/PPS in caps so downstream (appsink) and any TS mux
        // sees them as streamheader=... on every keyframe. Without this the
        // encapsulated stream is just NAL units with no parameter sets for
        // clients that join mid-stream.
        // h264parse puts SPS/PPS in the caps so downstream consumers see them
        // attached to every keyframe. We relink through it instead of just
        // passing the encoder output straight to appsink.
        let h264parse = make_element("h264parse")?;
        h264parse.set_property_from_str("config-interval", "1");
        pipeline
            .add(&h264parse)
            .map_err(|e| EncodeError::Pipeline(format!("add parse: {e}")))?;
        gstreamer::Element::link_many([
            appsrc.upcast_ref(),
            &videoconvert,
            &encoder,
            &h264parse,
            appsink.upcast_ref(),
        ])
        .map_err(|e| EncodeError::Pipeline(format!("link parse: {e}")))?;

        let (tx, rx) = mpsc::unbounded_channel::<EncodedChunk>();
        let enc_samples = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let enc_samples_cb = enc_samples.clone();
        let enc_samples_pub = enc_samples.clone();
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
                    let n = enc_samples_cb.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    if n <= 5 || n % 60 == 0 {
                        info!("encoder sample #{n}");
                    }
                    let buffer = sample.buffer().ok_or_else(|| {
                        warn!("sample had no buffer");
                        gstreamer::FlowError::Eos
                    })?;
                    let map = buffer
                        .map_readable()
                        .map_err(|_| gstreamer::FlowError::Eos)?;
                    let bytes = map.to_vec();
                    let is_keyframe = !buffer.flags().contains(gstreamer::BufferFlags::DELTA_UNIT);
                    let pts_ns = buffer.pts().map(|t| t.nseconds()).unwrap_or(0);
                    tx.send(EncodedChunk {
                        bytes,
                        is_keyframe,
                        pts_ns,
                    })
                    .ok();
                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .eos(move |_| {
                    info!("encoder EOS");
                })
                .build(),
        );
        let enc_samples_alive = enc_samples_pub;
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let n = enc_samples_alive.load(std::sync::atomic::Ordering::Relaxed);
            if n > 0 {
                info!("encoder alive: {n} samples emitted");
            }
        });

        pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| EncodeError::Pipeline(format!("set_state Playing: {e}")))?;

        Ok(Self {
            pipeline,
            appsrc,
            kind,
            rx: Some(rx),
        })
    }

    pub fn subscribe(&mut self) -> Option<mpsc::UnboundedReceiver<EncodedChunk>> {
        self.rx.take()
    }

    /// Push one BGRA frame with the given timestamp; block-free unless the
    /// appsrc queue is full, in which case a flow error is returned.
    pub fn push_frame(
        &self,
        frame: &[u8],
        _width: u32,
        _height: u32,
        pts_ns: u64,
    ) -> Result<(), EncodeError> {
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
        self.appsrc
            .push_buffer(buffer)
            .map_err(|e| EncodeError::Pipeline(format!("push_buffer: {e}")))?;
        Ok(())
    }

    /// Send end-of-stream so the encoder flushes its delayed tail frames
    /// through the pipeline, then tear everything down.
    pub fn stop(&self) {
        if let Err(e) = self.appsrc.end_of_stream() {
            warn!("failed to signal EOS on stop: {e}");
        }
        let _ = self.pipeline.set_state(gstreamer::State::Null);
    }

    /// Duration of one frame in nanoseconds at `framerate` fps.
    pub fn frame_duration_ns(framerate: u32) -> u64 {
        1_000_000_000 / u64::from(framerate)
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
        assert_eq!(EncoderKind::Vaapi.gst_element(), "vaapih264enc");
    }

    #[test]
    fn default_params_target_full_hd() {
        let params = EncodeParams::default();
        assert_eq!(params.width, 1920);
        assert_eq!(params.height, 1080);
        assert_eq!(params.framerate, 60);
        assert_eq!(params.kind, EncoderKind::X264);
    }

    #[test]
    fn init_is_idempotent() {
        init().unwrap();
        init().unwrap();
    }

    #[test]
    fn detect_available_returns_a_known_kind() {
        init().unwrap();
        let kind = detect_available(EncoderKind::X264);
        assert!(matches!(
            kind,
            EncoderKind::X264 | EncoderKind::Vaapi | EncoderKind::Nvenc,
        ));
    }

    #[test]
    fn frame_duration_ns_matches_framerate() {
        assert_eq!(Encoder::frame_duration_ns(60), 16_666_666);
        assert_eq!(Encoder::frame_duration_ns(30), 33_333_333);
    }
}
