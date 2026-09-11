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
    pub output_name: Option<String>,
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

pub const VIRTUAL_OUTPUT_CONNECTOR: &str = "Virtual-ORBISCREEN";

pub fn kwin_output_config_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config/kwinoutputconfig.json"))
}

pub fn forget_saved_virtual_output(path: &Path, connector: &str) -> std::io::Result<bool> {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    let mut root: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return Ok(false),
    };
    fn strip(value: &mut serde_json::Value, connector: &str) -> bool {
        let mut changed = false;
        match value {
            serde_json::Value::Array(items) => {
                let before = items.len();
                items.retain(
                    |entry| match entry.get("connectorName").and_then(|v| v.as_str()) {
                        Some(name) => {
                            name != connector && !name.starts_with(&format!("{connector}-"))
                        }
                        None => true,
                    },
                );
                if items.len() != before {
                    changed = true;
                }
                for item in items.iter_mut() {
                    if strip(item, connector) {
                        changed = true;
                    }
                }
            }
            serde_json::Value::Object(map) => {
                for child in map.values_mut() {
                    if strip(child, connector) {
                        changed = true;
                    }
                }
            }
            _ => {}
        }
        changed
    }
    if !strip(&mut root, connector) {
        return Ok(false);
    }
    let pretty = serde_json::to_string_pretty(&root)
        .map_err(|e| std::io::Error::other(format!("serialize kwin output config: {e}")))?;
    atomic_write(path, &pretty)?;
    Ok(true)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KscreenOutput {
    pub name: String,
    pub uuid: Option<String>,
    pub enabled: bool,
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn parse_kscreen_outputs(text: &str) -> Vec<KscreenOutput> {
    let text = strip_ansi(text);
    let mut current: Option<KscreenOutput> = None;
    let mut out = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("Output: ") {
            if let Some(done) = current.take() {
                out.push(done);
            }
            let mut parts = rest.split_whitespace();
            let _idx = parts.next();
            if let Some(name) = parts.next() {
                let uuid = parts
                    .next()
                    .filter(|tok| tok.contains('-'))
                    .map(str::to_string);
                current = Some(KscreenOutput {
                    name: name.to_string(),
                    uuid,
                    enabled: false,
                });
            }
        } else if line.trim() == "enabled" {
            if let Some(cur) = current.as_mut() {
                cur.enabled = true;
            }
        } else if line.trim() == "disabled" {
            if let Some(cur) = current.as_mut() {
                cur.enabled = false;
            }
        }
    }
    if let Some(done) = current {
        out.push(done);
    }
    out
}

fn is_portal_virtual_output(name: &str) -> bool {
    name.starts_with("Virtual-virtual-xdp-kde")
}

pub fn list_kscreen_outputs() -> Vec<KscreenOutput> {
    let out = std::process::Command::new("kscreen-doctor")
        .arg("-o")
        .env("TERM", "dumb")
        .env("NO_COLOR", "1")
        .output();
    match out {
        Ok(out) if out.status.success() => {
            parse_kscreen_outputs(&String::from_utf8_lossy(&out.stdout))
        }
        _ => Vec::new(),
    }
}

pub fn select_tablet_output(outputs: &[KscreenOutput], preferred: Option<&str>) -> Option<String> {
    let enabled: Vec<&str> = outputs
        .iter()
        .filter(|o| o.enabled)
        .map(|o| o.name.as_str())
        .collect();
    if let Some(name) = preferred {
        if enabled.contains(&name) {
            return Some(name.to_string());
        }
    }
    if let Some(name) = enabled
        .iter()
        .copied()
        .find(|n| n.starts_with("Virtual-ORBISCREEN-"))
    {
        return Some(name.to_string());
    }
    if enabled.contains(&VIRTUAL_OUTPUT_CONNECTOR) {
        return Some(VIRTUAL_OUTPUT_CONNECTOR.to_string());
    }
    None
}

pub fn output_uuid(name: &str) -> Option<String> {
    list_kscreen_outputs()
        .into_iter()
        .find(|o| o.enabled && o.name == name)
        .and_then(|o| o.uuid)
}

pub fn preferred_tablet_output(preferred: Option<&str>) -> Option<String> {
    select_tablet_output(&list_kscreen_outputs(), preferred)
}

