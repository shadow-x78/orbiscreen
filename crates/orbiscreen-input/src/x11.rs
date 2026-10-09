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
    mk_name: String,
    ts_name: String,
    tab_name: String,
    output_name: Option<String>,
    pointer_frame: Option<crate::PointerFrame>,
    prod_offset: u16,
    width: u32,
    height: u32,
    cursor_x: f64,
    cursor_y: f64,
    button_1_pressed: bool,
    touch_slot_active: [bool; crate::MAX_TOUCH_SLOTS],
    touch_slot_id: [i32; crate::MAX_TOUCH_SLOTS],
    touch_active_count: u8,
    button_touch_down: bool,
    pen_release_at: Option<std::time::Instant>,
}

impl UinputInjector {
    pub fn open(spec: VirtualTouchscreenSpec) -> Result<Self, InputError> {
        let mut mk_keys: Vec<Key> = (1u16..=248).map(Key::from_raw).collect();
        mk_keys.extend((0x110u16..=0x117).map(Key::from_raw));

        let width_axis = AbsInfo::new(0, spec.width.saturating_sub(1) as i32);
        let height_axis = AbsInfo::new(0, spec.height.saturating_sub(1) as i32);
        let (mouse_x_axis, mouse_y_axis) = mouse_axes(spec.pointer_frame, spec.width, spec.height);

        let is_secondary = spec.output_name.as_deref().is_some_and(is_secondary_output);
        let prod_offset = if is_secondary { 0x0010 } else { 0x0000 };
        let (mk_name, ts_name, tab_name) =
            if let Some(label) = spec.device_label.as_deref().filter(|s| !s.is_empty()) {
                (
                    format!("{label} Mouse"),
                    format!("{label} Touch"),
                    format!("{label} Pen"),
                )
            } else {
                let prefix = if is_secondary {
                    "Orbiscreen 2"
                } else {
                    "Orbiscreen"
                };
                (
                    format!("{prefix} Virtual Mouse and Keyboard"),
                    format!("{prefix} Virtual Touchscreen"),
                    format!("{prefix} Virtual Tablet"),
                )
            };

        let mouse_keyboard = UinputDevice::builder()?
            .with_input_id(InputId::new(
                Bus::VIRTUAL,
                0x0BEE,
                0x0001 + prod_offset,
                0x0001,
            ))?
            .with_props([InputProp::POINTER])?
            .with_abs_axes([
                AbsSetup::new(Abs::X, mouse_x_axis),
                AbsSetup::new(Abs::Y, mouse_y_axis),
            ])?
            .with_rel_axes([Rel::WHEEL])?
            .with_keys(mk_keys)?
            .build(&mk_name)?;

        let slot_axis = AbsInfo::new(0, (crate::MAX_TOUCH_SLOTS as i32) - 1);
        let tracking_axis = AbsInfo::new(-1, i32::MAX);
        let touchscreen = UinputDevice::builder()?
            .with_input_id(InputId::new(
                Bus::VIRTUAL,
                0x0BEE,
                0x0002 + prod_offset,
                0x0001,
            ))?
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
            .with_keys([Key::BTN_TOUCH, Key::BTN_LEFT])?
            .build(&ts_name)?;

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
            Key::BTN_LEFT,
            Key::BTN_RIGHT,
            Key::BTN_MIDDLE,
        ];

        let tablet = UinputDevice::builder()?
            .with_input_id(InputId::new(
                Bus::VIRTUAL,
                0x0BEE,
                0x0003 + prod_offset,
                0x0001,
            ))?
            .with_props([InputProp::DIRECT])?
            .with_abs_axes([
                AbsSetup::new(Abs::X, res_w_axis),
                AbsSetup::new(Abs::Y, res_h_axis),
                AbsSetup::new(Abs::PRESSURE, pressure_axis),
                AbsSetup::new(Abs::TILT_X, tilt_axis),
                AbsSetup::new(Abs::TILT_Y, tilt_axis),
            ])?
            .with_keys(tablet_keys)?
            .build(&tab_name)?;

