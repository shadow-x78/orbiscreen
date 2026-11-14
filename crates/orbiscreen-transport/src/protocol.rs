pub const TYPE_VIDEO: u8 = 1;
pub const TYPE_HELLO: u8 = 2;
pub const TYPE_HELLO_ACK: u8 = 3;
pub const TYPE_PING: u8 = 4;
pub const TYPE_PONG: u8 = 5;
pub const TYPE_IDR: u8 = 6;
pub const TYPE_PROBE: u8 = 7;
pub const TYPE_PROBE_ACK: u8 = 8;
pub const TYPE_PMTU: u8 = 9;
pub const TYPE_BYE: u8 = 10;

pub const UDP_MAGIC: &[u8; 4] = b"ORB1";
pub const UDP_SEAL_MAGIC: &[u8; 4] = b"ORB2";
pub const MAX_FRAME: usize = 4 * 1024 * 1024;
pub const VIDEO_HEADER_LEN: usize = 28;
pub const WT_VIDEO_PREFIX: usize = 18;
pub const WT_DATAGRAM_HEADER: usize = 24;
pub const DEFAULT_WT_DATAGRAM: usize = 1024;
