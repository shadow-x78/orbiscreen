// Orbiscreen - capabilities.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Wayland,
    X11,
    Unknown,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wayland => write!(f, "wayland"),
            Self::X11 => write!(f, "x11"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compositor {
    Kde,
    Gnome,
    Cosmic,
    Hyprland,
    Sway,
    Wayfire,
    Labwc,
    River,
    Gamescope,
    OtherWayland,
    X11,
    Unknown,
}

impl fmt::Display for Compositor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kde => write!(f, "KDE Plasma"),
            Self::Gnome => write!(f, "GNOME (Mutter)"),
            Self::Cosmic => write!(f, "COSMIC (cosmic-comp)"),
            Self::Hyprland => write!(f, "Hyprland"),
            Self::Sway => write!(f, "Sway"),
            Self::Wayfire => write!(f, "Wayfire"),
            Self::Labwc => write!(f, "Labwc"),
            Self::River => write!(f, "river"),
            Self::Gamescope => write!(f, "Gamescope"),
            Self::OtherWayland => write!(f, "other Wayland compositor"),
            Self::X11 => write!(f, "X11"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    pub session: SessionType,
    pub compositor: Compositor,
    pub current_desktop: Option<String>,
}

impl Capabilities {
    pub fn from_env() -> Self {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    pub fn from_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        let xdg_session = lookup("XDG_SESSION_TYPE")
            .map(|v| v.to_ascii_lowercase())
            .unwrap_or_default();
        let wayland_display = lookup("WAYLAND_DISPLAY");
        let x_display = lookup("DISPLAY");
        let current_desktop = lookup("XDG_CURRENT_DESKTOP").filter(|v| !v.trim().is_empty());

        let session = match (xdg_session.as_str(), &wayland_display, &x_display) {
            ("wayland", _, _) | ("mir", _, _) => SessionType::Wayland,
            ("x11", _, _) | ("tty", _, _) => SessionType::X11,
            (_, Some(_), _) => SessionType::Wayland,
            (_, None, Some(_)) => SessionType::X11,
            _ => SessionType::Unknown,
        };

        let compositor = match session {
            SessionType::Wayland => {
                let desktop = current_desktop.as_deref().unwrap_or("");
                let desktop_tokens: Vec<String> = desktop
                    .split(':')
                    .map(|t| t.trim().to_ascii_lowercase())
                    .filter(|t| !t.is_empty())
                    .collect();
                if lookup("HYPRLAND_INSTANCE_SIGNATURE").is_some_and(|v| !v.trim().is_empty()) {
                    Compositor::Hyprland
                } else if lookup("SWAYSOCK").is_some_and(|v| !v.trim().is_empty()) {
                    Compositor::Sway
                } else if lookup("GAMESCOPE_WAYLAND_DISPLAY").is_some() {
                    Compositor::Gamescope
                } else if desktop_tokens.iter().any(|t| t == "kde")
                    || (desktop_tokens.is_empty()
                        && lookup("KDE_FULL_SESSION").as_deref() == Some("true"))
                {
                    Compositor::Kde
                } else if desktop_tokens
                    .iter()
                    .any(|t| t == "gnome" || t.contains("unity"))
                {
                    Compositor::Gnome
                } else if desktop_tokens.iter().any(|t| t == "cosmic")
                    || lookup("XDG_SESSION_DESKTOP").as_deref() == Some("cosmic")
                    || lookup("COSMIC_DATA_CONTROL").is_some()
                {
                    Compositor::Cosmic
                } else if desktop_tokens.iter().any(|t| t == "wayfire") {
                    Compositor::Wayfire
                } else if desktop_tokens.iter().any(|t| t == "labwc") {
                    Compositor::Labwc
                } else if desktop_tokens.iter().any(|t| t == "river") {
                    Compositor::River
                } else {
                    Compositor::OtherWayland
                }
            }
            SessionType::X11 => Compositor::X11,
            SessionType::Unknown => Compositor::Unknown,
        };

        Self {
            session,
            compositor,
            current_desktop,
        }
    }

    pub fn is_wlroots(&self) -> bool {
        matches!(
            self.compositor,
            Compositor::Sway
                | Compositor::Hyprland
                | Compositor::Wayfire
                | Compositor::Labwc
                | Compositor::River
                | Compositor::Gamescope
        )
    }

    pub fn auto_chain(&self) -> Vec<CaptureStep> {
        match self.session {
            SessionType::Wayland => match self.compositor {
                Compositor::Kde => vec![CaptureStep::KwinVirtual, CaptureStep::Portal],
                Compositor::Cosmic => vec![CaptureStep::Evdi, CaptureStep::Portal],
                Compositor::Sway
                | Compositor::Hyprland
                | Compositor::Wayfire
                | Compositor::Labwc
                | Compositor::River
                | Compositor::Gamescope => vec![
                    CaptureStep::WlrootsVirtual,
                    CaptureStep::WlrScreencopy,
                    CaptureStep::Portal,
                    CaptureStep::Evdi,
                ],
                _ => vec![CaptureStep::Portal],
            },
            SessionType::X11 => vec![CaptureStep::Evdi, CaptureStep::X11Root],
            SessionType::Unknown => match self.current_desktop {
                Some(_) => vec![CaptureStep::Portal, CaptureStep::X11Root],
                None => vec![CaptureStep::Evdi, CaptureStep::X11Root],
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureStep {
    KwinVirtual,
    WlrootsVirtual,
    WlrScreencopy,
    Portal,
    Evdi,
    X11Root,
}

impl fmt::Display for CaptureStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KwinVirtual => write!(f, "kwin-virtual"),
            Self::WlrootsVirtual => write!(f, "wlroots-virtual"),
            Self::WlrScreencopy => write!(f, "wlr-screencopy"),
            Self::Portal => write!(f, "portal"),
            Self::Evdi => write!(f, "evdi"),
            Self::X11Root => write!(f, "x11-root"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn caps(vars: &[(&str, &str)]) -> Capabilities {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        Capabilities::from_lookup(|key| map.get(key).cloned())
    }

    #[test]
    fn kde_wayland_is_detected() {
        let c = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("WAYLAND_DISPLAY", "wayland-0"),
            ("XDG_CURRENT_DESKTOP", "KDE"),
        ]);
        assert_eq!(c.session, SessionType::Wayland);
        assert_eq!(c.compositor, Compositor::Kde);
    }

    #[test]
    fn kde_via_full_session_flag() {
        let c = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("KDE_FULL_SESSION", "true"),
        ]);
        assert_eq!(c.compositor, Compositor::Kde);
    }

    #[test]
    fn gnome_wayland_is_detected() {
        let c = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("XDG_CURRENT_DESKTOP", "ubuntu:GNOME"),
        ]);
        assert_eq!(c.session, SessionType::Wayland);
        assert_eq!(c.compositor, Compositor::Gnome);
    }

    #[test]
    fn cosmic_wayland_is_detected() {
        let c = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("WAYLAND_DISPLAY", "wayland-0"),
            ("XDG_CURRENT_DESKTOP", "COSMIC"),
        ]);
        assert_eq!(c.session, SessionType::Wayland);
        assert_eq!(c.compositor, Compositor::Cosmic);
        assert!(!c.is_wlroots());
    }

    #[test]
    fn cosmic_pop_is_detected() {
        let c = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("XDG_CURRENT_DESKTOP", "COSMIC:Pop"),
        ]);
        assert_eq!(c.compositor, Compositor::Cosmic);
        assert!(!c.is_wlroots());
    }

    #[test]
    fn cosmic_via_session_desktop() {
        let c = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("XDG_SESSION_DESKTOP", "cosmic"),
        ]);
        assert_eq!(c.compositor, Compositor::Cosmic);
    }

    #[test]
    fn cosmic_via_data_control() {
        let c = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("COSMIC_DATA_CONTROL", "1"),
        ]);
        assert_eq!(c.compositor, Compositor::Cosmic);
    }

    #[test]
    fn hyprland_wins_over_desktop_tokens() {
        let c = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("HYPRLAND_INSTANCE_SIGNATURE", "abc123"),
            ("XDG_CURRENT_DESKTOP", "Hyprland"),
        ]);
        assert_eq!(c.compositor, Compositor::Hyprland);
        assert!(c.is_wlroots());
    }

    #[test]
    fn sway_is_detected_from_socket() {
        let c = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("SWAYSOCK", "/run/user/1000/sway-ipc.sock"),
        ]);
        assert_eq!(c.compositor, Compositor::Sway);
        assert!(c.is_wlroots());
    }

    #[test]
    fn labwc_and_river_are_wlroots() {
        let labwc = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("XDG_CURRENT_DESKTOP", "Labwc"),
        ]);
        assert_eq!(labwc.compositor, Compositor::Labwc);
        assert!(labwc.is_wlroots());
        let river = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("XDG_CURRENT_DESKTOP", "river"),
        ]);
        assert_eq!(river.compositor, Compositor::River);
        assert!(river.is_wlroots());
    }

    #[test]
    fn x11_session_is_detected_with_or_without_xdg_type() {
        let with_type = caps(&[("XDG_SESSION_TYPE", "x11"), ("XDG_CURRENT_DESKTOP", "XFCE")]);
        assert_eq!(with_type.session, SessionType::X11);
        assert_eq!(with_type.compositor, Compositor::X11);
        let with_display_only = caps(&[("DISPLAY", ":0")]);
        assert_eq!(with_display_only.session, SessionType::X11);
    }

    #[test]
    fn wayland_display_alone_implies_wayland() {
        let c = caps(&[("WAYLAND_DISPLAY", "wayland-1")]);
        assert_eq!(c.session, SessionType::Wayland);
        assert_eq!(c.compositor, Compositor::OtherWayland);
    }

    #[test]
    fn empty_env_is_unknown() {
        let c = caps(&[]);
        assert_eq!(c.session, SessionType::Unknown);
        assert_eq!(c.compositor, Compositor::Unknown);
    }

    #[test]
    fn kde_chain_prefers_native_virtual_first() {
        let c = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("XDG_CURRENT_DESKTOP", "KDE"),
        ]);
        assert_eq!(
            c.auto_chain(),
            vec![CaptureStep::KwinVirtual, CaptureStep::Portal]
        );
    }

    #[test]
    fn gnome_chain_uses_portal_only() {
        let c = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("XDG_CURRENT_DESKTOP", "GNOME"),
        ]);
        assert_eq!(c.auto_chain(), vec![CaptureStep::Portal]);
    }

    #[test]
    fn cosmic_chain_prefers_evdi_then_portal() {
        let c = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("XDG_CURRENT_DESKTOP", "COSMIC"),
        ]);
        assert_eq!(c.auto_chain(), vec![CaptureStep::Evdi, CaptureStep::Portal]);
    }

    #[test]
    fn wlroots_chain_prefers_virtual_output_then_screencopy() {
        let expected = vec![
            CaptureStep::WlrootsVirtual,
            CaptureStep::WlrScreencopy,
            CaptureStep::Portal,
            CaptureStep::Evdi,
        ];
        let sway = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("SWAYSOCK", "/run/sway.sock"),
        ]);
        assert!(sway.is_wlroots());
        assert_eq!(sway.auto_chain(), expected);
        let hyprland = caps(&[
            ("XDG_SESSION_TYPE", "wayland"),
            ("HYPRLAND_INSTANCE_SIGNATURE", "instance"),
        ]);
        assert!(hyprland.is_wlroots());
        assert_eq!(hyprland.auto_chain(), expected);
    }

    #[test]
    fn x11_chain_tries_evdi_before_root_capture() {
        let c = caps(&[("XDG_SESSION_TYPE", "x11"), ("XDG_CURRENT_DESKTOP", "MATE")]);
        assert_eq!(
            c.auto_chain(),
            vec![CaptureStep::Evdi, CaptureStep::X11Root]
        );
    }

    #[test]
    fn capture_step_display_names_are_stable() {
        assert_eq!(CaptureStep::KwinVirtual.to_string(), "kwin-virtual");
        assert_eq!(CaptureStep::WlrootsVirtual.to_string(), "wlroots-virtual");
        assert_eq!(CaptureStep::WlrScreencopy.to_string(), "wlr-screencopy");
        assert_eq!(CaptureStep::Portal.to_string(), "portal");
        assert_eq!(CaptureStep::Evdi.to_string(), "evdi");
        assert_eq!(CaptureStep::X11Root.to_string(), "x11-root");
    }

    #[test]
    fn cosmic_display_name_is_expected() {
        assert_eq!(Compositor::Cosmic.to_string(), "COSMIC (cosmic-comp)");
    }
}
