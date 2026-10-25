// Orbiscreen - HTTPS listener identity (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

//! Long-lived self-signed identity for the HTTPS listener on `wt_port`.
//!
//! WebTransport caps hash-pinned certificates at 14 days, so the QUIC endpoint keeps the
//! short-lived `wt-cert.pem` and renews it at start. The HTTPS listener shares the port but
//! not the constraint: it serves the pairing API, the web UI and the Android pinned proxy,
//! and paired Android clients pin the SHA-256 fingerprint of whatever certificate it
//! presents. Serving the 14-day certificate there broke every paired device at least every
//! two weeks (expiry while running, then a new fingerprint on restart). This identity lives
//! for ten years next to the WebTransport one, so the pinned fingerprint stays stable.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rustls::pki_types::pem::PemObject as _;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use sha2::{Digest as _, Sha256};
use tracing::{debug, info, warn};

use super::wt_stream::{advertised_hosts, cert_usable, write_secret_file};

/// Lifetime of a freshly generated HTTPS certificate.
pub const HTTPS_CERT_VALIDITY: Duration = Duration::from_secs(10 * 365 * 24 * 60 * 60);
/// Renew at daemon start when less than this remains.
pub const HTTPS_CERT_RENEW_SLACK: Duration = Duration::from_secs(30 * 24 * 60 * 60);
/// Back-date `not_before` so a client whose clock runs slightly behind still accepts it.
const NOT_BEFORE_SKEW: Duration = Duration::from_secs(24 * 60 * 60);

/// Certificate and private key for the HTTPS listener, PEM encoded for `axum_server`.
#[derive(Debug, Clone)]
pub struct HttpsIdentity {
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
    fingerprint: String,
}

fn default_paths() -> (PathBuf, PathBuf) {
    (
        orbiscreen_core::default_https_cert_path(),
        orbiscreen_core::default_https_key_path(),
    )
}

/// Upper-case hex SHA-256 of the DER certificate: the exact string the Android client pins
/// and shows on its confirmation screen.
pub fn fingerprint_hex(cert_der: &[u8]) -> String {
    Sha256::digest(cert_der)
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect()
}

fn pem_pair_to_identity(cert_pem: Vec<u8>, key_pem: Vec<u8>) -> Result<HttpsIdentity, String> {
    let cert_der =
        CertificateDer::from_pem_slice(&cert_pem).map_err(|e| format!("cert pem: {e}"))?;
    PrivateKeyDer::from_pem_slice(&key_pem).map_err(|e| format!("key pem: {e}"))?;
    let fingerprint = fingerprint_hex(&cert_der);
    Ok(HttpsIdentity {
        cert_pem,
        key_pem,
        fingerprint,
    })
}

fn load(cert_path: &Path, key_path: &Path) -> Result<HttpsIdentity, String> {
    let cert_pem = std::fs::read(cert_path).map_err(|e| format!("read cert: {e}"))?;
    let key_pem = std::fs::read(key_path).map_err(|e| format!("read key: {e}"))?;
    let cert_der =
        CertificateDer::from_pem_slice(&cert_pem).map_err(|e| format!("cert pem: {e}"))?;
    if !cert_usable(&cert_der, HTTPS_CERT_RENEW_SLACK) {
        return Err("certificate expired or expiring".into());
    }
    pem_pair_to_identity(cert_pem, key_pem)
}

fn generate(
    sans: &[String],
    not_before: time::OffsetDateTime,
    not_after: time::OffsetDateTime,
) -> Result<HttpsIdentity, String> {
    let key = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)
        .map_err(|e| format!("key: {e}"))?;
    let mut params =
        rcgen::CertificateParams::new(sans.to_vec()).map_err(|e| format!("params: {e}"))?;
    params.not_before = not_before;
    params.not_after = not_after;
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "Orbiscreen host");
    let cert = params
        .self_signed(&key)
        .map_err(|e| format!("self-sign: {e}"))?;
    // PEM encoding goes through wtransport's types so no extra PEM crate is pulled in.
    let cert_pem = wtransport::tls::Certificate::from_der(cert.der().to_vec())
        .map_err(|e| format!("cert der: {e}"))?
        .to_pem();
    let key_pem = wtransport::tls::PrivateKey::from_der_pkcs8(key.serialize_der()).to_secret_pem();
    pem_pair_to_identity(cert_pem.into_bytes(), key_pem.into_bytes())
}

