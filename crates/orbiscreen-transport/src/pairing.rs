use std::collections::HashMap;
use std::fmt;
use std::io;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MAX_PENDING_REQUESTS: usize = 8;
const MAX_PENDING_PER_PEER: usize = 2;
const MAX_CLIENTS: usize = 16;
const REQUEST_TTL_SECS: u64 = 600;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientStatus {
    Approved,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingClient {
    pub client_id: String,
    pub label: String,
    pub created_at: u64,
    pub last_seen: u64,
    pub status: ClientStatus,
}

#[derive(Clone, Serialize)]
pub struct PairingRequest {
    pub request_id: String,
    pub label: String,
    pub peer: String,
    pub created_at: u64,
    pub approved: bool,
}

impl fmt::Debug for PairingRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PairingRequest")
            .field("label", &self.label)
            .field("peer", &self.peer)
            .field("created_at", &self.created_at)
            .field("approved", &self.approved)
            .finish_non_exhaustive()
    }
}

#[derive(Serialize, Deserialize)]
struct StoredClient {
    #[serde(flatten)]
    client: PairingClient,
    credential_hash: String,
}

struct PendingRequest {
    request: PairingRequest,
    expires_at: Instant,
    client_id: Option<String>,
    credential: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
struct RegistryData {
    #[serde(default)]
    clients: Vec<StoredClient>,
    #[serde(skip)]
    requests: Vec<PendingRequest>,
    #[serde(skip)]
    failed: bool,
    #[serde(skip)]
    sessions: HashMap<String, String>,
}

impl RegistryData {
    fn expire(&mut self) {
        let now = Instant::now();
        let wall_now = now_secs();
        self.requests.retain(|pending| {
            now < pending.expires_at
                && wall_now
                    .checked_sub(pending.request.created_at)
                    .is_some_and(|age| age < REQUEST_TTL_SECS)
        });
    }
}

pub struct PairingRegistry {
    path: Option<PathBuf>,
    data: RwLock<RegistryData>,
}

impl fmt::Debug for PairingRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PairingRegistry").finish_non_exhaustive()
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn hash_credential(credential: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(credential.as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

pub fn generate_credential() -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine as _;
    let mut bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rng(), &mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn sanitize_label(label: &str, fallback: &str) -> String {
    let cleaned: String = label
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
        .take(32)
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn unavailable() -> io::Error {
    io::Error::other("pairing registry unavailable")
}

impl PairingRegistry {
    pub fn load() -> io::Result<Self> {
        let path = pairing_state_path().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "pairing state directory unavailable",
            )
        })?;
        Self::load_from(Some(path))
    }

    pub fn load_from(path: Option<PathBuf>) -> io::Result<Self> {
        let data = match path.as_deref() {
            Some(path) => load_registry(path)?,
            None => RegistryData::default(),
        };
        Ok(Self {
            path,
            data: RwLock::new(data),
        })
    }

    pub fn request(&self, label: &str, peer: &str) -> Option<String> {
        self.request_pairing(label, peer).map(|r| r.request_id)
    }

    pub fn request_pairing(&self, label: &str, peer: &str) -> Option<PairingRequest> {
        let peer = peer.parse::<IpAddr>().ok()?.to_string();
        let mut data = self.data.write().ok()?;
        if data.failed {
            return None;
        }
        data.expire();
        if data.requests.len() >= MAX_PENDING_REQUESTS {
            return None;
        }
        let from_this_peer = data
            .requests
            .iter()
            .filter(|p| p.request.peer == peer)
            .count();
        if from_this_peer >= MAX_PENDING_PER_PEER {
            return None;
        }
        let request = PairingRequest {
            request_id: generate_credential(),
            label: sanitize_label(label, "device"),
            peer,
            created_at: now_secs(),
            approved: false,
        };
        data.requests.push(PendingRequest {
            request: request.clone(),
            expires_at: Instant::now() + Duration::from_secs(REQUEST_TTL_SECS),
            client_id: None,
            credential: None,
        });
        Some(request)
    }

    pub fn pending_requests(&self) -> Vec<PairingRequest> {
        let Ok(mut data) = self.data.write() else {
            return Vec::new();
        };
        if data.failed {
            return Vec::new();
        }
        data.expire();
        data.requests.iter().map(|p| p.request.clone()).collect()
    }

    pub fn approve(&self, request_id: &str) -> io::Result<Option<PairingClient>> {
        let mut data = self.data.write().map_err(|_| unavailable())?;
        if data.failed {
            return Err(unavailable());
        }
        data.expire();
        let Some(index) = data
            .requests
            .iter()
            .position(|p| crate::token_eq(&p.request.request_id, request_id))
        else {
            return Ok(None);
        };
        if data.requests[index].request.approved || data.clients.len() >= MAX_CLIENTS {
            return Ok(None);
        }
        let credential = generate_credential();
        let now = now_secs();
        let client = PairingClient {
            client_id: generate_credential(),
            label: data.requests[index].request.label.clone(),
            created_at: now,
            last_seen: now,
            status: ClientStatus::Approved,
        };
        data.clients.push(StoredClient {
            client: client.clone(),
            credential_hash: hash_credential(&credential),
        });
        if let Err(error) = self.persist(&mut data) {
            data.clients.pop();
            return Err(error);
        }
        let pending = &mut data.requests[index];
        pending.request.approved = true;
        pending.client_id = Some(client.client_id.clone());
        pending.credential = Some(credential);
        Ok(Some(client))
    }

    pub fn claim(&self, request_id: &str, peer: &str) -> Option<String> {
        let peer = peer.parse::<IpAddr>().ok()?.to_string();
        let mut data = self.data.write().ok()?;
        if data.failed {
            return None;
        }
        data.expire();
        let index = data.requests.iter().position(|p| {
            crate::token_eq(&p.request.request_id, request_id) && p.request.peer == peer
        })?;
        let client_id = data.requests[index].client_id.as_deref()?;
        if !data
            .clients
            .iter()
            .any(|c| c.client.client_id == client_id && c.client.status == ClientStatus::Approved)
        {
            data.requests.remove(index);
            return None;
        }
        let credential = data.requests[index].credential.take()?;
        data.requests.remove(index);
        Some(credential)
    }

    pub fn deny(&self, request_id: &str) -> io::Result<bool> {
        let mut data = self.data.write().map_err(|_| unavailable())?;
        if data.failed {
            return Err(unavailable());
        }
        data.expire();
        let Some(index) = data
            .requests
            .iter()
            .position(|p| crate::token_eq(&p.request.request_id, request_id))
        else {
            return Ok(false);
        };
        let pending = data.requests.remove(index);
        if let Some(client_id) = pending.client_id {
            if let Some(client) = data
                .clients
                .iter_mut()
                .find(|c| c.client.client_id == client_id)
            {
                client.client.status = ClientStatus::Revoked;
            }
            data.sessions.retain(|_, owner| owner != &client_id);
            self.persist(&mut data)?;
        }
        Ok(true)
    }

    pub fn revoke(&self, client_id: &str) -> io::Result<bool> {
        let mut data = self.data.write().map_err(|_| unavailable())?;
        if data.failed {
            return Err(unavailable());
        }
        let Some(client) = data
            .clients
            .iter_mut()
            .find(|c| c.client.client_id == client_id)
        else {
            return Ok(false);
        };
        client.client.status = ClientStatus::Revoked;
        data.requests
            .retain(|p| p.client_id.as_deref() != Some(client_id));
        data.sessions.retain(|_, owner| owner != client_id);
        self.persist(&mut data)?;
        Ok(true)
    }

    pub fn clients(&self) -> Vec<PairingClient> {
        self.data
            .read()
            .ok()
            .filter(|data| !data.failed)
            .map(|data| data.clients.iter().map(|c| c.client.clone()).collect())
            .unwrap_or_default()
    }

    pub fn verify(&self, credential: &str) -> Option<PairingClient> {
        let hash = hash_credential(credential);
        let mut data = self.data.write().ok()?;
        if data.failed {
            return None;
        }
        let mut matched = None;
        for (index, client) in data.clients.iter().enumerate() {
            if crate::token_eq(&client.credential_hash, &hash)
                && client.client.status == ClientStatus::Approved
            {
                matched = Some(index);
            }
        }
        let client = &mut data.clients[matched?].client;
        client.last_seen = now_secs();
        Some(client.clone())
    }

    pub fn bind_session(&self, client_id: &str, session_id: &str) -> bool {
        let Ok(mut data) = self.data.write() else {
            return false;
        };
        if data.failed
            || session_id.is_empty()
            || !data.clients.iter().any(|c| {
                c.client.client_id == client_id && c.client.status == ClientStatus::Approved
            })
        {
            return false;
        }
        match data.sessions.entry(session_id.to_string()) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.get() == client_id,
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(client_id.to_string());
                true
            }
        }
    }

    pub fn owns_session(&self, client_id: &str, session_id: &str) -> bool {
        self.data.read().is_ok_and(|data| {
            !data.failed
                && data
                    .sessions
                    .get(session_id)
                    .is_some_and(|owner| owner == client_id)
                && data.clients.iter().any(|c| {
                    c.client.client_id == client_id && c.client.status == ClientStatus::Approved
                })
        })
    }

    pub fn session_owner(&self, session_id: &str) -> Option<String> {
        self.data
            .read()
            .ok()
            .filter(|data| !data.failed)
            .and_then(|data| {
                let owner = data.sessions.get(session_id)?.clone();
                let still_valid = data.clients.iter().any(|c| {
                    c.client.client_id == owner && c.client.status == ClientStatus::Approved
                });
                still_valid.then_some(owner)
            })
    }

    pub fn forget_session(&self, session_id: &str) {
        if let Ok(mut data) = self.data.write() {
            data.sessions.remove(session_id);
        }
    }

    pub fn owned_sessions(&self, client_id: &str) -> Vec<String> {
        let Ok(data) = self.data.read() else {
            return Vec::new();
        };
        if data.failed
            || !data.clients.iter().any(|c| {
                c.client.client_id == client_id && c.client.status == ClientStatus::Approved
            })
        {
            return Vec::new();
        }
        let mut sessions: Vec<_> = data
            .sessions
            .iter()
            .filter(|(_, owner)| owner.as_str() == client_id)
            .map(|(session, _)| session.clone())
            .collect();
        sessions.sort();
        sessions
    }

    fn persist(&self, data: &mut RegistryData) -> io::Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        if let Err(error) = save_registry(data, path) {
            data.failed = true;
            data.requests.clear();
            data.sessions.clear();
            return Err(error);
        }
        Ok(())
    }
}

