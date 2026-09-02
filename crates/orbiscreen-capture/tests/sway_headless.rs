// Orbiscreen - sway_headless.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use std::io::{Read as _, Write as _};
use std::os::unix::fs::FileTypeExt as _;
use std::os::unix::io::IntoRawFd;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use orbiscreen_capture::wlr_screencopy::{WlrScreencopyCapture, WlrScreencopySpec};
use orbiscreen_capture::wlr_virtual_output::{VirtualOutputSpec, WlrootsVirtualOutput};
use orbiscreen_capture::CapturedFrame;

const WAYLAND_DISPLAY: &str = "wayland-orbiscreen-test";

struct SwaySession {
    child: Child,
    runtime_dir: PathBuf,
    socket: PathBuf,
    wayland_socket: PathBuf,
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

fn which(binary: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(binary))
        .find(|path| path.is_file())
}

fn wait_for_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let wanted = dir.join(name);
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if wanted.exists() {
            return Some(wanted);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    None
}

fn wait_for_wayland_socket(dir: &Path) -> Option<String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let is_socket = entry.file_type().map(|t| t.is_socket()).unwrap_or(false);
                if is_socket {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.starts_with("wayland-") {
                        return Some(name);
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    None
}

fn start_sway() -> Option<SwaySession> {
    let sway = which("sway")?;
    let runtime_dir =
        std::env::temp_dir().join(format!("orbiscreen-sway-test-{}", std::process::id()));
    std::fs::create_dir_all(&runtime_dir).ok()?;
    let config_path = runtime_dir.join("sway-config");
    std::fs::write(&config_path, "xwayland disable\n").ok()?;
    let log_path = runtime_dir.join("sway.log");
    let log_file = std::fs::File::create(&log_path).ok()?;
    let log_err = log_file.try_clone().ok()?;

    let mut command = Command::new(sway);
    command
        .args(["-d", "-c", config_path.to_str()?])
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_err))
        .env("WLR_BACKENDS", "headless")
        .env("WLR_RENDERER", "pixman")
        .env("WLR_LIBINPUT_NO_DEVICES", "1")
        .env("XDG_RUNTIME_DIR", &runtime_dir)
        .env("WAYLAND_DISPLAY", WAYLAND_DISPLAY)
        .env_remove("SWAYSOCK");
    let mut child = command.spawn().ok()?;
    let pid = child.id();
    #[allow(unsafe_code)]
    let uid = unsafe { libc::getuid() };
    let ipc_name = format!("sway-ipc.{uid}.{pid}.sock");
    let fail = |mut c: Child, reason: &str| -> Option<SwaySession> {
        let _ = c.kill();
        let _ = c.wait();
        let log = std::fs::read_to_string(&log_path).unwrap_or_default();
        eprintln!("sway headless test setup failed ({reason}); sway log:\n{log}");
        let _ = std::fs::remove_dir_all(&runtime_dir);
        None
    };
    let Some(socket) = wait_for_file(&runtime_dir, &ipc_name) else {
        return fail(child, "IPC socket never appeared");
    };
    let Some(wayland_name) = wait_for_wayland_socket(&runtime_dir) else {
        return fail(child, "wayland socket never appeared");
    };
    let wayland_socket = runtime_dir.join(&wayland_name);
    if let Err(e) = UnixStream::connect(&wayland_socket) {
        return fail(
            child,
            &format!("discovered wayland socket {wayland_socket:?} is not connectable: {e}"),
        );
    }
    match child.try_wait() {
        Ok(None) => {}
        status => return fail(child, &format!("sway exited early: {status:?}")),
    }

    let keys = [
        "WAYLAND_DISPLAY",
        "XDG_RUNTIME_DIR",
        "SWAYSOCK",
        "WAYLAND_SOCKET",
    ];
    let saved = keys
        .iter()
        .map(|key| (*key, std::env::var_os(key)))
        .collect();
    std::env::remove_var("WAYLAND_DISPLAY");
    std::env::set_var("XDG_RUNTIME_DIR", &runtime_dir);
    std::env::set_var("SWAYSOCK", &socket);

    Some(SwaySession {
        child,
        runtime_dir,
        socket,
        wayland_socket,
        saved,
    })
}

