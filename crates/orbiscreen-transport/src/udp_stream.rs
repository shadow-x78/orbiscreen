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
        self.low.saturating_sub(VIDEO_HEADER_LEN).max(1)
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
    let mut out = Vec::with_capacity(5 + token.len());
    out.extend_from_slice(MAGIC);
    out.push(TYPE_HELLO);
    out.extend_from_slice(token.as_bytes());
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
    Hello(String),
    HelloAck,
    Ping(u64),
    Pong { t0_ns: u64, host_ns: u64 },
    Idr,
    Probe { id: u16 },
    ProbeAck { id: u16, recv: u16 },
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
                is_keyframe: buf[5] != 0,
                seq: u16::from_le_bytes([buf[6], buf[7]]),
                frag: u16::from_le_bytes([buf[8], buf[9]]),
                frags: u16::from_le_bytes([buf[10], buf[11]]),
                pts_ns: u64::from_le_bytes(buf[12..20].try_into().ok()?),
                sent_ns: u64::from_le_bytes(buf[20..28].try_into().ok()?),
                payload: buf[28..].to_vec(),
            }))
        }
        TYPE_HELLO => Some(Packet::Hello(
            String::from_utf8_lossy(&buf[5..]).trim().to_string(),
        )),
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
    let chunk = max_payload.max(1);
    let chunks: Vec<&[u8]> = pkt.bytes.chunks(chunk).collect();
    let frags = chunks.len() as u16;
    chunks
        .into_iter()
        .enumerate()
        .map(|(i, chunk)| {
            encode_video(
                seq,
                i as u16,
                frags,
                pkt.is_keyframe,
                pkt.pts_ns,
                sent_ns,
                chunk,
            )
        })
        .collect()
}

struct UdpClient {
    last_seen: Instant,
    pmtu: PmtuSearch,
    probe_id: u16,
    probe_deadline: Option<Instant>,
    announced: bool,
}

