// Orbiscreen - orbiscreen-input library (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

pub mod wayland;
pub mod x11;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PointerEvent {
    Move { x: f64, y: f64 },
    Button { button: u32, pressed: bool },
    Wheel { delta_y: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StylusEvent {
    Proximity {
        in_range: bool,
    },
    Pressure {
        x: f64,
        y: f64,
        pressure: f64,
    },
    Tilt {
        x: f64,
        y: f64,
        pressure: f64,
        tilt_x_deg: f64,
        tilt_y_deg: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEvent {
    pub code: u32,
    pub pressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchCalibration {
    pub display_width: u32,
    pub display_height: u32,
}

impl TouchCalibration {
    pub fn new(display_width: u32, display_height: u32) -> Self {
        Self {
            display_width,
            display_height,
        }
    }

    pub fn clamp_and_scale(&self, norm_x: f64, norm_y: f64) -> (f64, f64) {
        let x = norm_x.clamp(0.0, 1.0) * (self.display_width as f64);
        let y = norm_y.clamp(0.0, 1.0) * (self.display_height as f64);
        (x, y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputBackend {
    X11,
    Wayland,
}

pub fn detect_backend() -> InputBackend {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        InputBackend::Wayland
    } else {
        InputBackend::X11
    }
}

#[derive(Debug, Error)]
pub enum InputError {
    #[error("uinput error: {0}")]
    Uinput(String),
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub struct VirtualTouchscreenSpec {
    pub width: u32,
    pub height: u32,
}

#[allow(missing_debug_implementations)]
pub struct InputInjector {
    backend: InputBackend,
    x11: Option<x11::UinputInjector>,
    wayland: Option<wayland::WaylandInjector>,
}

impl InputInjector {
    pub fn open(spec: VirtualTouchscreenSpec) -> Result<Self, InputError> {
        match detect_backend() {
            InputBackend::X11 => Ok(Self {
                backend: InputBackend::X11,
                x11: Some(x11::UinputInjector::open(spec)?),
                wayland: None,
            }),
            InputBackend::Wayland => Err(InputError::NotImplemented(
                "Wayland input requires open_async",
            )),
        }
    }

    pub async fn open_async(spec: VirtualTouchscreenSpec) -> Result<Self, InputError> {
        match detect_backend() {
            InputBackend::X11 => Self::open(spec),
            InputBackend::Wayland => {
                let wayland = wayland::WaylandInjector::open().await?;
                Ok(Self {
                    backend: InputBackend::Wayland,
                    x11: None,
                    wayland: Some(wayland),
                })
            }
        }
    }

    pub fn backend(&self) -> InputBackend {
        self.backend
    }

    pub async fn inject_pointer(&mut self, event: PointerEvent) -> Result<(), InputError> {
        match (&mut self.x11, &mut self.wayland) {
            (Some(dev), _) => dev.inject_pointer(event),
            (_, Some(dev)) => dev.inject_pointer(event).await,
            (None, None) => Err(InputError::NotImplemented("no input backend open")),
        }
    }

    pub async fn inject_key(&mut self, event: KeyEvent) -> Result<(), InputError> {
        match (&mut self.x11, &mut self.wayland) {
            (Some(dev), _) => dev.inject_key(event.code, event.pressed),
            (_, Some(dev)) => dev.inject_key(event).await,
            (None, None) => Err(InputError::NotImplemented("no input backend open")),
        }
    }

    pub async fn inject_stylus(&mut self, event: StylusEvent) -> Result<(), InputError> {
        if let Some(dev) = self.x11.as_mut() {
            return dev.inject_stylus(event);
        }
        match event {
            StylusEvent::Pressure { x, y, .. } | StylusEvent::Tilt { x, y, .. } => {
                self.inject_pointer(PointerEvent::Move { x, y }).await
            }
            StylusEvent::Proximity { .. } => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_events_are_distinct_variants() {
        let a = PointerEvent::Move { x: 1.0, y: 2.0 };
        let b = PointerEvent::Button {
            button: 1,
            pressed: true,
        };
        assert_ne!(format!("{a:?}"), format!("{b:?}"));
    }

    #[test]
    fn key_event_carries_code_and_pressed() {
        let key = KeyEvent {
            code: 30,
            pressed: true,
        };
        assert_eq!(key.code, 30);
        assert!(key.pressed);
    }

    #[test]
    fn detect_prefers_wayland_when_present() {
        let prev = std::env::var_os("WAYLAND_DISPLAY");
        std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
        assert_eq!(detect_backend(), InputBackend::Wayland);
        match prev {
            Some(value) => std::env::set_var("WAYLAND_DISPLAY", value),
            None => std::env::remove_var("WAYLAND_DISPLAY"),
        }
    }

    #[test]
    fn button_codes_map_to_linux_buttons() {
        assert_eq!(x11::button_code(1), 0x110);
        assert_eq!(x11::button_code(2), 0x112);
        assert_eq!(x11::button_code(3), 0x111);
    }

    #[test]
    fn stylus_pressure_event_carries_pressure() {
        let event = StylusEvent::Pressure {
            x: 100.0,
            y: 200.0,
            pressure: 0.75,
        };
        let StylusEvent::Pressure { pressure, x, y } = event else {
            panic!("expected Pressure variant");
        };
        assert_eq!(x, 100.0);
        assert_eq!(y, 200.0);
        assert_eq!(pressure, 0.75);
    }

    #[test]
    fn stylus_tilt_event_carries_both_tilt_axes() {
        let event = StylusEvent::Tilt {
            x: 50.0,
            y: 60.0,
            pressure: 0.3,
            tilt_x_deg: 5.0,
            tilt_y_deg: -3.0,
        };
        let StylusEvent::Tilt {
            tilt_x_deg,
            tilt_y_deg,
            ..
        } = event
        else {
            panic!("expected Tilt variant");
        };
        assert_eq!(tilt_x_deg, 5.0);
        assert_eq!(tilt_y_deg, -3.0);
    }

    #[test]
    fn button_codes_stay_within_registered_keys() {
        // uinput drops events for keys never passed to with_keys(); the
        // device registers KEY_ESC..=KEY_KPDOT (1..=83) and BTN_LEFT..=BTN_TASK
        // (0x110..=0x117), so button_code must stay inside those ranges.
        for button in 1..=8 {
            let code = x11::button_code(button);
            assert!(
                (0x110..=0x117).contains(&code),
                "button {button} maps to {code:#x}, outside registered key range"
            );
        }
    }

    #[test]
    fn touch_calibration_clamps_and_scales() {
        let cal = TouchCalibration::new(1920, 1080);
        assert_eq!(cal.clamp_and_scale(0.5, 0.5), (960.0, 540.0));
        assert_eq!(cal.clamp_and_scale(-0.1, 1.2), (0.0, 1080.0));
    }

    #[test]
    fn key_event_json_uses_snake_case_field_names() {
        // The web client sends {"Key": {"code": ..., "pressed": ...}}.
        let json = serde_json::json!({"Key": {"code": 30, "pressed": true}});
        #[derive(serde::Deserialize)]
        struct W {
            #[serde(rename = "Key")]
            key: KeyEvent,
        }
        let w: W = serde_json::from_value(json).expect("key payload");
        assert_eq!(w.key.code, 30);
        assert!(w.key.pressed);
    }
}
