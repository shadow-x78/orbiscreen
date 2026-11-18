use std::os::fd::RawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use gstreamer::prelude::*;
use gstreamer_app::{AppSink, AppSinkCallbacks};
use orbiscreen_core::frame::CapturedFrame;
use orbiscreen_core::frame_pool::FramePool;
use tokio::sync::mpsc;

use super::sample_to_captured_frame;

const FRAME_CHANNEL_CAPACITY: usize = 8;

pub(crate) struct PipewireSpec {
    pub fd: Option<RawFd>,
    pub node_id: u32,
    pub width: u32,
    pub height: u32,
}

pub(crate) struct PipewireCapture {
    pipeline: gstreamer::Pipeline,
    rx: tokio::sync::Mutex<mpsc::Receiver<CapturedFrame>>,
    error: Arc<std::sync::Mutex<Option<String>>>,
    ended: Arc<AtomicBool>,
    shutting_down: Arc<AtomicBool>,
    stall: Arc<std::sync::Mutex<StallWatch>>,
}

const STALL_REPORT_AFTER: Duration = Duration::from_secs(5);

#[derive(Debug)]
struct StallWatch {
    opened_at: std::time::Instant,
    frames_seen: u64,
    reported: bool,
}

impl StallWatch {
    fn new() -> Self {
        Self {
            opened_at: std::time::Instant::now(),
            frames_seen: 0,
            reported: false,
        }
    }

    fn note_frame(&mut self) {
        self.frames_seen = self.frames_seen.saturating_add(1);
    }

    fn should_report(&mut self) -> bool {
        if self.frames_seen > 0 || self.reported {
            return false;
        }
        if self.opened_at.elapsed() < STALL_REPORT_AFTER {
            return false;
        }
        self.reported = true;
        true
    }
}

impl PipewireCapture {
    pub fn open(spec: PipewireSpec) -> Result<Self, String> {
        gstreamer::init().map_err(|e| format!("gst init: {e}"))?;
        let source = match spec.fd {
            Some(fd) => format!(
                "pipewiresrc fd={fd} path={} do-timestamp=true always-copy=true",
                spec.node_id
            ),
            None => format!(
                "pipewiresrc path={} do-timestamp=true always-copy=true",
                spec.node_id
            ),
        };
        let pipeline_str = format!(
            "{source} \
             ! video/x-raw \
             ! videoconvert n-threads=4 \
             ! videoscale \
             ! video/x-raw,format=BGRA,width={},height={} \
             ! appsink name=sink drop=true sync=false max-buffers=1 emit-signals=false",
            spec.width, spec.height
        );
        let pipeline = gstreamer::parse::launch(&pipeline_str)
            .map_err(|e| format!("gst launch: {e}"))?
            .downcast::<gstreamer::Pipeline>()
            .map_err(|_| "gst pipeline downcast".to_string())?;
        let appsink = pipeline
            .by_name("sink")
            .ok_or_else(|| "appsink not found".to_string())?
            .downcast::<AppSink>()
            .map_err(|_| "appsink downcast".to_string())?;

        let (tx, rx) = mpsc::channel::<CapturedFrame>(FRAME_CHANNEL_CAPACITY);
        let pool = Arc::new(FramePool::new());
        let stall = Arc::new(std::sync::Mutex::new(StallWatch::new()));
        let stall_for_sink = Arc::clone(&stall);
        appsink.set_callbacks(
            AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    let sample = match sink.pull_sample() {
                        Ok(sample) => sample,
                        Err(e) => {
                            tracing::warn!("pull_sample error: {e}");
                            return Err(gstreamer::FlowError::Eos);
                        }
                    };
                    if let Some(frame) = sample_to_captured_frame(&sample, &pool) {
                        stall_for_sink
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .note_frame();
                        if tx.try_send(frame).is_err() {
                            tracing::debug!("capture frame dropped: consumer channel full");
                        }
                    }
                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );

        let error_slot = Arc::new(std::sync::Mutex::new(None::<String>));
        let error_for_bus = Arc::clone(&error_slot);
        let ended_slot = Arc::new(AtomicBool::new(false));
        let ended_for_bus = Arc::clone(&ended_slot);
        let reached_playing = Arc::new(AtomicBool::new(false));
        let playing_for_bus = Arc::clone(&reached_playing);
        let shutting_down = Arc::new(AtomicBool::new(false));
        let shutting_down_for_bus = Arc::clone(&shutting_down);
        let pipeline_weak = pipeline.downgrade();
        if let Some(bus) = pipeline.bus() {
            bus.set_sync_handler(move |_bus, msg| {
                match msg.view() {
                    gstreamer::MessageView::Error(err) => {
                        tracing::error!(
                            "gstreamer capture error: {} (debug: {})",
                            err.error(),
                            err.debug().unwrap_or_default()
                        );
                        let mut slot = error_for_bus.lock().unwrap_or_else(|e| e.into_inner());
                        if slot.is_none() {
                            *slot = Some(format!(
                                "{} ({})",
                                err.error(),
                                err.debug().unwrap_or_default()
                            ));
                        }
                        ended_for_bus.store(true, Ordering::Relaxed);
                    }
                    gstreamer::MessageView::Eos(_) => {
                        tracing::warn!(
                            "gstreamer capture pipeline reached EOS; the portal stopped \
                             delivering buffers"
                        );
                        let mut slot = error_for_bus.lock().unwrap_or_else(|e| e.into_inner());
                        if slot.is_none() {
                            *slot = Some(
                                "portal screencast reached end-of-stream: the compositor \
                                 stopped sending frames"
                                    .to_string(),
                            );
                        }
                        ended_for_bus.store(true, Ordering::Relaxed);
                    }
                    gstreamer::MessageView::StateChanged(state) => {
                        if shutting_down_for_bus.load(Ordering::Relaxed) {
                            return gstreamer::BusSyncReply::Drop;
                        }
                        let from_pipeline = msg
                            .src()
                            .map(|src| {
                                pipeline_weak
                                    .upgrade()
                                    .map(|pipeline| {
                                        src == pipeline.upcast_ref::<gstreamer::Object>()
                                    })
                                    .unwrap_or(false)
                            })
                            .unwrap_or(false);
                        if !from_pipeline {
                            return gstreamer::BusSyncReply::Drop;
                        }
                        let current = state.current();
                        let pending = state.pending();
                        if current == gstreamer::State::Playing {
                            playing_for_bus.store(true, Ordering::Relaxed);
                            return gstreamer::BusSyncReply::Drop;
                        }
                        if !playing_for_bus.load(Ordering::Relaxed) {
                            return gstreamer::BusSyncReply::Drop;
                        }
                        if pending == gstreamer::State::Playing {
                            return gstreamer::BusSyncReply::Drop;
                        }
                        tracing::warn!(
                            "gstreamer capture pipeline left PLAYING for {:?}; \
                             the portal stopped delivering buffers",
                            current
                        );
                        let mut slot = error_for_bus.lock().unwrap_or_else(|e| e.into_inner());
                        if slot.is_none() {
                            *slot = Some(format!(
                                "portal screencast pipeline stopped in {current:?} \
                                 after it had started"
                            ));
                        }
                        ended_for_bus.store(true, Ordering::Relaxed);
                    }
                    gstreamer::MessageView::Warning(warn) => tracing::warn!(
                        "gstreamer capture warning: {} (debug: {})",
                        warn.error(),
                        warn.debug().unwrap_or_default()
                    ),
                    _ => {}
                }
                gstreamer::BusSyncReply::Drop
            });
        }

        pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| format!("state error: {e}"))?;

