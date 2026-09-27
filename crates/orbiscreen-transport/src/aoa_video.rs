// Orbiscreen - aoa_video.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

//! Native USB/AOA video: length-prefixed Annex-B access units on AOA frames,
//! not HTTP MPEG-TS. USB bulk is ordered and retried, so P-frames are dropped
//! only when the write queue is full (the analogue of a late UDP datagram).

use super::annexb::{self, SpsPps};
use super::wt_protocol;
use super::H264Packet;

pub const FRAME_FLAG_DATA: u8 = 0x01;
pub const FRAME_FLAG_OPEN: u8 = 0x02;
pub const FRAME_FLAG_CLOSE: u8 = 0x04;
pub const FRAME_FLAG_RESET: u8 = 0x08;
pub const FRAME_FLAG_VIDEO: u8 = 0x10;

pub const FRAME_HEADER_LEN: usize = 5;
pub const MAX_PAYLOAD_LEN: usize = 16384;
pub const VIDEO_STREAM_ID: u16 = 0;
pub const VIDEO_QUEUE_CAP: usize = 2;

pub fn max_aoa_payload() -> usize {
    MAX_PAYLOAD_LEN.saturating_sub(FRAME_HEADER_LEN)
}

pub fn is_video(flags: u8) -> bool {
    flags & FRAME_FLAG_VIDEO != 0
}

pub fn is_video_open(flags: u8) -> bool {
    is_video(flags) && flags & FRAME_FLAG_OPEN != 0
}

pub fn is_video_close(flags: u8) -> bool {
    is_video(flags) && flags & FRAME_FLAG_CLOSE != 0
}

pub fn encode_aoa_frame(stream_id: u16, flags: u8, payload: &[u8]) -> Option<Vec<u8>> {
    if payload.len() > max_aoa_payload() {
        return None;
    }
    let mut out = Vec::with_capacity(FRAME_HEADER_LEN + payload.len());
    out.extend_from_slice(&stream_id.to_be_bytes());
    out.push(flags);
    out.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    out.extend_from_slice(payload);
    Some(out)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFrame<'a> {
    pub stream_id: u16,
    pub flags: u8,
    pub payload: &'a [u8],
}

pub fn parse_aoa_frame(buf: &[u8]) -> Option<(ParsedFrame<'_>, usize)> {
    if buf.len() < FRAME_HEADER_LEN {
        return None;
    }
    let stream_id = u16::from_be_bytes([buf[0], buf[1]]);
    let flags = buf[2];
    let payload_len = u16::from_be_bytes([buf[3], buf[4]]) as usize;
    let total = FRAME_HEADER_LEN + payload_len;
    if buf.len() < total {
        return None;
    }
    Some((
        ParsedFrame {
            stream_id,
            flags,
            payload: &buf[FRAME_HEADER_LEN..total],
        },
        total,
    ))
}

/// Split a byte stream into complete AOA frames. Returns parsed frames and
/// leftover unparsed tail (incomplete header/payload).
pub fn drain_aoa_frames(buf: &[u8]) -> (Vec<(u16, u8, Vec<u8>)>, usize) {
    let mut frames = Vec::new();
    let mut offset = 0;
    while let Some((frame, used)) = parse_aoa_frame(&buf[offset..]) {
        frames.push((frame.stream_id, frame.flags, frame.payload.to_vec()));
        offset += used;
    }
    (frames, offset)
}

/// Pack a length-prefixed `encode_video` blob into one or more AOA VIDEO
/// frames concatenated so the USB writer can emit them as one AU.
pub fn pack_video_bytes(stream_id: u16, bytes: &[u8]) -> Vec<u8> {
    let max = max_aoa_payload();
    let mut out = Vec::new();
    if bytes.is_empty() {
        if let Some(frame) = encode_aoa_frame(stream_id, FRAME_FLAG_VIDEO, &[]) {
            out.extend_from_slice(&frame);
        }
        return out;
    }
    for chunk in bytes.chunks(max) {
        if let Some(frame) = encode_aoa_frame(stream_id, FRAME_FLAG_VIDEO, chunk) {
            out.extend_from_slice(&frame);
        }
    }
    out
}

pub fn encode_video_open(session_id: &str) -> Vec<u8> {
    encode_aoa_frame(
        VIDEO_STREAM_ID,
        FRAME_FLAG_VIDEO | FRAME_FLAG_OPEN,
        session_id.as_bytes(),
    )
    .expect("session id fits one AOA frame")
}

pub fn decode_video_open_session(payload: &[u8]) -> Option<String> {
    if payload.is_empty() {
        return Some(String::new());
    }
    String::from_utf8(payload.to_vec()).ok()
}

