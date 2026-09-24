// Orbiscreen - udp_stream.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rand::Rng;
use tokio::net::UdpSocket;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use super::H264Packet;

pub const MAGIC: &[u8; 4] = b"ORB1";
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
pub const VIDEO_HEADER_LEN: usize = 28;
pub const PROBE_HEADER_LEN: usize = 7;
pub const MIN_DATAGRAM: usize = 576;
pub const BASE_DATAGRAM: usize = 1200;
pub const DEFAULT_MAX_DATAGRAM: usize = 1472;
pub const MAX_PAYLOAD: usize = BASE_DATAGRAM - VIDEO_HEADER_LEN;
const CLIENT_TTL: Duration = Duration::from_secs(5);
const PROBE_TIMEOUT: Duration = Duration::from_millis(250);
const MAX_PROBE_ATTEMPTS: u8 = 3;
const PROBE_STEP: usize = 16;

pub fn default_udp_port(signaling_port: u16) -> u16 {
    signaling_port.saturating_add(1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UdpLimits {
    pub max_datagram: usize,
    pub drop_above: Option<usize>,
    pub loss_pct: u8,
}

impl Default for UdpLimits {
    fn default() -> Self {
        Self {
            max_datagram: DEFAULT_MAX_DATAGRAM,
            drop_above: None,
            loss_pct: 0,
        }
    }
}

impl UdpLimits {
    pub fn from_env() -> Self {
        let max_datagram = parse_env_usize("ORBISCREEN_UDP_MAX_DATAGRAM")
            .unwrap_or(DEFAULT_MAX_DATAGRAM)
            .clamp(MIN_DATAGRAM, 65_507);
        let drop_above =
            parse_env_usize("ORBISCREEN_UDP_DROP_ABOVE").map(|n| n.clamp(MIN_DATAGRAM, 65_507));
        let loss_pct = parse_env_usize("ORBISCREEN_UDP_LOSS_PCT")
            .unwrap_or(0)
            .min(90) as u8;
        Self {
            max_datagram,
            drop_above,
            loss_pct,
        }
    }
}

fn parse_env_usize(name: &str) -> Option<usize> {
    std::env::var(name).ok()?.parse().ok()
}

#[derive(Debug, Clone)]
pub struct PmtuSearch {
    low: usize,
    high: usize,
    probe: Option<usize>,
    attempts: u8,
}

impl PmtuSearch {
    pub fn new(max_datagram: usize) -> Self {
        let high = max_datagram.clamp(MIN_DATAGRAM, 65_507);
        let low = BASE_DATAGRAM.min(high).max(MIN_DATAGRAM);
        Self::from_range(low, high)
    }

    pub fn from_min(max_datagram: usize) -> Self {
        let high = max_datagram.clamp(MIN_DATAGRAM, 65_507);
        Self::from_range(MIN_DATAGRAM.min(high), high)
    }

    fn from_range(low: usize, high: usize) -> Self {
        let mut search = Self {
            low,
            high: high.max(low),
            probe: None,
            attempts: 0,
        };
        search.arm();
        search
    }

    pub fn confirmed(&self) -> usize {
        self.low
    }

    pub fn video_payload(&self) -> usize {
        self.low
            .saturating_sub(super::udp_crypto::SEAL_OVERHEAD)
            .saturating_sub(VIDEO_HEADER_LEN)
            .max(1)
    }

    pub fn is_complete(&self) -> bool {
        self.probe.is_none()
    }

    pub fn current_probe(&self) -> Option<usize> {
        self.probe
    }

    fn raise(&mut self, size: usize) {
        if size > self.low && size <= self.high {
            self.low = size;
        }
    }

    pub fn on_ack(&mut self, recv: usize) {
        let Some(probe) = self.probe else {
            return;
        };
        if recv < probe {
            self.raise(recv);
            self.fail();
            return;
        }
        self.low = probe;
        self.attempts = 0;
        self.arm();
    }

    pub fn on_timeout(&mut self) {
        if self.probe.is_none() {
            return;
        }
        self.attempts = self.attempts.saturating_add(1);
        if self.attempts >= MAX_PROBE_ATTEMPTS {
            self.fail();
        }
    }

    pub fn on_unsendable(&mut self) {
        self.fail();
    }

    fn fail(&mut self) {
        let Some(probe) = self.probe else {
            return;
        };
        if probe <= self.low.saturating_add(PROBE_STEP) {
            self.high = self.low;
            self.probe = None;
            self.attempts = 0;
            return;
        }
        self.high = probe.saturating_sub(1).max(self.low);
        self.attempts = 0;
        self.arm();
    }

    fn arm(&mut self) {
        if self.low >= self.high {
            self.probe = None;
            return;
        }
        let span = self.high - self.low;
        let next = if span <= PROBE_STEP {
            self.high
        } else {
            self.low
                .saturating_add(span / 2)
                .max(self.low.saturating_add(PROBE_STEP))
                .min(self.high)
        };
        if next <= self.low {
            self.probe = None;
            return;
        }
        self.probe = Some(next);
        self.attempts = 0;
    }
}

pub fn now_unix_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

pub fn encode_video(
    seq: u16,
    frag: u16,
    frags: u16,
    is_keyframe: bool,
    pts_ns: u64,
    sent_ns: u64,
    payload: &[u8],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(VIDEO_HEADER_LEN + payload.len());
    out.extend_from_slice(MAGIC);
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

pub fn encode_hello(token: &str) -> Vec<u8> {
    encode_hello_session(token, None)
}

pub fn encode_hello_session(token: &str, session: Option<&str>) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + token.len() + 16);
    out.extend_from_slice(MAGIC);
    out.push(TYPE_HELLO);
    out.extend_from_slice(token.as_bytes());
    if let Some(session) = session.filter(|s| !s.is_empty()) {
        out.push(0);
        out.extend_from_slice(session.as_bytes());
    }
    out
}

pub fn encode_bye(session: Option<&str>) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + 16);
    out.extend_from_slice(MAGIC);
    out.push(TYPE_BYE);
    if let Some(session) = session.filter(|s| !s.is_empty()) {
        out.extend_from_slice(session.as_bytes());
    }
    out
}

pub fn encode_hello_ack() -> Vec<u8> {
    let mut out = Vec::with_capacity(5);
    out.extend_from_slice(MAGIC);
    out.push(TYPE_HELLO_ACK);
    out
}

pub fn encode_ping(t0_ns: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(13);
    out.extend_from_slice(MAGIC);
    out.push(TYPE_PING);
    out.extend_from_slice(&t0_ns.to_le_bytes());
    out
}

pub fn encode_pong(t0_ns: u64, host_ns: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(21);
    out.extend_from_slice(MAGIC);
    out.push(TYPE_PONG);
    out.extend_from_slice(&t0_ns.to_le_bytes());
    out.extend_from_slice(&host_ns.to_le_bytes());
    out
}