fn now_utc() -> time::OffsetDateTime {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    time::OffsetDateTime::UNIX_EPOCH + since_epoch
}

fn store(identity: &HttpsIdentity, cert_path: &Path, key_path: &Path) -> Result<(), String> {
    write_secret_file(cert_path, &identity.cert_pem).map_err(|e| format!("write cert: {e}"))?;
    write_secret_file(key_path, &identity.key_pem).map_err(|e| format!("write key: {e}"))?;
    Ok(())
}

impl HttpsIdentity {
    /// Loads the persisted identity from the config directory, or generates and stores a new one.
    pub fn new() -> Result<Self, String> {
        let (cert_path, key_path) = default_paths();
        Self::load_or_create(&cert_path, &key_path)
    }

    pub fn load_or_create(cert_path: &Path, key_path: &Path) -> Result<Self, String> {
        match load(cert_path, key_path) {
            Ok(identity) => {
                info!(cert = %cert_path.display(), "reusing persisted HTTPS certificate");
                Ok(identity)
            }
            Err(reason) => {
                debug!(
                    cert = %cert_path.display(),
                    reason,
                    "generating a new HTTPS certificate"
                );
                let mut sans = vec!["localhost".to_string(), "127.0.0.1".to_string()];
                for host in advertised_hosts() {
                    if !sans.contains(&host) {
                        sans.push(host);
                    }
                }
                let now = now_utc();
                let identity = generate(&sans, now - NOT_BEFORE_SKEW, now + HTTPS_CERT_VALIDITY)?;
                match store(&identity, cert_path, key_path) {
                    Ok(()) => info!(cert = %cert_path.display(), "stored HTTPS certificate"),
                    Err(e) => warn!(
                        cert = %cert_path.display(),
                        "could not persist HTTPS certificate (paired devices will have to re-confirm after the next restart): {e}"
                    ),
                }
                Ok(identity)
            }
        }
    }

    /// Upper-case hex SHA-256 of the DER certificate, without separators.
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// `AA:BB:...` form, as the Android confirmation screen prints it.
    pub fn fingerprint_display(&self) -> String {
        self.fingerprint
            .as_bytes()
            .chunks(2)
            .map(|pair| std::str::from_utf8(pair).unwrap_or("??"))
            .collect::<Vec<_>>()
            .join(":")
    }

    pub fn cert_pem(&self) -> &[u8] {
        &self.cert_pem
    }