pub fn encode_video_open_ack(host_ns: u64) -> Vec<u8> {
    encode_aoa_frame(
        VIDEO_STREAM_ID,
        FRAME_FLAG_VIDEO | FRAME_FLAG_OPEN,
        &host_ns.to_le_bytes(),
    )
    .expect("ack is 8 bytes")
}

pub fn decode_video_open_ack(payload: &[u8]) -> Option<u64> {
    if payload.len() != 8 {
        return None;
    }
    Some(u64::from_le_bytes(payload.try_into().ok()?))
}

pub fn encode_video_close() -> Vec<u8> {
    encode_aoa_frame(VIDEO_STREAM_ID, FRAME_FLAG_VIDEO | FRAME_FLAG_CLOSE, &[])
        .expect("empty close")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lane {
    Priority,
    Video,
}

pub fn lane_for(is_keyframe: bool) -> Lane {
    if is_keyframe {
        Lane::Priority
    } else {
        Lane::Video
    }
}

/// USB bulk does not lose packets. A full video queue means the tablet is
/// behind; drop the P-frame rather than queueing seconds of picture.
pub fn drop_p_on_full_queue(try_send_ok: bool) -> bool {
    !try_send_ok
}

pub fn clock_offset_ns(host_ns: u64, t0_ns: u64, now_ns: u64) -> i64 {
    let rtt = now_ns.saturating_sub(t0_ns);
    host_ns.wrapping_add(rtt / 2).wrapping_sub(now_ns) as i64
}

/// The USB video task is the session's viewer. Transient send failures and a
/// closed broadcast must not drop that lease; only leaving the accessory does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoPumpEvent {
    BridgeStopped,
    BroadcastClosed,
    SendFailed,
}

pub fn video_pump_should_release(event: VideoPumpEvent) -> bool {
    matches!(event, VideoPumpEvent::BridgeStopped)
}

pub fn host_now_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

pub fn pack_h264_packet(
    pkt: &H264Packet,
    sent_ns: u64,
    cached_sps_pps: &mut Option<SpsPps>,
) -> Option<Vec<u8>> {
    let mut pkt = pkt.clone();
    if pkt.is_keyframe {
        let found = annexb::extract_sps_pps(&pkt.bytes);
        if !found.sps.is_empty() && !found.pps.is_empty() {
            *cached_sps_pps = Some(found);
        }
        pkt.bytes = annexb::with_parameter_sets(&pkt.bytes, cached_sps_pps.as_ref());
    }
    let framed = wt_protocol::encode_video(&pkt, sent_ns).ok()?;
    Some(pack_video_bytes(VIDEO_STREAM_ID, &framed))
}

