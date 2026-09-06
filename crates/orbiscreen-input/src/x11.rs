// Orbiscreen - x11.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::io;

use evdevil::event::{
    Abs, AbsEvent, InputEvent, Key, KeyEvent as KEv, KeyState, Rel, RelEvent, Syn, SynEvent,
};
use evdevil::uinput::{AbsSetup, UinputDevice};
use evdevil::{AbsInfo, Bus, InputId, InputProp};
use tracing::info;

use super::{InputError, PointerEvent, StylusEvent, TouchEvent, VirtualTouchscreenSpec};

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
    touchscreen: UinputDevice,
    tablet: UinputDevice,
    width: u32,
    height: u32,
    last_touch_x: i32,
    last_touch_y: i32,
    touch_slot_active: [bool; crate::MAX_TOUCH_SLOTS],
    touch_active_count: u8,
}

impl UinputInjector {
    pub fn open(spec: VirtualTouchscreenSpec) -> Result<Self, InputError> {
        let mut mk_keys: Vec<Key> = (1u16..=248).map(Key::from_raw).collect();
        mk_keys.extend((0x110u16..=0x117).map(Key::from_raw));

        let width_axis = AbsInfo::new(0, spec.width.saturating_sub(1) as i32);
        let height_axis = AbsInfo::new(0, spec.height.saturating_sub(1) as i32);

        let mouse_keyboard = UinputDevice::builder()?
            .with_input_id(InputId::new(Bus::VIRTUAL, 0x0BEE, 0x0001, 0x0001))?
            .with_props([InputProp::POINTER])?
            .with_abs_axes([
                AbsSetup::new(Abs::X, width_axis),
                AbsSetup::new(Abs::Y, height_axis),
            ])?
            .with_rel_axes([Rel::WHEEL])?
            .with_keys(mk_keys)?
            .build("Orbiscreen Virtual Mouse and Keyboard")?;

        let slot_axis = AbsInfo::new(0, (crate::MAX_TOUCH_SLOTS as i32) - 1);
        let tracking_axis = AbsInfo::new(-1, i32::MAX);
        let touchscreen = UinputDevice::builder()?
            .with_input_id(InputId::new(Bus::VIRTUAL, 0x0BEE, 0x0002, 0x0001))?
            .with_props([InputProp::DIRECT])?
            .with_abs_axes([
                AbsSetup::new(Abs::X, width_axis),
                AbsSetup::new(Abs::Y, height_axis),
                AbsSetup::new(Abs::MT_SLOT, slot_axis),
                AbsSetup::new(Abs::MT_TRACKING_ID, tracking_axis),
                AbsSetup::new(Abs::MT_POSITION_X, width_axis),
                AbsSetup::new(Abs::MT_POSITION_Y, height_axis),
            ])?
            .with_keys([Key::BTN_TOUCH])?
            .build("Orbiscreen Virtual Touchscreen")?;

        let res_w_axis = AbsInfo::new(0, spec.width.saturating_sub(1) as i32).with_resolution(10);
        let res_h_axis = AbsInfo::new(0, spec.height.saturating_sub(1) as i32).with_resolution(10);
        let pressure_axis = AbsInfo::new(0, PRESSURE_MAX);
        let tilt_axis = AbsInfo::new(TILT_MIN, TILT_MAX);
        let tablet_keys = vec![
            Key::BTN_TOOL_PEN,
            Key::BTN_TOOL_RUBBER,
            Key::BTN_TOUCH,
            Key::BTN_STYLUS,
            Key::BTN_STYLUS2,
        ];

        let tablet = UinputDevice::builder()?
            .with_input_id(InputId::new(Bus::VIRTUAL, 0x0BEE, 0x0003, 0x0001))?
            .with_props([InputProp::DIRECT])?
            .with_abs_axes([
                AbsSetup::new(Abs::X, res_w_axis),
                AbsSetup::new(Abs::Y, res_h_axis),
                AbsSetup::new(Abs::PRESSURE, pressure_axis),
                AbsSetup::new(Abs::TILT_X, tilt_axis),
                AbsSetup::new(Abs::TILT_Y, tilt_axis),
            ])?
            .with_keys(tablet_keys)?
            .build("Orbiscreen Virtual Tablet")?;

        info!("opened uinput devices: mouse/keyboard, touchscreen, and tablet");
        Ok(Self {
            mouse_keyboard,
            touchscreen,
            tablet,
            width: spec.width,
            height: spec.height,
            last_touch_x: 0,
            last_touch_y: 0,
            touch_slot_active: [false; crate::MAX_TOUCH_SLOTS],
            touch_active_count: 0,
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
                self.last_touch_x = xi;
                self.last_touch_y = yi;
                let events = vec![
                    AbsEvent::new(Abs::X, xi).into(),
                    AbsEvent::new(Abs::Y, yi).into(),
                    SynEvent::new(Syn::REPORT).into(),
                ];
                self.mouse_keyboard.write_events(&events)?;
            }
            PointerEvent::RelativeMove { dx, dy } => {
                let new_x = (f64::from(self.last_touch_x) + dx)
                    .clamp(0.0, f64::from(self.width.saturating_sub(1)))
                    as i32;
                let new_y = (f64::from(self.last_touch_y) + dy)
                    .clamp(0.0, f64::from(self.height.saturating_sub(1)))
                    as i32;
                self.last_touch_x = new_x;
                self.last_touch_y = new_y;
                let events = vec![
                    AbsEvent::new(Abs::X, new_x).into(),
                    AbsEvent::new(Abs::Y, new_y).into(),
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
                    AbsEvent::new(Abs::X, self.last_touch_x).into(),
                    AbsEvent::new(Abs::Y, self.last_touch_y).into(),
                    KEv::new(Key::from_raw(code as u16), state).into(),
                    SynEvent::new(Syn::REPORT).into(),
                ];
                self.mouse_keyboard.write_events(&events)?;
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
        if code == 0 || code > 248 {
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

    pub fn inject_touch(&mut self, event: TouchEvent) -> Result<(), InputError> {
        let slot = (event.slot as usize).min(crate::MAX_TOUCH_SLOTS - 1);
        let (xi, yi) = self.clamp_point(event.x, event.y);
        if event.pressed {
            self.last_touch_x = xi;
            self.last_touch_y = yi;
        }
        let tracking_id = if event.id >= 0 { event.id } else { slot as i32 };
        let becoming_active = event.pressed && !self.touch_slot_active[slot];
        let becoming_idle = !event.pressed && self.touch_slot_active[slot];
        if becoming_active {
            self.touch_slot_active[slot] = true;
            self.touch_active_count = self.touch_active_count.saturating_add(1);
        } else if becoming_idle {
            self.touch_slot_active[slot] = false;
            self.touch_active_count = self.touch_active_count.saturating_sub(1);
        } else if !event.pressed {
            return Ok(());
        }

        let mut writer = self.touchscreen.writer();
        let mut slot_writer = writer.slot(slot as u16)?;
        if becoming_active {
            slot_writer = slot_writer.set_tracking_id(tracking_id)?;
        }
        if event.pressed {
            slot_writer = slot_writer.set_position(xi, yi)?;
        }
        if becoming_idle {
            slot_writer = slot_writer.set_tracking_id(-1)?;
        }
        writer = slot_writer.finish_slot()?;
        if event.pressed {
            writer = writer.write_events(&[
                AbsEvent::new(Abs::X, xi).into(),
                AbsEvent::new(Abs::Y, yi).into(),
            ])?;
        }
        if becoming_active && self.touch_active_count == 1 {
            writer = writer.write_events(&[KEv::new(Key::BTN_TOUCH, KeyState::PRESSED).into()])?;
        } else if becoming_idle && self.touch_active_count == 0 {
            writer = writer.write_events(&[KEv::new(Key::BTN_TOUCH, KeyState::RELEASED).into()])?;
        }
        writer.finish()?;
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
        self.last_touch_x = xi;
        self.last_touch_y = yi;
        let pressure_val =
            (pressure * f64::from(PRESSURE_MAX)).clamp(0.0, f64::from(PRESSURE_MAX)) as i32;
        let is_touching = pressure_val > 0;
        let touch_state = if is_touching {
            KeyState::PRESSED
        } else {
            KeyState::RELEASED
        };
        let tool_state = if is_touching {
            KeyState::PRESSED
        } else {
            KeyState::RELEASED
        };

        let mut events: Vec<InputEvent> = Vec::with_capacity(8);
        events.push(AbsEvent::new(Abs::X, xi).into());
        events.push(AbsEvent::new(Abs::Y, yi).into());
        events.push(AbsEvent::new(Abs::PRESSURE, pressure_val).into());
        events.push(KEv::new(Key::BTN_TOOL_PEN, tool_state).into());
        events.push(KEv::new(Key::BTN_TOUCH, touch_state).into());
        if let Some((tx, ty)) = tilt {
            let tx = tx.clamp(f64::from(TILT_MIN), f64::from(TILT_MAX)) as i32;
            let ty = ty.clamp(f64::from(TILT_MIN), f64::from(TILT_MAX)) as i32;
            events.push(AbsEvent::new(Abs::TILT_X, tx).into());
            events.push(AbsEvent::new(Abs::TILT_Y, ty).into());
        }
        events.push(SynEvent::new(Syn::REPORT).into());
        let _ = self.tablet.write_events(&events);

        let mk_events = vec![
            AbsEvent::new(Abs::X, xi).into(),
            AbsEvent::new(Abs::Y, yi).into(),
            KEv::new(Key::BTN_LEFT, touch_state).into(),
            SynEvent::new(Syn::REPORT).into(),
        ];
        let _ = self.mouse_keyboard.write_events(&mk_events);
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