pub fn disable_stale_portal_virtual_outputs() {
    for output in list_kscreen_outputs() {
        if output.enabled && is_portal_virtual_output(&output.name) {
            let spec = format!("output.{}.disable", output.name);
            match std::process::Command::new("kscreen-doctor")
                .arg(&spec)
                .status()
            {
                Ok(status) if status.success() => tracing::info!(
                    output = %output.name,
                    "disabled portal virtual output"
                ),
                Ok(status) => tracing::warn!(
                    output = %output.name,
                    %status,
                    "kscreen-doctor failed to disable portal virtual output"
                ),
                Err(e) => tracing::warn!(
                    output = %output.name,
                    "could not run kscreen-doctor to disable portal virtual output: {e}"
                ),
            }
        }
    }
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
    connector: String,
    stop: Arc<AtomicBool>,
    ended: Arc<AtomicBool>,
    ended_notify: Arc<Notify>,
    event_thread: Option<std::thread::JoinHandle<()>>,
    _damage_pump: super::damage_pump::DamagePumpHandle,
}

impl KwinVirtualCapture {
    #[instrument(skip_all, fields(width = spec.width, height = spec.height))]
    pub fn open(spec: KwinVirtualSpec) -> Result<Self, KwinVirtualError> {
        let base_name = spec
            .output_name
            .clone()
            .unwrap_or_else(|| "ORBISCREEN".to_string());
        let default_conn = format!("Virtual-{base_name}");
        let pid_connector = format!("Virtual-{base_name}-{}", std::process::id());
        if let Some(path) = kwin_output_config_path() {
            for connector in [default_conn.as_str(), pid_connector.as_str()] {
                match forget_saved_virtual_output(&path, connector) {
                    Ok(true) => tracing::info!(
                        file = %path.display(),
                        connector,
                        "cleared stale KWin config"
                    ),
                    Ok(false) => {}
                    Err(e) => tracing::warn!(
                        file = %path.display(),
                        connector,
                        "could not clear stale KWin virtual output config: {e}"
                    ),
                }
            }
        }
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

        let names = if base_name == "ORBISCREEN" {
            vec![
                "ORBISCREEN".to_string(),
                format!("ORBISCREEN-{}", std::process::id()),
            ]
        } else {
            vec![
                base_name.clone(),
                format!("{}-{}", base_name, std::process::id()),
            ]
        };
        let mut last_err: Option<KwinVirtualError> = None;
        let mut stream = None;
        let mut node_id = None;
        let mut shared_ok: Option<Arc<StreamShared>> = None;
        let mut accepted_name: Option<String> = None;
        for name in names {
            let shared = Arc::new(StreamShared::default());
            let candidate = if version >= 4 {
                screencast.stream_virtual_output_with_description(
                    name.clone(),
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
                    name.clone(),
                    width,
                    height,
                    1.0,
                    POINTER_EMBEDDED,
                    &session.queue.handle(),
                    shared.clone(),
                )
            };

            let deadline = Instant::now() + HANDSHAKE_DEADLINE;
            let result = loop {
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
                    break Err(KwinVirtualError::StreamFailed(err));
                }
                if shared.closed.load(Ordering::Relaxed) {
                    break Err(KwinVirtualError::StreamFailed(
                        "compositor closed the stream during setup".into(),
                    ));
                }
                if let Some(node) = *shared.node_id.lock().unwrap_or_else(|e| e.into_inner()) {
                    break Ok(node);
                }
                if Instant::now() >= deadline {
                    break Err(KwinVirtualError::Timeout);
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            match result {
                Ok(id) => {
                    tracing::info!(name, "KWin virtual display created via zkde-screencast");
                    stream = Some(candidate);
                    node_id = Some(id);
                    shared_ok = Some(shared);
                    accepted_name = Some(format!("Virtual-{name}"));
                    break;
                }
                Err(e) => {
                    tracing::warn!(name, "KWin virtual output '{name}' failed: {e}");
                    drop(candidate);
                    last_err = Some(e);
                }
            }
        }
        let (stream, node_id, shared) = match (stream, node_id, shared_ok) {
            (Some(stream), Some(node_id), Some(shared)) => (stream, node_id, shared),
            _ => {
                return Err(last_err.unwrap_or_else(|| {
                    KwinVirtualError::StreamFailed("no virtual output".into())
                }));
            }
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
        let damage_pump = super::damage_pump::spawn(accepted_name.clone(), pump_interval);

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
            connector: accepted_name.unwrap_or(default_conn),
            stop,
            ended,
            ended_notify,
            event_thread: Some(event_thread),
            _damage_pump: damage_pump,
        })
    }