        info!("opened uinput devices: mouse/keyboard, touchscreen, and tablet");
        let output_name = spec.output_name.clone().filter(|s| !s.is_empty());
        if let Some(output) = output_name.as_deref() {
            configure_kwin_device(&[&mk_name, &ts_name, &tab_name], output);
        }

        let mut injector = Self {
            mouse_keyboard,
            touchscreen,
            tablet,
            mk_name,
            ts_name,
            tab_name,
            output_name,
            pointer_frame: spec.pointer_frame,
            prod_offset,
            width: spec.width,
            height: spec.height,
            cursor_x: f64::from(spec.width) / 2.0,
            cursor_y: f64::from(spec.height) / 2.0,
            button_1_pressed: false,
            touch_slot_active: [false; crate::MAX_TOUCH_SLOTS],
            touch_slot_id: [-1; crate::MAX_TOUCH_SLOTS],
            touch_active_count: 0,
            button_touch_down: false,
            pen_release_at: None,
        };
        let _ = injector.release_tools();
        let center_x = (spec.width.saturating_sub(1) / 2) as i32;
        let center_y = (spec.height.saturating_sub(1) / 2) as i32;
        let _ = injector.emit_pointer_position(center_x, center_y);
        Ok(injector)
    }

    fn clamp_point(&self, x: f64, y: f64) -> (i32, i32) {
        let cx = x.clamp(0.0, f64::from(self.width.saturating_sub(1))) as i32;
        let cy = y.clamp(0.0, f64::from(self.height.saturating_sub(1))) as i32;
        (cx, cy)
    }

    fn open_mouse_keyboard(&self, width: u32, height: u32) -> Result<UinputDevice, InputError> {
        let mut mk_keys: Vec<Key> = (1u16..=248).map(Key::from_raw).collect();
        mk_keys.extend((0x110u16..=0x117).map(Key::from_raw));
        let (mouse_x_axis, mouse_y_axis) = mouse_axes(self.pointer_frame, width, height);
        Ok(UinputDevice::builder()?
            .with_input_id(InputId::new(
                Bus::VIRTUAL,
                0x0BEE,
                0x0001 + self.prod_offset,
                0x0001,
            ))?
            .with_props([InputProp::POINTER])?
            .with_abs_axes([
                AbsSetup::new(Abs::X, mouse_x_axis),
                AbsSetup::new(Abs::Y, mouse_y_axis),
            ])?
            .with_rel_axes([Rel::WHEEL])?
            .with_keys(mk_keys)?
            .build(&self.mk_name)?)
    }

    fn open_touchscreen(&self, width: u32, height: u32) -> Result<UinputDevice, InputError> {
        let width_axis = AbsInfo::new(0, width.saturating_sub(1) as i32);
        let height_axis = AbsInfo::new(0, height.saturating_sub(1) as i32);
        let slot_axis = AbsInfo::new(0, (crate::MAX_TOUCH_SLOTS as i32) - 1);
        let tracking_axis = AbsInfo::new(-1, i32::MAX);
        Ok(UinputDevice::builder()?
            .with_input_id(InputId::new(
                Bus::VIRTUAL,
                0x0BEE,
                0x0002 + self.prod_offset,
                0x0001,
            ))?
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
            .with_keys([Key::BTN_TOUCH, Key::BTN_LEFT])?
            .build(&self.ts_name)?)
    }

    fn open_tablet(&self, width: u32, height: u32) -> Result<UinputDevice, InputError> {
        let res_w_axis = AbsInfo::new(0, width.saturating_sub(1) as i32).with_resolution(10);
        let res_h_axis = AbsInfo::new(0, height.saturating_sub(1) as i32).with_resolution(10);
        let pressure_axis = AbsInfo::new(0, PRESSURE_MAX);
        let tilt_axis = AbsInfo::new(TILT_MIN, TILT_MAX);
        let tablet_keys = vec![
            Key::BTN_TOOL_PEN,
            Key::BTN_TOOL_RUBBER,
            Key::BTN_TOUCH,
            Key::BTN_STYLUS,
            Key::BTN_STYLUS2,
            Key::BTN_LEFT,
            Key::BTN_RIGHT,
            Key::BTN_MIDDLE,
        ];
        Ok(UinputDevice::builder()?
            .with_input_id(InputId::new(
                Bus::VIRTUAL,
                0x0BEE,
                0x0003 + self.prod_offset,
                0x0001,
            ))?
            .with_props([InputProp::DIRECT])?
            .with_abs_axes([
                AbsSetup::new(Abs::X, res_w_axis),
                AbsSetup::new(Abs::Y, res_h_axis),
                AbsSetup::new(Abs::PRESSURE, pressure_axis),
                AbsSetup::new(Abs::TILT_X, tilt_axis),
                AbsSetup::new(Abs::TILT_Y, tilt_axis),
            ])?
            .with_keys(tablet_keys)?
            .build(&self.tab_name)?)
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), InputError> {
        if width == 0 || height == 0 || (width == self.width && height == self.height) {
            return Ok(());
        }
        self.release_tools()?;
        let mouse_keyboard = self.open_mouse_keyboard(width, height)?;
        let touchscreen = self.open_touchscreen(width, height)?;
        let tablet = self.open_tablet(width, height)?;
        self.mouse_keyboard = mouse_keyboard;
        self.touchscreen = touchscreen;
        self.tablet = tablet;
        self.width = width;
        self.height = height;
        self.touch_slot_active = [false; crate::MAX_TOUCH_SLOTS];
        self.touch_slot_id = [-1; crate::MAX_TOUCH_SLOTS];
        self.touch_active_count = 0;
        self.cursor_x = (self.cursor_x).clamp(0.0, f64::from(width.saturating_sub(1)));
        self.cursor_y = (self.cursor_y).clamp(0.0, f64::from(height.saturating_sub(1)));
        if let Some(output) = self.output_name.clone() {
            configure_kwin_device(&[&self.mk_name, &self.ts_name, &self.tab_name], &output);
        }
        info!(width, height, "recreated uinput devices for resized output");
        Ok(())
    }

    pub fn inject_pointer(&mut self, event: PointerEvent) -> Result<(), InputError> {
        match event {
            PointerEvent::Move { x, y } => {
                let (xi, yi) = self.clamp_point(x, y);
                self.cursor_x = f64::from(xi);
                self.cursor_y = f64::from(yi);
                self.emit_pointer_position(xi, yi)?;
            }
            PointerEvent::RelativeMove { dx, dy } => {
                let ((tx, ty), _) = clamped_relative(
                    (self.cursor_x, self.cursor_y),
                    (dx, dy),
                    (self.width, self.height),
                );
                self.cursor_x = f64::from(tx);
                self.cursor_y = f64::from(ty);
                self.emit_pointer_position(tx, ty)?;
            }
            PointerEvent::Button { button, pressed } => {
                let Some(btn_key) = button_key(button) else {
                    return Err(InputError::Uinput(format!("invalid button: {button}")));
                };
                let state = if pressed {
                    KeyState::PRESSED
                } else {
                    KeyState::RELEASED
                };
                let xi = self.cursor_x.round() as i32;
                let yi = self.cursor_y.round() as i32;
                let (gx, gy) = match self.pointer_frame {
                    Some(f) => (f.origin_x + xi, f.origin_y + yi),
                    None => (xi, yi),
                };
                let events = vec![
                    AbsEvent::new(Abs::X, gx).into(),
                    AbsEvent::new(Abs::Y, gy).into(),
                    KEv::new(btn_key, state).into(),
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

    fn emit_pointer_position(&mut self, x: i32, y: i32) -> Result<(), InputError> {
        let (gx, gy) = match self.pointer_frame {
            Some(f) => (f.origin_x + x, f.origin_y + y),
            None => (x, y),
        };
        let events = vec![
            AbsEvent::new(Abs::X, gx).into(),
            AbsEvent::new(Abs::Y, gy).into(),
            SynEvent::new(Syn::REPORT).into(),
        ];
        self.mouse_keyboard.write_events(&events)?;
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
            self.cursor_x = f64::from(xi);
            self.cursor_y = f64::from(yi);
        }
        let tracking_id = if event.id >= 0 { event.id } else { slot as i32 };

        if event.pressed {
            let was_active = self.touch_slot_active[slot];
            let id_changed = was_active && self.touch_slot_id[slot] != tracking_id;
            if (!was_active && self.touch_active_count == 0)
                || (id_changed && self.touch_active_count == 1)
            {
                self.release_touch_only()?;
            }
        }

        let mut writer = self.touchscreen.writer();
        let mut slot_writer = writer.slot(slot as u16)?;

        if event.pressed {
            let was_active = self.touch_slot_active[slot];
            let id_changed = was_active && self.touch_slot_id[slot] != tracking_id;

            if !was_active || id_changed {
                if id_changed {
                    slot_writer = slot_writer.set_tracking_id(-1)?;
                } else {
                    self.touch_active_count = self.touch_active_count.saturating_add(1);
                }
                slot_writer = slot_writer.set_tracking_id(tracking_id)?;
                self.touch_slot_active[slot] = true;
                self.touch_slot_id[slot] = tracking_id;
            }

            slot_writer = slot_writer.set_position(xi, yi)?;
            writer = slot_writer.finish_slot()?;

            writer = writer.write_events(&[
                AbsEvent::new(Abs::X, xi).into(),
                AbsEvent::new(Abs::Y, yi).into(),
            ])?;

            if !self.button_touch_down {
                self.button_touch_down = true;
                writer = writer.write_events(&[
                    KEv::new(Key::BTN_TOUCH, KeyState::PRESSED).into(),
                    KEv::new(Key::BTN_LEFT, KeyState::PRESSED).into(),
                ])?;
            }
        } else {
            if self.touch_slot_active[slot] {
                self.touch_slot_active[slot] = false;
                self.touch_slot_id[slot] = -1;
                self.touch_active_count = self.touch_active_count.saturating_sub(1);
                slot_writer = slot_writer.set_tracking_id(-1)?;
            }
            writer = slot_writer.finish_slot()?;

            if self.touch_active_count == 0 && self.button_touch_down {
                self.button_touch_down = false;
                writer = writer.write_events(&[
                    KEv::new(Key::BTN_TOUCH, KeyState::RELEASED).into(),
                    KEv::new(Key::BTN_LEFT, KeyState::RELEASED).into(),
                ])?;
            }
        }

        writer.finish()?;
        Ok(())
    }

    pub fn release_touch_only(&mut self) -> Result<(), InputError> {
        self.button_1_pressed = false;
        self.button_touch_down = false;
        let mut events: Vec<InputEvent> = Vec::new();
        for raw in 0x110..=0x117u16 {
            events.push(KEv::new(Key::from_raw(raw), KeyState::RELEASED).into());
        }
        for raw in 1..=248u16 {
            events.push(KEv::new(Key::from_raw(raw), KeyState::RELEASED).into());
        }
        events.push(SynEvent::new(Syn::REPORT).into());
        self.mouse_keyboard.write_events(&events)?;
        self.touchscreen
            .write_events(&[
                KEv::new(Key::BTN_TOUCH, KeyState::RELEASED).into(),
                KEv::new(Key::BTN_LEFT, KeyState::RELEASED).into(),
                SynEvent::new(Syn::REPORT).into(),
            ])
            .map_err(|e| InputError::Uinput(e.to_string()))
    }

    pub fn release_tools(&mut self) -> Result<(), InputError> {
        self.button_1_pressed = false;
        self.button_touch_down = false;
        let xi = self.cursor_x.round() as i32;
        let yi = self.cursor_y.round() as i32;
        let tablet_events = vec![
            AbsEvent::new(Abs::X, xi).into(),
            AbsEvent::new(Abs::Y, yi).into(),
            AbsEvent::new(Abs::PRESSURE, 0).into(),
            KEv::new(Key::BTN_TOUCH, KeyState::RELEASED).into(),
            KEv::new(Key::BTN_STYLUS, KeyState::RELEASED).into(),
            KEv::new(Key::BTN_STYLUS2, KeyState::RELEASED).into(),
            KEv::new(Key::BTN_TOOL_PEN, KeyState::RELEASED).into(),
            SynEvent::new(Syn::REPORT).into(),
        ];
        self.tablet.write_events(&tablet_events)?;

        let mut events: Vec<InputEvent> = Vec::new();
        for raw in 0x110..=0x117u16 {
            events.push(KEv::new(Key::from_raw(raw), KeyState::RELEASED).into());
        }
        for raw in 1..=248u16 {
            events.push(KEv::new(Key::from_raw(raw), KeyState::RELEASED).into());
        }
        events.push(SynEvent::new(Syn::REPORT).into());
        self.mouse_keyboard.write_events(&events)?;
        Ok(())
    }

    fn handle_pen_proximity(&mut self) -> Result<(), InputError> {
        match pen_proximity_action(self.pen_release_at.is_some(), false) {
            PenProximity::Release => {
                self.pen_release_at = None;
                self.release_tools()
            }
            PenProximity::Cancel => {
                self.pen_release_at = None;
                Ok(())
            }
            PenProximity::Ignore => {
                self.pen_release_at = Some(std::time::Instant::now());
                self.button_touch_down = false;
                self.tablet
                    .write_events(&[
                        KEv::new(Key::BTN_TOUCH, KeyState::RELEASED).into(),
                        KEv::new(Key::BTN_STYLUS, KeyState::RELEASED).into(),
                        KEv::new(Key::BTN_STYLUS2, KeyState::RELEASED).into(),
                        SynEvent::new(Syn::REPORT).into(),
                    ])
                    .map_err(|e| InputError::Uinput(e.to_string()))
            }
        }
    }

    pub fn inject_stylus(&mut self, event: StylusEvent) -> Result<(), InputError> {
        if let StylusEvent::Proximity {} = event {
            return self.handle_pen_proximity();
        }
        let (x, y, pressure, tilt) = match event {
            StylusEvent::Proximity {} => unreachable!("handled above"),
            StylusEvent::Pressure { x, y, pressure } => (x, y, pressure, None),
            StylusEvent::Tilt {
                x,
                y,
                pressure,
                tilt_x_deg,
                tilt_y_deg,
            } => (x, y, pressure, Some((tilt_x_deg, tilt_y_deg))),
        };
        match pen_proximity_action(
            self.pen_release_at.is_some(),
            self.pen_release_at
                .is_some_and(|at| std::time::Instant::now() >= at),
        ) {
            PenProximity::Release => {
                self.release_tools()?;
                self.pen_release_at = None;
                return Ok(());
            }
            PenProximity::Cancel => self.pen_release_at = None,
            PenProximity::Ignore => {}
        }
        let (xi, yi) = self.clamp_point(x, y);
        self.cursor_x = f64::from(xi);
        self.cursor_y = f64::from(yi);
        let pressure_val =
            (pressure * f64::from(PRESSURE_MAX)).clamp(0.0, f64::from(PRESSURE_MAX)) as i32;
        let is_touching = pressure_val > 0;
        let touch_state = if is_touching {
            KeyState::PRESSED
        } else {
            KeyState::RELEASED
        };
        let mut events: Vec<InputEvent> = Vec::with_capacity(8);
        events.push(AbsEvent::new(Abs::X, xi).into());
        events.push(AbsEvent::new(Abs::Y, yi).into());
        events.push(AbsEvent::new(Abs::PRESSURE, pressure_val).into());
        events.push(KEv::new(Key::BTN_TOOL_PEN, KeyState::PRESSED).into());
        events.push(KEv::new(Key::BTN_TOUCH, touch_state).into());
        if let Some((tx, ty)) = tilt {
            let tx = tx.clamp(f64::from(TILT_MIN), f64::from(TILT_MAX)) as i32;
            let ty = ty.clamp(f64::from(TILT_MIN), f64::from(TILT_MAX)) as i32;
            events.push(AbsEvent::new(Abs::TILT_X, tx).into());
            events.push(AbsEvent::new(Abs::TILT_Y, ty).into());
        }
        events.push(SynEvent::new(Syn::REPORT).into());
        self.tablet.write_events(&events)?;
        Ok(())
    }
}

pub(crate) fn is_secondary_output(name: &str) -> bool {
    name.ends_with("-2")
}

fn mouse_axes(frame: Option<crate::PointerFrame>, width: u32, height: u32) -> (AbsInfo, AbsInfo) {
    match frame {
        Some(f) => (
            AbsInfo::new(0, f.workspace_width.max(1) as i32),
            AbsInfo::new(0, f.workspace_height.max(1) as i32),
        ),
        None => (
            AbsInfo::new(0, width.saturating_sub(1) as i32),
            AbsInfo::new(0, height.saturating_sub(1) as i32),
        ),
    }
}

fn configure_kwin_device(device_names: &[&str], output_name: &str) {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return;
    }
    if let Ok(home) = std::env::var("HOME") {
        let kwinrc_path = format!("{}/.config/kwinrc", home);
        let content = std::fs::read_to_string(&kwinrc_path).unwrap_or_default();
        let mut lines: Vec<String> = content.lines().map(String::from).collect();

        for dev in device_names {
            let section = format!("[InputDevice][{}]", dev);
            let mut in_section = false;
            let mut replaced = false;
            let mut i = 0;
            while i < lines.len() {
                if lines[i].starts_with('[') {
                    if in_section {
                        break;
                    }
                    if lines[i] == section {
                        in_section = true;
                    }
                } else if in_section && lines[i].starts_with("OutputName=") {
                    lines[i] = format!("OutputName={}", output_name);
                    replaced = true;
                    break;
                }
                i += 1;
            }
            if !in_section {
                lines.push(section);
                lines.push(format!("OutputName={}", output_name));
            } else if !replaced {
                lines.insert(i, format!("OutputName={}", output_name));
            }
        }

        let _ = std::fs::write(&kwinrc_path, lines.join("\n") + "\n");
        let _ = std::process::Command::new("qdbus")
            .args(["org.kde.KWin", "/KWin", "reconfigure"])
            .status();
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PenProximity {
    Ignore,
    Cancel,
    Release,
}

pub(crate) fn pen_proximity_action(
    pending_release: bool,
    stylus_sample_since: bool,
) -> PenProximity {
    if !pending_release {
        PenProximity::Ignore
    } else if stylus_sample_since {
        PenProximity::Cancel
    } else {
        PenProximity::Release
    }
}

fn button_key(button: u32) -> Option<Key> {
    let raw = button_code(button);
    (1..=8).contains(&button).then(|| Key::from_raw(raw as u16))
}

pub fn clamped_relative(
    cursor: (f64, f64),
    delta: (f64, f64),
    bounds: (u32, u32),
) -> ((i32, i32), (i32, i32)) {
    let (max_x, max_y) = (
        bounds.0.saturating_sub(1) as f64,
        bounds.1.saturating_sub(1) as f64,
    );
    let target_x = (cursor.0 + delta.0).clamp(0.0, max_x);
    let target_y = (cursor.1 + delta.1).clamp(0.0, max_y);
    let from_x = cursor.0.round().clamp(0.0, max_x) as i32;
    let from_y = cursor.1.round().clamp(0.0, max_y) as i32;
    let to_x = target_x.round() as i32;
    let to_y = target_y.round() as i32;
    ((to_x, to_y), (to_x - from_x, to_y - from_y))
}

#[cfg(test)]
mod tests {
    use super::{
        button_code, button_key, clamped_relative, is_secondary_output, mouse_axes,
        pen_proximity_action, PenProximity,
    };

    #[test]
    fn buttons_six_through_eight_map_distinctly_not_left() {
        let left = button_key(1).unwrap();
        for button in 6..=8 {
            assert_ne!(button_key(button), Some(left));
        }
        assert!(button_key(0).is_none());
        assert!(button_key(9).is_none());
        assert_eq!(button_code(6), 0x115);
        assert_eq!(button_code(8), 0x117);
    }

    #[test]
    fn pointer_frame_stretches_the_mouse_axes_to_the_workspace() {
        let frame = crate::PointerFrame {
            origin_x: 4480,
            origin_y: 0,
            workspace_width: 7040,
            workspace_height: 1600,
        };
        let (x, y) = mouse_axes(Some(frame), 2560, 1600);
        assert_eq!((x.minimum(), x.maximum()), (0, 7040));
        assert_eq!((y.minimum(), y.maximum()), (0, 1600));
        let (lx, ly) = mouse_axes(None, 2560, 1600);
        assert_eq!((lx.minimum(), lx.maximum()), (0, 2559));
        assert_eq!((ly.minimum(), ly.maximum()), (0, 1599));
    }

    #[test]
    fn motion_inside_bounds_passes_through() {
        let (pos, delta) = clamped_relative((100.0, 100.0), (50.0, -20.0), (2560, 1600));
        assert_eq!(pos, (150, 80));
        assert_eq!(delta, (50, -20));
    }

    #[test]
    fn motion_at_right_edge_is_truncated_not_leaked() {
        let (pos, delta) = clamped_relative((2554.0, 800.0), (50.0, 0.0), (2560, 1600));
        assert_eq!(pos, (2559, 800));
        assert_eq!(delta, (5, 0));
    }

    #[test]
    fn motion_at_top_left_edge_is_truncated() {
        let (pos, delta) = clamped_relative((2.0, 3.0), (-40.0, -60.0), (2560, 1600));
        assert_eq!(pos, (0, 0));
        assert_eq!(delta, (-2, -3));
    }

    #[test]
    fn repeated_overflow_deltas_stay_confined() {
        let mut cursor = (2559.0, 1599.0);
        for _ in 0..5 {
            let (pos, delta) = clamped_relative(cursor, (200.0, 150.0), (2560, 1600));
            assert_eq!(pos, (2559, 1599));
            assert_eq!(delta, (0, 0));
            cursor = (f64::from(pos.0), f64::from(pos.1));
        }
    }

    #[test]
    fn absolute_move_from_a_foreign_cursor_cannot_emit_a_huge_jump() {
        let tablet = (2560.0, 1600.0);
        let cursor_on_other_monitor = (5000.0, 800.0);
        let target = (1200.0, 800.0);

        let raw_dx = (target.0 - cursor_on_other_monitor.0) as i32;
        assert!(raw_dx.abs() > 3000, "old path produced the runaway jump");

        let clamped_from = (tablet.0 - 1.0, target.1);
        let (pos, delta) = clamped_relative(
            clamped_from,
            (target.0 - clamped_from.0, target.1 - clamped_from.1),
            (tablet.0 as u32, tablet.1 as u32),
        );
        assert_eq!(pos, (1200, 800));
        assert_eq!(delta.1, 0);
        assert!(delta.0.abs() < 2000, "delta still too large: {delta:?}");
    }

    #[test]
    fn secondary_output_matches_the_names_capture_actually_produces() {
        assert!(!is_secondary_output("Virtual-ORBISCREEN"));
        assert!(is_secondary_output("Virtual-ORBISCREEN-2"));

        assert!(!is_secondary_output("Virtual-ORBISCREEN-12342"));
        assert!(!is_secondary_output("Virtual-ORBISCREEN-4321"));

        assert!(!is_secondary_output("Virtual-Orbi-9d41ad1e"));
        assert!(is_secondary_output("Virtual-Orbi-9d41ad1e-2"));
        assert!(!is_secondary_output("Virtual-Orbi-12345678"));
    }

    #[test]
    fn a_bursty_pen_edge_does_not_drop_the_tool() {
        assert_eq!(
            pen_proximity_action(false, false),
            PenProximity::Ignore,
            "an edge with nothing pending carries no information"
        );
        assert_eq!(
            pen_proximity_action(true, true),
            PenProximity::Cancel,
            "the pen spoke after the edge, so it never left"
        );
        assert_eq!(
            pen_proximity_action(true, false),
            PenProximity::Release,
            "the edge stood, so the pen really left"
        );
    }

    #[test]
    fn a_single_edge_alone_never_releases_the_tool() {
        assert_eq!(pen_proximity_action(false, false), PenProximity::Ignore);
    }
}
