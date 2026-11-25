use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct UsbReport {
    pub devices: Vec<String>,
    pub aoa_ready: bool,
    pub permission_denied: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisplayBackendKind {
    #[default]
    Unknown,
    NativeKwin,
    NativeWlroots,
    EvdiCompatible,
    EvdiOutdated,
    EvdiMissing,
    EvdiNoDeviceNode,
}

impl DisplayBackendKind {
    pub fn is_healthy(self) -> bool {
        matches!(
            self,
            Self::NativeKwin | Self::NativeWlroots | Self::EvdiCompatible
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DoctorReport {
    pub session: String,
    pub compositor: String,
    pub current_desktop: Option<String>,
    pub capture_plan: Vec<String>,
    pub display_backend: String,
    pub display_backend_kind: DisplayBackendKind,
    pub input_backend: String,
    pub uinput_writable: bool,
    pub portal_on_session_bus: Option<bool>,
    pub screencast_saved_token: bool,
    pub remote_desktop_saved_token: bool,
    pub swaymsg: bool,
    pub hyprctl: bool,
    pub cosmic_comp: bool,
    pub cosmic_randr: bool,
    pub wlroots_virtual_output_ipc: Option<String>,
    pub usb: UsbReport,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct StatusReport {
    pub running: bool,
    pub frames_forwarded: u64,
    pub active_clients: usize,
    pub total_clients: u64,
    pub auth_failures: u64,
    pub usb_devices: usize,
    pub usb_connected_devices: Vec<String>,
    pub usb_aoa_ready: bool,
    pub encoder: String,
    pub capture_backend: String,
    pub display_width: u32,
    pub display_height: u32,
    pub display_fps: u32,
    pub signaling_port: u16,
    pub version: String,
}

impl StatusReport {
    pub fn new() -> Self {
        Self {
            display_width: 1920,
            display_height: 1080,
            display_fps: 60,
            signaling_port: crate::ports::DEFAULT_SIGNALING_PORT,
            encoder: "auto".to_string(),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_report_parses_the_daemon_envelope() {
        let raw = r#"{
            "running": true,
            "frames_forwarded": 42,
            "active_clients": 1,
            "total_clients": 3,
            "auth_failures": 0,
            "usb_devices": 1,
            "usb_connected_devices": ["Tablet"],
            "usb_aoa_ready": true,
            "encoder": "vaapi",
            "capture_backend": "kwin-virtual",
            "display_width": 2560,
            "display_height": 1440,
            "display_fps": 120,
            "signaling_port": 8788,
            "version": "0.45.1"
        }"#;
        let report: StatusReport = serde_json::from_str(raw).expect("status");
        assert!(report.running);
        assert_eq!(report.display_width, 2560);
        assert_eq!(report.encoder, "vaapi");
    }

    #[test]
    fn doctor_report_defaults_missing_fields() {
        let report: DoctorReport =
            serde_json::from_str(r#"{"session":"wayland","usb":{"devices":[]}}"#).expect("doctor");
        assert_eq!(report.session, "wayland");
        assert_eq!(report.display_backend_kind, DisplayBackendKind::Unknown);
        assert!(!report.usb.aoa_ready);
    }

    #[test]
    fn display_backend_kind_serializes_as_snake_case() {
        let json = serde_json::to_string(&DisplayBackendKind::NativeKwin).unwrap();
        assert_eq!(json, "\"native_kwin\"");
    }
}