    pub fn connector_name(&self) -> &str {
        &self.connector
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forget_saved_virtual_output_drops_connector() {
        let dir = std::env::temp_dir().join(format!("orbi-kwin-cfg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("kwinoutputconfig.json");
        std::fs::write(
            &path,
            r#"[{
                "data": [
                    {"connectorName": "eDP-1"},
                    {"connectorName": "Virtual-ORBISCREEN", "uuid": "dead"}
                ]
            }]"#,
        )
        .unwrap();
        assert!(forget_saved_virtual_output(&path, VIRTUAL_OUTPUT_CONNECTOR).unwrap());
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("Virtual-ORBISCREEN"));
        assert!(raw.contains("eDP-1"));
        assert!(!forget_saved_virtual_output(&path, VIRTUAL_OUTPUT_CONNECTOR).unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_kscreen_doctor_outputs() {
        let text = "\
\u{1b}[01;32mOutput: \u{1b}[0;0m1 eDP-1 8d4cd7b2-4072-46fe-9076-a472ff599d3e
\u{1b}[01;32menabled\u{1b}[0;0m
connected
Geometry: 0,0 1920x1200
Output: 2 DP-6 2185e147-5700-4a76-95c7-4ca01c705bea
enabled
Geometry: 1920,0 2560x1440
Output: 3 Virtual-virtual-xdp-kde- 3ab51ad6-f454-4c80-bee1-bb69124801de
enabled
Geometry: 4480,0 1920x1080
Output: 4 Virtual-ORBISCREEN deadbeef-0000-0000-0000-000000000000
disabled
";
        let outs = parse_kscreen_outputs(text);
        assert_eq!(
            outs,
            vec![
                KscreenOutput {
                    name: "eDP-1".into(),
                    uuid: Some("8d4cd7b2-4072-46fe-9076-a472ff599d3e".into()),
                    enabled: true
                },
                KscreenOutput {
                    name: "DP-6".into(),
                    uuid: Some("2185e147-5700-4a76-95c7-4ca01c705bea".into()),
                    enabled: true
                },
                KscreenOutput {
                    name: "Virtual-virtual-xdp-kde-".into(),
                    uuid: Some("3ab51ad6-f454-4c80-bee1-bb69124801de".into()),
                    enabled: true
                },
                KscreenOutput {
                    name: "Virtual-ORBISCREEN".into(),
                    uuid: Some("deadbeef-0000-0000-0000-000000000000".into()),
                    enabled: false
                },
            ]
        );
        assert_eq!(
            select_tablet_output(&outs, Some(VIRTUAL_OUTPUT_CONNECTOR)).as_deref(),
            None
        );
        let mut with_orbi = outs.clone();
        with_orbi[3].enabled = true;
        assert_eq!(
            select_tablet_output(&with_orbi, Some(VIRTUAL_OUTPUT_CONNECTOR)).as_deref(),
            Some(VIRTUAL_OUTPUT_CONNECTOR)
        );
        with_orbi.push(KscreenOutput {
            name: "Virtual-ORBISCREEN-123".into(),
            uuid: Some("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".into()),
            enabled: true,
        });
        assert_eq!(
            select_tablet_output(&with_orbi, Some("Virtual-ORBISCREEN-123")).as_deref(),
            Some("Virtual-ORBISCREEN-123")
        );
        assert_eq!(
            select_tablet_output(&with_orbi, Some(VIRTUAL_OUTPUT_CONNECTOR)).as_deref(),
            Some(VIRTUAL_OUTPUT_CONNECTOR)
        );
        assert_eq!(
            select_tablet_output(&with_orbi, None).as_deref(),
            Some("Virtual-ORBISCREEN-123")
        );
    }
}
