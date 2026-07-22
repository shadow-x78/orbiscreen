// Orbiscreen — orbiscreen-input library (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

pub mod wayland;
pub mod x11;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerEvent {
    Move { x: f64, y: f64 },
    Button { button: u32, pressed: bool },
    Wheel { delta_y: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: u32,
    pub pressed: bool,
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
}
