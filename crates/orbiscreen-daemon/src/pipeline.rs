use std::sync::Arc;

use orbiscreen_encode::{EncodedChunk, Encoder};
use orbiscreen_input::{InputInjector, PointerEvent, StylusEvent, TouchEvent};
use orbiscreen_transport::{H264Packet, IncomingInput};
use tokio::sync::mpsc;

pub fn spawn_idr_pump(
    mut idr_rx: mpsc::Receiver<()>,
    encoder: Arc<Encoder>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while idr_rx.recv().await.is_some() {
            encoder.request_keyframe();
        }
    })
}

pub fn scale_point(
    x: f64,
    y: f64,
    target_w: u32,
    target_h: u32,
    source_w: u32,
    source_h: u32,
) -> (f64, f64) {
    if !x.is_finite() || !y.is_finite() {
        return (0.0, 0.0);
    }
    let x = (x * f64::from(target_w) / f64::from(source_w.max(1))).clamp(0.0, f64::from(target_w));
    let y = (y * f64::from(target_h) / f64::from(source_h.max(1))).clamp(0.0, f64::from(target_h));
    (x, y)
}

pub async fn dispatch_input(
    injector: &mut InputInjector,
    event: IncomingInput,
    scale: impl Fn(f64, f64) -> (f64, f64),
) {
    match event {
        IncomingInput::Pointer(pointer) => {
            let pointer = match pointer {
                PointerEvent::Move { x, y } => {
                    let (x, y) = scale(x, y);
                    PointerEvent::Move { x, y }
                }
                other => other,
            };
            let _ = injector.inject_pointer(pointer).await;
        }
        IncomingInput::Key(key) => {
            let _ = injector.inject_key(key).await;
        }
        IncomingInput::Stylus(stylus) => {
            let stylus = match stylus {
                StylusEvent::Pressure { x, y, pressure } => {
                    let (x, y) = scale(x, y);
                    StylusEvent::Pressure { x, y, pressure }
                }
                StylusEvent::Tilt {
                    x,
                    y,
                    pressure,
                    tilt_x_deg,
                    tilt_y_deg,
                } => {
                    let (x, y) = scale(x, y);
                    StylusEvent::Tilt {
                        x,
                        y,
                        pressure,
                        tilt_x_deg,
                        tilt_y_deg,
                    }
                }
                other => other,
            };
            let _ = injector.inject_stylus(stylus).await;
        }
        IncomingInput::Touch(touch) => {
            let (x, y) = scale(touch.x, touch.y);
            let _ = injector.inject_touch(TouchEvent { x, y, ..touch }).await;
        }
        IncomingInput::RawPointer { x, y } => {
            let (x, y) = scale(x, y);
            let _ = injector.inject_pointer(PointerEvent::Move { x, y }).await;
        }
        IncomingInput::Resize { width, height } => {
            injector.resize(width, height);
        }
    }
}

#[derive(Debug, Default)]
pub struct TimestampBase {
    base: Option<u64>,
}

impl TimestampBase {
    pub fn normalize(&mut self, chunk: EncodedChunk) -> H264Packet {
        let base = *self.base.get_or_insert(chunk.pts_ns);
        H264Packet {
            bytes: chunk.bytes,
            is_keyframe: chunk.is_keyframe,
            pts_ns: chunk.pts_ns.saturating_sub(base),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(pts_ns: u64) -> EncodedChunk {
        EncodedChunk {
            bytes: vec![1, 2, 3],
            is_keyframe: true,
            pts_ns,
        }
    }

    #[test]
    fn scale_point_maps_into_the_target_extent() {
        assert_eq!(
            scale_point(960.0, 540.0, 1920, 1080, 1920, 1080),
            (960.0, 540.0)
        );
        assert_eq!(
            scale_point(960.0, 540.0, 1280, 720, 1920, 1080),
            (640.0, 360.0)
        );
        assert_eq!(
            scale_point(-10.0, 5000.0, 1920, 1080, 1920, 1080),
            (0.0, 1080.0)
        );
        assert_eq!(
            scale_point(f64::NAN, 1.0, 1920, 1080, 1920, 1080),
            (0.0, 0.0)
        );
    }

    #[test]
    fn timestamp_base_normalizes_the_first_packet_to_zero() {
        let mut base = TimestampBase::default();
        assert_eq!(base.normalize(chunk(1_000)).pts_ns, 0);
        assert_eq!(base.normalize(chunk(1_500)).pts_ns, 500);
        assert_eq!(base.normalize(chunk(2_000)).pts_ns, 1_000);
    }

    #[test]
    fn timestamp_base_never_underflows_on_out_of_order_packets() {
        let mut base = TimestampBase::default();
        assert_eq!(base.normalize(chunk(2_000)).pts_ns, 0);
        assert_eq!(base.normalize(chunk(1_000)).pts_ns, 0);
    }
}