    /// `(certificate PEM, private key PEM)` for `axum_server::tls_rustls::RustlsConfig::from_pem`.
    pub fn into_pem(self) -> (Vec<u8>, Vec<u8>) {
        (self.cert_pem, self.key_pem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x509_parser::prelude::FromDer as _;

    fn temp_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("orbiscreen-https-{}-{}", std::process::id(), nonce));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn validity(identity: &HttpsIdentity) -> (i64, i64) {
        let der = CertificateDer::from_pem_slice(identity.cert_pem()).unwrap();
        let (_, cert) = x509_parser::certificate::X509Certificate::from_der(&der).unwrap();
        (
            cert.validity().not_before.timestamp(),
            cert.validity().not_after.timestamp(),
        )
    }

    #[test]
    fn generated_identity_is_long_lived_and_persisted() {
        let dir = temp_dir();
        let (cert, key) = (dir.join("https-cert.pem"), dir.join("https-key.pem"));
        let identity = HttpsIdentity::load_or_create(&cert, &key).expect("identity");
        assert!(cert.is_file());
        assert!(key.is_file());
        assert_eq!(identity.fingerprint().len(), 64);
        assert!(identity
            .fingerprint()
            .chars()
            .all(|c| c.is_ascii_digit() || ('A'..='F').contains(&c)));
        assert_eq!(identity.fingerprint_display().len(), 64 + 31);
        let now = now_utc().unix_timestamp();
        let (not_before, not_after) = validity(&identity);
        assert!(not_before <= now - NOT_BEFORE_SKEW.as_secs() as i64 + 60);
        assert!(not_after - now >= 9 * 365 * 24 * 60 * 60);
        assert!(cert_usable(
            &CertificateDer::from_pem_slice(identity.cert_pem()).unwrap(),
            HTTPS_CERT_RENEW_SLACK
        ));
    }

    #[test]
    fn persisted_identity_keeps_its_fingerprint() {
        let dir = temp_dir();
        let (cert, key) = (dir.join("https-cert.pem"), dir.join("https-key.pem"));
        let first = HttpsIdentity::load_or_create(&cert, &key).expect("first");
        let second = HttpsIdentity::load_or_create(&cert, &key).expect("second");
        assert_eq!(first.fingerprint(), second.fingerprint());
        assert_eq!(first.cert_pem(), second.cert_pem());
    }

    #[test]
    fn fingerprint_is_sha256_of_der_in_upper_hex() {
        let dir = temp_dir();
        let identity = HttpsIdentity::load_or_create(&dir.join("c.pem"), &dir.join("k.pem"))
            .expect("identity");
        let der = CertificateDer::from_pem_slice(identity.cert_pem()).unwrap();
        let expected: String = Sha256::digest(der.as_ref())
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect();
        assert_eq!(identity.fingerprint(), expected);
    }

    #[test]
    fn expired_or_expiring_identity_is_replaced() {
        let dir = temp_dir();
        let (cert, key) = (dir.join("https-cert.pem"), dir.join("https-key.pem"));
        let sans = vec!["localhost".to_string()];
        let now = now_utc();
        let expiring = generate(
            &sans,
            now - time::Duration::days(365),
            now + time::Duration::days(1),
        )
        .expect("expiring");
        store(&expiring, &cert, &key).expect("store");
        assert!(load(&cert, &key).is_err());
        let renewed = HttpsIdentity::load_or_create(&cert, &key).expect("renewed");
        assert_ne!(renewed.fingerprint(), expiring.fingerprint());
        let (_, not_after) = validity(&renewed);
        assert!(not_after - now.unix_timestamp() >= 9 * 365 * 24 * 60 * 60);
        assert_eq!(
            HttpsIdentity::load_or_create(&cert, &key)
                .expect("reload")
                .fingerprint(),
            renewed.fingerprint()
        );
    }

    #[test]
    fn missing_key_regenerates_identity() {
        let dir = temp_dir();
        let (cert, key) = (dir.join("https-cert.pem"), dir.join("https-key.pem"));
        let first = HttpsIdentity::load_or_create(&cert, &key).expect("first");
        std::fs::remove_file(&key).expect("drop key");
        let second = HttpsIdentity::load_or_create(&cert, &key).expect("second");
        assert_ne!(first.fingerprint(), second.fingerprint());
    }

    #[test]
    fn https_fingerprint_survives_webtransport_renewal() {
        use crate::wt_stream::WtHub;
        let dir = temp_dir();
        let (https_cert, https_key) = (dir.join("https-cert.pem"), dir.join("https-key.pem"));
        let (wt_cert, wt_key) = (dir.join("wt-cert.pem"), dir.join("wt-key.pem"));
        let https = HttpsIdentity::load_or_create(&https_cert, &https_key).expect("https");
        let wt_first = WtHub::load_or_create(0, &wt_cert, &wt_key).expect("wt first");
        // Simulate the 14-day renewal: drop the WebTransport identity and start again.
        std::fs::remove_file(&wt_cert).expect("drop wt cert");
        std::fs::remove_file(&wt_key).expect("drop wt key");
        let wt_second = WtHub::load_or_create(0, &wt_cert, &wt_key).expect("wt second");
        assert_ne!(wt_first.offer.cert_sha256, wt_second.offer.cert_sha256);
        let https_again =
            HttpsIdentity::load_or_create(&https_cert, &https_key).expect("https again");
        assert_eq!(https.fingerprint(), https_again.fingerprint());
        // The two identities are distinct certificates: the HTTPS one is not hash-pinned by
        // browsers, so it may outlive the WebTransport 14-day cap.
        assert_ne!(
            https.fingerprint(),
            fingerprint_hex(
                wt_second
                    .https_pem()
                    .map(|(c, _)| { CertificateDer::from_pem_slice(&c).unwrap().to_vec() })
                    .unwrap()
                    .as_slice()
            )
        );
    }

    #[tokio::test]
    async fn pem_pair_is_accepted_by_the_https_server_config() {
        let dir = temp_dir();
        let identity = HttpsIdentity::load_or_create(&dir.join("c.pem"), &dir.join("k.pem"))
            .expect("identity");
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (cert_pem, key_pem) = identity.into_pem();
        axum_server::tls_rustls::RustlsConfig::from_pem(cert_pem, key_pem)
            .await
            .expect("rustls config from the generated PEM pair");
    }
}
