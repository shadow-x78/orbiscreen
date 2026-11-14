use std::sync::Mutex;
use std::time::{Duration, Instant};

use tokio::sync::{broadcast, mpsc};

use crate::annexb;
use crate::display::{AttachedDisplay, DisplayCtl, DisplayInfo};
use crate::{H264Packet, IncomingInput};

pub const IDR_DEBOUNCE: Duration = Duration::from_millis(250);

pub fn now_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
pub struct SessionControl {
    id: Option<String>,
    displays: Option<DisplayCtl>,
    global_idr: Option<mpsc::Sender<()>>,
    idr_debounce: Duration,
    last_idr: std::sync::Arc<Mutex<Instant>>,
}

impl SessionControl {
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn request_idr(&self) {
        let now = Instant::now();
        let mut last = self.last_idr.lock().unwrap_or_else(|e| e.into_inner());
        if now.duration_since(*last) < self.idr_debounce {
            return;
        }
        *last = now;
        drop(last);
        if let (Some(ctl), Some(id)) = (&self.displays, &self.id) {
            let ctl = ctl.clone();
            let id = id.clone();
            tokio::spawn(async move { ctl.idr(&id).await });
        } else if let Some(tx) = &self.global_idr {
            let _ = tx.try_send(());
        }
    }

    pub async fn send_input(&self, event: IncomingInput) {
        if let (Some(ctl), Some(id)) = (&self.displays, &self.id) {
            ctl.input(Some(id.clone()), event).await;
        }
    }
}

impl Drop for SessionControl {
    fn drop(&mut self) {
        if let (Some(ctl), Some(id)) = (self.displays.take(), self.id.take()) {
            tokio::spawn(async move { ctl.detach(&id).await });
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct SessionAttachment {
    pub info: Option<DisplayInfo>,
    pub video: broadcast::Receiver<H264Packet>,
    pub control: SessionControl,
}

impl SessionAttachment {
    pub async fn open(
        displays: Option<&DisplayCtl>,
        video_tx: &broadcast::Sender<H264Packet>,
        global_idr: Option<mpsc::Sender<()>>,
        session: Option<String>,
        key: Option<String>,
        idr_debounce: Duration,
    ) -> Result<Self, String> {
        let (info, video, id, displays) = match displays {
            Some(ctl) => {
                let AttachedDisplay { info, video } = ctl.attach(session, key).await?;
                (
                    Some(info.clone()),
                    video,
                    Some(info.id.clone()),
                    Some(ctl.clone()),
                )
            }
            None => (None, video_tx.subscribe(), None, None),
        };
        let last_idr = Instant::now()
            .checked_sub(idr_debounce)
            .unwrap_or_else(Instant::now);
        Ok(Self {
            info,
            video,
            control: SessionControl {
                id,
                displays,
                global_idr,
                idr_debounce,
                last_idr: std::sync::Arc::new(Mutex::new(last_idr)),
            },
        })
    }

    pub fn split(self) -> (broadcast::Receiver<H264Packet>, SessionControl) {
        (self.video, self.control)
    }
}

#[derive(Debug, Default)]
pub struct KeyframeGate {
    waiting: bool,
}

impl KeyframeGate {
    pub fn new() -> Self {
        Self { waiting: true }
    }

    pub fn reset(&mut self) {
        self.waiting = true;
    }

    pub fn admit(&mut self, packet: &H264Packet) -> bool {
        if !self.waiting {
            return true;
        }
        if packet.is_keyframe {
            self.waiting = false;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
pub struct AnnexBPreparer {
    parameter_sets: Option<annexb::SpsPps>,
}

impl AnnexBPreparer {
    pub fn prepare(&mut self, mut packet: H264Packet) -> H264Packet {
        if packet.is_keyframe {
            let found = annexb::extract_sps_pps(&packet.bytes);
            if !found.sps.is_empty() && !found.pps.is_empty() {
                self.parameter_sets = Some(found);
            }
            packet.bytes = annexb::with_parameter_sets(&packet.bytes, self.parameter_sets.as_ref());
        }
        packet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(is_keyframe: bool) -> H264Packet {
        H264Packet {
            bytes: vec![0, 0, 0, 1, if is_keyframe { 0x65 } else { 0x41 }],
            is_keyframe,
            pts_ns: 0,
        }
    }

    #[test]
    fn keyframe_gate_drops_until_the_first_keyframe() {
        let mut gate = KeyframeGate::new();
        assert!(!gate.admit(&packet(false)));
        assert!(!gate.admit(&packet(false)));
        assert!(gate.admit(&packet(true)));
        assert!(gate.admit(&packet(false)));
    }

    #[test]
    fn keyframe_gate_reset_waits_again() {
        let mut gate = KeyframeGate::new();
        assert!(gate.admit(&packet(true)));
        gate.reset();
        assert!(!gate.admit(&packet(false)));
        assert!(gate.admit(&packet(true)));
    }

    #[test]
    fn timestamp_helper_is_nonzero() {
        assert!(now_ns() > 0);
    }
}
