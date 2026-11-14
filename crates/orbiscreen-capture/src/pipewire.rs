use std::os::fd::RawFd;
use std::sync::Arc;

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
}

impl PipewireCapture {
    pub fn open(spec: PipewireSpec) -> Result<Self, String> {
        gstreamer::init().map_err(|e| format!("gst init: {e}"))?;
        let source = match spec.fd {
            Some(fd) => format!(
                "pipewiresrc fd={fd} path={} do-timestamp=true",
                spec.node_id
            ),
            None => format!("pipewiresrc path={} do-timestamp=true", spec.node_id),
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
                        if tx.try_send(frame).is_err() {
                            tracing::debug!("capture frame dropped: consumer channel full");
                        }
                    }
                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );

        if let Some(bus) = pipeline.bus() {
            bus.set_sync_handler(|_bus, msg| {
                match msg.view() {
                    gstreamer::MessageView::Error(err) => tracing::error!(
                        "gstreamer capture error: {} (debug: {})",
                        err.error(),
                        err.debug().unwrap_or_default()
                    ),
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
        })
    }

    pub async fn next_frame(&self) -> Result<CapturedFrame, String> {
        let mut rx = self.rx.lock().await;
        rx.recv()
            .await
            .ok_or_else(|| "GStreamer pipeline closed".to_string())
    }
}

impl Drop for PipewireCapture {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gstreamer::State::Null);
    }
}
