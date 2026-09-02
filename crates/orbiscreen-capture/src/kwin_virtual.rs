// Orbiscreen - kwin_virtual.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
use std::io::Write as _;
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gstreamer::prelude::*;
use gstreamer_app::{AppSink, AppSinkCallbacks};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::Notify;
use tracing::instrument;
use wayland_client::backend::WaylandError;
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols_plasma::screencast::v1::client::zkde_screencast_stream_unstable_v1::{
    Event as StreamEvent, ZkdeScreencastStreamUnstableV1,
};
use wayland_protocols_plasma::screencast::v1::client::zkde_screencast_unstable_v1::ZkdeScreencastUnstableV1;

use super::{sample_to_captured_frame, CaptureError, CapturedFrame};

const POINTER_EMBEDDED: u32 = 2;

const FRAME_CHANNEL_CAPACITY: usize = 2;

const HANDSHAKE_DEADLINE: Duration = Duration::from_secs(5);
const EVENT_POLL_TIMEOUT_MS: i32 = 100;

const KWIN_INTERFACES_KEY: &str = "X-KDE-Wayland-Interfaces=zkde_screencast_unstable_v1";
const PERMISSION_FILE_NAME: &str = "orbiscreen.kwin.desktop";

#[derive(Debug, Clone)]
pub struct KwinVirtualSpec {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Error)]
pub enum KwinVirtualError {
    #[error("KWin screencast protocol (zkde_screencast_unstable_v1) is not available on this compositor")]
    ProtocolUnavailable,
    #[error("compositor is too old: virtual output streaming needs protocol version >= 2")]
    ProtocolTooOld,
    #[error("requested size {0}x{1} exceeds the protocol's i32 dimensions")]
    UnsupportedSize(u32, u32),
    #[error("KWin rejected the virtual output stream: {0}")]
    StreamFailed(String),
    #[error("timed out waiting for KWin to create the virtual output stream")]
    Timeout,
    #[error("wayland error: {0}")]
    Wayland(String),
}

impl From<KwinVirtualError> for CaptureError {
    fn from(error: KwinVirtualError) -> Self {
        CaptureError::Io(error.to_string())
    }
}

#[derive(Debug, Default)]
struct StreamShared {
    node_id: Mutex<Option<u32>>,
    failed: Mutex<Option<String>>,
    closed: AtomicBool,
}

#[derive(Debug, Default)]
struct HandshakeState {
    registry: Option<WlRegistry>,
    screencast_global: Option<(u32, u32)>,
}

struct WaylandSession {
    conn: Connection,
    queue: EventQueue<HandshakeState>,
    state: HandshakeState,
}

impl WaylandSession {
    fn connect() -> Result<Self, KwinVirtualError> {
        let conn =
            Connection::connect_to_env().map_err(|e| KwinVirtualError::Wayland(e.to_string()))?;
        let mut queue: EventQueue<HandshakeState> = conn.new_event_queue();
        let qh = queue.handle();
        let mut state = HandshakeState::default();

        conn.display().get_registry(&qh, ());
        queue
            .roundtrip(&mut state)
            .map_err(|e| KwinVirtualError::Wayland(e.to_string()))?;
        Ok(Self { conn, queue, state })
    }
}

impl Dispatch<WlRegistry, ()> for HandshakeState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == <ZkdeScreencastUnstableV1 as Proxy>::interface().name {
                state.registry = Some(registry.clone());
                state.screencast_global = Some((name, version));
            }
        }
    }
}

impl Dispatch<ZkdeScreencastUnstableV1, ()> for HandshakeState {
    fn event(
        _: &mut Self,
        _: &ZkdeScreencastUnstableV1,
        _: <ZkdeScreencastUnstableV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZkdeScreencastStreamUnstableV1, Arc<StreamShared>> for HandshakeState {
    fn event(
        _: &mut Self,
        _: &ZkdeScreencastStreamUnstableV1,
        event: <ZkdeScreencastStreamUnstableV1 as Proxy>::Event,
        shared: &Arc<StreamShared>,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            StreamEvent::Created { node } => {
                *shared.node_id.lock().unwrap_or_else(|e| e.into_inner()) = Some(node);
            }
            StreamEvent::Failed { error } => {
                *shared.failed.lock().unwrap_or_else(|e| e.into_inner()) = Some(error);
            }
            StreamEvent::Closed => {
                shared.closed.store(true, Ordering::Relaxed);
            }
            _ => {}
        }
    }
}

fn is_kde_session() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|v| {
            v.to_ascii_lowercase()
                .split(':')
                .any(|component| component == "kde")
        })
        .unwrap_or(false)
}

