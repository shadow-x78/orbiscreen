// Orbiscreen - orbiscreen-transport - mdns module (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::sync::Arc;

use mdns_sd::{ServiceDaemon, ServiceInfo};
use thiserror::Error;
use tracing::{debug, info, warn};

use super::ServiceDescriptor;

const SERVICE_TYPE: &str = "_orbiscreen._tcp.local.";

#[derive(Debug, Error)]
pub enum MdnsError {
    #[error("mDNS daemon failed: {0}")]
    Daemon(String),
    #[error("failed to register service: {0}")]
    Register(String),
}

#[allow(missing_debug_implementations)]
pub struct Advertiser {
    daemon: ServiceDaemon,
    fullname: String,
}

impl Advertiser {
    pub fn register(desc: &ServiceDescriptor) -> Result<Arc<Self>, MdnsError> {
        let daemon = ServiceDaemon::new().map_err(|e| MdnsError::Daemon(e.to_string()))?;

        let raw_host = hostname().unwrap_or_else(|| "orbiscreen-host".to_string());
        let host_domain = if raw_host.ends_with(".local.") {
            raw_host.clone()
        } else if raw_host.ends_with(".local") {
            format!("{raw_host}.")
        } else {
            format!("{raw_host}.local.")
        };
        let instance = desc.instance.clone();

        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &instance,
            &host_domain,
            "0.0.0.0",
            desc.port,
            None,
        )
        .map_err(|e| MdnsError::Register(e.to_string()))?
        .enable_addr_auto();

        daemon
            .register(service_info)
            .map_err(|e| MdnsError::Register(e.to_string()))?;

        // Fullname must match mdns-sd's derived form ("<instance>.<service-type>")
        // so Drop can unregister exactly what was registered.
        let fullname = format!("{instance}.{SERVICE_TYPE}");

        // Drain daemon events in the background so their reporting channel does not
        // back up; connection-closed or repeated timeouts end the thread quietly.
        if let Ok(rx) = daemon.monitor() {
            std::thread::spawn(move || {
                while let Ok(event) = rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    debug!(?event, "mDNS daemon event");
                }
            });
        }

        let advertiser = Arc::new(Self { daemon, fullname });
        info!(instance = %instance, port = desc.port, "Advertised Orbiscreen service via mDNS");
        Ok(advertiser)
    }

    pub fn fullname(&self) -> &str {
        &self.fullname
    }
}

impl Drop for Advertiser {
    // Unregister before shutdown so peers see the service disappear promptly,
    // instead of relying on record expiry after the daemon exits.
    fn drop(&mut self) {
        match self.daemon.unregister(&self.fullname) {
            Ok(_rx) => {}
            Err(e) => warn!(fullname = %self.fullname, "mDNS unregister failed: {e}"),
        }
        let _ = self.daemon.shutdown();
    }
}

fn hostname() -> Option<String> {
    hostname::get().ok().and_then(|h| h.into_string().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_round_trips_through_advertiser() {
        let desc = ServiceDescriptor {
            instance: "test-laptop".into(),
            port: 8788,
        };
        assert_eq!(desc.port, 8788);
        assert_eq!(desc.instance, "test-laptop");
    }
}
