// Orbiscreen - orbiscreen-capture integration tests (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use std::io::{Read as _, Write as _};
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
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

fn which(binary: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(binary))
        .find(|path| path.is_file())
}

fn wait_for_socket(runtime_dir: &Path, uid: u32, pid: u32) -> Option<PathBuf> {
    let wanted = format!("sway-ipc.{uid}.{pid}.sock");
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        if let Ok(entries) = std::fs::read_dir(runtime_dir) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy() == wanted {
                    return Some(entry.path());
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

    let mut command = Command::new(sway);
    command
        .args(["-c", config_path.to_str()?])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("WLR_BACKENDS", "headless")
        .env("WLR_LIBINPUT_NO_DEVICES", "1")
        .env("XDG_RUNTIME_DIR", &runtime_dir)
        .env("WAYLAND_DISPLAY", WAYLAND_DISPLAY)
        .env_remove("SWAYSOCK");
    let child = command.spawn().ok()?;
    let pid = child.id();
    #[allow(unsafe_code)]
    let uid = unsafe { libc::getuid() };
    let socket = wait_for_socket(&runtime_dir, uid, pid);
    let Some(socket) = socket else {
        let mut c = child;
        let mut log = String::new();
        if let Some(mut err) = c.stderr.take() {
            let _ = err.read_to_string(&mut log);
        }
        let _ = c.kill();
        let _ = c.wait();
        eprintln!("sway did not open its IPC socket (log: {log})");
        return None;
    };

    let keys = ["WAYLAND_DISPLAY", "XDG_RUNTIME_DIR", "SWAYSOCK"];
    let saved = keys
        .iter()
        .map(|key| (*key, std::env::var_os(key)))
        .collect();
    std::env::set_var("WAYLAND_DISPLAY", WAYLAND_DISPLAY);
    std::env::set_var("XDG_RUNTIME_DIR", &runtime_dir);
    std::env::set_var("SWAYSOCK", &socket);

    Some(SwaySession {
        child,
        runtime_dir,
        socket,
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

fn sway_output_names(socket: &Path) -> Vec<String> {
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
        .filter_map(|o| o.get("name").and_then(|n| n.as_str()))
        .map(str::to_string)
        .collect()
}

#[test]
fn sway_headless_capture_and_virtual_output() {
    if which("sway").is_none() {
        eprintln!("sway is not installed — skipping headless integration test");
        return;
    }
    let Some(session) = start_sway() else {
        panic!("failed to start sway in headless mode");
    };

    let capture = WlrScreencopyCapture::open(WlrScreencopySpec::default())
        .expect("screencopy capture opens against headless sway");
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
        sway_output_names(&session.socket).contains(&virtual_name),
        "the created output is visible to sway IPC"
    );

    let virtual_capture = WlrScreencopyCapture::open(WlrScreencopySpec {
        output_name: Some(virtual_name.clone()),
    })
    .expect("screencopy capture opens on the virtual output by name");
    assert_eq!(virtual_capture.dimensions(), virtual_output.dimensions());
    let frame = collect_frame(&virtual_capture);
    assert_eq!(frame.data.len() as u32, frame.width * frame.height * 4);
    drop(virtual_capture);

    drop(virtual_output);
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut names = sway_output_names(&session.socket);
    while names.contains(&virtual_name) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
        names = sway_output_names(&session.socket);
    }
    assert!(
        !names.contains(&virtual_name),
        "dropping the guard removes the virtual output"
    );
}