pub fn encode_idr() -> Vec<u8> {
    let mut out = Vec::with_capacity(5);
    out.extend_from_slice(MAGIC);
    out.push(TYPE_IDR);
    out
}

pub fn encode_probe(id: u16, datagram_len: usize) -> Vec<u8> {
    let len = datagram_len.max(PROBE_HEADER_LEN);
    let mut out = vec![0u8; len];
    out[0..4].copy_from_slice(MAGIC);
    out[4] = TYPE_PROBE;
    out[5..7].copy_from_slice(&id.to_le_bytes());
    out
}

pub fn encode_probe_ack(id: u16, recv_len: u16) -> Vec<u8> {
    let mut out = Vec::with_capacity(9);
    out.extend_from_slice(MAGIC);
    out.push(TYPE_PROBE_ACK);
    out.extend_from_slice(&id.to_le_bytes());
    out.extend_from_slice(&recv_len.to_le_bytes());
    out
}

pub fn encode_pmtu(datagram_len: u16) -> Vec<u8> {
    let mut out = Vec::with_capacity(7);
    out.extend_from_slice(MAGIC);
    out.push(TYPE_PMTU);
    out.extend_from_slice(&datagram_len.to_le_bytes());
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoFragment {
    pub seq: u16,
    pub frag: u16,
    pub frags: u16,
    pub is_keyframe: bool,
    pub pts_ns: u64,
    pub sent_ns: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Packet {
    Video(VideoFragment),
    Hello {
        token: String,
        session: Option<String>,
    },
    HelloAck,
    Bye(Option<String>),
    Ping(u64),
    Pong {
        t0_ns: u64,
        host_ns: u64,
    },
    Idr,
    Probe {
        id: u16,
    },
    ProbeAck {
        id: u16,
        recv: u16,
    },
    Pmtu(u16),
}

pub fn parse_packet(buf: &[u8]) -> Option<Packet> {
    if buf.len() < 5 || buf[0..4] != MAGIC[..] {
        return None;
    }
    match buf[4] {
        TYPE_VIDEO => {
            if buf.len() < 28 {
                return None;
            }
            Some(Packet::Video(VideoFragment {
                is_keyframe: buf[5] & 1 != 0,
                seq: u16::from_le_bytes([buf[6], buf[7]]),
                frag: u16::from_le_bytes([buf[8], buf[9]]),
                frags: u16::from_le_bytes([buf[10], buf[11]]),
                pts_ns: u64::from_le_bytes(buf[12..20].try_into().ok()?),
                sent_ns: u64::from_le_bytes(buf[20..28].try_into().ok()?),
                payload: buf[28..].to_vec(),
            }))
        }
        TYPE_HELLO => {
            let rest = String::from_utf8_lossy(&buf[5..]);
            let (token, session) = match rest.split_once('\0') {
                Some((token, session)) => (
                    token.trim().to_string(),
                    Some(session.trim())
                        .filter(|s| !s.is_empty())
                        .map(str::to_string),
                ),
                None => (rest.trim().to_string(), None),
            };
            Some(Packet::Hello { token, session })
        }
        TYPE_BYE => {
            let session = String::from_utf8_lossy(&buf[5..]);
            let session = session.trim();
            Some(Packet::Bye(if session.is_empty() {
                None
            } else {
                Some(session.to_string())
            }))
        }
        TYPE_HELLO_ACK => Some(Packet::HelloAck),
        TYPE_PING => {
            if buf.len() < 13 {
                return None;
            }
            Some(Packet::Ping(u64::from_le_bytes(
                buf[5..13].try_into().ok()?,
            )))
        }
        TYPE_PONG => {
            if buf.len() < 21 {
                return None;
            }
            Some(Packet::Pong {
                t0_ns: u64::from_le_bytes(buf[5..13].try_into().ok()?),
                host_ns: u64::from_le_bytes(buf[13..21].try_into().ok()?),
            })
        }
        TYPE_IDR => Some(Packet::Idr),
        TYPE_PROBE => {
            if buf.len() < PROBE_HEADER_LEN {
                return None;
            }
            Some(Packet::Probe {
                id: u16::from_le_bytes([buf[5], buf[6]]),
            })
        }
        TYPE_PROBE_ACK => {
            if buf.len() < 9 {
                return None;
            }
            Some(Packet::ProbeAck {
                id: u16::from_le_bytes([buf[5], buf[6]]),
                recv: u16::from_le_bytes([buf[7], buf[8]]),
            })
        }
        TYPE_PMTU => {
            if buf.len() < 7 {
                return None;
            }
            Some(Packet::Pmtu(u16::from_le_bytes([buf[5], buf[6]])))
        }
        _ => None,
    }
}

pub fn fragment_video(
    seq: u16,
    pkt: &H264Packet,
    sent_ns: u64,
    max_payload: usize,
) -> Vec<Vec<u8>> {
    if pkt.bytes.is_empty() {
        return Vec::new();
    }
    encode_block_packets(pkt, max_payload, 5, |frag, frags, key, payload| {
        encode_video(seq, frag, frags, key, pkt.pts_ns, sent_ns, payload)
    })
}

pub(crate) fn encode_block_packets(
    pkt: &H264Packet,
    max_payload: usize,
    flag_index: usize,
    mut encode: impl FnMut(u16, u16, bool, &[u8]) -> Vec<u8>,
) -> Vec<Vec<u8>> {
    super::fec::shard_au_blocks(&pkt.bytes, max_payload.max(1))
        .into_iter()
        .flat_map(|block| {
            let frags = block.shards.data.len() as u16;
            let mut packets =
                Vec::with_capacity(block.shards.data.len() + block.shards.parity.len());
            for (i, part) in block.shards.data.iter().enumerate() {
                let payload = super::fec::block_payload(block.index, block.count, part);
                let mut packet = encode(i as u16, frags, pkt.is_keyframe, &payload);
                if block.count > 1 && packet.len() > flag_index {
                    packet[flag_index] |= super::fec::FEC_BLOCK_FLAG;
                }
                packets.push(packet);
            }
            for (i, par) in block.shards.parity.iter().enumerate() {
                let payload = super::fec::block_payload(block.index, block.count, par);
                let mut packet = encode(frags + i as u16, frags, pkt.is_keyframe, &payload);
                if block.count > 1 && packet.len() > flag_index {
                    packet[flag_index] |= super::fec::FEC_BLOCK_FLAG;
                }
                packets.push(packet);
            }
            packets
        })
        .collect()
}

struct UdpClient {
    last_seen: Instant,
    prune_pending: Option<Instant>,
    pmtu: PmtuSearch,
    probe_id: u16,
    probe_deadline: Option<Instant>,
    announced: bool,
    session: Option<String>,
    key_id: Option<[u8; 16]>,
    payload: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    video_task: Option<tokio::task::JoinHandle<()>>,
}

fn new_client(limits: UdpLimits) -> UdpClient {
    let pmtu = if limits.drop_above.is_some() {
        PmtuSearch::from_min(limits.max_datagram)
    } else {
        PmtuSearch::new(limits.max_datagram)
    };
    UdpClient {
        last_seen: Instant::now(),
        prune_pending: None,
        pmtu,
        probe_id: 0,
        probe_deadline: None,
        announced: false,
        session: None,
        key_id: None,
        payload: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        video_task: None,
    }
}

fn would_drop(len: usize, drop_above: Option<usize>) -> bool {
    drop_above.is_some_and(|limit| len > limit)
}

fn should_lose(loss_pct: u8) -> bool {
    loss_pct > 0 && rand::rng().random_range(0u8..100) < loss_pct
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SendOutcome {
    Sent,
    TooBig,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct AuSendResult {
    attempted: usize,
    sent: usize,
    failed: usize,
    too_big: bool,
}

impl AuSendResult {
    fn request_idr(self) -> bool {
        self.failed > 0 || self.too_big
    }
}

fn note_send(out: &mut AuSendResult, outcome: SendOutcome) -> bool {
    out.attempted += 1;
    match outcome {
        SendOutcome::Sent => {
            out.sent += 1;
            true
        }
        SendOutcome::Failed => {
            out.failed += 1;
            true
        }
        SendOutcome::TooBig => {
            out.too_big = true;
            false
        }
    }
}

#[cfg(test)]
fn dispatch_au_sends<F>(n: usize, mut send: F) -> AuSendResult
where
    F: FnMut(usize) -> SendOutcome,
{
    let mut out = AuSendResult::default();
    for i in 0..n {
        if !note_send(&mut out, send(i)) {
            break;
        }
    }
    out
}

fn seal_frames(
    keys: &super::udp_crypto::UdpKeyRing,
    key_id: &[u8; 16],
    frames: &[Vec<u8>],
) -> Vec<Vec<u8>> {
    let mut sealed = Vec::with_capacity(frames.len());
    for frame in frames {
        match keys.seal_host(key_id, frame) {
            Some(pkt) => sealed.push(pkt),
            None => break,
        }
    }
    sealed
}

async fn send_au_fragments(
    sock: &UdpSocket,
    frames: &[Vec<u8>],
    addr: SocketAddr,
    limits: UdpLimits,
) -> AuSendResult {
    let mut out = AuSendResult::default();
    for frame in frames {
        if !note_send(&mut out, send_datagram(sock, frame, addr, limits).await) {
            break;
        }
    }
    out
}

fn is_msg_too_big(err: &std::io::Error) -> bool {
    err.raw_os_error() == Some(libc::EMSGSIZE)
}

async fn send_datagram(
    sock: &UdpSocket,
    buf: &[u8],
    addr: SocketAddr,
    limits: UdpLimits,
) -> SendOutcome {
    if would_drop(buf.len(), limits.drop_above) {
        return SendOutcome::TooBig;
    }
    if should_lose(limits.loss_pct) {
        return SendOutcome::Failed;
    }
    match sock.send_to(buf, addr).await {
        Ok(_) => SendOutcome::Sent,
        Err(e) if is_msg_too_big(&e) => {
            debug!(%addr, len = buf.len(), "UDP datagram too big: {e}");
            SendOutcome::TooBig
        }
        Err(e) => {
            debug!(%addr, "UDP send failed: {e}");
            SendOutcome::Failed
        }
    }
}

async fn send_probe(
    sock: &UdpSocket,
    addr: SocketAddr,
    client: &mut UdpClient,
    limits: UdpLimits,
    keys: &super::udp_crypto::UdpKeyRing,
) {
    let Some(key_id) = client.key_id else {
        return;
    };
    loop {
        let Some(size) = client.pmtu.current_probe() else {
            return;
        };
        if client.probe_id == 0 {
            client.probe_id = 1;
        }
        let plain_len = size.saturating_sub(super::udp_crypto::SEAL_OVERHEAD);
        if plain_len < PROBE_HEADER_LEN {
            client.probe_deadline = None;
            client.pmtu.on_unsendable();
            bump_probe_id(client);
            continue;
        }
        let plain = encode_probe(client.probe_id, plain_len);
        let Some(pkt) = keys.seal_host_to_len(&key_id, &plain, size) else {
            client.probe_deadline = Some(Instant::now() + PROBE_TIMEOUT);
            return;
        };
        match send_datagram(sock, &pkt, addr, limits).await {
            SendOutcome::Sent => {
                client.probe_deadline = Some(Instant::now() + PROBE_TIMEOUT);
                debug!(%addr, size, id = client.probe_id, "UDP PMTU probe");
                return;
            }
            SendOutcome::TooBig => {
                debug!(
                    %addr,
                    size,
                    id = client.probe_id,
                    "UDP PMTU probe rejected locally"
                );
                client.probe_deadline = None;
                client.pmtu.on_unsendable();
                bump_probe_id(client);
            }
            SendOutcome::Failed => {
                client.probe_deadline = Some(Instant::now() + PROBE_TIMEOUT);
                debug!(
                    %addr,
                    size,
                    id = client.probe_id,
                    sent = false,
                    "UDP PMTU probe"
                );
                return;
            }
        }
    }
}

async fn announce_pmtu(
    sock: &UdpSocket,
    addr: SocketAddr,
    client: &mut UdpClient,
    limits: UdpLimits,
    keys: &super::udp_crypto::UdpKeyRing,
) {
    if client.announced || !client.pmtu.is_complete() {
        return;
    }
    client.announced = true;
    let datagram = client.pmtu.confirmed();
    let payload = client.pmtu.video_payload();
    client
        .payload
        .store(payload, std::sync::atomic::Ordering::Relaxed);
    info!(%addr, datagram, payload, "UDP PMTU confirmed");
    let Some(key_id) = client.key_id else {
        return;
    };
    if let Some(pkt) = keys.seal_host(&key_id, &encode_pmtu(datagram as u16)) {
        let _ = send_datagram(sock, &pkt, addr, limits).await;
    }
}

fn bump_probe_id(client: &mut UdpClient) {
    client.probe_id = client.probe_id.wrapping_add(1);
    if client.probe_id == 0 {
        client.probe_id = 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AckEffect {
    Ignored,
    Continue,
    Completed,
}

fn apply_probe_ack(client: &mut UdpClient, id: u16, recv: usize) -> AckEffect {
    if id != client.probe_id || client.pmtu.is_complete() {
        return AckEffect::Ignored;
    }
    let before = client.pmtu.current_probe();
    client.probe_deadline = None;
    client.pmtu.on_ack(recv);
    if client.pmtu.current_probe() != before {
        bump_probe_id(client);
    }
    if client.pmtu.is_complete() {
        AckEffect::Completed
    } else {
        AckEffect::Continue
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn set_dont_fragment(sock: &std::net::UdpSocket) {
    use std::os::fd::AsRawFd;
    let fd = sock.as_raw_fd();
    let val: libc::c_int = libc::IP_PMTUDISC_PROBE;
    #[allow(unsafe_code)]
    let rc = unsafe {
        libc::setsockopt(
            fd,
            libc::IPPROTO_IP,
            libc::IP_MTU_DISCOVER,
            std::ptr::addr_of!(val).cast(),
            std::mem::size_of_val(&val) as libc::socklen_t,
        )
    };
    if rc != 0 {
        warn!(
            "IP_MTU_DISCOVER PROBE failed: {}",
            std::io::Error::last_os_error()
        );
    }
}

fn bind_udp_socket(port: u16) -> std::io::Result<UdpSocket> {
    let std_sock = std::net::UdpSocket::bind(("0.0.0.0", port))?;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    set_dont_fragment(&std_sock);
    std_sock.set_nonblocking(true)?;
    UdpSocket::from_std(std_sock)
}

#[allow(clippy::too_many_arguments)]
pub async fn run_udp_hub(
    port: u16,
    _token: String,
    mut video_rx: broadcast::Receiver<H264Packet>,
    idr_tx: Option<tokio::sync::mpsc::Sender<()>>,
    stats: std::sync::Arc<super::Stats>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
    limits: UdpLimits,
    displays: Option<super::DisplayCtl>,
    _registry: Option<std::sync::Arc<crate::pairing::PairingRegistry>>,
    keys: super::udp_crypto::UdpKeyRing,
) {
    let sock = match bind_udp_socket(port) {
        Ok(s) => s,
        Err(e) => {
            warn!("UDP video bind {port} failed: {e}");
            return;
        }
    };
    info!(
        max_datagram = limits.max_datagram,
        drop_above = ?limits.drop_above,
        loss_pct = limits.loss_pct,
        "UDP video listening on 0.0.0.0:{port}"
    );
    let sock = std::sync::Arc::new(sock);
    let clients: std::sync::Arc<tokio::sync::Mutex<HashMap<SocketAddr, UdpClient>>> =
        std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    let recv_sock = sock.clone();
    let recv_clients = clients.clone();
    let recv_keys = keys.clone();
    let recv_idr = idr_tx.clone();
    let recv_stats = stats.clone();
    let recv_limits = limits;
    let recv_displays = displays.clone();
    let mut recv_shutdown = shutdown.clone();
    tokio::spawn(async move {
        let mut buf = vec![0u8; 65535];
        loop {
            tokio::select! {
                _ = recv_shutdown.changed() => break,
                res = recv_sock.recv_from(&mut buf) => {
                    let Ok((n, addr)) = res else { continue };
                    let ctx = IncomingCtx {
                        keys: &recv_keys,
                        sock: &recv_sock,
                        clients: &recv_clients,
                        idr_tx: recv_idr.as_ref(),
                        stats: &recv_stats,
                        limits: recv_limits,
                        displays: recv_displays.as_ref(),
                    };
                    handle_incoming(&buf[..n], addr, &ctx).await;
                }
            }
        }
    });

    let mut seq: u16 = 0;
    let mut prune = tokio::time::interval(Duration::from_secs(1));
    let mut probe_tick = tokio::time::interval(Duration::from_millis(20));
    probe_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = shutdown.changed() => break,
            _ = prune.tick() => {
                let mut map = clients.lock().await;
                let now = Instant::now();
                let expired: Vec<SocketAddr> = map
                    .iter()
                    .filter_map(|(addr, c)| {
                        if c.last_seen.elapsed() < CLIENT_TTL {
                            None
                        } else if let Some(pending) = c.prune_pending {
                            if pending.elapsed() >= Duration::from_secs(2) {
                                Some(*addr)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                for c in map.values_mut() {
                    if c.last_seen.elapsed() < CLIENT_TTL {
                        c.prune_pending = None;
                    } else if c.prune_pending.is_none() {
                        c.prune_pending = Some(now);
                    }
                }
                for addr in expired {
                    if let Some(mut client) = map.remove(&addr) {
                        if let Some(task) = client.video_task.take() {
                            task.abort();
                        }
                        if let (Some(ctl), Some(id)) = (displays.as_ref(), client.session.take()) {
                            ctl.detach(&id).await;
                        }
                        stats.client_stopped();
                    }
                }
            }
            _ = probe_tick.tick() => {
                let now = Instant::now();
                let expired_addrs: Vec<SocketAddr> = {
                    let map = clients.lock().await;
                    map.iter()
                        .filter(|(_, client)| client.probe_deadline.is_some_and(|deadline| now >= deadline))
                        .map(|(addr, _)| *addr)
                        .collect()
                };
                for addr in expired_addrs {
                    let mut map = clients.lock().await;
                    if let Some(client) = map.get_mut(&addr) {
                        let expired = client
                            .probe_deadline
                            .is_some_and(|deadline| now >= deadline);
                        if expired {
                            let before = client.pmtu.current_probe();
                            client.probe_deadline = None;
                            client.pmtu.on_timeout();
                            if client.pmtu.current_probe() != before {
                                bump_probe_id(client);
                            }
                            if client.pmtu.current_probe().is_some() {
                                send_probe(&sock, addr, client, limits, &keys).await;
                            }
                            if client.pmtu.is_complete() {
                                enable_udp_video(
                                    &sock,
                                    addr,
                                    client,
                                    limits,
                                    displays.as_ref(),
                                    idr_tx.as_ref(),
                                    &keys,
                                )
                                .await;
                            }
                        }
                    }
                }
            }
            pkt = video_rx.recv(), if displays.is_none() => {
                let pkt = match pkt {
                    Ok(p) => p,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        debug!("UDP hub lagged {n} packets; requesting IDR");
                        request_idr_sender(idr_tx.as_ref());
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                let targets: Vec<(SocketAddr, usize, [u8; 16])> = {
                    let map = clients.lock().await;
                    map.iter()
                        .filter(|(_, c)| c.pmtu.is_complete() && c.key_id.is_some())
                        .filter_map(|(addr, c)| {
                            c.key_id
                                .map(|key_id| (*addr, c.pmtu.video_payload(), key_id))
                        })
                        .collect()
                };
                if targets.is_empty() {
                    continue;
                }
                seq = seq.wrapping_add(1);
                let sent_ns = now_unix_ns();
                for (addr, payload, key_id) in targets {
                    let frames = fragment_video(seq, &pkt, sent_ns, payload);
                    let sealed = seal_frames(&keys, &key_id, &frames);
                    if sealed.len() != frames.len() {
                        request_idr_sender(idr_tx.as_ref());
                        continue;
                    }
                    let result = send_au_fragments(&sock, &sealed, addr, limits).await;
                    if result.request_idr() {
                        request_idr_sender(idr_tx.as_ref());
                    }
                }
            }
        }
    }
    let map = clients.lock().await;
    let bye = encode_bye(None);
    for (addr, client) in map.iter() {
        let Some(key_id) = client.key_id else {
            continue;
        };
        let Some(pkt) = keys.seal_host(&key_id, &bye) else {
            continue;
        };
        let _ = sock.send_to(&pkt, addr).await;
    }
}

#[allow(missing_debug_implementations)]
struct IncomingCtx<'a> {
    keys: &'a super::udp_crypto::UdpKeyRing,
    sock: &'a std::sync::Arc<UdpSocket>,
    clients: &'a tokio::sync::Mutex<HashMap<SocketAddr, UdpClient>>,
    idr_tx: Option<&'a tokio::sync::mpsc::Sender<()>>,
    stats: &'a super::Stats,
    limits: UdpLimits,
    displays: Option<&'a super::DisplayCtl>,
}

struct UdpForward {
    sock: std::sync::Arc<UdpSocket>,
    addr: SocketAddr,
    payload: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    limits: UdpLimits,
    displays: Option<super::DisplayCtl>,
    session: Option<String>,
    idr_tx: Option<tokio::sync::mpsc::Sender<()>>,
    keys: super::udp_crypto::UdpKeyRing,
    key_id: [u8; 16],
}

async fn forward_udp_video(fwd: UdpForward, mut video_rx: broadcast::Receiver<H264Packet>) {
    let mut seq: u16 = 0;
    loop {
        let pkt = match video_rx.recv().await {
            Ok(p) => p,
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        };
        let chunk = fwd.payload.load(std::sync::atomic::Ordering::Relaxed);
        if chunk == 0 {
            continue;
        }
        seq = seq.wrapping_add(1);
        let frames = fragment_video(seq, &pkt, now_unix_ns(), chunk);
        let sealed = seal_frames(&fwd.keys, &fwd.key_id, &frames);
        if sealed.len() != frames.len() {
            request_client_idr(
                fwd.displays.as_ref(),
                fwd.session.as_deref(),
                fwd.idr_tx.as_ref(),
            )
            .await;
            continue;
        }
        let result = send_au_fragments(&fwd.sock, &sealed, fwd.addr, fwd.limits).await;
        if result.request_idr() {
            request_client_idr(
                fwd.displays.as_ref(),
                fwd.session.as_deref(),
                fwd.idr_tx.as_ref(),
            )
            .await;
        }
    }
}

async fn handle_incoming(buf: &[u8], addr: SocketAddr, ctx: &IncomingCtx<'_>) {
    if buf.len() >= 4 && buf[0..4] == MAGIC[..] {
        if buf.len() >= 5 && buf[4] == TYPE_HELLO {
            warn!(%addr, "UDP cleartext hello rejected");
            ctx.stats.note_auth_failure();
        }
        return;
    }
    let Some(key_id) = super::udp_crypto::datagram_key_id(buf) else {
        return;
    };
    let Some(nonce) = super::udp_crypto::datagram_nonce(buf) else {
        return;
    };
    let Some(key) = ctx.keys.lookup(&key_id) else {
        warn!(%addr, "UDP datagram rejected");
        ctx.stats.note_auth_failure();
        return;
    };
    let Some(plain) = super::udp_crypto::open(&key.secret, buf) else {
        warn!(%addr, "UDP datagram rejected");
        ctx.stats.note_auth_failure();
        return;
    };
    if !ctx.keys.accept_client_nonce(&key_id, &nonce) {
        return;
    }
    match parse_packet(&plain) {
        Some(Packet::Hello {
            token: got,
            session,
        }) => {
            let bound = ctx.keys.bound_session(&key_id).flatten();
            let decision = super::udp_crypto::decide_hello(
                &got,
                session.as_deref(),
                &key_id,
                bound.as_deref(),
            );
            let super::udp_crypto::HelloDecision::Accept { session } = decision else {
                warn!(%addr, "UDP hello rejected");
                ctx.stats.note_auth_failure();
                return;
            };
            let mut map = ctx.clients.lock().await;
            let joining = !map.contains_key(&addr);
            let client = map.entry(addr).or_insert_with(|| new_client(ctx.limits));
            client.last_seen = Instant::now();
            client.prune_pending = None;
            if joining {
                client.key_id = Some(key_id);
                ctx.stats.client_started();
                info!(
                    %addr,
                    start = client.pmtu.confirmed(),
                    payload = client.pmtu.video_payload(),
                    "UDP client joined"
                );
                if let Some(ctl) = ctx.displays {
                    match ctl.attach(session.clone(), None).await {
                        Ok(att) => {
                            client.session = Some(att.info.id.clone());
                            let fwd = UdpForward {
                                sock: ctx.sock.clone(),
                                addr,
                                payload: client.payload.clone(),
                                limits: ctx.limits,
                                displays: ctx.displays.cloned(),
                                session: client.session.clone(),
                                idr_tx: ctx.idr_tx.cloned(),
                                keys: ctx.keys.clone(),
                                key_id,
                            };
                            client.video_task = Some(tokio::spawn(async move {
                                forward_udp_video(fwd, att.video).await;
                            }));
                        }
                        Err(e) => warn!(%addr, "UDP attach failed: {e}"),
                    }
                }
            }
            let send_id = client.key_id.unwrap_or(key_id);
            if let Some(ack) = ctx.keys.seal_host(&send_id, &encode_hello_ack()) {
                let _ = send_datagram(ctx.sock, &ack, addr, ctx.limits).await;
            }
            if joining {
                if client.pmtu.current_probe().is_some() {
                    send_probe(ctx.sock, addr, client, ctx.limits, ctx.keys).await;
                }
                if client.pmtu.is_complete() {
                    enable_udp_video(
                        ctx.sock,
                        addr,
                        client,
                        ctx.limits,
                        ctx.displays,
                        ctx.idr_tx,
                        ctx.keys,
                    )
                    .await;
                }
            }
        }
        Some(Packet::Bye(_)) => {
            let mut map = ctx.clients.lock().await;
            if let Some(mut client) = map.remove(&addr) {
                if let Some(task) = client.video_task.take() {
                    task.abort();
                }
                let id = client.session.take();
                if let (Some(ctl), Some(id)) = (ctx.displays, id) {
                    ctl.detach(&id).await;
                    ctl.release(&id).await;
                }
                ctx.stats.client_stopped();
                info!(%addr, "UDP client bye");
            }
        }
        Some(Packet::Ping(t0)) => {
            let key_id = {
                let mut map = ctx.clients.lock().await;
                let Some(client) = map.get_mut(&addr) else {
                    return;
                };
                client.last_seen = Instant::now();
                client.prune_pending = None;
                client.key_id
            };
            let Some(key_id) = key_id else {
                return;
            };
            let Some(pong) = ctx.keys.seal_host(&key_id, &encode_pong(t0, now_unix_ns())) else {
                return;
            };
            let _ = send_datagram(ctx.sock, &pong, addr, ctx.limits).await;
        }
        Some(Packet::Idr) => {
            let session = {
                let mut map = ctx.clients.lock().await;
                let Some(client) = map.get_mut(&addr) else {
                    return;
                };
                client.last_seen = Instant::now();
                client.session.clone()
            };
            request_client_idr(ctx.displays, session.as_deref(), ctx.idr_tx).await;
        }
        Some(Packet::ProbeAck { id, recv }) => {
            let mut map = ctx.clients.lock().await;
            let Some(client) = map.get_mut(&addr) else {
                return;
            };
            client.last_seen = Instant::now();
            client.prune_pending = None;
            let recv = recv as usize;
            match apply_probe_ack(client, id, recv) {
                AckEffect::Ignored => {}
                AckEffect::Completed | AckEffect::Continue => {
                    debug!(
                        %addr,
                        recv,
                        confirmed = client.pmtu.confirmed(),
                        "UDP PMTU ack"
                    );
                    if client.pmtu.current_probe().is_some() {
                        send_probe(ctx.sock, addr, client, ctx.limits, ctx.keys).await;
                    }
                    if client.pmtu.is_complete() {
                        enable_udp_video(
                            ctx.sock,
                            addr,
                            client,
                            ctx.limits,
                            ctx.displays,
                            ctx.idr_tx,
                            ctx.keys,
                        )
                        .await;
                    }
                }
            }
        }
        Some(
            Packet::HelloAck
            | Packet::Pong { .. }
            | Packet::Video(_)
            | Packet::Probe { .. }
            | Packet::Pmtu(_),
        ) => {}
        None => {}
    }
}

fn request_idr_sender(idr_tx: Option<&tokio::sync::mpsc::Sender<()>>) {
    if let Some(tx) = idr_tx {
        let _ = tx.try_send(());
    }
}

async fn request_client_idr(
    displays: Option<&super::DisplayCtl>,
    session: Option<&str>,
    idr_tx: Option<&tokio::sync::mpsc::Sender<()>>,
) {
    if let (Some(ctl), Some(id)) = (displays, session.filter(|s| !s.is_empty())) {
        info!(session = %id, "UDP IDR for session");
        ctl.idr(id).await;
        return;
    }
    info!(session = ?session, "UDP IDR via global encoder");
    request_idr_sender(idr_tx);
}

async fn enable_udp_video(
    sock: &UdpSocket,
    addr: SocketAddr,
    client: &mut UdpClient,
    limits: UdpLimits,
    displays: Option<&super::DisplayCtl>,
    idr_tx: Option<&tokio::sync::mpsc::Sender<()>>,
    keys: &super::udp_crypto::UdpKeyRing,
) {
    let first = !client.announced;
    announce_pmtu(sock, addr, client, limits, keys).await;
    if first && client.announced {
        request_client_idr(displays, client.session.as_deref(), idr_tx).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn paired_udp_is_rejected_without_attach_or_client_state() {
        let registry = crate::pairing::PairingRegistry::load_from(None).unwrap();
        let request = registry.request_pairing("tablet", "192.0.2.1").unwrap();
        let client = registry.approve(&request.request_id).unwrap().unwrap();
        let credential = registry.claim(&request.request_id, "192.0.2.1").unwrap();
        assert!(registry.bind_session(&client.client_id, "owned-session"));
        let sock = std::sync::Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let clients = tokio::sync::Mutex::new(HashMap::new());
        let stats = super::super::Stats::default();
        let (commands, mut commands_rx) = tokio::sync::mpsc::channel(8);
        let displays = super::super::DisplayCtl::new(commands);
        let keys = crate::udp_crypto::UdpKeyRing::new();
        let ctx = IncomingCtx {
            keys: &keys,
            sock: &sock,
            clients: &clients,
            idr_tx: None,
            stats: &stats,
            limits: UdpLimits::default(),
            displays: Some(&displays),
        };
        for session in [None, Some("owned-session"), Some("other-session")] {
            handle_incoming(
                &encode_hello_session(&credential, session),
                peer.local_addr().unwrap(),
                &ctx,
            )
            .await;
            assert!(clients.lock().await.is_empty());
            assert!(commands_rx.try_recv().is_err());
        }
        handle_incoming(&encode_hello("shared"), peer.local_addr().unwrap(), &ctx).await;
        assert!(clients.lock().await.is_empty());
        assert!(commands_rx.try_recv().is_err());
        assert!(registry.revoke(&client.client_id).unwrap());
    }

    #[tokio::test]
    async fn udp_bye_uses_attached_session_not_packet_session() {
        let sock = std::sync::Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr = peer.local_addr().unwrap();
        let mut client = new_client(UdpLimits::default());
        client.session = Some("attached-session".into());
        let clients = tokio::sync::Mutex::new(HashMap::from([(addr, client)]));
        let stats = super::super::Stats::default();
        stats.client_started();
        let (commands, mut commands_rx) = tokio::sync::mpsc::channel(8);
        let displays = super::super::DisplayCtl::new(commands);
        let keys = crate::udp_crypto::UdpKeyRing::new();
        let key = crate::udp_crypto::UdpSealKey {
            id: [3; 16],
            secret: [4; 32],
        };
        keys.insert(
            key.clone(),
            Some("attached-session".into()),
            std::time::Instant::now() + std::time::Duration::from_secs(60),
        );
        let ctx = IncomingCtx {
            keys: &keys,
            sock: &sock,
            clients: &clients,
            idr_tx: None,
            stats: &stats,
            limits: UdpLimits::default(),
            displays: Some(&displays),
        };
        handle_incoming(&encode_bye(Some("other-session")), addr, &ctx).await;
        assert!(commands_rx.try_recv().is_err());
        assert!(clients.lock().await.contains_key(&addr));
        let bye = crate::udp_crypto::seal(
            &key,
            &crate::udp_crypto::nonce(crate::udp_crypto::DIR_CLIENT, 1),
            &encode_bye(Some("other-session")),
        );
        handle_incoming(&bye, addr, &ctx).await;
        assert!(
            matches!(commands_rx.try_recv().unwrap(), crate::DisplayCommand::Detach { id } if id == "attached-session")
        );
        assert!(
            matches!(commands_rx.try_recv().unwrap(), crate::DisplayCommand::Release { id } if id == "attached-session")
        );
        assert!(clients.lock().await.is_empty());
    }

    #[tokio::test]
    async fn sealed_issued_key_hello_joins_and_cleartext_token_does_not() {
        let keys = crate::udp_crypto::UdpKeyRing::new();
        let key = crate::udp_crypto::UdpSealKey {
            id: [0x22; 16],
            secret: [0x11; 32],
        };
        keys.insert(
            key.clone(),
            Some("sess".into()),
            std::time::Instant::now() + std::time::Duration::from_secs(60),
        );
        let sock = std::sync::Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr = peer.local_addr().unwrap();
        let clients = tokio::sync::Mutex::new(HashMap::new());
        let stats = super::super::Stats::default();
        let ctx = IncomingCtx {
            keys: &keys,
            sock: &sock,
            clients: &clients,
            idr_tx: None,
            stats: &stats,
            limits: UdpLimits::default(),
            displays: None,
        };
        let shared = crate::udp_crypto::seal(
            &key,
            &crate::udp_crypto::nonce(crate::udp_crypto::DIR_CLIENT, 1),
            &encode_hello_session("shared", Some("sess")),
        );
        handle_incoming(&shared, addr, &ctx).await;
        assert!(clients.lock().await.is_empty());

        let wrong_session = crate::udp_crypto::seal(
            &key,
            &crate::udp_crypto::nonce(crate::udp_crypto::DIR_CLIENT, 2),
            &encode_hello_session(&crate::udp_crypto::key_id_hex(&key.id), Some("other")),
        );
        handle_incoming(&wrong_session, addr, &ctx).await;
        assert!(clients.lock().await.is_empty());

        let hello = crate::udp_crypto::seal(
            &key,
            &crate::udp_crypto::nonce(crate::udp_crypto::DIR_CLIENT, 3),
            &encode_hello_session(&crate::udp_crypto::key_id_hex(&key.id), Some("sess")),
        );
        handle_incoming(&hello, addr, &ctx).await;
        assert_eq!(clients.lock().await.len(), 1);
        let mut buf = vec![0u8; 256];
        let (n, _) =
            tokio::time::timeout(std::time::Duration::from_secs(1), peer.recv_from(&mut buf))
                .await
                .unwrap()
                .unwrap();
        let opened = crate::udp_crypto::open(&key.secret, &buf[..n]).expect("sealed ack");
        assert_eq!(parse_packet(&opened), Some(Packet::HelloAck));
        assert!(buf[..n].starts_with(crate::udp_crypto::SEAL_MAGIC));
    }

    #[test]
    fn video_roundtrip() {
        let pkt = H264Packet {
            bytes: vec![0, 0, 0, 1, 0x65, 1, 2, 3],
            is_keyframe: true,
            pts_ns: 42,
        };
        let frames = fragment_video(7, &pkt, 99, MAX_PAYLOAD);
        assert_eq!(frames.len(), 1);
        match parse_packet(&frames[0]) {
            Some(Packet::Video(v)) => {
                assert_eq!(v.seq, 7);
                assert!(v.is_keyframe);
                assert_eq!(v.pts_ns, 42);
                assert_eq!(v.sent_ns, 99);
                assert_eq!(v.payload, pkt.bytes);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn fragments_large_au() {
        let pkt = H264Packet {
            bytes: vec![7u8; MAX_PAYLOAD + 50],
            is_keyframe: false,
            pts_ns: 1,
        };
        let frames = fragment_video(1, &pkt, 2, MAX_PAYLOAD);
        assert_eq!(frames.len(), 2);
        let a = match parse_packet(&frames[0]) {
            Some(Packet::Video(v)) => v,
            _ => panic!(),
        };
        let b = match parse_packet(&frames[1]) {
            Some(Packet::Video(v)) => v,
            _ => panic!(),
        };
        assert_eq!(a.frags, 2);
        assert_eq!(b.frag, 1);
        assert_eq!(a.payload.len() + b.payload.len(), pkt.bytes.len());
    }

    #[test]
    fn hello_and_control_roundtrip() {
        let hello = parse_packet(&encode_hello("tok"));
        assert_eq!(
            hello,
            Some(Packet::Hello {
                token: "tok".into(),
                session: None
            })
        );
        assert_eq!(
            parse_packet(&encode_hello_session("tok", Some("ab"))),
            Some(Packet::Hello {
                token: "tok".into(),
                session: Some("ab".into())
            })
        );
        assert_eq!(
            parse_packet(&encode_bye(Some("ab"))),
            Some(Packet::Bye(Some("ab".into())))
        );
        assert_eq!(parse_packet(&encode_hello_ack()), Some(Packet::HelloAck));
        assert_eq!(parse_packet(&encode_idr()), Some(Packet::Idr));
        match parse_packet(&encode_pong(1, 2)) {
            Some(Packet::Pong { t0_ns, host_ns }) => {
                assert_eq!(t0_ns, 1);
                assert_eq!(host_ns, 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn default_port_is_signaling_plus_one() {
        assert_eq!(default_udp_port(8788), 8789);
    }

    #[test]
    fn probe_roundtrip() {
        let pkt = encode_probe(9, 900);
        assert_eq!(pkt.len(), 900);
        assert_eq!(parse_packet(&pkt), Some(Packet::Probe { id: 9 }));
        assert_eq!(
            parse_packet(&encode_probe_ack(9, 900)),
            Some(Packet::ProbeAck { id: 9, recv: 900 })
        );
        assert_eq!(parse_packet(&encode_pmtu(900)), Some(Packet::Pmtu(900)));
    }

    fn drive_search(mut search: PmtuSearch, path_mtu: usize) -> usize {
        let mut steps = 0;
        while !search.is_complete() {
            steps += 1;
            assert!(steps < 80, "search did not converge");
            let probe = search.current_probe().expect("probe while incomplete");
            if probe <= path_mtu {
                search.on_ack(probe);
            } else {
                search.on_timeout();
            }
        }
        search.confirmed()
    }

    #[test]
    fn pmtu_reaches_ceiling_when_path_is_open() {
        let found = drive_search(PmtuSearch::from_min(DEFAULT_MAX_DATAGRAM), 10_000);
        assert_eq!(found, DEFAULT_MAX_DATAGRAM);
    }

    #[test]
    fn pmtu_finds_artificial_limits() {
        for limit in [700usize, 900, 1200, 1400] {
            let found = drive_search(PmtuSearch::from_min(DEFAULT_MAX_DATAGRAM), limit);
            assert!(
                found <= limit,
                "limit {limit}: found {found} above path MTU"
            );
            assert!(
                limit - found <= PROBE_STEP,
                "limit {limit}: found {found}, farther than {PROBE_STEP}"
            );
        }
    }

    #[test]
    fn zero_loss_pct_never_drops() {
        for _ in 0..200 {
            assert!(!should_lose(0));
        }
    }

    #[test]
    fn failed_datagram_still_sends_remaining_fragments() {
        let script = [
            SendOutcome::Sent,
            SendOutcome::Failed,
            SendOutcome::Sent,
            SendOutcome::Sent,
        ];
        let result = dispatch_au_sends(script.len(), |i| script[i]);
        assert_eq!(result.attempted, 4);
        assert_eq!(result.sent, 3);
        assert_eq!(result.failed, 1);
        assert!(!result.too_big);
        assert!(result.request_idr());
    }

    #[test]
    fn too_big_stops_remaining_fragments() {
        let script = [SendOutcome::Sent, SendOutcome::TooBig, SendOutcome::Sent];
        let result = dispatch_au_sends(script.len(), |i| script[i]);
        assert_eq!(result.attempted, 2);
        assert_eq!(result.sent, 1);
        assert_eq!(result.failed, 0);
        assert!(result.too_big);
        assert!(result.request_idr());
    }

    #[test]
    fn one_lost_fragment_does_not_drop_a_twenty_fragment_au() {
        let n = 20;
        let lost = 3;
        let result = dispatch_au_sends(n, |i| {
            if i == lost {
                SendOutcome::Failed
            } else {
                SendOutcome::Sent
            }
        });
        assert_eq!(result.attempted, n);
        assert_eq!(result.sent, n - 1);
        assert_eq!(result.failed, 1);
        assert!(result.request_idr());
    }

    #[test]
    fn intact_au_does_not_request_idr() {
        let result = dispatch_au_sends(8, |_| SendOutcome::Sent);
        assert_eq!(result.attempted, 8);
        assert_eq!(result.sent, 8);
        assert_eq!(result.failed, 0);
        assert!(!result.request_idr());
    }

    #[test]
    fn complete_search_ignores_further_acks() {
        let mut search = PmtuSearch::new(BASE_DATAGRAM);
        assert!(search.is_complete());
        search.on_ack(DEFAULT_MAX_DATAGRAM);
        assert_eq!(search.confirmed(), BASE_DATAGRAM);
    }

    #[test]
    fn mismatched_or_late_probe_ack_is_ignored() {
        let mut client = new_client(UdpLimits::default());
        client.probe_id = 3;
        let before = client.pmtu.confirmed();
        assert_eq!(
            apply_probe_ack(&mut client, 2, DEFAULT_MAX_DATAGRAM),
            AckEffect::Ignored
        );
        assert_eq!(client.pmtu.confirmed(), before);

        client.pmtu = PmtuSearch::new(BASE_DATAGRAM);
        client.probe_id = 5;
        assert!(client.pmtu.is_complete());
        assert_eq!(
            apply_probe_ack(&mut client, 5, DEFAULT_MAX_DATAGRAM),
            AckEffect::Ignored
        );
        assert_eq!(client.pmtu.confirmed(), BASE_DATAGRAM);
    }

    #[test]
    fn production_search_starts_at_base() {
        let search = PmtuSearch::new(DEFAULT_MAX_DATAGRAM);
        assert_eq!(search.confirmed(), BASE_DATAGRAM);
        assert_eq!(
            search.video_payload(),
            BASE_DATAGRAM - VIDEO_HEADER_LEN - crate::udp_crypto::SEAL_OVERHEAD
        );
        assert!(!search.is_complete());
    }

    #[test]
    fn default_max_is_ipv4_ethernet_udp_payload() {
        assert_eq!(DEFAULT_MAX_DATAGRAM, 1500 - 20 - 8);
    }

    #[test]
    fn truncated_ack_rejects_probe_size() {
        let mut search = PmtuSearch::from_min(DEFAULT_MAX_DATAGRAM);
        let probe = search.current_probe().expect("armed");
        let floor = search.confirmed();
        search.on_ack(probe / 2);
        assert_ne!(search.current_probe(), Some(probe));
        assert_eq!(search.confirmed(), floor);
    }

    #[test]
    fn unsendable_probe_narrows_search() {
        let mut search = PmtuSearch::from_min(DEFAULT_MAX_DATAGRAM);
        let probe = search.current_probe().expect("armed");
        search.on_unsendable();
        assert!(search.confirmed() < probe);
        assert!(
            search.is_complete() || search.current_probe().is_some_and(|next| next < probe),
            "next probe should be smaller than the rejected size"
        );
    }

    #[tokio::test]
    async fn loopback_discovers_drop_above() {
        let drop_above = 900usize;
        let server = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client_addr = client.local_addr().unwrap();

        let client_task = tokio::spawn(async move {
            let mut buf = vec![0u8; 2048];
            let mut confirmed = None;
            while confirmed.is_none() {
                let (n, addr) = client.recv_from(&mut buf).await.unwrap();
                match parse_packet(&buf[..n]) {
                    Some(Packet::Probe { id }) => {
                        if n <= drop_above {
                            let _ = client.send_to(&encode_probe_ack(id, n as u16), addr).await;
                        }
                    }
                    Some(Packet::Pmtu(size)) => confirmed = Some(size),
                    _ => {}
                }
            }
            confirmed
        });

        let mut search = PmtuSearch::from_min(DEFAULT_MAX_DATAGRAM);
        let mut id = 0u16;
        let mut buf = vec![0u8; 2048];
        let mut steps = 0;
        while !search.is_complete() {
            steps += 1;
            assert!(steps < 80, "loopback search did not converge");
            let size = search.current_probe().unwrap();
            id = id.wrapping_add(1);
            let pkt = encode_probe(id, size);
            if pkt.len() > drop_above {
                search.on_timeout();
                continue;
            }
            server.send_to(&pkt, client_addr).await.unwrap();
            match tokio::time::timeout(Duration::from_millis(200), server.recv_from(&mut buf)).await
            {
                Ok(Ok((n, _))) => match parse_packet(&buf[..n]) {
                    Some(Packet::ProbeAck { id: ack_id, recv }) if ack_id == id => {
                        search.on_ack(recv as usize)
                    }
                    _ => search.on_timeout(),
                },
                _ => search.on_timeout(),
            }
        }
        server
            .send_to(&encode_pmtu(search.confirmed() as u16), client_addr)
            .await
            .unwrap();
        let client_seen = tokio::time::timeout(Duration::from_secs(1), client_task)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(client_seen, Some(search.confirmed() as u16));
        assert!(search.confirmed() <= drop_above);
        assert!(drop_above - search.confirmed() <= PROBE_STEP);
    }
}