fn new_client(limits: UdpLimits) -> UdpClient {
    let pmtu = if limits.drop_above.is_some() {
        PmtuSearch::from_min(limits.max_datagram)
    } else {
        PmtuSearch::new(limits.max_datagram)
    };
    UdpClient {
        last_seen: Instant::now(),
        pmtu,
        probe_id: 0,
        probe_deadline: None,
        announced: false,
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

async fn send_probe(sock: &UdpSocket, addr: SocketAddr, client: &mut UdpClient, limits: UdpLimits) {
    loop {
        let Some(size) = client.pmtu.current_probe() else {
            return;
        };
        if client.probe_id == 0 {
            client.probe_id = 1;
        }
        let pkt = encode_probe(client.probe_id, size);
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
) {
    if client.announced || !client.pmtu.is_complete() {
        return;
    }
    client.announced = true;
    let datagram = client.pmtu.confirmed();
    let payload = client.pmtu.video_payload();
    info!(%addr, datagram, payload, "UDP PMTU confirmed");
    let _ = send_datagram(sock, &encode_pmtu(datagram as u16), addr, limits).await;
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

pub async fn run_udp_hub(
    port: u16,
    token: String,
    mut video_rx: broadcast::Receiver<H264Packet>,
    idr_tx: Option<tokio::sync::mpsc::Sender<()>>,
    stats: std::sync::Arc<super::Stats>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
    limits: UdpLimits,
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
    let recv_token = token.clone();
    let recv_idr = idr_tx.clone();
    let recv_stats = stats.clone();
    let recv_limits = limits;
    let mut recv_shutdown = shutdown.clone();
    tokio::spawn(async move {
        let mut buf = vec![0u8; 4096];
        loop {
            tokio::select! {
                _ = recv_shutdown.changed() => break,
                res = recv_sock.recv_from(&mut buf) => {
                    let Ok((n, addr)) = res else { continue };
                    let ctx = IncomingCtx {
                        token: &recv_token,
                        sock: &recv_sock,
                        clients: &recv_clients,
                        idr_tx: recv_idr.as_ref(),
                        stats: &recv_stats,
                        limits: recv_limits,
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
                let before = map.len();
                map.retain(|_, c| c.last_seen.elapsed() < CLIENT_TTL);
                let dropped = before.saturating_sub(map.len());
                for _ in 0..dropped {
                    stats.client_stopped();
                }
            }
            _ = probe_tick.tick() => {
                let now = Instant::now();
                let mut map = clients.lock().await;
                for (addr, client) in map.iter_mut() {
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
                            send_probe(&sock, *addr, client, limits).await;
                        }
                        if client.pmtu.is_complete() {
                            announce_pmtu(&sock, *addr, client, limits).await;
                            request_idr_sender(idr_tx.as_ref());
                        }
                    }
                }
            }
            pkt = video_rx.recv() => {
                let pkt = match pkt {
                    Ok(p) => p,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                let targets: Vec<(SocketAddr, usize)> = {
                    let map = clients.lock().await;
                    map.iter()
                        .filter(|(_, c)| c.pmtu.is_complete())
                        .map(|(addr, c)| (*addr, c.pmtu.video_payload()))
                        .collect()
                };
                if targets.is_empty() {
                    continue;
                }
                seq = seq.wrapping_add(1);
                let sent_ns = now_unix_ns();
                for (addr, payload) in targets {
                    let frames = fragment_video(seq, &pkt, sent_ns, payload);
                    for frame in &frames {
                        if send_datagram(&sock, frame, addr, limits).await != SendOutcome::Sent {
                            break;
                        }
                    }
                }
            }
        }
    }
}

#[allow(missing_debug_implementations)]
struct IncomingCtx<'a> {
    token: &'a str,
    sock: &'a UdpSocket,
    clients: &'a tokio::sync::Mutex<HashMap<SocketAddr, UdpClient>>,
    idr_tx: Option<&'a tokio::sync::mpsc::Sender<()>>,
    stats: &'a super::Stats,
    limits: UdpLimits,
}

async fn handle_incoming(buf: &[u8], addr: SocketAddr, ctx: &IncomingCtx<'_>) {
    match parse_packet(buf) {
        Some(Packet::Hello(got)) => {
            if !super::token_eq(&got, ctx.token) {
                warn!(%addr, "UDP hello rejected");
                ctx.stats.note_auth_failure();
                return;
            }
            let mut map = ctx.clients.lock().await;
            let joining = !map.contains_key(&addr);
            let client = map.entry(addr).or_insert_with(|| new_client(ctx.limits));
            client.last_seen = Instant::now();
            if joining {
                ctx.stats.client_started();
                info!(
                    %addr,
                    start = client.pmtu.confirmed(),
                    payload = client.pmtu.video_payload(),
                    "UDP client joined"
                );
                request_idr_sender(ctx.idr_tx);
            }
            let _ = send_datagram(ctx.sock, &encode_hello_ack(), addr, ctx.limits).await;
            if joining {
                if client.pmtu.current_probe().is_some() {
                    send_probe(ctx.sock, addr, client, ctx.limits).await;
                }
                if client.pmtu.is_complete() {
                    announce_pmtu(ctx.sock, addr, client, ctx.limits).await;
                }
            }
        }
        Some(Packet::Ping(t0)) => {
            touch_client(ctx.clients, addr).await;
            let _ =
                send_datagram(ctx.sock, &encode_pong(t0, now_unix_ns()), addr, ctx.limits).await;
        }
        Some(Packet::Idr) => {
            if touch_client(ctx.clients, addr).await {
                request_idr_sender(ctx.idr_tx);
            }
        }
        Some(Packet::ProbeAck { id, recv }) => {
            let mut map = ctx.clients.lock().await;
            let Some(client) = map.get_mut(&addr) else {
                return;
            };
            client.last_seen = Instant::now();
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
                        send_probe(ctx.sock, addr, client, ctx.limits).await;
                    }
                    if client.pmtu.is_complete() {
                        announce_pmtu(ctx.sock, addr, client, ctx.limits).await;
                        request_idr_sender(ctx.idr_tx);
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

async fn touch_client(
    clients: &tokio::sync::Mutex<HashMap<SocketAddr, UdpClient>>,
    addr: SocketAddr,
) -> bool {
    let mut map = clients.lock().await;
    if let Some(c) = map.get_mut(&addr) {
        c.last_seen = Instant::now();
        true
    } else {
        false
    }
}

fn request_idr_sender(idr_tx: Option<&tokio::sync::mpsc::Sender<()>>) {
    if let Some(tx) = idr_tx {
        let _ = tx.try_send(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(hello, Some(Packet::Hello("tok".into())));
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
        assert_eq!(search.video_payload(), BASE_DATAGRAM - VIDEO_HEADER_LEN);
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
