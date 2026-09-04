// Orbiscreen - x11.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::io;

use evdevil::event::{
    Abs, AbsEvent, InputEvent, Key, KeyEvent as KEv, KeyState, Rel, RelEvent, Syn, SynEvent,
};
use evdevil::uinput::{AbsSetup, UinputDevice};
use evdevil::{AbsInfo, Bus, InputId, InputProp};
use tracing::info;

use super::{InputError, PointerEvent, StylusEvent, VirtualTouchscreenSpec};

const PRESSURE_MAX: i32 = 1024;
const TILT_MIN: i32 = -90;
const TILT_MAX: i32 = 90;

impl From<io::Error> for InputError {
    fn from(error: io::Error) -> Self {
        InputError::Uinput(error.to_string())
    }
}

#[allow(missing_debug_implementations)]
pub struct UinputInjector {
    mouse_keyboard: UinputDevice,
    touch_tablet: UinputDevice,
    width: u32,
    height: u32,
}

impl UinputInjector {
    pub fn open(spec: VirtualTouchscreenSpec) -> Result<Self, InputError> {
        let mut mk_keys: Vec<Key> = (1u16..=0x10F).map(Key::from_raw).collect();
        mk_keys.extend((0x110u16..=0x117).map(Key::from_raw));
        mk_keys.extend((0x160u16..=0x2FF).map(Key::from_raw));

        let width_axis = AbsInfo::new(0, spec.width.saturating_sub(1) as i32).with_resolution(10);
        let height_axis = AbsInfo::new(0, spec.height.saturating_sub(1) as i32).with_resolution(10);

        let mouse_keyboard = UinputDevice::builder()?
            .with_input_id(InputId::new(Bus::VIRTUAL, 0x0BEE, 0x0001, 0x0001))?
            .with_rel_axes([Rel::X, Rel::Y, Rel::WHEEL])?
            .with_keys(mk_keys)?
            .build("Orbiscreen Virtual Mouse and Keyboard")?;

        let pressure_axis = AbsInfo::new(0, PRESSURE_MAX).with_resolution(1);
        let tilt_axis = AbsInfo::new(TILT_MIN, TILT_MAX).with_resolution(1);

        let tablet_keys: Vec<Key> = (0x140u16..=0x14F).map(Key::from_raw).collect();

        let touch_tablet = UinputDevice::builder()?
            .with_input_id(InputId::new(Bus::VIRTUAL, 0x0BEE, 0x0002, 0x0001))?
            .with_props([InputProp::DIRECT])?
            .with_abs_axes([
                AbsSetup::new(Abs::X, width_axis),
                AbsSetup::new(Abs::Y, height_axis),
                AbsSetup::new(Abs::PRESSURE, pressure_axis),
                AbsSetup::new(Abs::TILT_X, tilt_axis),
                AbsSetup::new(Abs::TILT_Y, tilt_axis),
            ])?
            .with_keys(tablet_keys)?
            .build("Orbiscreen Virtual Touchscreen")?;

        info!("opened uinput devices: mouse/keyboard (relative) and touch/tablet (direct, resolution=10)");
        Ok(Self {
            mouse_keyboard,
            touch_tablet,
            width: spec.width,
            height: spec.height,
        })
    }

