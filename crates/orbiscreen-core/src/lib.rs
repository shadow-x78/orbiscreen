// Orbiscreen - orbiscreen-core library (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub encode: EncodeConfig,
    #[serde(default)]
    pub transport: TransportConfig,
}

impl Config {
    /// Clamp all sections to their valid ranges; call after [`load_config`].
    pub fn sanitize(&mut self) {
        self.display.sanitize();
        self.encode.sanitize();
        self.transport.sanitize();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
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

impl DisplayConfig {
    /// Minimum resolution (QVGA) and refresh rate a display/encoder can be
    /// driven at; `MAX_REFRESH_RATE_HZ` caps entries beyond any consumer panel.
    pub const MIN_WIDTH: u32 = 320;
    pub const MIN_HEIGHT: u32 = 240;
    pub const MIN_REFRESH_RATE_HZ: u32 = 1;
    pub const MAX_REFRESH_RATE_HZ: u32 = 480;

    /// Clamp to the supported ranges; degenerate values (0 fps/dimensions)
    /// would divide-by-zero or fail downstream in capture and encode.
    pub fn sanitize(&mut self) {
        self.width = self.width.max(Self::MIN_WIDTH);
        self.height = self.height.max(Self::MIN_HEIGHT);
        self.refresh_rate_hz = self
            .refresh_rate_hz
            .clamp(Self::MIN_REFRESH_RATE_HZ, Self::MAX_REFRESH_RATE_HZ);
        self.count = self.count.max(1);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
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

impl EncodeConfig {
    /// Bitrate limits in kbit/s, matching the `u32` property ranges of
    /// x264enc/vaapih264enc/nvh264enc (100 kbit/s .. 100 Mbit/s).
    pub const MIN_BITRATE_KBPS: u32 = 100;
    pub const MAX_BITRATE_KBPS: u32 = 100_000;

    /// Clamp bitrate and fall back to x264 for unknown encoder names.
    pub fn sanitize(&mut self) {
        self.bitrate_kbps = self
            .bitrate_kbps
            .clamp(Self::MIN_BITRATE_KBPS, Self::MAX_BITRATE_KBPS);
        if !matches!(
            self.preferred_encoder.to_ascii_lowercase().as_str(),
            "vaapi" | "nvenc" | "x264"
        ) {
            self.preferred_encoder = "x264".to_string();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TransportConfig {
    pub signaling_port: u16,
    /// Unused placeholder kept for config-file compatibility: WebRTC was
    /// removed in favor of MPEG-TS over HTTP, and no ports are bound from
    /// this range.
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

impl TransportConfig {
    /// Fallback signaling port used when the configured port is privileged
    /// (and cannot be bound without root) or zero (ephemeral).
    pub const DEFAULT_SIGNALING_PORT: u16 = 8788;

    /// Replace unusable ports (0 or privileged) and re-order an inverted
    /// WebRTC port range.
    pub fn sanitize(&mut self) {
        if self.signaling_port == 0 || self.signaling_port < 1024 {
            self.signaling_port = Self::DEFAULT_SIGNALING_PORT;
        }
        let (lo, hi) = self.webrtc_port_range;
        if lo > hi {
            self.webrtc_port_range = (hi, lo);
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

/// Parse a TOML config and clamp every field to its valid range.
pub fn load_config(toml_str: &str) -> Result<Config, CoreError> {
    let mut config: Config = toml::from_str(toml_str)?;
    config.sanitize();
    Ok(config)
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
    fn partial_toml_falls_back_to_defaults() {
        let parsed = load_config("[display]\nwidth = 2560\n").expect("parse partial config");
        assert_eq!(parsed.display.width, 2560);
        assert_eq!(parsed.display.height, 1080);
        assert_eq!(parsed.encode, EncodeConfig::default());
        assert_eq!(parsed.transport, TransportConfig::default());
    }

    #[test]
    fn out_of_range_config_is_clamped() {
        let toml = "\
[display]
width = 0
height = 0
refresh_rate_hz = 0
count = 0
[encode]
bitrate_kbps = 500000
preferred_encoder = \"vp9\"
[transport]
signaling_port = 80
webrtc_port_range = [50100, 50000]
mdns_advertise = true
";
        let cfg = load_config(toml).expect("parse bad config");
        assert_eq!(cfg.display.width, DisplayConfig::MIN_WIDTH);
        assert_eq!(cfg.display.height, DisplayConfig::MIN_HEIGHT);
        assert_eq!(cfg.display.refresh_rate_hz, 1);
        assert_eq!(cfg.display.count, 1);
        assert_eq!(cfg.encode.bitrate_kbps, EncodeConfig::MAX_BITRATE_KBPS);
        assert_eq!(cfg.encode.preferred_encoder, "x264");
        assert_eq!(cfg.transport.signaling_port, 8788);
        assert_eq!(cfg.transport.webrtc_port_range, (50_000, 50_100));
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