        Ok(Self {
            pipeline,
            rx: tokio::sync::Mutex::new(rx),
            error: error_slot,
            ended: ended_slot,
            shutting_down,
            stall,
        })
    }

    pub async fn next_frame(&self) -> Result<CapturedFrame, String> {
        let mut rx = self.rx.lock().await;
        match rx.recv().await {
            Some(frame) => Ok(frame),
            None => {
                let detail = self.error.lock().unwrap_or_else(|e| e.into_inner()).clone();
                Err(detail.unwrap_or_else(|| "GStreamer pipeline closed".to_string()))
            }
        }
    }

    pub fn is_ended(&self) -> bool {
        self.ended.load(Ordering::Relaxed)
    }

    pub fn check_stall(&self) {
        if !self
            .stall
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .should_report()
        {
            return;
        }
        tracing::warn!(
            "the screencast portal negotiated the stream but delivered no frame after {:?}; \
             the compositor may not be drawing on the shared surface. Check that \
             xdg-desktop-portal-cosmic matches the running cosmic-comp and try \
             GST_DEBUG=3 to see the negotiated caps",
            STALL_REPORT_AFTER
        );
    }
}

impl Drop for PipewireCapture {
    fn drop(&mut self) {
        self.shutting_down.store(true, Ordering::Relaxed);
        let _ = self.pipeline.set_state(gstreamer::State::Null);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stall_is_silent_before_the_window_elapses() {
        let mut watch = StallWatch::new();
        assert!(!watch.should_report());
    }

    #[test]
    fn stall_is_silent_once_a_frame_arrived() {
        let mut watch = StallWatch {
            opened_at: std::time::Instant::now() - Duration::from_secs(60),
            frames_seen: 0,
            reported: false,
        };
        watch.note_frame();
        assert!(!watch.should_report());
    }

    #[test]
    fn stall_reports_exactly_once() {
        let mut watch = StallWatch {
            opened_at: std::time::Instant::now() - Duration::from_secs(60),
            frames_seen: 0,
            reported: false,
        };
        assert!(watch.should_report());
        assert!(!watch.should_report());
    }

    #[test]
    fn frame_count_saturates_instead_of_wrapping() {
        let mut watch = StallWatch::new();
        watch.frames_seen = u64::MAX;
        watch.note_frame();
        assert_eq!(watch.frames_seen, u64::MAX);
    }
}
