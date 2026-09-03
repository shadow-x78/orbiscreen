// Orbiscreen - lib.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

pub mod frame_pool;
pub mod portal_state;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub capture: CaptureConfig,
    #[serde(default)]
    pub encode: EncodeConfig,
    #[serde(default)]
    pub transport: TransportConfig,
}

impl Config {
    pub fn sanitize(&mut self) {
        self.display.sanitize();
        self.capture.sanitize();
        self.encode.sanitize();
        self.transport.sanitize();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct CaptureConfig {
    pub preferred: String,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            preferred: "auto".to_string(),
        }
    }
}

impl CaptureConfig {
    const PREFERENCES: &'static [&'static str] = &[
        "auto",
        "kwin-virtual",
        "screencopy",
        "portal",
        "evdi",
        "mirror",
    ];

    pub fn sanitize(&mut self) {
        if !Self::PREFERENCES.contains(&self.preferred.as_str()) {
            self.preferred = "auto".to_string();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DisplayConfig {
    pub width: u32,
    pub height: u32,
    pub refresh_rate_hz: u32,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            refresh_rate_hz: 60,
        }
    }
}

impl DisplayConfig {
    pub const MIN_WIDTH: u32 = 320;
    pub const MIN_HEIGHT: u32 = 240;
    pub const MAX_WIDTH: u32 = 7680;
    pub const MAX_HEIGHT: u32 = 4320;
    pub const MIN_REFRESH_RATE_HZ: u32 = 1;
    pub const MAX_REFRESH_RATE_HZ: u32 = 480;

    pub fn sanitize(&mut self) {
        self.width = self.width.clamp(Self::MIN_WIDTH, Self::MAX_WIDTH);
        self.height = self.height.clamp(Self::MIN_HEIGHT, Self::MAX_HEIGHT);
        self.refresh_rate_hz = self
            .refresh_rate_hz
            .clamp(Self::MIN_REFRESH_RATE_HZ, Self::MAX_REFRESH_RATE_HZ);
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
            preferred_encoder: "auto".to_string(),
        }
    }
}

impl EncodeConfig {
    pub const MIN_BITRATE_KBPS: u32 = 100;
    pub const MAX_BITRATE_KBPS: u32 = 100_000;

    pub fn sanitize(&mut self) {
        self.bitrate_kbps = self
            .bitrate_kbps
            .clamp(Self::MIN_BITRATE_KBPS, Self::MAX_BITRATE_KBPS);
        if !matches!(
            self.preferred_encoder.to_ascii_lowercase().as_str(),
            "auto" | "nvenc" | "vaapi" | "x264"
        ) {
            self.preferred_encoder = "auto".to_string();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TransportConfig {
    pub signaling_port: u16,
    pub mdns_advertise: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            signaling_port: 8788,
            mdns_advertise: true,
        }
    }
}

impl TransportConfig {
    pub const DEFAULT_SIGNALING_PORT: u16 = 8788;

    pub fn sanitize(&mut self) {
        if self.signaling_port < 1024 {
            self.signaling_port = Self::DEFAULT_SIGNALING_PORT;
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
    let mut config: Config = toml::from_str(toml_str)?;
    config.sanitize();
    Ok(config)
}

pub fn dump_config(config: &Config) -> Result<String, CoreError> {
    Ok(toml::to_string_pretty(config)?)
}

pub fn default_config_path() -> std::path::PathBuf {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty() && std::path::Path::new(v).is_absolute())
    {
        return std::path::PathBuf::from(xdg).join("orbiscreen/orbiscreen.toml");
    }
    if let Some(home) = std::env::var_os("HOME").filter(|v| !v.is_empty()) {
        return std::path::PathBuf::from(home).join(".config/orbiscreen/orbiscreen.toml");
    }
    std::path::PathBuf::from("orbiscreen.toml")
}

pub fn default_token_path() -> std::path::PathBuf {
    default_config_path().with_file_name("token")
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
[encode]
bitrate_kbps = 500000
preferred_encoder = \"vp9\"
[transport]
signaling_port = 80
mdns_advertise = true
";
        let cfg = load_config(toml).expect("parse bad config");
        assert_eq!(cfg.display.width, DisplayConfig::MIN_WIDTH);
        assert_eq!(cfg.display.height, DisplayConfig::MIN_HEIGHT);
        assert_eq!(cfg.display.refresh_rate_hz, 1);
        assert_eq!(cfg.encode.bitrate_kbps, EncodeConfig::MAX_BITRATE_KBPS);
        assert_eq!(cfg.encode.preferred_encoder, "auto");
        assert_eq!(cfg.transport.signaling_port, 8788);
    }

    #[test]
    fn oversized_display_is_clamped_to_maximums() {
        let toml = "\
[display]
width = 999999
height = 999999
";
        let cfg = load_config(toml).expect("parse oversized config");
        assert_eq!(cfg.display.width, DisplayConfig::MAX_WIDTH);
        assert_eq!(cfg.display.height, DisplayConfig::MAX_HEIGHT);
    }

    #[test]
    fn legacy_webrtc_port_range_key_is_ignored() {
        let toml = "[transport]\nwebrtc_port_range = [50100, 50000]\n";
        let cfg = load_config(toml).expect("legacy key must not break parsing");
        assert_eq!(cfg.transport, TransportConfig::default());
    }

    #[test]
    fn default_display_is_1080p60() {
        let display = DisplayConfig::default();
        assert_eq!(display.width, 1920);
        assert_eq!(display.height, 1080);
        assert_eq!(display.refresh_rate_hz, 60);
    }

    #[test]
    fn unknown_capture_preference_falls_back_to_auto() {
        let parsed =
            load_config("[capture]\npreferred = \"magic\"\n").expect("parse capture config");
        assert_eq!(parsed.capture.preferred, "auto");
    }

    #[test]
    fn capture_preference_accepts_known_values() {
        for value in CaptureConfig::PREFERENCES {
            let parsed =
                load_config(&format!("[capture]\npreferred = \"{value}\"\n")).expect("parse");
            assert_eq!(parsed.capture.preferred, *value);
        }
    }

    #[test]
    fn default_config_path_uses_xdg_config_home_when_set() {
        let prev = std::env::var_os("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/orbiscreen-test-xdg");
        let path = default_config_path();
        assert_eq!(
            path,
            std::path::PathBuf::from("/tmp/orbiscreen-test-xdg/orbiscreen/orbiscreen.toml")
        );
        match prev {
            Some(value) => std::env::set_var("XDG_CONFIG_HOME", value),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
    }
}