pub fn pairing_state_path() -> Option<PathBuf> {
    pairing_state_path_from(|key| std::env::var_os(key))
}

fn pairing_state_path_from(
    mut lookup: impl FnMut(&str) -> Option<std::ffi::OsString>,
) -> Option<PathBuf> {
    if let Some(xdg) =
        lookup("XDG_STATE_HOME").filter(|v| !v.is_empty() && PathBuf::from(&v).is_absolute())
    {
        return Some(PathBuf::from(xdg).join("orbiscreen/clients.json"));
    }
    if let Some(home) = lookup("HOME").filter(|v| !v.is_empty() && PathBuf::from(&v).is_absolute())
    {
        return Some(PathBuf::from(home).join(".local/state/orbiscreen/clients.json"));
    }
    None
}

fn load_registry(path: &Path) -> io::Result<RegistryData> {
    use std::io::Read as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let file = match std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(RegistryData::default());
        }
        Err(error) => return Err(error),
    };
    if !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid pairing state",
        ));
    }
    let mut content = Vec::new();
    file.take(1_048_577).read_to_end(&mut content)?;
    if content.len() > 1_048_576 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "pairing state too large",
        ));
    }
    let data: RegistryData = serde_json::from_slice(&content)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid pairing state"))?;
    if data.clients.len() > MAX_CLIENTS
        || data.clients.iter().enumerate().any(|(index, c)| {
            c.client.client_id.is_empty()
                || c.credential_hash.len() != 64
                || !c
                    .credential_hash
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                || data.clients[..index]
                    .iter()
                    .any(|other| other.client.client_id == c.client.client_id)
        })
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid pairing clients",
        ));
    }
    Ok(data)
}