impl Drop for SwaySession {
    fn drop(&mut self) {
        for (key, value) in self.saved.drain(..) {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.runtime_dir);
    }
}

fn open_capture(socket_path: &Path, spec: WlrScreencopySpec, what: &str) -> WlrScreencopyCapture {
    let mut last_err: Option<String> = None;
    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline {
        match UnixStream::connect(socket_path) {
            Ok(stream) => {
                std::env::set_var("WAYLAND_SOCKET", stream.into_raw_fd().to_string());
                match WlrScreencopyCapture::open(spec.clone()) {
                    Ok(capture) => return capture,
                    Err(e) => last_err = Some(e.to_string()),
                }
            }
            Err(e) => last_err = Some(format!("connect {socket_path:?}: {e}")),
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("{what} did not open within the deadline: {:?}", last_err);
}

fn collect_frame(capture: &WlrScreencopyCapture) -> CapturedFrame {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    runtime
        .block_on(async {
            tokio::time::timeout(Duration::from_secs(20), capture.next_frame()).await
        })
        .expect("frame arrived within the deadline")
        .expect("frame without error")
}

fn sway_output_states(socket: &Path) -> Vec<(String, bool)> {
    const GET_OUTPUTS: u32 = 3;
    let payload = "";
    let mut message = Vec::with_capacity(14 + payload.len());
    message.extend_from_slice(b"i3-ipc");
    message.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    message.extend_from_slice(&GET_OUTPUTS.to_le_bytes());

    let mut stream = UnixStream::connect(socket).expect("sway IPC connect");
    stream.write_all(&message).expect("sway IPC write");
    let mut header = [0u8; 14];
    stream.read_exact(&mut header).expect("sway IPC header");
    let len = u32::from_le_bytes(header[6..10].try_into().expect("length")) as usize;
    let mut body = vec![0u8; len];
    stream.read_exact(&mut body).expect("sway IPC body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("sway IPC outputs JSON");
    json.as_array()
        .expect("outputs list")
        .iter()
        .filter_map(|o| {
            o.get("name").and_then(|n| n.as_str()).map(|name| {
                let active = o.get("active").and_then(|a| a.as_bool()).unwrap_or(true);
                (name.to_string(), active)
            })
        })
        .collect()
}

#[test]
fn sway_headless_capture_and_virtual_output() {
    if which("sway").is_none() {
        eprintln!("sway is not installed; skipping headless integration test");
        return;
    }
    let Some(session) = start_sway() else {
        panic!("failed to start sway in headless mode");
    };

    let capture = open_capture(
        &session.wayland_socket,
        WlrScreencopySpec::default(),
        "headless output capture",
    );
    let (w, h) = capture.dimensions();
    assert!(w > 0 && h > 0, "headless output reports dimensions");
    let frame = collect_frame(&capture);
    assert_eq!(frame.width, w);
    assert_eq!(frame.height, h);
    assert_eq!(frame.data.len() as u32, w * h * 4);
    drop(capture);

    let virtual_output = WlrootsVirtualOutput::create(VirtualOutputSpec {
        width: 1024,
        height: 768,
        refresh_rate_hz: 60,
    })
    .expect("virtual output created via sway IPC");
    let virtual_name = virtual_output.name().to_string();
    assert!(
        sway_output_states(&session.socket)
            .iter()
            .any(|(name, _)| name == &virtual_name),
        "the created output is visible to sway IPC"
    );

    let virtual_capture = open_capture(
        &session.wayland_socket,
        WlrScreencopySpec {
            output_name: Some(virtual_name.clone()),
        },
        "virtual-output capture",
    );
    assert_eq!(virtual_capture.dimensions(), virtual_output.dimensions());
    let frame = collect_frame(&virtual_capture);
    assert_eq!(frame.data.len() as u32, frame.width * frame.height * 4);
    drop(virtual_capture);

    drop(virtual_output);
    let deadline = Instant::now() + Duration::from_secs(5);
    let still_active = |states: &[(String, bool)]| {
        states
            .iter()
            .any(|(name, active)| name == &virtual_name && *active)
    };
    let mut states = sway_output_states(&session.socket);
    while still_active(&states) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
        states = sway_output_states(&session.socket);
    }
    assert!(
        !still_active(&states),
        "dropping the guard removes the virtual output (or disables it when the compositor has no removal command)"
    );
}
