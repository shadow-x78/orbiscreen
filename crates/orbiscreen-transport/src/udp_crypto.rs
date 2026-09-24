// Orbiscreen - udp_crypto.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

//! AES-256-GCM wrapper for UDP datagrams.
//!
//! The clear header is `ORB2 || key_id || nonce`. The AEAD binds that header
//! as additional data, and the plaintext is an ordinary ORB1 packet.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};

pub const SEAL_MAGIC: &[u8; 4] = b"ORB2";
pub const KEY_ID_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 16;
pub const SEAL_HEADER_LEN: usize = 4 + KEY_ID_LEN + NONCE_LEN;
pub const SEAL_OVERHEAD: usize = SEAL_HEADER_LEN + TAG_LEN;
pub const DIR_CLIENT: u8 = 0;
pub const DIR_HOST: u8 = 1;
const REPLAY_WINDOW: u64 = 256;
pub const UDP_KEY_TTL: Duration = Duration::from_secs(600);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UdpSealKey {
    pub id: [u8; 16],
    pub secret: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MintKind {
    Anonymous,
    SharedToken,
    Paired { owns_session: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelloDecision {
    Accept { session: Option<String> },
    Reject,
}

pub fn nonce(direction: u8, counter: u64) -> [u8; 12] {
    let mut out = [0u8; NONCE_LEN];
    out[0] = direction;
    out[1..9].copy_from_slice(&counter.to_le_bytes());
    out
}

pub fn peer_counter(nonce: &[u8; 12], expected_direction: u8) -> Option<u64> {
    if nonce[0] != expected_direction || nonce[9] != 0 || nonce[10] != 0 || nonce[11] != 0 {
        return None;
    }
    let counter = u64::from_le_bytes(nonce[1..9].try_into().ok()?);
    (counter != 0).then_some(counter)
}

pub fn key_id_hex(id: &[u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(KEY_ID_LEN * 2);
    for byte in id {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

pub fn hello_token_matches(token: &str, key_id: &[u8; 16]) -> bool {
    let expected = key_id_hex(key_id);
    if token.len() != expected.len() {
        return false;
    }
    let mut diff = 0u8;
    for (got, want) in token.bytes().zip(expected.bytes()) {
        diff |= got.to_ascii_lowercase() ^ want;
    }
    diff == 0
}

pub fn seal(key: &UdpSealKey, nonce_bytes: &[u8; NONCE_LEN], plaintext: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(SEAL_HEADER_LEN + plaintext.len() + TAG_LEN);
    out.extend_from_slice(SEAL_MAGIC);
    out.extend_from_slice(&key.id);
    out.extend_from_slice(nonce_bytes);
    let header = out.clone();
    let mut body = plaintext.to_vec();
    let sealing = LessSafeKey::new(
        UnboundKey::new(&AES_256_GCM, &key.secret).expect("AES-256 key is 32 bytes"),
    );
    let nonce = Nonce::try_assume_unique_for_key(nonce_bytes).expect("nonce is 12 bytes");
    sealing
        .seal_in_place_append_tag(nonce, Aad::from(header), &mut body)
        .expect("AES-GCM seal");
    out.extend_from_slice(&body);
    out
}

pub fn datagram_key_id(datagram: &[u8]) -> Option<[u8; KEY_ID_LEN]> {
    if datagram.len() < SEAL_HEADER_LEN || datagram[0..4] != SEAL_MAGIC[..] {
        return None;
    }
    let mut id = [0u8; KEY_ID_LEN];
    id.copy_from_slice(&datagram[4..4 + KEY_ID_LEN]);
    Some(id)
}

pub fn datagram_nonce(datagram: &[u8]) -> Option<[u8; NONCE_LEN]> {
    if datagram.len() < SEAL_HEADER_LEN || datagram[0..4] != SEAL_MAGIC[..] {
        return None;
    }
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&datagram[SEAL_HEADER_LEN - NONCE_LEN..SEAL_HEADER_LEN]);
    Some(nonce)
}

pub fn open(secret: &[u8; 32], datagram: &[u8]) -> Option<Vec<u8>> {
    if datagram.len() < SEAL_OVERHEAD || datagram[0..4] != SEAL_MAGIC[..] {
        return None;
    }
    let nonce_bytes = &datagram[SEAL_HEADER_LEN - NONCE_LEN..SEAL_HEADER_LEN];
    let header = &datagram[..SEAL_HEADER_LEN];
    let mut body = datagram[SEAL_HEADER_LEN..].to_vec();
    let opening = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, secret).ok()?);
    let nonce = Nonce::try_assume_unique_for_key(nonce_bytes).ok()?;
    let plain = opening
        .open_in_place(nonce, Aad::from(header), &mut body)
        .ok()?;
    Some(plain.to_vec())
}

pub fn seal_to_len(
    key: &UdpSealKey,
    nonce_bytes: &[u8; NONCE_LEN],
    plaintext: &[u8],
    wire_len: usize,
) -> Option<Vec<u8>> {
    let plain_len = wire_len.checked_sub(SEAL_OVERHEAD)?;
    if plaintext.len() > plain_len {
        return None;
    }
    let mut padded = plaintext.to_vec();
    padded.resize(plain_len, 0);
    let wire = seal(key, nonce_bytes, &padded);
    (wire.len() == wire_len).then_some(wire)
}

pub fn decide_hello(
    token: &str,
    claimed_session: Option<&str>,
    key_id: &[u8; 16],
    bound_session: Option<&str>,
) -> HelloDecision {
    if !hello_token_matches(token, key_id) {
        return HelloDecision::Reject;
    }
    match bound_session {
        Some(bound) => {
            if claimed_session == Some(bound) {
                HelloDecision::Accept {
                    session: Some(bound.to_string()),
                }
            } else {
                HelloDecision::Reject
            }
        }
        None => HelloDecision::Accept {
            session: claimed_session.map(str::to_string),
        },
    }
}

pub fn may_mint_udp_key(kind: MintKind, session: &str) -> bool {
    if session.trim().is_empty() {
        return false;
    }
    match kind {
        MintKind::Anonymous => false,
        MintKind::SharedToken => true,
        MintKind::Paired { owns_session } => owns_session,
    }
}

#[derive(Debug, Clone)]
pub struct ReplayWindow {
    highest: u64,
    bits: [u64; 4],
}

impl Default for ReplayWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayWindow {
    pub fn new() -> Self {
        Self {
            highest: 0,
            bits: [0; 4],
        }
    }

    pub fn accept(&mut self, counter: u64) -> bool {
        if counter == 0 {
            return false;
        }
        if self.highest == 0 {
            self.highest = counter;
            self.bits = [0; 4];
            set_bit(&mut self.bits, 0);
            return true;
        }
        if counter > self.highest {
            let shift = counter - self.highest;
            if shift >= REPLAY_WINDOW {
                self.bits = [0; 4];
            } else {
                shift_bits(&mut self.bits, shift);
            }
            self.highest = counter;
            set_bit(&mut self.bits, 0);
            return true;
        }
        let age = self.highest - counter;
        if age >= REPLAY_WINDOW || bit_set(&self.bits, age) {
            return false;
        }
        set_bit(&mut self.bits, age);
        true
    }
}

fn set_bit(bits: &mut [u64; 4], age: u64) {
    let word = (age / 64) as usize;
    let bit = age % 64;
    bits[word] |= 1u64 << bit;
}

fn bit_set(bits: &[u64; 4], age: u64) -> bool {
    let word = (age / 64) as usize;
    let bit = age % 64;
    bits[word] & (1u64 << bit) != 0
}

fn shift_bits(bits: &mut [u64; 4], shift: u64) {
    let mut next = [0u64; 4];
    for age in 0..REPLAY_WINDOW {
        if bit_set(bits, age) {
            let moved = age + shift;
            if moved < REPLAY_WINDOW {
                set_bit(&mut next, moved);
            }
        }
    }
    *bits = next;
}

#[derive(Debug)]
struct IssuedKey {
    secret: [u8; 32],
    session: Option<String>,
    ttl: Duration,
    expires: Instant,
    replay: ReplayWindow,
    host_counter: u64,
}

#[derive(Debug, Clone)]
pub struct UdpKeyRing {
    inner: std::sync::Arc<std::sync::Mutex<HashMap<[u8; 16], IssuedKey>>>,
}

impl Default for UdpKeyRing {
    fn default() -> Self {
        Self::new()
    }
}

impl UdpKeyRing {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<[u8; 16], IssuedKey>> {
        self.inner.lock().unwrap_or_else(|err| err.into_inner())
    }

    pub fn issue(&self, session: Option<String>, ttl: Duration) -> UdpSealKey {
        let mut id = [0u8; KEY_ID_LEN];
        let mut secret = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rng(), &mut id);
        rand::RngCore::fill_bytes(&mut rand::rng(), &mut secret);
        let key = UdpSealKey { id, secret };
        self.insert(key.clone(), session, Instant::now() + ttl);
        key
    }

    pub fn insert(&self, key: UdpSealKey, session: Option<String>, expires: Instant) {
        let ttl = expires.saturating_duration_since(Instant::now());
        self.lock().insert(
            key.id,
            IssuedKey {
                secret: key.secret,
                session,
                ttl,
                expires,
                replay: ReplayWindow::new(),
                host_counter: 0,
            },
        );
    }

    fn touch(issued: &mut IssuedKey) {
        issued.expires = Instant::now() + issued.ttl;
    }

    fn live<'a>(
        map: &'a mut HashMap<[u8; 16], IssuedKey>,
        id: &[u8; 16],
    ) -> Option<&'a mut IssuedKey> {
        let expires = map.get(id)?.expires;
        if Instant::now() >= expires {
            return None;
        }
        map.get_mut(id)
    }

    pub fn lookup(&self, id: &[u8; 16]) -> Option<UdpSealKey> {
        let mut map = self.lock();
        let issued = Self::live(&mut map, id)?;
        Some(UdpSealKey {
            id: *id,
            secret: issued.secret,
        })
    }

    pub fn bound_session(&self, id: &[u8; 16]) -> Option<Option<String>> {
        let mut map = self.lock();
        Some(Self::live(&mut map, id)?.session.clone())
    }

    pub fn revoke_session(&self, session: &str) {
        self.lock()
            .retain(|_, issued| issued.session.as_deref() != Some(session));
    }

    pub fn accept_client_nonce(&self, id: &[u8; 16], nonce_bytes: &[u8; NONCE_LEN]) -> bool {
        let Some(counter) = peer_counter(nonce_bytes, DIR_CLIENT) else {
            return false;
        };
        let mut map = self.lock();
        let Some(issued) = Self::live(&mut map, id) else {
            return false;
        };
        if !issued.replay.accept(counter) {
            return false;
        }
        Self::touch(issued);
        true
    }

    pub fn seal_host(&self, id: &[u8; 16], plaintext: &[u8]) -> Option<Vec<u8>> {
        let (key, nonce_bytes) = self.next_host(id)?;
        Some(seal(&key, &nonce_bytes, plaintext))
    }

    pub fn seal_host_to_len(
        &self,
        id: &[u8; 16],
        plaintext: &[u8],
        wire_len: usize,
    ) -> Option<Vec<u8>> {
        let (key, nonce_bytes) = self.next_host(id)?;
        seal_to_len(&key, &nonce_bytes, plaintext, wire_len)
    }

    fn next_host(&self, id: &[u8; 16]) -> Option<(UdpSealKey, [u8; NONCE_LEN])> {
        let mut map = self.lock();
        let issued = Self::live(&mut map, id)?;
        Self::touch(issued);
        issued.host_counter = issued.host_counter.saturating_add(1);
        if issued.host_counter == 0 {
            return None;
        }
        let nonce_bytes = nonce(DIR_HOST, issued.host_counter);
        Some((
            UdpSealKey {
                id: *id,
                secret: issued.secret,
            },
            nonce_bytes,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VECTOR_PLAIN: &str =
        "4f524231023232323232323232323232323232323232323232323232323232323232323232";
    const VECTOR_WIRE: &str =
        "4f5242322222222222222222222222222222222200010000000000000000000022cb09a9c9492fce188e3803bea3912cc0a18914d5ba4a68886d06e62ac703d79a8bd5bb2d96ca3b5088ef074d8d79b1e0264142fe";

    fn decode_hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    fn vector_key() -> UdpSealKey {
        UdpSealKey {
            id: [0x22; 16],
            secret: [0x11; 32],
        }
    }

    #[test]
    fn matches_independent_aes_gcm_vector() {
        let key = vector_key();
        let plain = decode_hex(VECTOR_PLAIN);
        let wire = seal(&key, &nonce(DIR_CLIENT, 1), &plain);
        assert_eq!(wire, decode_hex(VECTOR_WIRE));
        assert_eq!(open(&key.secret, &wire).as_deref(), Some(plain.as_slice()));
    }

    #[test]
    fn tampered_ciphertext_wrong_key_and_cleartext_are_rejected() {
        let key = vector_key();
        let plain = decode_hex(VECTOR_PLAIN);
        let wire = seal(&key, &nonce(DIR_CLIENT, 1), &plain);
        assert_eq!(open(&key.secret, &wire).as_deref(), Some(plain.as_slice()));
        let mut flipped = wire.clone();
        let last = flipped.len() - 1;
        flipped[last] ^= 0x01;
        assert!(open(&key.secret, &flipped).is_none());
        let mut header = wire.clone();
        header[4] ^= 0x01;
        assert!(open(&key.secret, &header).is_none());
        let mut other = key.secret;
        other[0] ^= 0x01;
        assert!(open(&other, &wire).is_none());
        assert!(open(&key.secret, &plain).is_none());
    }

    #[test]
    fn probe_seal_preserves_wire_length() {
        let key = vector_key();
        let plain = b"ORB1probe";
        let wire = seal_to_len(&key, &nonce(DIR_HOST, 3), plain, 600).expect("pad");
        assert_eq!(wire.len(), 600);
        let opened = open(&key.secret, &wire).expect("open");
        assert!(opened.starts_with(plain));
        assert_eq!(opened.len(), 600 - SEAL_OVERHEAD);
    }

    #[test]
    fn issued_key_hello_is_accepted_and_shared_token_is_not() {
        let id = [0x22; 16];
        let token = key_id_hex(&id);
        assert_eq!(
            decide_hello(&token, Some("sess"), &id, Some("sess")),
            HelloDecision::Accept {
                session: Some("sess".into())
            }
        );
        assert_eq!(
            decide_hello("shared-token", Some("sess"), &id, Some("sess")),
            HelloDecision::Reject
        );
        assert_eq!(
            decide_hello(&token, Some("other"), &id, Some("sess")),
            HelloDecision::Reject
        );
        assert!(hello_token_matches(&token.to_ascii_uppercase(), &id));
    }

    #[test]
    fn udp_key_mint_requires_a_session_the_caller_owns() {
        assert!(!may_mint_udp_key(MintKind::Anonymous, "sess"));
        assert!(!may_mint_udp_key(MintKind::SharedToken, ""));
        assert!(!may_mint_udp_key(MintKind::SharedToken, "   "));
        assert!(may_mint_udp_key(MintKind::SharedToken, "sess"));
        assert!(may_mint_udp_key(
            MintKind::Paired { owns_session: true },
            "sess"
        ));
        assert!(!may_mint_udp_key(
            MintKind::Paired { owns_session: false },
            "sess"
        ));
    }

    #[test]
    fn replay_window_rejects_duplicates_and_keeps_recent_reorder() {
        let mut window = ReplayWindow::new();
        assert!(!window.accept(0));
        assert!(window.accept(5));
        assert!(!window.accept(5));
        assert!(window.accept(4));
        assert!(!window.accept(4));
        assert!(window.accept(5 + 200));
        assert!(!window.accept(5));
        assert!(window.accept(5 + 200 + REPLAY_WINDOW));
        assert!(!window.accept(5 + 200));
    }

    #[test]
    fn ring_issues_lookup_expiry_and_host_seal() {
        let ring = UdpKeyRing::new();
        let key = ring.issue(Some("sess".into()), Duration::from_secs(60));
        assert_ne!(key.secret, [0; 32]);
        assert_eq!(ring.lookup(&key.id).unwrap().secret, key.secret);
        assert_eq!(ring.bound_session(&key.id), Some(Some("sess".into())));
        let client = nonce(DIR_CLIENT, 1);
        assert!(ring.accept_client_nonce(&key.id, &client));
        assert!(!ring.accept_client_nonce(&key.id, &client));
        assert!(!ring.accept_client_nonce(&key.id, &nonce(DIR_HOST, 1)));
        let sealed = ring.seal_host(&key.id, b"ORB1").expect("host seal");
        let opened = open(&key.secret, &sealed).expect("open host");
        assert_eq!(opened, b"ORB1");
        assert_eq!(sealed[20], DIR_HOST);
        ring.revoke_session("sess");
        assert!(ring.lookup(&key.id).is_none());

        let expired = UdpSealKey {
            id: [0x44; 16],
            secret: [0x55; 32],
        };
        ring.insert(
            expired.clone(),
            Some("old".into()),
            Instant::now() - Duration::from_secs(1),
        );
        assert!(ring.lookup(&expired.id).is_none());
        assert!(!ring.accept_client_nonce(&expired.id, &nonce(DIR_CLIENT, 1)));
        assert!(ring.lookup(&expired.id).is_none());
    }

    #[test]
    fn traffic_extends_a_live_key_past_its_original_deadline() {
        let ttl = Duration::from_millis(80);
        let ring = UdpKeyRing::new();
        let key = ring.issue(Some("sess".into()), ttl);
        std::thread::sleep(Duration::from_millis(50));
        assert!(ring.seal_host(&key.id, b"ping").is_some());
        std::thread::sleep(Duration::from_millis(50));
        assert!(ring.lookup(&key.id).is_some());
        assert!(ring.seal_host(&key.id, b"still").is_some());
        assert!(ring.accept_client_nonce(&key.id, &nonce(DIR_CLIENT, 1)));
    }

    #[test]
    fn an_unused_key_expires_at_its_idle_deadline() {
        let ring = UdpKeyRing::new();
        let key = ring.issue(Some("sess".into()), Duration::from_millis(40));
        std::thread::sleep(Duration::from_millis(70));
        assert!(ring.lookup(&key.id).is_none());
        assert!(ring.seal_host(&key.id, b"late").is_none());
    }
}