fn save_registry(data: &RegistryData, path: &Path) -> io::Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _, PermissionsExt as _};
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no parent dir"))?;
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(parent)?;
    let directory = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(parent)?;
    directory.set_permissions(std::fs::Permissions::from_mode(0o700))?;
    let content = serde_json::to_vec_pretty(data)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "pairing serialization failed"))?;
    let tmp_path = parent.join(format!(
        ".clients.tmp-{}-{}",
        std::process::id(),
        generate_credential()
    ));
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW)
        .mode(0o600)
        .open(&tmp_path)?;
    let result = (|| {
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        file.write_all(&content)?;
        file.sync_all()?;
        std::fs::rename(&tmp_path, path)?;
        directory.sync_all()
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp_path);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const PEER: &str = "192.0.2.9";

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("orbi-pair-test-{}", generate_credential()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> PathBuf {
            self.0.join("state/clients.json")
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn registry() -> PairingRegistry {
        PairingRegistry::load_from(None).unwrap()
    }

    fn expire(registry: &PairingRegistry) {
        for pending in &mut registry.data.write().unwrap().requests {
            pending.expires_at = Instant::now();
        }
    }

    #[test]
    fn full_lifecycle_is_explicit_and_one_time() {
        let reg = registry();
        let request = reg.request("Tablet", PEER).unwrap();
        assert_eq!(request.len(), 43);
        assert!(reg.claim(&request, PEER).is_none());
        assert!(!reg.pending_requests()[0].approved);
        let client = reg.approve(&request).unwrap().unwrap();
        assert!(reg.approve(&request).unwrap().is_none());
        assert!(reg.pending_requests()[0].approved);
        let credential = reg.claim(&request, PEER).unwrap();
        assert!(reg.claim(&request, PEER).is_none());
        assert!(reg.pending_requests().is_empty());
        assert_eq!(reg.verify(&credential).unwrap().client_id, client.client_id);
        assert!(reg.verify("wrong").is_none());
        assert!(reg.revoke(&client.client_id).unwrap());
        assert!(reg.verify(&credential).is_none());
    }

    #[test]
    fn duplicate_labels_and_ips_are_independent() {
        let reg = registry();
        let first = reg.request("Tablet", PEER).unwrap();
        let second = reg.request("Tablet", PEER).unwrap();
        assert_ne!(first, second);
        let a = reg.approve(&first).unwrap().unwrap();
        let b = reg.approve(&second).unwrap().unwrap();
        assert_ne!(a.client_id, b.client_id);
        let secret_a = reg.claim(&first, PEER).unwrap();
        let secret_b = reg.claim(&second, PEER).unwrap();
        assert_ne!(secret_a, secret_b);
        assert!(reg.revoke(&b.client_id).unwrap());
        assert!(reg.verify(&secret_b).is_none());
        assert_eq!(reg.verify(&secret_a).unwrap().client_id, a.client_id);
    }

    #[test]
    fn pending_capacity_rejects_without_eviction() {
        let reg = registry();
        let peer = |i: usize| format!("192.0.2.{i}");
        let requests: Vec<_> = (0..MAX_PENDING_REQUESTS)
            .map(|i| reg.request("Tablet", &peer(i + 1)).unwrap())
            .collect();
        assert!(reg
            .request("Tablet", &peer(MAX_PENDING_REQUESTS + 1))
            .is_none());
        assert_eq!(reg.pending_requests()[0].request_id, requests[0]);
        assert!(reg.deny(&requests[0]).unwrap());
        assert!(reg
            .request("Tablet", &peer(MAX_PENDING_REQUESTS + 1))
            .is_some());
        expire(&reg);
        assert!(reg
            .request("Tablet", &peer(MAX_PENDING_REQUESTS + 2))
            .is_some());
        assert_eq!(reg.pending_requests().len(), 1);
    }

    #[test]
    fn client_capacity_rejects_without_eviction() {
        let reg = registry();
        let credentials: Vec<_> = (0..MAX_CLIENTS)
            .map(|_| {
                let request = reg.request("Tablet", PEER).unwrap();
                reg.approve(&request).unwrap().unwrap();
                reg.claim(&request, PEER).unwrap()
            })
            .collect();
        let extra = reg.request("Tablet", PEER).unwrap();
        assert!(reg.approve(&extra).unwrap().is_none());
        assert!(reg.claim(&extra, PEER).is_none());
        assert_eq!(reg.clients().len(), MAX_CLIENTS);
        assert!(credentials.iter().all(|c| reg.verify(c).is_some()));
    }

    #[test]
    fn expiry_is_enforced_at_approval_claim_and_list() {
        let reg = registry();
        let request = reg.request("Tablet", PEER).unwrap();
        expire(&reg);
        assert!(reg.approve(&request).unwrap().is_none());
        let request = reg.request("Tablet", PEER).unwrap();
        reg.approve(&request).unwrap().unwrap();
        expire(&reg);
        assert!(reg.claim(&request, PEER).is_none());
        reg.request("Tablet", PEER).unwrap();
        expire(&reg);
        assert!(reg.pending_requests().is_empty());
    }

    #[test]
    fn wall_clock_expiry_and_rollback_fail_closed() {
        for created_at in [now_secs() - REQUEST_TTL_SECS, now_secs() + 60] {
            let reg = registry();
            let request = reg.request("Tablet", PEER).unwrap();
            reg.data.write().unwrap().requests[0].request.created_at = created_at;
            assert!(reg.approve(&request).unwrap().is_none());
        }
    }

    #[test]
    fn revoke_and_deny_before_claim_destroy_only_matching_secrets() {
        for revoke in [false, true] {
            let reg = registry();
            let first = reg.request("Tablet", PEER).unwrap();
            let second = reg.request("Tablet", PEER).unwrap();
            let client = reg.approve(&first).unwrap().unwrap();
            reg.approve(&second).unwrap().unwrap();
            let revoked_credential = reg.data.read().unwrap().requests[0]
                .credential
                .clone()
                .unwrap();
            if revoke {
                assert!(reg.revoke(&client.client_id).unwrap());
            } else {
                assert!(reg.deny(&first).unwrap());
            }
            assert!(reg.claim(&first, PEER).is_none());
            assert!(reg.verify(&revoked_credential).is_none());
            assert!(reg.claim(&second, PEER).is_some());
        }
    }

    #[test]
    fn claim_requires_capability_and_ip_without_port() {
        let reg = registry();
        assert!(reg.request("Tablet", "192.0.2.9:1000").is_none());
        let request = reg.request("Tablet", PEER).unwrap();
        reg.approve(&request).unwrap().unwrap();
        assert!(reg.claim("unknown", PEER).is_none());
        assert!(reg.claim(&request, "192.0.2.10").is_none());
        assert!(reg.claim(&request, "192.0.2.9:1000").is_none());
        assert!(reg.claim(&request, PEER).is_some());
        let ipv6 = reg.request("Tablet", "2001:db8:0:0::1").unwrap();
        reg.approve(&ipv6).unwrap().unwrap();
        assert!(reg.claim(&ipv6, "2001:db8::1").is_some());
    }

    #[test]
    fn persistence_contains_only_client_hashes_and_survives_restart() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = TestDir::new();
        let path = dir.path();
        let reg = PairingRegistry::load_from(Some(path.clone())).unwrap();
        let request = reg.request("Tablet", PEER).unwrap();
        assert!(!path.exists());
        let client = reg.approve(&request).unwrap().unwrap();
        let credential = reg.claim(&request, PEER).unwrap();
        let pending = reg.request("Tablet", PEER).unwrap();
        reg.approve(&pending).unwrap().unwrap();
        let unclaimed = reg.data.read().unwrap().requests[0]
            .credential
            .clone()
            .unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains(&hash_credential(&credential)));
        for secret in [&credential, &unclaimed, &request, &pending] {
            assert!(!content.contains(secret));
        }
        assert!(!content.contains("requests"));
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        drop(reg);
        let reloaded = PairingRegistry::load_from(Some(path)).unwrap();
        assert!(reloaded.pending_requests().is_empty());
        assert!(reloaded.claim(&pending, PEER).is_none());
        assert_eq!(
            reloaded.verify(&credential).unwrap().client_id,
            client.client_id
        );
        reloaded.revoke(&client.client_id).unwrap();
        let reloaded = PairingRegistry::load_from(Some(dir.path())).unwrap();
        assert!(reloaded.verify(&credential).is_none());
    }

    #[test]
    fn failed_approval_disables_registry_and_cleans_temporary_file() {
        let dir = TestDir::new();
        let path = dir.path();
        let reg = PairingRegistry::load_from(Some(path.clone())).unwrap();
        let request = reg.request("Tablet", PEER).unwrap();
        std::fs::create_dir_all(&path).unwrap();
        assert!(reg.approve(&request).is_err());
        assert!(reg.clients().is_empty());
        assert!(reg.pending_requests().is_empty());
        assert!(reg.claim(&request, PEER).is_none());
        assert!(reg.request("Tablet", PEER).is_none());
        assert_eq!(
            std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
            1
        );
    }

    #[test]
    fn failed_revocation_or_denial_blocks_authentication_and_claims() {
        for revoke in [false, true] {
            let dir = TestDir::new();
            let path = dir.path();
            let reg = PairingRegistry::load_from(Some(path.clone())).unwrap();
            let request = reg.request("Tablet", PEER).unwrap();
            let client = reg.approve(&request).unwrap().unwrap();
            let credential = reg.data.read().unwrap().requests[0]
                .credential
                .clone()
                .unwrap();
            std::fs::remove_file(&path).unwrap();
            std::fs::create_dir(&path).unwrap();
            if revoke {
                assert!(reg.revoke(&client.client_id).is_err());
            } else {
                assert!(reg.deny(&request).is_err());
            }
            assert!(reg.verify(&credential).is_none());
            assert!(reg.claim(&request, PEER).is_none());
            assert!(reg.approve(&request).is_err());
            assert!(reg.deny(&request).is_err());
            assert!(reg.revoke(&client.client_id).is_err());
        }
    }

    #[test]
    fn corrupt_and_symlink_state_fail_loading() {
        use std::os::unix::fs::symlink;
        let dir = TestDir::new();
        let path = dir.0.join("clients.json");
        std::fs::write(&path, "invalid").unwrap();
        assert!(PairingRegistry::load_from(Some(path.clone())).is_err());
        let link = dir.0.join("link.json");
        symlink(&path, &link).unwrap();
        assert!(PairingRegistry::load_from(Some(link)).is_err());
    }

    #[test]
    fn debug_and_public_serialization_never_expose_credentials_or_hashes() {
        let reg = registry();
        let request = reg.request_pairing("Tablet", PEER).unwrap();
        let client = reg.approve(&request.request_id).unwrap().unwrap();
        let credential = reg.data.read().unwrap().requests[0]
            .credential
            .clone()
            .unwrap();
        let hash = hash_credential(&credential);
        let debug = format!(
            "{reg:?} {request:?} {client:?} {:?}",
            reg.pending_requests()
        );
        let json = serde_json::to_string(&(reg.clients(), reg.pending_requests())).unwrap();
        for secret in [&credential, &hash] {
            assert!(!debug.contains(secret));
            assert!(!json.contains(secret));
        }
        assert!(!debug.contains(&request.request_id));
        assert!(!json.contains("credential"));
    }

    #[test]
    fn session_ownership_is_exclusive_and_requires_approved_clients() {
        let reg = registry();
        let first = reg.request("Tablet", PEER).unwrap();
        let second = reg.request("Tablet", PEER).unwrap();
        let a = reg.approve(&first).unwrap().unwrap();
        let b = reg.approve(&second).unwrap().unwrap();
        assert!(!reg.bind_session("unknown", "one"));
        assert!(!reg.bind_session(&a.client_id, ""));
        assert!(reg.bind_session(&a.client_id, "one"));
        assert!(reg.bind_session(&a.client_id, "one"));
        assert!(!reg.bind_session(&b.client_id, "one"));
        assert!(reg.owns_session(&a.client_id, "one"));
        assert!(!reg.owns_session(&b.client_id, "one"));
        assert!(reg.bind_session(&a.client_id, "two"));
        assert_eq!(reg.owned_sessions(&a.client_id), vec!["one", "two"]);
        reg.forget_session("one");
        reg.forget_session("unknown");
        assert!(!reg.owns_session(&a.client_id, "one"));
        assert!(reg.bind_session(&b.client_id, "one"));
        reg.revoke(&a.client_id).unwrap();
        assert!(!reg.owns_session(&a.client_id, "two"));
        assert!(!reg.bind_session(&a.client_id, "three"));
        assert!(reg.owned_sessions(&a.client_id).is_empty());
        assert!(reg.owns_session(&b.client_id, "one"));
        reg.deny(&second).unwrap();
        assert!(!reg.owns_session(&b.client_id, "one"));
        assert!(reg.owned_sessions(&b.client_id).is_empty());
    }

    #[test]
    fn session_ownership_is_memory_only_and_fails_closed() {
        let dir = TestDir::new();
        let path = dir.path();
        let reg = PairingRegistry::load_from(Some(path.clone())).unwrap();
        let request = reg.request("Tablet", PEER).unwrap();
        let client = reg.approve(&request).unwrap().unwrap();
        assert!(reg.bind_session(&client.client_id, "session-only-in-memory"));
        let other = reg.request("Tablet", PEER).unwrap();
        reg.approve(&other).unwrap().unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(!content.contains("session-only-in-memory"));
        assert!(!content.contains("sessions"));
        let reloaded = PairingRegistry::load_from(Some(path.clone())).unwrap();
        assert!(!reloaded.owns_session(&client.client_id, "session-only-in-memory"));
        assert!(reloaded.owned_sessions(&client.client_id).is_empty());
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(reg.revoke(&client.client_id).is_err());
        assert!(!reg.owns_session(&client.client_id, "session-only-in-memory"));
        assert!(!reg.bind_session(&client.client_id, "new-session"));
        assert!(reg.owned_sessions(&client.client_id).is_empty());
    }

    #[test]
    fn concurrent_claim_delivers_exactly_once() {
        let reg = registry();
        let request = reg.request("Tablet", PEER).unwrap();
        reg.approve(&request).unwrap().unwrap();
        std::thread::scope(|scope| {
            let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
            let handles: Vec<_> = (0..2)
                .map(|_| {
                    let barrier = barrier.clone();
                    let reg = &reg;
                    let request = &request;
                    scope.spawn(move || {
                        barrier.wait();
                        reg.claim(request, PEER)
                    })
                })
                .collect();
            let delivered = handles
                .into_iter()
                .filter_map(|handle| handle.join().unwrap())
                .count();
            assert_eq!(delivered, 1);
        });
    }

    #[test]
    fn concurrent_approvals_persist_every_client_under_write_lock() {
        let dir = TestDir::new();
        let path = dir.path();
        let reg = PairingRegistry::load_from(Some(path.clone())).unwrap();
        let requests: Vec<_> = (0..MAX_PENDING_REQUESTS)
            .map(|i| reg.request("Tablet", &format!("192.0.2.{i}")).unwrap())
            .collect();
        std::thread::scope(|scope| {
            let handles: Vec<_> = requests
                .iter()
                .map(|request| scope.spawn(|| reg.approve(request).unwrap().unwrap()))
                .collect();
            for handle in handles {
                handle.join().unwrap();
            }
        });
        let reloaded = PairingRegistry::load_from(Some(path.clone())).unwrap();
        assert_eq!(reloaded.clients().len(), MAX_PENDING_REQUESTS);
        assert_eq!(
            std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
            1
        );
        for (i, request) in requests.iter().enumerate() {
            let credential = reg.claim(request, &format!("192.0.2.{i}")).unwrap();
            assert!(reloaded.verify(&credential).is_some());
        }
    }

    #[test]
    fn credential_hash_is_stable_sha256_hex() {
        assert_eq!(
            hash_credential("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_ne!(hash_credential("abc"), hash_credential("abd"));
    }
}