fn user_applications_dir() -> Option<PathBuf> {
    if let Some(data_home) = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
    {
        return Some(data_home.join("applications"));
    }
    let home = std::env::var_os("HOME")?;
    let mut path = PathBuf::from(home);
    path.push(".local/share/applications");
    Some(path)
}

fn permission_file_matches(path: &Path, exe: &str) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(meta) if !meta.file_type().is_symlink() => {}
        _ => return false,
    }
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    content
        .lines()
        .any(|line| line.trim() == KWIN_INTERFACES_KEY)
        && content.lines().any(|line| {
            line.strip_prefix("Exec=")
                .map(|e| e.trim() == exe)
                .unwrap_or(false)
        })
}

fn atomic_write(path: &Path, contents: &str) -> std::io::Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| std::io::Error::other("permission file has no parent directory"))?;
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join(format!(
        ".{name}.tmp{}",
        std::process::id(),
        name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("orbiscreen.kwin.desktop")
    ));
    {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o644)
            .open(&tmp)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
    }

    std::fs::rename(&tmp, path)
}

fn client_executable() -> std::io::Result<PathBuf> {
    if let Some(appimage) = std::env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty() && p.is_file())
    {
        return Ok(appimage);
    }
    std::env::current_exe().map_err(|e| std::io::Error::other(format!("resolve current exe: {e}")))
}

