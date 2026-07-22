// Orbiscreen — orbiscreen-core library (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub display: DisplayConfig,
    pub encode: EncodeConfig,
    pub transport: TransportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisplayConfig {
    pub width: u32,
    pub height: u32,
    pub refresh_rate_hz: u32,
    pub count: u32,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            refresh_rate_hz: 60,
            count: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncodeConfig {
    pub bitrate_kbps: u32,
    pub preferred_encoder: String,
}

impl Default for EncodeConfig {
    fn default() -> Self {
        Self {
            bitrate_kbps: 8000,
            preferred_encoder: "x264".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransportConfig {
    pub signaling_port: u16,
    pub webrtc_port_range: (u16, u16),
    pub mdns_advertise: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            signaling_port: 8788,
            webrtc_port_range: (50_000, 50_100),
            mdns_advertise: true,
        }
    }
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("failed to parse configuration: {0}")]
    ConfigParse(#[from] toml::de::Error),
    #[error("failed to serialize configuration: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),
}

pub fn load_config(toml_str: &str) -> Result<Config, CoreError> {
    Ok(toml::from_str(toml_str)?)
}

pub fn dump_config(config: &Config) -> Result<String, CoreError> {
    Ok(toml::to_string_pretty(config)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrips_through_toml() {
        let config = Config::default();
        let serialized = toml::to_string(&config).expect("serialize default config");
        let parsed = load_config(&serialized).expect("parse default config");
        assert_eq!(config, parsed);
    }

    #[test]
    fn default_display_is_1080p60() {
        let display = DisplayConfig::default();
        assert_eq!(display.width, 1920);
        assert_eq!(display.height, 1080);
        assert_eq!(display.refresh_rate_hz, 60);
        assert_eq!(display.count, 1);
    }
}
