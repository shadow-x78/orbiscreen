// Orbiscreen — orbiscreen-input — x11 module (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::io;

use evdevil::event::{
    Abs, AbsEvent, InputEvent, Key, KeyEvent as KEv, KeyState, Rel, RelEvent, Syn, SynEvent,
};
use evdevil::uinput::{AbsSetup, UinputDevice};
use evdevil::{AbsInfo, Bus, InputId};
use tracing::info;

use super::{InputError, PointerEvent, StylusEvent, VirtualTouchscreenSpec};

const MAX_KEY_COUNT: usize = 200;
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
    device: UinputDevice,
    width: u32,
    height: u32,
}

impl UinputInjector {
    pub fn open(spec: VirtualTouchscreenSpec) -> Result<Self, InputError> {
        let width_axis = AbsInfo::new(0, spec.width.saturating_sub(1) as i32);
        let height_axis = AbsInfo::new(0, spec.height.saturating_sub(1) as i32);
        let pressure_axis = AbsInfo::new(0, PRESSURE_MAX);
        let tilt_axis = AbsInfo::new(TILT_MIN, TILT_MAX);

        let keys: Vec<Key> = (Key::KEY_ESC.raw()..=Key::KEY_KPDOT.raw())
            .take(MAX_KEY_COUNT)
            .map(Key::from_raw)
            .collect();

        let device = UinputDevice::builder()?
            .with_input_id(InputId::new(Bus::VIRTUAL, 0x0BEE, 0x0001, 0x0001))?
            .with_abs_axes([
                AbsSetup::new(Abs::X, width_axis),
                AbsSetup::new(Abs::Y, height_axis),
                AbsSetup::new(Abs::PRESSURE, pressure_axis),
                AbsSetup::new(Abs::TILT_X, tilt_axis),
                AbsSetup::new(Abs::TILT_Y, tilt_axis),
            ])?
            .with_keys(keys)?
            .build("Orbiscreen Virtual Touchscreen")?;
        info!("opened uinput device: orbiscreen virtual touchscreen");
        Ok(Self {
            device,
            width: spec.width,
            height: spec.height,
        })
    }

    fn clamp_point(&self, x: f64, y: f64) -> (i32, i32) {
        let cx = x.clamp(0.0, (self.width - 1) as f64) as i32;
        let cy = y.clamp(0.0, (self.height - 1) as f64) as i32;
        (cx, cy)
    }

    pub fn inject_pointer(&mut self, event: PointerEvent) -> Result<(), InputError> {
        let events: Vec<InputEvent> = match event {
            PointerEvent::Move { x, y } => {
                let (xi, yi) = self.clamp_point(x, y);
                vec![
                    AbsEvent::new(Abs::X, xi).into(),
                    AbsEvent::new(Abs::Y, yi).into(),
                    SynEvent::new(Syn::REPORT).into(),
                ]
            }
            PointerEvent::Button { button, pressed } => {
                let code = button_code(button);
                let state = if pressed {
                    KeyState::PRESSED
                } else {
                    KeyState::RELEASED
                };
                vec![
                    KEv::new(Key::from_raw(code as u16), state).into(),
                    SynEvent::new(Syn::REPORT).into(),
                ]
            }
            PointerEvent::Wheel { delta_y } => {
                vec![
                    RelEvent::new(Rel::WHEEL, delta_y as i32).into(),
                    SynEvent::new(Syn::REPORT).into(),
                ]
            }
        };
        self.device.write_events(&events)?;
        Ok(())
    }

    pub fn inject_key(&mut self, code: u32, pressed: bool) -> Result<(), InputError> {
        let state = if pressed {
            KeyState::PRESSED
        } else {
            KeyState::RELEASED
        };
        self.device.write_events(&[
            KEv::new(Key::from_raw(code as u16), state).into(),
            SynEvent::new(Syn::REPORT).into(),
        ])?;
        Ok(())
    }

    pub fn inject_stylus(&mut self, event: StylusEvent) -> Result<(), InputError> {
        if matches!(event, StylusEvent::Proximity { .. }) {
            return Ok(());
        }
        let (x, y, pressure, tilt) = match event {
            StylusEvent::Pressure { x, y, pressure } => (x, y, pressure, None),
            StylusEvent::Tilt {
                x,
                y,
                pressure,
                tilt_x_deg,
                tilt_y_deg,
            } => (x, y, pressure, Some((tilt_x_deg, tilt_y_deg))),
            StylusEvent::Proximity { .. } => unreachable!(),
        };
        let (xi, yi) = self.clamp_point(x, y);
        let pressure = (pressure * PRESSURE_MAX as f64).clamp(0.0, PRESSURE_MAX as f64) as i32;

        let mut events: Vec<InputEvent> = Vec::with_capacity(6);
        events.push(AbsEvent::new(Abs::X, xi).into());
        events.push(AbsEvent::new(Abs::Y, yi).into());
        events.push(AbsEvent::new(Abs::PRESSURE, pressure).into());
        if let Some((tx, ty)) = tilt {
            events.push(AbsEvent::new(Abs::TILT_X, tx as i32).into());
            events.push(AbsEvent::new(Abs::TILT_Y, ty as i32).into());
        }
        events.push(SynEvent::new(Syn::REPORT).into());
        self.device.write_events(&events)?;
        Ok(())
    }
}

pub fn button_code(button: u32) -> u32 {
    match button {
        1 => 0x110,
        2 => 0x112,
        3 => 0x111,
        n => n + 0x110,
    }
}