fn ensure_kwin_permission_file() {
    let write = |path: &Path, exe: &str| -> std::io::Result<()> {
        atomic_write(
            path,
            &format!(
                "[Desktop Entry]\n\
                 Exec={exe}\n\
                 {KWIN_INTERFACES_KEY}\n\
                 Type=Application\n\
                 Name=Orbiscreen KWin screencast permission\n\
                 Comment=Allows the Orbiscreen daemon to create virtual displays\n\
                 NoDisplay=true\n"
            ),
        )
    };

    let Ok(exe) = client_executable() else {
        tracing::warn!("could not resolve the daemon executable for the KWin permission file");
        return;
    };
    let exe = match exe.to_str() {
        Some(exe) => exe,
        None => {
            tracing::warn!(
                "daemon executable path is not valid UTF-8; skipping KWin permission file"
            );
            return;
        }
    };

    let Some(apps_dir) = user_applications_dir() else {
        tracing::warn!(
            "cannot determine a user applications directory (set HOME or XDG_DATA_HOME); \
             KWin may not advertise the screencast protocol"
        );
        return;
    };
    let permission_file = apps_dir.join(PERMISSION_FILE_NAME);
    if permission_file_matches(&permission_file, exe) {
        return;
    }
    match write(&permission_file, exe) {
        Ok(()) => tracing::info!(
            file = %permission_file.display(),
            "granted KWin screencast access via user desktop file"
        ),
        Err(e) => {
            tracing::warn!("could not write the KWin permission file: {e}");
            return;
        }
    }

    for candidate in ["/usr/bin/kbuildsycoca6", "/usr/local/bin/kbuildsycoca6"] {
        if Path::new(candidate).is_file() {
            let _ = std::process::Command::new(candidate)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            break;
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct KwinVirtualCapture {
    _pipeline: gstreamer::Pipeline,
    stream: ZkdeScreencastStreamUnstableV1,
    rx: tokio::sync::Mutex<mpsc::Receiver<CapturedFrame>>,
    width: u32,
    height: u32,
    stop: Arc<AtomicBool>,
    ended: Arc<AtomicBool>,
    ended_notify: Arc<Notify>,
    event_thread: Option<std::thread::JoinHandle<()>>,
    _damage_pump: super::damage_pump::DamagePumpHandle,
}

impl KwinVirtualCapture {
    #[instrument(skip_all, fields(width = spec.width, height = spec.height))]
    pub fn open(spec: KwinVirtualSpec) -> Result<Self, KwinVirtualError> {
        let Ok(width) = i32::try_from(spec.width) else {
            return Err(KwinVirtualError::UnsupportedSize(spec.width, spec.height));
        };
        let Ok(height) = i32::try_from(spec.height) else {
            return Err(KwinVirtualError::UnsupportedSize(spec.width, spec.height));
        };
        gstreamer::init().map_err(|e| KwinVirtualError::Wayland(format!("gst init: {e}")))?;

        let mut session = WaylandSession::connect()?;
        if session.state.screencast_global.is_none() && is_kde_session() {
            ensure_kwin_permission_file();
            for _ in 0..5 {
                std::thread::sleep(Duration::from_millis(500));
                session = WaylandSession::connect()?;
                if session.state.screencast_global.is_some() {
                    break;
                }
            }
        }
        if session.state.screencast_global.is_none() {
            return Err(KwinVirtualError::ProtocolUnavailable);
        }

        let (global_name, advertised_version) = session
            .state
            .screencast_global
            .ok_or(KwinVirtualError::ProtocolUnavailable)?;

        let client_max = <ZkdeScreencastUnstableV1 as Proxy>::interface().version;
        let version = advertised_version.min(client_max);
        if version < 2 {
            return Err(KwinVirtualError::ProtocolTooOld);
        }
        let registry = session
            .state
            .registry
            .clone()
            .ok_or(KwinVirtualError::ProtocolUnavailable)?;
        let screencast: ZkdeScreencastUnstableV1 =
            registry.bind(global_name, version, &session.queue.handle(), ());

        let shared = Arc::new(StreamShared::default());
        let stream = if version >= 4 {
            screencast.stream_virtual_output_with_description(
                "ORBISCREEN".to_string(),
                "Orbiscreen Virtual Display".to_string(),
                width,
                height,
                1.0,
                POINTER_EMBEDDED,
                &session.queue.handle(),
                shared.clone(),
            )
        } else {
            screencast.stream_virtual_output(
                "ORBISCREEN".to_string(),
                width,
                height,
                1.0,
                POINTER_EMBEDDED,
                &session.queue.handle(),
                shared.clone(),
            )
        };

        let deadline = Instant::now() + HANDSHAKE_DEADLINE;
        let node_id = loop {
            session
                .queue
                .roundtrip(&mut session.state)
                .map_err(|e| KwinVirtualError::Wayland(e.to_string()))?;
            if let Some(err) = shared
                .failed
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone()
            {
                return Err(KwinVirtualError::StreamFailed(err));
            }
            if shared.closed.load(Ordering::Relaxed) {
                return Err(KwinVirtualError::StreamFailed(
                    "compositor closed the stream during setup".into(),
                ));
            }
            if let Some(node) = *shared.node_id.lock().unwrap_or_else(|e| e.into_inner()) {
                break node;
            }
            if Instant::now() >= deadline {
                return Err(KwinVirtualError::Timeout);
            }
            std::thread::sleep(Duration::from_millis(10));
        };

        let pipeline_str = format!(
            "pipewiresrc path={node_id} do-timestamp=true \
             ! video/x-raw \
             ! videoconvert \
             ! videoscale \
             ! video/x-raw,format=BGRA,width={},height={} \
             ! appsink name=sink drop=true sync=false max-buffers=1 emit-signals=false",
            spec.width, spec.height
        );
        let pipeline = gstreamer::parse::launch(&pipeline_str)
            .map_err(|e| KwinVirtualError::Wayland(format!("gst launch: {e}")))?
            .downcast::<gstreamer::Pipeline>()
            .map_err(|_| KwinVirtualError::Wayland("gst pipeline downcast".into()))?;

        let appsink = pipeline
            .by_name("sink")
            .ok_or_else(|| KwinVirtualError::Wayland("appsink not found".into()))?
            .downcast::<AppSink>()
            .map_err(|_| KwinVirtualError::Wayland("appsink downcast".into()))?;

        let (tx, rx) = mpsc::channel::<CapturedFrame>(FRAME_CHANNEL_CAPACITY);
        let pool = orbiscreen_core::frame_pool::FramePool::new();
        appsink.set_callbacks(
            AppSinkCallbacks::builder()
                .new_sample(move |sink| {
                    let sample = match sink.pull_sample() {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::warn!("pull_sample error: {e}");
                            return Err(gstreamer::FlowError::Eos);
                        }
                    };
                    if let Some(frame) = sample_to_captured_frame(&sample, &pool) {
                        if tx.try_send(frame).is_err() {
                            tracing::debug!("capture frame dropped: consumer channel full");
                        }
                    }
                    Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );

        if let Some(bus) = pipeline.bus() {
            bus.set_sync_handler(|_bus, msg| {
                match msg.view() {
                    gstreamer::MessageView::Error(err) => tracing::error!(
                        target: "orbiscreen_capture::kwin_virtual",
                        "gstreamer capture error: {} (debug: {})",
                        err.error(),
                        err.debug().unwrap_or_default()
                    ),
                    gstreamer::MessageView::Warning(warn) => tracing::warn!(
                        target: "orbiscreen_capture::kwin_virtual",
                        "gstreamer capture warning: {} (debug: {})",
                        warn.error(),
                        warn.debug().unwrap_or_default()
                    ),
                    _ => {}
                }
                gstreamer::BusSyncReply::Drop
            });
        }

        pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| KwinVirtualError::Wayland(format!("State error: {e}")))?;

        let pump_interval = Duration::from_millis(16);
        let damage_pump = super::damage_pump::spawn(pump_interval);

        let ended = Arc::new(AtomicBool::new(false));
        let ended_notify = Arc::new(Notify::new());
        let stop = Arc::new(AtomicBool::new(false));
        let event_thread = std::thread::Builder::new()
            .name("orbiscreen-kwin-events".into())
            .spawn({
                let shared = Arc::clone(&shared);
                let ended = Arc::clone(&ended);
                let ended_notify = Arc::clone(&ended_notify);
                let stop = Arc::clone(&stop);
                move || {
                    pump_events(
                        session.conn,
                        session.queue,
                        session.state,
                        shared,
                        stop,
                        ended,
                        ended_notify,
                    )
                }
            })
            .map_err(|e| KwinVirtualError::Wayland(format!("spawn event thread: {e}")))?;

        Ok(Self {
            _pipeline: pipeline,
            stream,
            rx: tokio::sync::Mutex::new(rx),
            width: spec.width,
            height: spec.height,
            stop,
            ended,
            ended_notify,
            event_thread: Some(event_thread),
            _damage_pump: damage_pump,
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn is_ended(&self) -> bool {
        self.ended.load(Ordering::Relaxed)
    }

    pub async fn next_frame(&self) -> Result<CapturedFrame, CaptureError> {
        let mut rx = self.rx.lock().await;
        loop {
            if self.ended.load(Ordering::Relaxed) {
                return Err(CaptureError::Io(
                    "KWin virtual output stream was closed".into(),
                ));
            }
            tokio::select! {
                frame = rx.recv() => {
                    return match frame {
                        Some(frame) => Ok(frame),
                        None => Err(CaptureError::Io(
                            "KWin virtual output pipeline closed".into(),
                        )),
                    };
                }
                _ = self.ended_notify.notified() => continue,
            }
        }
    }
}

impl Drop for KwinVirtualCapture {
    fn drop(&mut self) {
        let _ = self._pipeline.set_state(gstreamer::State::Null);
        self.stop.store(true, Ordering::Relaxed);

        self.stream.close();
        if let Some(handle) = self.event_thread.take() {
            let _ = handle.join();
        }
    }
}

#[allow(unsafe_code)]
fn pump_events(
    conn: Connection,
    mut queue: EventQueue<HandshakeState>,
    mut state: HandshakeState,
    shared: Arc<StreamShared>,
    stop: Arc<AtomicBool>,
    ended: Arc<AtomicBool>,
    ended_notify: Arc<Notify>,
) {
    loop {
        if stop.load(Ordering::Relaxed) || shared.closed.load(Ordering::Relaxed) {
            break;
        }
        if let Err(e) = queue.dispatch_pending(&mut state) {
            tracing::warn!("kwin event dispatch failed: {e}");
            break;
        }
        if stop.load(Ordering::Relaxed) || shared.closed.load(Ordering::Relaxed) {
            break;
        }

        let Some(guard) = conn.prepare_read() else {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        };
        let fd = guard.connection_fd().as_raw_fd();
        let mut fds = [libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        }];
        let ready = unsafe { libc::poll(fds.as_mut_ptr(), 1, EVENT_POLL_TIMEOUT_MS) };
        if ready > 0 && fds[0].revents & libc::POLLIN != 0 {
            if let Err(e) = guard.read() {
                let would_block = matches!(&e, WaylandError::Io(io) if io.kind() == std::io::ErrorKind::WouldBlock);
                if !would_block {
                    tracing::warn!("kwin read_events failed: {e}");
                    break;
                }
            }
        }
        let _ = conn.flush();
    }
    ended.store(true, Ordering::Relaxed);
    ended_notify.notify_one();
}
