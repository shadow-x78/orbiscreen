// Orbiscreen - mpegts_mux_timestamps.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use gstreamer::prelude::*;
use gstreamer_app::{AppSink, AppSinkCallbacks, AppSrc};
use orbiscreen_encode::{EncodeParams, Encoder};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

const W: u32 = 320;
const H: u32 = 180;
const FPS: u32 = 60;
const FRAMES: usize = 45;
const PRIME_FRAMES: usize = 8;

fn frame_dur_ns() -> u64 {
    Encoder::frame_duration_ns(FPS)
}

fn encode_frames(pts_base: u64) -> Vec<(Vec<u8>, bool, u64)> {
    gstreamer::init().unwrap();
    let mut enc = Encoder::new(EncodeParams {
        width: W,
        height: H,
        framerate: FPS,
        bitrate_kbps: 400,
        ..Default::default()
    })
    .unwrap();
    let mut rx = enc.subscribe().unwrap();
    let data = vec![0x40u8; (W * H * 4) as usize];
    for i in 0..FRAMES + PRIME_FRAMES {
        enc.push_frame(&data, W, H, pts_base + (i as u64) * frame_dur_ns())
            .unwrap();
        std::thread::sleep(Duration::from_millis(2));
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let mut out = Vec::new();
    while out.len() < FRAMES && std::time::Instant::now() < deadline {
        match rx.try_recv() {
            Ok(c) => out.push((c.bytes, c.is_keyframe, c.pts_ns)),
            Err(_) => std::thread::sleep(Duration::from_millis(5)),
        }
    }
    assert_eq!(out.len(), FRAMES, "encoder did not emit all frames");
    assert!(
        out[0].0.starts_with(&[0, 0, 0, 1]),
        "encoder output must be Annex B byte-stream, got {:02x?}",
        &out[0].0[..out[0].0.len().min(4)]
    );
    out
}

#[test]
fn mpegts_muxer_emits_for_daemon_normalized_timestamps() {
    let raw = encode_frames(0);
    let input_bytes: usize = raw.iter().map(|(b, _, _)| b.len()).sum();

    let base = raw[0].2;
    let normalized: Vec<_> = raw
        .iter()
        .map(|(b, k, p)| (b.clone(), *k, p.saturating_sub(base)))
        .collect();
    for pair in normalized.windows(2) {
        assert!(
            pair[0].2 <= pair[1].2,
            "normalized timestamps must be monotonic"
        );
    }

    let got = mux_to_ts(normalized);
    eprintln!("mpegts output: {got} B from {input_bytes} B of H.264 input");
    assert!(
        got * 2 >= input_bytes && got > 0,
        "expected real MPEG-TS output (in={input_bytes} B), got {got} B"
    );
}

fn mux_to_ts(chunks: Vec<(Vec<u8>, bool, u64)>) -> usize {
    let pipeline = gstreamer::parse::launch(
        "appsrc name=src format=time is-live=false \
         ! video/x-h264,stream-format=byte-stream,alignment=au,framerate=60/1 \
         ! h264parse config-interval=1 \
         ! mpegtsmux alignment=7 \
         ! appsink name=sink drop=false sync=false max-buffers=256",
    )
    .unwrap()
    .downcast::<gstreamer::Pipeline>()
    .unwrap();

    if let Some(bus) = pipeline.bus() {
        bus.set_sync_handler(|_bus, msg| {
            let t = format!("{:?}", msg.type_());
            if t.contains("error") || t.contains("warning") {
                eprintln!("BUS {t}: {msg:?}");
            }
            gstreamer::BusSyncReply::Drop
        });
    }

    let appsrc = pipeline
        .by_name("src")
        .unwrap()
        .downcast::<AppSrc>()
        .unwrap();
    appsrc.set_property("max-bytes", 50_000_000u64);
    appsrc.set_caps(Some(
        &gstreamer::Caps::builder("video/x-h264")
            .field("stream-format", "byte-stream")
            .field("alignment", "au")
            .field("framerate", gstreamer::Fraction::new(60, 1))
            .build(),
    ));
    appsrc.set_format(gstreamer::Format::Time);

    let got = Arc::new(AtomicUsize::new(0));
    let sink_got = Arc::clone(&got);
    let appsink = pipeline
        .by_name("sink")
        .unwrap()
        .downcast::<AppSink>()
        .unwrap();
    appsink.set_callbacks(
        AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                if let Ok(sample) = sink.pull_sample() {
                    if let Some(buffer) = sample.buffer() {
                        sink_got.fetch_add(
                            buffer.map_readable().map(|m| m.size()).unwrap_or(0),
                            Ordering::SeqCst,
                        );
                    }
                }
                Ok(gstreamer::FlowSuccess::Ok)
            })
            .build(),
    );

    pipeline.set_state(gstreamer::State::Playing).unwrap();

    for (bytes, kf, pts) in &chunks {
        let mut buffer = gstreamer::Buffer::with_size(bytes.len()).unwrap();
        {
            let b = buffer.get_mut().unwrap();
            b.copy_from_slice(0, bytes).unwrap();
            b.set_pts(gstreamer::ClockTime::from_nseconds(*pts));
            if *kf {
                b.set_flags(gstreamer::BufferFlags::MARKER);
            }
        }
        appsrc.push_buffer(buffer).unwrap();
    }

    std::thread::sleep(Duration::from_secs(4));
    let _ = pipeline.set_state(gstreamer::State::Null);
    got.load(Ordering::SeqCst)
}
