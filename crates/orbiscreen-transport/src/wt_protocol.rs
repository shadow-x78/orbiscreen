// Orbiscreen - wt_protocol.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use super::H264Packet;

pub const TYPE_VIDEO: u8 = 1;
pub const TYPE_HELLO: u8 = 2;
pub const TYPE_HELLO_ACK: u8 = 3;
pub const TYPE_PING: u8 = 4;
pub const TYPE_PONG: u8 = 5;
pub const TYPE_IDR: u8 = 6;
pub const TYPE_BYE: u8 = 10;
pub const MAX_FRAME: usize = 4 * 1024 * 1024;
pub const VIDEO_PREFIX: usize = 1 + 1 + 8 + 8;
pub const DATAGRAM_HEADER: usize = 1 + 1 + 2 + 2 + 2 + 8 + 8;
pub const DEFAULT_DATAGRAM: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hello {
    pub token: String,
    pub session: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloAck {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoFrame {
    pub is_keyframe: bool,
    pub pts_ns: u64,
    pub sent_ns: u64,
    pub au: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoDatagram {
    pub is_keyframe: bool,
    pub seq: u16,
    pub frag: u16,
    pub frags: u16,
    pub pts_ns: u64,
    pub sent_ns: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Hello(Hello),
    HelloAck(HelloAck),
    Video(VideoFrame),
    Ping(u64),
    Pong { t0_ns: u64, host_ns: u64 },
    Idr,
    Bye,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    Empty,
    UnknownType(u8),
    Truncated,
    TooLarge(usize),
    BadUtf8,
}

pub fn encode_frame(body: &[u8]) -> Result<Vec<u8>, CodecError> {
    if body.len() > MAX_FRAME {
        return Err(CodecError::TooLarge(body.len()));
    }
    let mut out = Vec::with_capacity(4 + body.len());
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(body);
    Ok(out)
}

pub fn split_frame(buf: &[u8]) -> Result<Option<(&[u8], usize)>, CodecError> {
    if buf.len() < 4 {
        return Ok(None);
    }
    let len = u32::from_be_bytes(buf[0..4].try_into().unwrap_or([0, 0, 0, 0])) as usize;
    if len > MAX_FRAME {
        return Err(CodecError::TooLarge(len));
    }
    let total = 4 + len;
    if buf.len() < total {
        return Ok(None);
    }
    Ok(Some((&buf[4..total], total)))
}

pub fn encode_hello(token: &str, session: &str) -> Result<Vec<u8>, CodecError> {
    if token.len() > u16::MAX as usize || session.len() > u16::MAX as usize {
        return Err(CodecError::TooLarge(token.len().max(session.len())));
    }
    let mut body = Vec::with_capacity(5 + token.len() + session.len());
    body.push(TYPE_HELLO);
    body.extend_from_slice(&(token.len() as u16).to_le_bytes());
    body.extend_from_slice(token.as_bytes());
    body.extend_from_slice(&(session.len() as u16).to_le_bytes());
    body.extend_from_slice(session.as_bytes());
    encode_frame(&body)
}

pub fn encode_hello_ack(width: u16, height: u16) -> Result<Vec<u8>, CodecError> {
    let mut body = Vec::with_capacity(5);
    body.push(TYPE_HELLO_ACK);
    body.extend_from_slice(&width.to_le_bytes());
    body.extend_from_slice(&height.to_le_bytes());
    encode_frame(&body)
}

pub fn encode_video_datagram(
    seq: u16,
    frag: u16,
    frags: u16,
    is_keyframe: bool,
    pts_ns: u64,
    sent_ns: u64,
    payload: &[u8],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(DATAGRAM_HEADER + payload.len());
    out.push(TYPE_VIDEO);
    out.push(u8::from(is_keyframe));
    out.extend_from_slice(&seq.to_le_bytes());
    out.extend_from_slice(&frag.to_le_bytes());
    out.extend_from_slice(&frags.to_le_bytes());
    out.extend_from_slice(&pts_ns.to_le_bytes());
    out.extend_from_slice(&sent_ns.to_le_bytes());
    out.extend_from_slice(payload);
    out
}

pub fn parse_video_datagram(buf: &[u8]) -> Option<VideoDatagram> {
    if buf.len() < DATAGRAM_HEADER || buf[0] != TYPE_VIDEO {
        return None;
    }
    Some(VideoDatagram {
        is_keyframe: buf[1] & 1 != 0,
        seq: u16::from_le_bytes([buf[2], buf[3]]),
        frag: u16::from_le_bytes([buf[4], buf[5]]),
        frags: u16::from_le_bytes([buf[6], buf[7]]),
        pts_ns: u64::from_le_bytes(buf[8..16].try_into().ok()?),
        sent_ns: u64::from_le_bytes(buf[16..24].try_into().ok()?),
        payload: buf[24..].to_vec(),
    })
}

pub fn datagram_payload_size(max_datagram: usize) -> usize {
    max_datagram.saturating_sub(DATAGRAM_HEADER).max(1)
}

pub fn fragment_video_datagrams(
    seq: u16,
    pkt: &H264Packet,
    sent_ns: u64,
    max_datagram: usize,
) -> Vec<Vec<u8>> {
    if pkt.bytes.is_empty() {
        return Vec::new();
    }
    crate::udp_stream::encode_block_packets(
        pkt,
        datagram_payload_size(max_datagram),
        1,
        |frag, frags, key, payload| {
            encode_video_datagram(seq, frag, frags, key, pkt.pts_ns, sent_ns, payload)
        },
    )
}

pub fn seq_delta(cur: u16, prev: u16) -> u16 {
    cur.wrapping_sub(prev)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCarrier {
    Reliable,
    Datagram,
}

pub fn video_carrier(is_keyframe: bool, datagrams_ok: bool) -> VideoCarrier {
    if is_keyframe || !datagrams_ok {
        VideoCarrier::Reliable
    } else {
        VideoCarrier::Datagram
    }
}

pub fn advance_datagram_seq(is_keyframe: bool) -> bool {
    !is_keyframe
}

pub fn encode_video(pkt: &H264Packet, sent_ns: u64) -> Result<Vec<u8>, CodecError> {
    if pkt.bytes.len() > MAX_FRAME.saturating_sub(VIDEO_PREFIX) {
        return Err(CodecError::TooLarge(pkt.bytes.len()));
    }
    let mut body = Vec::with_capacity(VIDEO_PREFIX + pkt.bytes.len());
    body.push(TYPE_VIDEO);
    body.push(u8::from(pkt.is_keyframe));
    body.extend_from_slice(&pkt.pts_ns.to_le_bytes());
    body.extend_from_slice(&sent_ns.to_le_bytes());
    body.extend_from_slice(&pkt.bytes);
    encode_frame(&body)
}

pub fn encode_ctrl(kind: u8) -> Result<Vec<u8>, CodecError> {
    encode_frame(&[kind])
}

pub fn encode_ping(t0_ns: u64) -> Result<Vec<u8>, CodecError> {
    let mut body = Vec::with_capacity(9);
    body.push(TYPE_PING);
    body.extend_from_slice(&t0_ns.to_le_bytes());
    encode_frame(&body)
}

pub fn encode_pong(t0_ns: u64, host_ns: u64) -> Result<Vec<u8>, CodecError> {
    let mut body = Vec::with_capacity(17);
    body.push(TYPE_PONG);
    body.extend_from_slice(&t0_ns.to_le_bytes());
    body.extend_from_slice(&host_ns.to_le_bytes());
    encode_frame(&body)
}

pub fn decode_message(body: &[u8]) -> Result<Message, CodecError> {
    let Some((kind, rest)) = body.split_first() else {
        return Err(CodecError::Empty);
    };
    match *kind {
        TYPE_HELLO => {
            let token = take_len_str(rest)?;
            let after = &rest[2 + token.len()..];
            let session = take_len_str(after)?;
            Ok(Message::Hello(Hello { token, session }))
        }
        TYPE_HELLO_ACK => {
            if rest.len() < 4 {
                return Err(CodecError::Truncated);
            }
            Ok(Message::HelloAck(HelloAck {
                width: u16::from_le_bytes([rest[0], rest[1]]),
                height: u16::from_le_bytes([rest[2], rest[3]]),
            }))
        }
        TYPE_VIDEO => {
            if rest.len() < 17 {
                return Err(CodecError::Truncated);
            }
            Ok(Message::Video(VideoFrame {
                is_keyframe: rest[0] != 0,
                pts_ns: u64::from_le_bytes(rest[1..9].try_into().unwrap_or([0; 8])),
                sent_ns: u64::from_le_bytes(rest[9..17].try_into().unwrap_or([0; 8])),
                au: rest[17..].to_vec(),
            }))
        }
        TYPE_PING => Ok(Message::Ping(take_u64(rest)?)),
        TYPE_PONG => {
            if rest.len() < 16 {
                return Err(CodecError::Truncated);
            }
            Ok(Message::Pong {
                t0_ns: u64::from_le_bytes(rest[0..8].try_into().unwrap_or([0; 8])),
                host_ns: u64::from_le_bytes(rest[8..16].try_into().unwrap_or([0; 8])),
            })
        }
        TYPE_IDR => Ok(Message::Idr),
        TYPE_BYE => Ok(Message::Bye),
        other => Err(CodecError::UnknownType(other)),
    }
}

fn take_u64(rest: &[u8]) -> Result<u64, CodecError> {
    rest.get(..8)
        .and_then(|s| s.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or(CodecError::Truncated)
}

fn take_len_str(buf: &[u8]) -> Result<String, CodecError> {
    if buf.len() < 2 {
        return Err(CodecError::Truncated);
    }
    let n = u16::from_le_bytes([buf[0], buf[1]]) as usize;
    let bytes = buf.get(2..2 + n).ok_or(CodecError::Truncated)?;
    String::from_utf8(bytes.to_vec()).map_err(|_| CodecError::BadUtf8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_roundtrip() {
        let bytes = encode_hello("tok", "sess-1").unwrap();
        let (body, used) = split_frame(&bytes).unwrap().unwrap();
        assert_eq!(used, bytes.len());
        match decode_message(body).unwrap() {
            Message::Hello(h) => {
                assert_eq!(h.token, "tok");
                assert_eq!(h.session, "sess-1");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn hello_ack_and_ctrl() {
        let bytes = encode_hello_ack(2560, 1600).unwrap();
        let (body, _) = split_frame(&bytes).unwrap().unwrap();
        match decode_message(body).unwrap() {
            Message::HelloAck(a) => {
                assert_eq!(a.width, 2560);
                assert_eq!(a.height, 1600);
            }
            other => panic!("{other:?}"),
        }
        let idr = encode_ctrl(TYPE_IDR).unwrap();
        let (body, _) = split_frame(&idr).unwrap().unwrap();
        assert!(matches!(decode_message(body), Ok(Message::Idr)));
    }

    #[test]
    fn video_roundtrip_preserves_annexb() {
        let pkt = H264Packet {
            bytes: vec![0, 0, 0, 1, 0x65, 1, 2, 3],
            is_keyframe: true,
            pts_ns: 33_333_333,
        };
        let bytes = encode_video(&pkt, 99).unwrap();
        let (body, _) = split_frame(&bytes).unwrap().unwrap();
        match decode_message(body).unwrap() {
            Message::Video(v) => {
                assert!(v.is_keyframe);
                assert_eq!(v.pts_ns, 33_333_333);
                assert_eq!(v.sent_ns, 99);
                assert_eq!(v.au, pkt.bytes);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn ping_pong_roundtrip() {
        let ping = encode_ping(7).unwrap();
        let (body, _) = split_frame(&ping).unwrap().unwrap();
        assert_eq!(decode_message(body).unwrap(), Message::Ping(7));
        let pong = encode_pong(7, 11).unwrap();
        let (body, _) = split_frame(&pong).unwrap().unwrap();
        assert_eq!(
            decode_message(body).unwrap(),
            Message::Pong {
                t0_ns: 7,
                host_ns: 11
            }
        );
    }

    #[test]
    fn split_frame_waits_for_complete_payload() {
        let full = encode_ctrl(TYPE_BYE).unwrap();
        assert!(split_frame(&full[..3]).unwrap().is_none());
        assert!(split_frame(&full[..4]).unwrap().is_none());
        let (body, n) = split_frame(&full).unwrap().unwrap();
        assert_eq!(n, full.len());
        assert!(matches!(decode_message(body), Ok(Message::Bye)));
    }

    #[test]
    fn rejects_oversized_length() {
        let mut buf = (MAX_FRAME as u32 + 1).to_be_bytes().to_vec();
        buf.extend_from_slice(&[0u8; 8]);
        assert!(matches!(split_frame(&buf), Err(CodecError::TooLarge(_))));
    }

    #[test]
    fn unknown_type_is_error() {
        assert!(matches!(
            decode_message(&[99]),
            Err(CodecError::UnknownType(99))
        ));
    }

    #[test]
    fn datagram_roundtrip_and_fragments() {
        let pkt = H264Packet {
            bytes: vec![7u8; datagram_payload_size(DEFAULT_DATAGRAM) + 50],
            is_keyframe: true,
            pts_ns: 9,
        };
        let dgrams = fragment_video_datagrams(3, &pkt, 11, DEFAULT_DATAGRAM);
        assert_eq!(dgrams.len(), 2);
        assert!(dgrams.iter().all(|d| d.len() <= DEFAULT_DATAGRAM));
        let a = parse_video_datagram(&dgrams[0]).unwrap();
        let b = parse_video_datagram(&dgrams[1]).unwrap();
        assert_eq!(a.seq, 3);
        assert_eq!(a.frag, 0);
        assert_eq!(b.frag, 1);
        assert_eq!(a.frags, 2);
        assert!(a.is_keyframe && b.is_keyframe);
        let mut au = a.payload;
        au.extend_from_slice(&b.payload);
        assert_eq!(au, pkt.bytes);
    }

    #[test]
    fn fec_parity_is_appended_and_not_counted_in_frags() {
        let chunk = datagram_payload_size(DEFAULT_DATAGRAM);
        let pkt = H264Packet {
            bytes: vec![3u8; chunk * 4],
            is_keyframe: false,
            pts_ns: 1,
        };
        let dgrams = fragment_video_datagrams(7, &pkt, 2, DEFAULT_DATAGRAM);
        let parsed: Vec<_> = dgrams
            .iter()
            .map(|d| parse_video_datagram(d).unwrap())
            .collect();
        assert_eq!(parsed[0].frags, 5);
        let k = parsed[0].frags as usize;
        let m = crate::fec::parity_count(k);
        assert_eq!(parsed.len(), k + m);
        assert!(parsed.iter().take(k).all(|p| (p.frag as usize) < k));
        assert!(parsed.iter().skip(k).all(|p| (p.frag as usize) >= k));
        assert!(parsed.iter().all(|p| p.frags as usize == k));
    }

    #[test]
    fn datagram_rejects_truncated_and_wrong_type() {
        assert!(parse_video_datagram(&[TYPE_VIDEO, 1, 0, 0]).is_none());
        assert!(parse_video_datagram(&encode_ctrl(TYPE_IDR).unwrap()).is_none());
    }

    #[test]
    fn seq_delta_wraps() {
        assert_eq!(seq_delta(1, 0), 1);
        assert_eq!(seq_delta(0, 65535), 1);
    }

    #[test]
    fn keyframe_uses_reliable_carrier_when_datagrams_exist() {
        assert_eq!(video_carrier(true, true), VideoCarrier::Reliable);
    }

    #[test]
    fn p_frame_uses_datagram_when_peer_has_datagrams() {
        assert_eq!(video_carrier(false, true), VideoCarrier::Datagram);
    }

    #[test]
    fn everything_is_reliable_without_datagrams() {
        assert_eq!(video_carrier(true, false), VideoCarrier::Reliable);
        assert_eq!(video_carrier(false, false), VideoCarrier::Reliable);
    }

    #[test]
    fn datagram_seq_does_not_advance_for_reliable_keyframes() {
        assert!(!advance_datagram_seq(true));
        assert!(advance_datagram_seq(false));
    }

    #[test]
    fn empty_session_hello() {
        let bytes = encode_hello("abc", "").unwrap();
        let (body, _) = split_frame(&bytes).unwrap().unwrap();
        match decode_message(body).unwrap() {
            Message::Hello(h) => {
                assert_eq!(h.token, "abc");
                assert!(h.session.is_empty());
            }
            other => panic!("{other:?}"),
        }
    }
}
