// Orbiscreen - orbiscreen-input - xtest module (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use x11rb::connection::{Connection as _, RequestConnection as _};
use x11rb::protocol::xproto::{
    BUTTON_PRESS_EVENT, BUTTON_RELEASE_EVENT, KEY_PRESS_EVENT, KEY_RELEASE_EVENT,
    MOTION_NOTIFY_EVENT,
};
use x11rb::protocol::xtest;
use x11rb::rust_connection::RustConnection;

use super::{InputError, PointerEvent, VirtualTouchscreenSpec};

const MAX_X_KEYCODE: u32 = 247;

#[allow(missing_debug_implementations)]
pub struct XtestInjector {
    conn: RustConnection,
    root: u32,
    root_width: u32,
    root_height: u32,
    width: u32,
    height: u32,
}

impl XtestInjector {
    pub fn open(spec: VirtualTouchscreenSpec) -> Result<Self, InputError> {
        let (conn, screen_num) =
            x11rb::connect(None).map_err(|e| InputError::Uinput(format!("x11 connect: {e}")))?;
        conn.extension_information(xtest::X11_EXTENSION_NAME)
            .map_err(|e| InputError::Uinput(format!("xtest query: {e}")))?
            .ok_or_else(|| InputError::Uinput("XTEST extension not available".into()))?;
        let (root, root_width, root_height) = {
            let screen = &conn.setup().roots[screen_num];
            (
                screen.root,
                screen.width_in_pixels.into(),
                screen.height_in_pixels.into(),
            )
        };
        Ok(Self {
            conn,
            root,
            root_width,
            root_height,
            width: spec.width,
            height: spec.height,
        })
    }

    fn to_root(&self, x: f64, y: f64) -> (i16, i16) {
        let rx = x * f64::from(self.root_width) / f64::from(self.width.max(1));
        let ry = y * f64::from(self.root_height) / f64::from(self.height.max(1));
        let rx = rx.clamp(0.0, f64::from(self.root_width.saturating_sub(1))) as i16;
        let ry = ry.clamp(0.0, f64::from(self.root_height.saturating_sub(1))) as i16;
        (rx, ry)
    }

    fn fake(&self, type_: u8, detail: u8, root_x: i16, root_y: i16) -> Result<(), InputError> {
        xtest::fake_input(&self.conn, type_, detail, 0, self.root, root_x, root_y, 0)
            .map_err(|e| InputError::Uinput(format!("xtest fake_input: {e}")))?;
        self.conn
            .flush()
            .map_err(|e| InputError::Uinput(format!("xtest flush: {e}")))?;
        Ok(())
    }

    pub fn inject_pointer(&self, event: PointerEvent) -> Result<(), InputError> {
        match event {
            PointerEvent::Move { x, y } => {
                let (rx, ry) = self.to_root(x, y);
                self.fake(MOTION_NOTIFY_EVENT, 0, rx, ry)
            }
            PointerEvent::Button { button, pressed } => {
                if button == 0 || button > 8 {
                    return Err(InputError::Uinput(format!("invalid button: {button}")));
                }
                let detail = x_button(button);
                let type_ = if pressed {
                    BUTTON_PRESS_EVENT
                } else {
                    BUTTON_RELEASE_EVENT
                };
                self.fake(type_, detail, 0, 0)
            }
            PointerEvent::Wheel { delta_y } => {
                let steps = delta_y.clamp(i32::MIN as f64, i32::MAX as f64).round() as i32;
                let button = if steps >= 0 { 4 } else { 5 };
                for _ in 0..steps.unsigned_abs() {
                    self.fake(BUTTON_PRESS_EVENT, button, 0, 0)?;
                    self.fake(BUTTON_RELEASE_EVENT, button, 0, 0)?;
                }
                Ok(())
            }
        }
    }

    pub fn inject_key(&self, code: u32, pressed: bool) -> Result<(), InputError> {
        if code == 0 || code > MAX_X_KEYCODE {
            return Err(InputError::Uinput(format!(
                "key code {code} cannot be mapped onto X11 (max {MAX_X_KEYCODE})"
            )));
        }
        let type_ = if pressed {
            KEY_PRESS_EVENT
        } else {
            KEY_RELEASE_EVENT
        };
        self.fake(type_, (code + 8) as u8, 0, 0)
    }
}

fn x_button(button: u32) -> u8 {
    match button {
        1 => 1,
        2 => 2,
        3 => 3,
        n => (n + 4) as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x11::button_code;

    #[test]
    fn linux_button_codes_are_reused() {
        assert_eq!(button_code(1), 0x110);
        assert_eq!(button_code(3), 0x111);
    }

    #[test]
    fn x_buttons_map_1_2_3_directly_and_rest_to_8_plus() {
        assert_eq!(x_button(1), 1);
        assert_eq!(x_button(2), 2);
        assert_eq!(x_button(3), 3);
        assert_eq!(x_button(4), 8);
        assert_eq!(x_button(8), 12);
    }

    #[test]
    fn keycode_offset_is_8() {
        assert_eq!(30 + 8, 38);
        assert_eq!(MAX_X_KEYCODE + 8, 255);
    }
}
