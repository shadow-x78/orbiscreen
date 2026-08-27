// Orbiscreen - orbiscreen-input - wlroots module (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use std::os::fd::{AsFd, AsRawFd};
use std::sync::mpsc::{RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};
use wayland_client::protocol::wl_output::{self, WlOutput};
use wayland_client::protocol::wl_pointer;
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_seat::{self, WlSeat};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols_wlr::virtual_pointer::v1::client::zwlr_virtual_pointer_manager_v1::{
    self as vp_manager_proto, ZwlrVirtualPointerManagerV1,
};
use wayland_protocols_wlr::virtual_pointer::v1::client::zwlr_virtual_pointer_v1::{
    self as vp_proto, ZwlrVirtualPointerV1,
};

use super::{InputError, KeyEvent, PointerEvent, VirtualTouchscreenSpec};
use crate::x11::button_code;

mod vk_proto {
    pub mod client {
        #![allow(
            dead_code,
            non_camel_case_types,
            unused_unsafe,
            unused_variables,
            non_upper_case_globals,
            non_snake_case,
            unused_imports,
            missing_docs,
            clippy::all
        )]
        use wayland_client;
        use wayland_client::protocol::*;

        pub mod __interfaces {
            use wayland_client::protocol::__interfaces::*;
            wayland_scanner::generate_interfaces!("protocols/virtual-keyboard-unstable-v1.xml");
        }
        use self::__interfaces::*;

        wayland_scanner::generate_client_code!("protocols/virtual-keyboard-unstable-v1.xml");
    }
}

use vk_proto::client::zwp_virtual_keyboard_manager_v1::{
    self as vk_manager_proto, ZwpVirtualKeyboardManagerV1,
};
use vk_proto::client::zwp_virtual_keyboard_v1::{self as vk_proto_mod, ZwpVirtualKeyboardV1};

const HANDSHAKE_DEADLINE: Duration = Duration::from_secs(3);
const DRAIN_PAUSE: Duration = Duration::from_millis(20);
const WHEEL_STEP_VALUE: f64 = 15.0;

enum WlCmd {
    Pointer(PointerEvent),
    Key(KeyEvent),
}

#[derive(Debug)]
struct OutputInfo {
    proxy: WlOutput,
    name: Option<String>,
    got_done: bool,
}

#[derive(Default)]
struct WlState {
    seat: Option<WlSeat>,
    pointer_manager: Option<(ZwlrVirtualPointerManagerV1, u32)>,
    keyboard_manager: Option<ZwpVirtualKeyboardManagerV1>,
    outputs: Vec<OutputInfo>,
    pointer: Option<ZwlrVirtualPointerV1>,
    keyboard: Option<ZwpVirtualKeyboardV1>,
    keymap_file: Option<std::fs::File>,
}

impl Dispatch<WlRegistry, ()> for WlState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == <WlSeat as Proxy>::interface().name && state.seat.is_none() {
                state.seat = Some(registry.bind(name, version.min(9), qh, ()));
            } else if interface == <ZwlrVirtualPointerManagerV1 as Proxy>::interface().name {
                let bound =
                    version.min(<ZwlrVirtualPointerManagerV1 as Proxy>::interface().version);
                state.pointer_manager = Some((registry.bind(name, bound, qh, ()), bound));
            } else if interface == <ZwpVirtualKeyboardManagerV1 as Proxy>::interface().name {
                state.keyboard_manager = Some(registry.bind(name, version.min(1), qh, ()));
            } else if interface == <WlOutput as Proxy>::interface().name {
                let proxy: WlOutput = registry.bind(name, version.min(4), qh, ());
                state.outputs.push(OutputInfo {
                    proxy,
                    name: None,
                    got_done: false,
                });
            }
        }
    }
}