/// Concatenate VIDEO payloads (in USB order) and peel complete `encode_video`
/// frames. Used by tests to mimic the Android `IdrFrames.Reader`.
pub fn reassemble_video_payloads(payloads: &[Vec<u8>]) -> Vec<wt_protocol::VideoFrame> {
    let mut buf = Vec::new();
    for p in payloads {
        buf.extend_from_slice(p);
    }
    let mut out = Vec::new();
    while buf.len() >= 4 {
        match wt_protocol::split_frame(&buf) {
            Ok(Some((body, used))) => {
                if let Ok(wt_protocol::Message::Video(v)) = wt_protocol::decode_message(body) {
                    out.push(v);
                }
                buf.drain(..used);
            }
            _ => break,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_au(key: bool, n: usize) -> H264Packet {
        let mut bytes = vec![0, 0, 0, 1, if key { 0x65 } else { 0x41 }];
        bytes.resize(n.max(5), 0xAB);
        H264Packet {
            bytes,
            is_keyframe: key,
            pts_ns: 33_333_333,
        }
    }

    #[test]
    fn header_roundtrip() {
        let frame = encode_aoa_frame(7, FRAME_FLAG_DATA, &[1, 2, 3]).unwrap();
        let (parsed, used) = parse_aoa_frame(&frame).unwrap();
        assert_eq!(used, frame.len());
        assert_eq!(parsed.stream_id, 7);
        assert_eq!(parsed.flags, FRAME_FLAG_DATA);
        assert_eq!(parsed.payload, &[1, 2, 3]);
    }

    #[test]
    fn parse_waits_for_full_payload() {
        let frame = encode_aoa_frame(1, FRAME_FLAG_VIDEO, &[9; 40]).unwrap();
        assert!(parse_aoa_frame(&frame[..4]).is_none());
        assert!(parse_aoa_frame(&frame[..FRAME_HEADER_LEN]).is_none());
        assert!(parse_aoa_frame(&frame[..frame.len() - 1]).is_none());
        assert!(parse_aoa_frame(&frame).is_some());
    }

    #[test]
    fn rejects_payload_over_one_urb() {
        let too_big = vec![0u8; max_aoa_payload() + 1];
        assert!(encode_aoa_frame(0, FRAME_FLAG_VIDEO, &too_big).is_none());
        let ok = vec![0u8; max_aoa_payload()];
        let frame = encode_aoa_frame(0, FRAME_FLAG_VIDEO, &ok).unwrap();
        assert_eq!(frame.len(), MAX_PAYLOAD_LEN);
    }

    #[test]
    fn pack_splits_large_au_on_urb_boundary() {
        let pkt = sample_au(true, 40_000);
        let packed = pack_h264_packet(&pkt, 99, &mut None).unwrap();
        let (frames, used) = drain_aoa_frames(&packed);
        assert_eq!(used, packed.len());
        assert!(frames.len() >= 2);
        for (id, flags, payload) in &frames {
            assert_eq!(*id, VIDEO_STREAM_ID);
            assert_eq!(*flags, FRAME_FLAG_VIDEO);
            assert!(payload.len() <= max_aoa_payload());
            assert!(FRAME_HEADER_LEN + payload.len() <= MAX_PAYLOAD_LEN);
        }
        let payloads: Vec<Vec<u8>> = frames.into_iter().map(|(_, _, p)| p).collect();
        let videos = reassemble_video_payloads(&payloads);
        assert_eq!(videos.len(), 1);
        assert!(videos[0].is_keyframe);
        assert_eq!(videos[0].sent_ns, 99);
        assert_eq!(videos[0].au, pkt.bytes);
    }

    #[test]
    fn usb_packet_splits_still_reassemble() {
        let pkt = sample_au(false, 20_000);
        let packed = pack_h264_packet(&pkt, 7, &mut None).unwrap();
        let mut payloads = Vec::new();
        let (frames, _) = drain_aoa_frames(&packed);
        for (_, _, p) in frames {
            payloads.push(p);
        }
        let videos = reassemble_video_payloads(&payloads);
        assert_eq!(videos.len(), 1);
        assert!(!videos[0].is_keyframe);
        assert_eq!(videos[0].au, pkt.bytes);
    }

    #[test]
    fn open_ack_clock_offset_matches_udp_formula() {
        let open = encode_video_open("sess-1");
        let (parsed, _) = parse_aoa_frame(&open).unwrap();
        assert!(is_video_open(parsed.flags));
        assert_eq!(decode_video_open_session(parsed.payload).unwrap(), "sess-1");

        let t0 = 1_000_000_000u64;
        let host = 2_000_000_000u64;
        let now = t0 + 4_000_000;
        let ack = encode_video_open_ack(host);
        let (parsed, _) = parse_aoa_frame(&ack).unwrap();
        assert_eq!(decode_video_open_ack(parsed.payload).unwrap(), host);
        assert_eq!(clock_offset_ns(host, t0, now), 998_000_000);
    }

    #[test]
    fn keyframes_use_priority_lane_pframes_drop_when_full() {
        assert_eq!(lane_for(true), Lane::Priority);
        assert_eq!(lane_for(false), Lane::Video);
        assert!(!drop_p_on_full_queue(true));
        assert!(drop_p_on_full_queue(false));
        assert_eq!(VIDEO_QUEUE_CAP, 2);
    }

    #[test]
    fn drain_leaves_partial_tail() {
        let a = encode_aoa_frame(1, FRAME_FLAG_VIDEO, &[1, 2]).unwrap();
        let b = encode_aoa_frame(1, FRAME_FLAG_VIDEO, &[3, 4, 5]).unwrap();
        let mut buf = a.clone();
        buf.extend_from_slice(&b);
        buf.extend_from_slice(&b[..4]);
        let (frames, used) = drain_aoa_frames(&buf);
        assert_eq!(frames.len(), 2);
        assert_eq!(used, a.len() + b.len());
        assert_eq!(&buf[used..], &b[..4]);
    }

    #[test]
    fn video_pump_keeps_viewer_on_transient_failure() {
        assert!(video_pump_should_release(VideoPumpEvent::BridgeStopped));
        assert!(!video_pump_should_release(VideoPumpEvent::BroadcastClosed));
        assert!(!video_pump_should_release(VideoPumpEvent::SendFailed));
    }

    #[test]
    fn video_close_flag() {
        let frame = encode_video_close();
        let (parsed, _) = parse_aoa_frame(&frame).unwrap();
        assert!(is_video_close(parsed.flags));
        assert!(parsed.payload.is_empty());
    }
}