    fn clamp_point(&self, x: f64, y: f64) -> (i32, i32) {
        let cx = x.clamp(0.0, f64::from(self.width.saturating_sub(1))) as i32;
        let cy = y.clamp(0.0, f64::from(self.height.saturating_sub(1))) as i32;
        (cx, cy)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
        }
    }

    pub fn inject_pointer(&mut self, event: PointerEvent) -> Result<(), InputError> {
        match event {
            PointerEvent::Move { x, y } => {
                let (xi, yi) = self.clamp_point(x, y);
                let events = vec![
                    AbsEvent::new(Abs::X, xi).into(),
                    AbsEvent::new(Abs::Y, yi).into(),
                    SynEvent::new(Syn::REPORT).into(),
                ];
                self.touch_tablet.write_events(&events)?;
            }
            PointerEvent::RelativeMove { dx, dy } => {
                let r_dx = dx.round() as i32;
                let r_dy = dy.round() as i32;
                let events = vec![
                    RelEvent::new(Rel::X, r_dx).into(),
                    RelEvent::new(Rel::Y, r_dy).into(),
                    SynEvent::new(Syn::REPORT).into(),
                ];
                self.mouse_keyboard.write_events(&events)?;
            }
            PointerEvent::Button { button, pressed } => {
                if button == 0 || button > 8 {
                    return Err(InputError::Uinput(format!("invalid button: {button}")));
                }
                let code = button_code(button);
                let state = if pressed {
                    KeyState::PRESSED
                } else {
                    KeyState::RELEASED
                };
                let events = vec![
                    KEv::new(Key::from_raw(code as u16), state).into(),
                    SynEvent::new(Syn::REPORT).into(),
                ];
                self.mouse_keyboard.write_events(&events)?;
                if button == 1 {
                    let touch_events = vec![
                        KEv::new(Key::BTN_TOUCH, state).into(),
                        SynEvent::new(Syn::REPORT).into(),
                    ];
                    let _ = self.touch_tablet.write_events(&touch_events);
                }
            }
            PointerEvent::Wheel { delta_y } => {
                let mut events: Vec<InputEvent> = Vec::new();
                let steps = delta_y
                    .clamp(
                        -(crate::MAX_WHEEL_STEPS as f64),
                        crate::MAX_WHEEL_STEPS as f64,
                    )
                    .round() as i32;
                for _ in 0..steps.unsigned_abs() {
                    events.push(RelEvent::new(Rel::WHEEL, -steps.signum()).into());
                    events.push(SynEvent::new(Syn::REPORT).into());
                }
                self.mouse_keyboard.write_events(&events)?;
            }
        }
        Ok(())
    }

    pub fn inject_key(&mut self, code: u32, pressed: bool) -> Result<(), InputError> {
        if code == 0 || code > 0x2FF {
            return Err(InputError::Uinput(format!("invalid key code: {code}")));
        }
        let state = if pressed {
            KeyState::PRESSED
        } else {
            KeyState::RELEASED
        };
        let events = vec![
            KEv::new(Key::from_raw(code as u16), state).into(),
            SynEvent::new(Syn::REPORT).into(),
        ];
        self.mouse_keyboard.write_events(&events)?;
        Ok(())
    }

    pub fn inject_stylus(&mut self, event: StylusEvent) -> Result<(), InputError> {
        let (x, y, pressure, tilt) = match event {
            StylusEvent::Proximity { .. } => return Ok(()),
            StylusEvent::Pressure { x, y, pressure } => (x, y, pressure, None),
            StylusEvent::Tilt {
                x,
                y,
                pressure,
                tilt_x_deg,
                tilt_y_deg,
            } => (x, y, pressure, Some((tilt_x_deg, tilt_y_deg))),
        };
        let (xi, yi) = self.clamp_point(x, y);
        let pressure =
            (pressure * f64::from(PRESSURE_MAX)).clamp(0.0, f64::from(PRESSURE_MAX)) as i32;
        let touch_state = if pressure > 0 {
            KeyState::PRESSED
        } else {
            KeyState::RELEASED
        };
        let pen_state = if pressure > 0 {
            KeyState::PRESSED
        } else {
            KeyState::RELEASED
        };

        let mut events: Vec<InputEvent> = Vec::with_capacity(8);
        events.push(AbsEvent::new(Abs::X, xi).into());
        events.push(AbsEvent::new(Abs::Y, yi).into());
        events.push(AbsEvent::new(Abs::PRESSURE, pressure).into());
        events.push(KEv::new(Key::BTN_TOOL_PEN, pen_state).into());
        events.push(KEv::new(Key::BTN_TOUCH, touch_state).into());
        if let Some((tx, ty)) = tilt {
            let tx = tx.clamp(f64::from(TILT_MIN), f64::from(TILT_MAX)) as i32;
            let ty = ty.clamp(f64::from(TILT_MIN), f64::from(TILT_MAX)) as i32;
            events.push(AbsEvent::new(Abs::TILT_X, tx).into());
            events.push(AbsEvent::new(Abs::TILT_Y, ty).into());
        }
        events.push(SynEvent::new(Syn::REPORT).into());
        self.touch_tablet.write_events(&events)?;
        Ok(())
    }
}

pub fn button_code(button: u32) -> u32 {
    match button {
        1 => 0x110,
        2 => 0x112,
        3 => 0x111,
        n => n + 0x10F,
    }
}