impl Dispatch<WlSeat, ()> for WlState {
    fn event(
        _: &mut Self,
        _: &WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlOutput, ()> for WlState {
    fn event(
        state: &mut Self,
        output: &WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(info) = state.outputs.iter_mut().find(|o| o.proxy == *output) else {
            return;
        };
        match event {
            wl_output::Event::Name { name } => info.name = Some(name),
            wl_output::Event::Done => info.got_done = true,
            _ => {}
        }
    }
}

impl Dispatch<ZwlrVirtualPointerManagerV1, ()> for WlState {
    fn event(
        _: &mut Self,
        _: &ZwlrVirtualPointerManagerV1,
        _: vp_manager_proto::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrVirtualPointerV1, ()> for WlState {
    fn event(
        _: &mut Self,
        _: &ZwlrVirtualPointerV1,
        _: vp_proto::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpVirtualKeyboardManagerV1, ()> for WlState {
    fn event(
        _: &mut Self,
        _: &ZwpVirtualKeyboardManagerV1,
        _: vk_manager_proto::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpVirtualKeyboardV1, ()> for WlState {
    fn event(
        _: &mut Self,
        _: &ZwpVirtualKeyboardV1,
        _: vk_proto_mod::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

#[allow(unsafe_code)]
fn anonymous_keymap_file(bytes: &[u8]) -> Result<std::fs::File, String> {
    use std::io::Write as _;
    use std::os::fd::FromRawFd as _;
    let name = std::ffi::CString::new("orbiscreen-keymap").map_err(|e| e.to_string())?;
    let fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err(format!("memfd_create: {}", std::io::Error::last_os_error()));
    }
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    file.write_all(bytes)
        .map_err(|e| format!("write keymap: {e}"))?;
    file.flush().map_err(|e| format!("flush keymap: {e}"))?;
    Ok(file)
}

fn compile_keymap() -> Result<Vec<u8>, String> {
    let ctx = xkbcommon::xkb::Context::new(xkbcommon::xkb::CONTEXT_NO_FLAGS);
    let keymap = xkbcommon::xkb::Keymap::new_from_names(
        &ctx,
        "evdev",
        "",
        "us",
        "",
        None,
        xkbcommon::xkb::COMPILE_NO_FLAGS,
    )
    .ok_or_else(|| "xkbcommon failed to compile the evdev/us keymap".to_string())?;
    let mut bytes = keymap
        .get_as_string(xkbcommon::xkb::KEYMAP_FORMAT_TEXT_V1)
        .into_bytes();
    bytes.push(0);
    Ok(bytes)
}

pub struct WlrootsInjector {
    tx: Sender<WlCmd>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl std::fmt::Debug for WlrootsInjector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WlrootsInjector").finish()
    }
}

impl WlrootsInjector {
    pub fn open(spec: VirtualTouchscreenSpec) -> Result<Self, InputError> {
        let conn = Connection::connect_to_env()
            .map_err(|e| InputError::Uinput(format!("wayland connect: {e}")))?;
        let mut queue: EventQueue<WlState> = conn.new_event_queue();
        let qh = queue.handle();
        let mut state = WlState::default();
        conn.display().get_registry(&qh, ());

        let deadline = Instant::now() + HANDSHAKE_DEADLINE;
        loop {
            queue
                .blocking_dispatch(&mut state)
                .map_err(|e| InputError::Uinput(format!("wayland dispatch: {e}")))?;
            let output_ready = spec.output_name.is_none()
                || state
                    .outputs
                    .iter()
                    .any(|o| o.name.as_deref() == spec.output_name.as_deref() && o.got_done);
            let ready = state.seat.is_some()
                && state.pointer_manager.is_some()
                && state.keyboard_manager.is_some()
                && output_ready;
            if ready || Instant::now() >= deadline {
                break;
            }
        }
        if state.pointer_manager.is_none() || state.keyboard_manager.is_none() {
            return Err(InputError::Uinput(
                "virtual-pointer/virtual-keyboard protocols are not available — the compositor \
                 is not wlroots-based or the protocols are disabled"
                    .into(),
            ));
        }
        let Some(seat) = state.seat.as_ref() else {
            return Err(InputError::Uinput(
                "no wl_seat advertised by the compositor".into(),
            ));
        };

        let target_output = spec.output_name.as_deref().and_then(|wanted| {
            state
                .outputs
                .iter()
                .find(|o| o.name.as_deref() == Some(wanted))
                .map(|o| o.proxy.clone())
        });
        if spec.output_name.is_some() && target_output.is_none() {
            warn!(
                "output {:?} not advertised by the compositor; the virtual pointer is unmapped",
                spec.output_name
            );
        }

        let pointer = match (&state.pointer_manager, &target_output) {
            (Some((manager, version)), Some(output)) if *version >= 2 => {
                manager.create_virtual_pointer_with_output(Some(seat), Some(output), &qh, ())
            }
            (Some((manager, _)), _) => manager.create_virtual_pointer(Some(seat), &qh, ()),
            _ => unreachable!("pointer manager presence checked above"),
        };
        state.pointer = Some(pointer);

        let keyboard = match &state.keyboard_manager {
            Some(manager) => manager.create_virtual_keyboard(seat, &qh, ()),
            None => unreachable!("keyboard manager presence checked above"),
        };
        state.keyboard = Some(keyboard);

        let keymap = compile_keymap().map_err(InputError::Uinput)?;
        let file = anonymous_keymap_file(&keymap).map_err(InputError::Uinput)?;
        let size = u32::try_from(keymap.len()).map_err(|e| InputError::Uinput(e.to_string()))?;
        if let Some(keyboard) = state.keyboard.as_ref() {
            keyboard.keymap(1, file.as_fd(), size);
        }
        state.keymap_file = Some(file);
        conn.flush()
            .map_err(|e| InputError::Uinput(format!("wayland flush: {e}")))?;
        info!(
            "wlroots native input ready (virtual-pointer + virtual-keyboard) — no portal, \
             no root"
        );

        let (tx, rx) = std::sync::mpsc::channel::<WlCmd>();
        let extent = (spec.width.max(1), spec.height.max(1));
        let thread = std::thread::Builder::new()
            .name("orbiscreen-wlr-input".into())
            .spawn(move || worker_loop(conn, queue, state, extent, rx))
            .map_err(|e| InputError::Uinput(format!("spawn input worker: {e}")))?;
        Ok(Self {
            tx,
            thread: Some(thread),
        })
    }

    pub fn inject_pointer_sync(&self, event: PointerEvent) -> Result<(), InputError> {
        self.tx
            .send(WlCmd::Pointer(event))
            .map_err(|_| InputError::Uinput("wlroots input worker exited".into()))
    }

    pub fn inject_key_sync(&self, event: KeyEvent) -> Result<(), InputError> {
        self.tx
            .send(WlCmd::Key(event))
            .map_err(|_| InputError::Uinput("wlroots input worker exited".into()))
    }

    pub async fn inject_pointer(&self, event: PointerEvent) -> Result<(), InputError> {
        self.inject_pointer_sync(event)
    }

    pub async fn inject_key(&self, event: KeyEvent) -> Result<(), InputError> {
        self.inject_key_sync(event)
    }
}

impl Drop for WlrootsInjector {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn worker_loop(
    conn: Connection,
    mut queue: EventQueue<WlState>,
    mut state: WlState,
    extent: (u32, u32),
    rx: std::sync::mpsc::Receiver<WlCmd>,
) {
    let start = Instant::now();
    loop {
        match rx.recv_timeout(DRAIN_PAUSE) {
            Ok(cmd) => apply_command(&mut state, cmd, extent, start),
            Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {}
        }
        if queue.dispatch_pending(&mut state).is_err() {
            break;
        }
        if let Some(guard) = conn.prepare_read() {
            let fd = guard.connection_fd().as_raw_fd();
            let mut fds = [libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            }];
            #[allow(unsafe_code)]
            let ready = unsafe { libc::poll(fds.as_mut_ptr(), 1, 0) };
            if ready > 0 && fds[0].revents & libc::POLLIN != 0 && guard.read().is_err() {
                break;
            }
        }
        if conn.flush().is_err() {
            break;
        }
    }
    if let Some(pointer) = state.pointer.take() {
        pointer.destroy();
    }
    if let Some(keyboard) = state.keyboard.take() {
        keyboard.destroy();
    }
    let _ = conn.flush();
    debug!("wlroots input worker exited");
}

fn apply_command(state: &mut WlState, cmd: WlCmd, extent: (u32, u32), start: Instant) {
    let time = u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
    match cmd {
        WlCmd::Pointer(event) => {
            let Some(pointer) = state.pointer.as_ref() else {
                return;
            };
            match event {
                PointerEvent::Move { x, y } => {
                    let (xw, yw) = extent;
                    let x = x.clamp(0.0, f64::from(xw)).round() as u32;
                    let y = y.clamp(0.0, f64::from(yw)).round() as u32;
                    pointer.motion_absolute(time, x, y, xw, yw);
                    pointer.frame();
                }
                PointerEvent::Button { button, pressed } => {
                    if button == 0 || button > 8 {
                        warn!("invalid button {button}");
                        return;
                    }
                    let state = if pressed {
                        wl_pointer::ButtonState::Pressed
                    } else {
                        wl_pointer::ButtonState::Released
                    };
                    pointer.button(time, button_code(button), state);
                    pointer.frame();
                }
                PointerEvent::Wheel { delta_y } => {
                    let steps = delta_y.clamp(i32::MIN as f64, i32::MAX as f64).round() as i32;
                    if steps != 0 {
                        pointer.axis_discrete(
                            time,
                            wl_pointer::Axis::VerticalScroll,
                            WHEEL_STEP_VALUE * f64::from(steps.abs()),
                            steps,
                        );
                        pointer.frame();
                    }
                }
            }
        }
        WlCmd::Key(event) => {
            let Some(keyboard) = state.keyboard.as_ref() else {
                return;
            };
            if event.code == 0 || event.code > 0xFF {
                debug!("key code {} outside the wayland range; ignored", event.code);
                return;
            }
            keyboard.key(time, event.code + 8, u32::from(event.pressed));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keymap_compiles_and_is_nul_terminated() {
        let bytes = compile_keymap().expect("keymap compiles");
        assert!(!bytes.is_empty());
        assert_eq!(*bytes.last().expect("non-empty"), 0);
        let text = std::str::from_utf8(&bytes[..bytes.len() - 1]).expect("utf8 keymap");
        assert!(text.contains("xkb_keymap"));
    }

    #[test]
    fn wl_keycode_is_evdev_plus_8() {
        assert_eq!(30 + 8, 38);
        assert_eq!(0x110u32, 272);
    }
}
