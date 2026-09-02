// Orbiscreen - lib.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
pub mod wayland;
pub mod wlroots;
pub mod x11;
pub mod xtest;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PointerEvent {
    Move { x: f64, y: f64 },
    Button { button: u32, pressed: bool },
    Wheel { delta_y: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StylusEvent {
    Proximity {},
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
}

#[derive(Debug, Clone)]
pub struct VirtualTouchscreenSpec {
    pub width: u32,
    pub height: u32,
    pub output_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectorKind {
    Uinput,
    Xtest,
    Portal,
    Wlroots,
}

pub(crate) const MAX_WHEEL_STEPS: i32 = 12;

#[allow(missing_debug_implementations)]
enum InjectorInner {
    Uinput(Box<x11::UinputInjector>),
    Xtest(Box<xtest::XtestInjector>),
    Portal(Box<wayland::WaylandInjector>),
    Wlroots(Box<wlroots::WlrootsInjector>),
}

#[allow(missing_debug_implementations)]
pub struct InputInjector {
    inner: InjectorInner,
}

impl InputInjector {
    pub async fn open_async(spec: VirtualTouchscreenSpec) -> Result<Self, InputError> {
        match detect_backend() {
            InputBackend::X11 => {
                let spec_for_uinput = spec.clone();
                match xtest::XtestInjector::open(spec) {
                    Ok(injector) => {
                        info!("input injection via XTEST (rootless)");
                        Ok(Self {
                            inner: InjectorInner::Xtest(Box::new(injector)),
                        })
                    }
                    Err(e) => {
                        warn!("XTEST injection unavailable ({e}); falling back to uinput");
                        Ok(Self {
                            inner: InjectorInner::Uinput(Box::new(x11::UinputInjector::open(
                                spec_for_uinput,
                            )?)),
                        })
                    }
                }
            }
            InputBackend::Wayland => {
                let spec_for_uinput = spec.clone();
                if let Ok(injector) = x11::UinputInjector::open(spec_for_uinput) {
                    info!(
                        "input injection via kernel uinput device (accessible via user ACL/group)"
                    );
                    return Ok(Self {
                        inner: InjectorInner::Uinput(Box::new(injector)),
                    });
                }

                let spec_for_wlr = spec.clone();
                let wlr_result = tokio::task::spawn_blocking(move || {
                    wlroots::WlrootsInjector::open(spec_for_wlr)
                })
                .await
                .map_err(|e| InputError::Uinput(format!("wlroots input task: {e}")))?;
                if let Ok(injector) = wlr_result {
                    info!("input injection via wlroots virtual protocols");
                    return Ok(Self {
                        inner: InjectorInner::Wlroots(Box::new(injector)),
                    });
                }

                info!("falling back to RemoteDesktop portal");
                match tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    wayland::WaylandInjector::open(),
                )
                .await
                {
                    Ok(Ok(injector)) => Ok(Self {
                        inner: InjectorInner::Portal(Box::new(injector)),
                    }),
                    Ok(Err(portal_error)) => {
                        warn!("portal RemoteDesktop failed: {portal_error}");
                        Err(InputError::Uinput(
                            "no usable input injector found (uinput and portal both failed)".into(),
                        ))
                    }
                    Err(_) => {
                        warn!("portal RemoteDesktop timed out after 5s");
                        Err(InputError::Uinput("portal RemoteDesktop timed out".into()))
                    }
                }
            }
        }
    }

    pub fn backend(&self) -> InjectorKind {
        match &self.inner {
            InjectorInner::Uinput(_) => InjectorKind::Uinput,
            InjectorInner::Xtest(_) => InjectorKind::Xtest,
            InjectorInner::Portal(_) => InjectorKind::Portal,
            InjectorInner::Wlroots(_) => InjectorKind::Wlroots,
        }
    }

    pub async fn inject_pointer(&mut self, event: PointerEvent) -> Result<(), InputError> {
        match &mut self.inner {
            InjectorInner::Uinput(injector) => injector.inject_pointer(event),
            InjectorInner::Xtest(injector) => injector.inject_pointer(event),
            InjectorInner::Portal(injector) => injector.inject_pointer(event).await,
            InjectorInner::Wlroots(injector) => injector.inject_pointer(event).await,
        }
    }

    pub async fn inject_key(&mut self, event: KeyEvent) -> Result<(), InputError> {
        match &mut self.inner {
            InjectorInner::Uinput(injector) => injector.inject_key(event.code, event.pressed),
            InjectorInner::Xtest(injector) => injector.inject_key(event.code, event.pressed),
            InjectorInner::Portal(injector) => injector.inject_key(event).await,
            InjectorInner::Wlroots(injector) => injector.inject_key(event).await,
        }
    }

    pub async fn inject_stylus(&mut self, event: StylusEvent) -> Result<(), InputError> {
        match event {
            StylusEvent::Pressure { x, y, .. } | StylusEvent::Tilt { x, y, .. } => {
                match &mut self.inner {
                    InjectorInner::Uinput(injector) => injector.inject_stylus(event),
                    _ => self.inject_pointer(PointerEvent::Move { x, y }).await,
                }
            }
            StylusEvent::Proximity {} => match &mut self.inner {
                InjectorInner::Uinput(injector) => injector.inject_stylus(event),
                _ => Ok(()),
            },
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
        assert_eq!(x11::button_code(4), 0x113);
        assert_eq!(x11::button_code(5), 0x114);
        assert_eq!(x11::button_code(6), 0x115);
        assert_eq!(x11::button_code(7), 0x116);
        assert_eq!(x11::button_code(8), 0x117);
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
        for button in 1..=8 {
            let code = x11::button_code(button);
            assert!(
                (0x110..=0x117).contains(&code),
                "button {button} maps to {code:#x}, outside registered key range"
            );
        }
    }

    #[test]
    fn key_event_json_uses_snake_case_field_names() {
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
