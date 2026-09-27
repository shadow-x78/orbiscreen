use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use orbiscreen_capture::kwin_virtual::{protocol_names_for, KwinVirtualCapture, KwinVirtualSpec};
use orbiscreen_encode::{EncodeParams, Encoder, EncoderKind};
use orbiscreen_input::{InputInjector, PointerEvent, VirtualTouchscreenSpec};
use orbiscreen_transport::{DisplayCommand, DisplayCtl, DisplayInfo, H264Packet, IncomingInput};
use tokio::sync::{broadcast, mpsc, watch};
use tracing::{info, warn};

const IDLE_AFTER_LAST_VIEWER: Duration = Duration::from_secs(120);
const WAITING_FOR_FIRST_VIEWER: Duration = Duration::from_secs(30);

/// A session is reaped only when nobody is attached and nothing is still
/// subscribed to its video broadcast. A USB/AOA reader can outlive a
/// mismatched `viewers` count; closing then kills both tablets at this timeout.
pub(crate) fn should_reap_idle_session(
    viewers: usize,
    video_receivers: usize,
    ever_attached: bool,
    idle_for: Duration,
) -> bool {
    if viewers > 0 || video_receivers > 0 {
        return false;
    }
    let limit = if ever_attached {
        IDLE_AFTER_LAST_VIEWER
    } else {
        WAITING_FOR_FIRST_VIEWER
    };
    idle_for >= limit
}

#[derive(Clone, Debug)]
pub struct HubConfig {
    pub encode_kind: EncoderKind,
    pub bitrate_kbps: u32,
    pub refresh_hz: u32,
    pub default_width: u32,
    pub default_height: u32,
    pub has_explicit_override: bool,
}

struct Session {
    info: DisplayInfo,
    client_name: String,
    client_key: Option<String>,

    bitrate_kbps: Option<u32>,
    video_tx: broadcast::Sender<H264Packet>,
    idr_tx: mpsc::Sender<()>,
    input_tx: mpsc::Sender<IncomingInput>,
    viewers: usize,
    ever_attached: bool,
    shutdown: watch::Sender<bool>,
    capture: Option<Arc<KwinVirtualCapture>>,
    encoder: Option<Arc<Encoder>>,
}

pub fn spawn_hub(cfg: HubConfig) -> DisplayCtl {
    let (tx, rx) = mpsc::channel(64);
    tokio::spawn(run_hub(cfg, rx));
    DisplayCtl::new(tx)
}

async fn run_hub(mut cfg: HubConfig, mut rx: mpsc::Receiver<DisplayCommand>) {
    let mut sessions: HashMap<String, Session> = HashMap::new();
    let mut idle_at: HashMap<String, tokio::time::Instant> = HashMap::new();
    let mut idle_tick = tokio::time::interval(Duration::from_millis(250));
    idle_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            cmd = rx.recv() => {
                let Some(cmd) = cmd else { break };
                if let DisplayCommand::Shutdown { reply } = cmd {
                    let ids: Vec<String> = sessions.keys().cloned().collect();
                    for id in ids {
                        close_session(&mut sessions, &id);
                    }
                    let _ = reply.send(());
                    break;
                }
                handle_cmd(&mut cfg, &mut sessions, &mut idle_at, cmd).await;
            }
            _ = idle_tick.tick() => {
                let now = tokio::time::Instant::now();
                let due: Vec<String> = idle_at
                    .iter()
                    .filter_map(|(id, at)| {
                        let Some(session) = sessions.get(id) else {
                            return Some(id.clone());
                        };
                        let idle_for = now.saturating_duration_since(*at);
                        let limit = if session.ever_attached {
                            IDLE_AFTER_LAST_VIEWER
                        } else {
                            WAITING_FOR_FIRST_VIEWER
                        };
                        if idle_for >= limit {
                            Some(id.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                for id in due {
                    let reap = sessions.get(&id).is_none_or(|session| {
                        should_reap_idle_session(
                            session.viewers,
                            session.video_tx.receiver_count(),
                            session.ever_attached,
                            IDLE_AFTER_LAST_VIEWER,
                        )
                    });
                    if reap {
                        if let Some(session) = sessions.get(&id) {
                            info!(
                                viewers = session.viewers,
                                receivers = session.video_tx.receiver_count(),
                                ever_attached = session.ever_attached,
                                "reaping idle client display"
                            );
                        }
                        idle_at.remove(&id);
                        if sessions.contains_key(&id) {
                            close_session(&mut sessions, &id);
                        }
                    } else {
                        // Still watched. Drop the stamp so the next real detach
                        // starts a fresh idle window.
                        idle_at.remove(&id);
                    }
                }
            }
        }
    }
    let ids: Vec<String> = sessions.keys().cloned().collect();
    for id in ids {
        close_session(&mut sessions, &id);
    }
}

pub(crate) fn target_resolution(
    cfg: &HubConfig,
    client_width: u32,
    client_height: u32,
) -> (u32, u32) {
    if cfg.default_width > 0 && cfg.default_height > 0 {
        (cfg.default_width, cfg.default_height)
    } else if client_width > 0 && client_height > 0 {
        (client_width, client_height)
    } else {
        (1920, 1080)
    }
}

async fn handle_cmd(
    cfg: &mut HubConfig,
    sessions: &mut HashMap<String, Session>,
    idle_at: &mut HashMap<String, tokio::time::Instant>,
    cmd: DisplayCommand,
) {
    match cmd {
        DisplayCommand::SetDefaults {
            width,
            height,
            refresh_hz,
        } => {
            cfg.default_width = width;
            cfg.default_height = height;
            cfg.refresh_hz = refresh_hz;
            cfg.has_explicit_override = true;
        }
        DisplayCommand::Acquire {
            name,
            key,
            width,
            height,
            bitrate_kbps,
            reply,
        } => {
            let (target_w, target_h) = target_resolution(cfg, width, height);
            if let Some(ref k) = key {
                if let Some((existing_id, existing_session)) = sessions
                    .iter_mut()
                    .find(|(_, s)| s.client_key.as_ref() == Some(k))
                {
                    if existing_session.info.width == target_w
                        && existing_session.info.height == target_h
                    {
                        if existing_session.viewers == 0 {
                            idle_at.insert(existing_id.clone(), tokio::time::Instant::now());
                        } else {
                            idle_at.remove(existing_id);
                        }
                        let info = existing_session.info.clone();
                        let _ = reply.send(Ok(info));
                        return;
                    }
                    let id_to_remove = existing_id.clone();
                    if let Some(old) = sessions.remove(&id_to_remove) {
                        close_session_inner(old);
                    }
                }
            }
            let result = open_session(cfg, name, key, target_w, target_h, bitrate_kbps).await;
            if let Ok(session) = result {
                let info = session.info.clone();
                idle_at.insert(info.id.clone(), tokio::time::Instant::now());
                sessions.insert(info.id.clone(), session);
                let _ = reply.send(Ok(info));
            } else if let Err(e) = result {
                let _ = reply.send(Err(e));
            }
        }
        DisplayCommand::Release { id } => {
            idle_at.remove(&id);
            close_session(sessions, &id);
        }
        DisplayCommand::Attach { id, key, reply } => {
            let chosen = resolve_attach_session_id(sessions, id.as_deref(), key.as_deref());
            let sid = match chosen {
                Some(sid) => sid,
                None if (id.is_none() || id.as_deref() == Some("")) && sessions.is_empty() => {
                    let w = if cfg.default_width > 0 {
                        cfg.default_width
                    } else {
                        1920
                    };
                    let h = if cfg.default_height > 0 {
                        cfg.default_height
                    } else {
                        1080
                    };
                    match open_session(cfg, "default".to_string(), None, w, h, None).await {
                        Ok(session) => {
                            let sid = session.info.id.clone();
                            idle_at.insert(sid.clone(), tokio::time::Instant::now());
                            sessions.insert(sid.clone(), session);
                            sid
                        }
                        Err(e) => {
                            let _ = reply.send(Err(format!("no display session: {e}")));
                            return;
                        }
                    }
                }
                None => {
                    let _ = reply.send(Err("no display session".into()));
                    return;
                }
            };
            if let Some(session) = sessions.get_mut(&sid) {
                session.viewers = session.viewers.saturating_add(1);
                session.ever_attached = true;
                idle_at.remove(&sid);
                let attached = orbiscreen_transport::AttachedDisplay {
                    info: session.info.clone(),
                    video: session.video_tx.subscribe(),
                };
                let _ = reply.send(Ok(attached));
            }
        }
        DisplayCommand::Detach { id } => {
            if let Some(session) = sessions.get_mut(&id) {
                session.viewers = session.viewers.saturating_sub(1);
                if session.viewers == 0 {
                    idle_at.insert(id, tokio::time::Instant::now());
                }
            }
        }
        DisplayCommand::Idr { id } => {
            let chosen = resolve_id(
                sessions,
                if id.is_empty() {
                    None
                } else {
                    Some(id.as_str())
                },
            );
            if let Some(sid) = chosen {
                if let Some(session) = sessions.get(&sid) {
                    let _ = session.idr_tx.try_send(());
                }
            } else if id.is_empty() {
                for session in sessions.values() {
                    let _ = session.idr_tx.try_send(());
                }
            } else if let Some(session) = sessions
                .values()
                .find(|s| s.client_key.as_deref() == Some(&id))
            {
                let _ = session.idr_tx.try_send(());
            }
        }
        DisplayCommand::Resize {
            id,
            width,
            height,
            reply,
        } => {
            let Some(old) = sessions.get(&id) else {
                let _ = reply.send(Err("unknown session".into()));
                return;
            };
            if old.info.width == width && old.info.height == height {
                let _ = reply.send(Ok(old.info.clone()));
                return;
            }
            let request = ResizeCarry {
                name: old.client_name.clone(),
                key: old.client_key.clone(),
                bitrate_kbps: old.bitrate_kbps,
                viewers: old.viewers,
                ever_attached: old.ever_attached,
            };
            match open_session(
                cfg,
                request.name.clone(),
                request.key.clone(),
                width,
                height,
                request.bitrate_kbps,
            )
            .await
            {
                Ok(session) => {
                    let Some(old) = sessions.remove(&id) else {
                        let _ = reply.send(Err("unknown session".into()));
                        return;
                    };
                    close_session_inner(old);
                    let info = session.info.clone();
                    let sid = info.id.clone();
                    let carry = adopt_resize_state(session, request, idle_at);
                    sessions.insert(sid, carry);
                    let _ = reply.send(Ok(info));
                }
                Err(e) => {
                    let _ = reply.send(Err(e));
                }
            }
        }
        DisplayCommand::Lookup { id, reply } => {
            let chosen = resolve_id(sessions, id.as_deref());
            let info = chosen.and_then(|sid| sessions.get(&sid).map(|s| s.info.clone()));
            let _ = reply.send(info);
        }
        DisplayCommand::Input { id, event } => {
            let chosen = resolve_id(sessions, id.as_deref());
            if let Some(sid) = chosen {
                if let Some(session) = sessions.get(&sid) {
                    let _ = session.input_tx.try_send(event);
                }
            } else {
                warn!(
                    session = ?id,
                    open = sessions.len(),
                    "dropping input; no unambiguous display session"
                );
            }
        }
        DisplayCommand::Shutdown { reply } => {
            let _ = reply.send(());
        }
    }
}

fn resolve_attach_session_id(
    sessions: &HashMap<String, Session>,
    id: Option<&str>,
    key: Option<&str>,
) -> Option<String> {
    if let Some(req_id) = id.filter(|s| !s.is_empty()) {
        if sessions.contains_key(req_id) {
            return Some(req_id.to_string());
        }
        if let Some(s) = sessions
            .values()
            .find(|s| s.client_key.as_deref() == Some(req_id))
        {
            return Some(s.info.id.clone());
        }
        return None;
    }
    if let Some(req_key) = key.filter(|s| !s.is_empty()) {
        if let Some(s) = sessions
            .values()
            .find(|s| s.client_key.as_deref() == Some(req_key))
        {
            return Some(s.info.id.clone());
        }
    }
    if sessions.len() == 1 {
        return sessions.keys().next().cloned();
    }
    if sessions.is_empty() {
        return None;
    }
    let unattached: Vec<&Session> = sessions.values().filter(|s| s.viewers == 0).collect();
    if !unattached.is_empty() {
        return unattached
            .into_iter()
            .max_by_key(|s| u64::from_str_radix(&s.info.id, 16).unwrap_or(0))
            .map(|s| s.info.id.clone());
    }
    sessions
        .values()
        .max_by_key(|s| u64::from_str_radix(&s.info.id, 16).unwrap_or(0))
        .map(|s| s.info.id.clone())
}

fn resolve_id(sessions: &HashMap<String, Session>, id: Option<&str>) -> Option<String> {
    if let Some(req) = id.filter(|s| !s.is_empty()) {
        if sessions.contains_key(req) {
            return Some(req.to_string());
        }
        if let Some(s) = sessions
            .values()
            .find(|s| s.client_key.as_deref() == Some(req))
        {
            return Some(s.info.id.clone());
        }
        return None;
    }
    resolve_session_id(sessions.keys().map(String::as_str), id)
}

struct ResizeCarry {
    name: String,
    key: Option<String>,
    bitrate_kbps: Option<u32>,
    viewers: usize,
    ever_attached: bool,
}

fn adopt_resize_state(
    mut session: Session,
    carry: ResizeCarry,
    idle_at: &mut HashMap<String, tokio::time::Instant>,
) -> Session {
    session.viewers = carry.viewers;
    session.ever_attached = carry.ever_attached;
    let deadline = if session.ever_attached {
        tokio::time::Instant::now() + IDLE_AFTER_LAST_VIEWER
    } else {
        tokio::time::Instant::now() + WAITING_FOR_FIRST_VIEWER
    };
    idle_at.insert(session.info.id.clone(), deadline);
    session
}

fn resolve_session_id<'a>(
    keys: impl IntoIterator<Item = &'a str>,
    id: Option<&str>,
) -> Option<String> {
    let keys: Vec<&str> = keys.into_iter().collect();
    if let Some(id) = id.filter(|s| !s.is_empty()) {
        return keys.contains(&id).then(|| id.to_string());
    }
    if keys.len() == 1 {
        return Some(keys[0].to_string());
    }
    None
}

fn next_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{n:x}")
}

fn encoder_label(kind: EncoderKind) -> &'static str {
    match kind {
        EncoderKind::Auto => "auto",
        EncoderKind::Vaapi => "vaapi",
        EncoderKind::Nvenc => "nvenc",
        EncoderKind::X264 => "x264",
    }
}

async fn open_session(
    cfg: &HubConfig,
    name: String,
    key: Option<String>,
    width: u32,
    height: u32,
    bitrate_kbps: Option<u32>,
) -> Result<Session, String> {
    let width = width.clamp(320, 7680);
    let height = height.clamp(240, 4320);
    let (description, names) = protocol_names_for(&name, key.as_deref());
    let spec = KwinVirtualSpec {
        width,
        height,
        names,
        description,
    };
    let capture = tokio::task::spawn_blocking(move || KwinVirtualCapture::open(spec))
        .await
        .map_err(|e| format!("kwin open task: {e}"))?
        .map_err(|e| e.to_string())?;
    let capture = Arc::new(capture);
    let connector = capture.connector_name().to_string();
    let (actual_w, actual_h) = capture.dimensions();
    info!(connector = %connector, width = actual_w, height = actual_h, "client virtual output created");

    {
        let name = connector.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            let enable_spec = format!("output.{name}.enable");
            let scale_spec = format!("output.{name}.scale.1");
            let next_x = orbiscreen_capture::kwin_virtual::next_available_output_x(&name);
            let pos_spec = format!("output.{name}.position.{next_x},0");
            for _ in 0..5 {
                let status = tokio::process::Command::new("kscreen-doctor")
                    .arg(&enable_spec)
                    .arg(&scale_spec)
                    .arg(&pos_spec)
                    .status()
                    .await;
                if let Ok(s) = status {
                    if s.success() {
                        info!("Enabled and scaled KWin output {name} at position {next_x},0");
                        break;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }
        });
    }

    let target_bitrate = bitrate_kbps.unwrap_or_else(|| {
        if cfg.bitrate_kbps > 0 && cfg.bitrate_kbps != 8000 {
            cfg.bitrate_kbps
        } else {
            orbiscreen_encode::suggested_bitrate_kbps(actual_w, actual_h, cfg.refresh_hz)
        }
    });

    let mut encoder = Encoder::new(EncodeParams {
        kind: cfg.encode_kind,
        bitrate_kbps: target_bitrate,
        width: actual_w,
        height: actual_h,
        framerate: cfg.refresh_hz,
    })
    .map_err(|e| e.to_string())?;
    let encoder_name = encoder_label(encoder.kind());
    let mut encoded_rx = encoder
        .subscribe()
        .ok_or_else(|| "encoder returned no rx".to_string())?;
    let encoder = Arc::new(encoder);
    let (idr_tx, mut idr_rx) = mpsc::channel::<()>(8);
    let encoder_for_idr = Arc::clone(&encoder);
    tokio::spawn(async move {
        while idr_rx.recv().await.is_some() {
            encoder_for_idr.request_keyframe();
        }
    });

    let (video_tx, _) = broadcast::channel::<H264Packet>(64);
    let video_out = video_tx.clone();
    tokio::spawn(async move {
        let mut ts_base: Option<u64> = None;
        while let Some(chunk) = encoded_rx.recv().await {
            let base = *ts_base.get_or_insert(chunk.pts_ns);
            let pkt = H264Packet {
                bytes: chunk.bytes,
                is_keyframe: chunk.is_keyframe,
                pts_ns: chunk.pts_ns.saturating_sub(base),
            };
            let _ = video_out.send(pkt);
        }
    });

    let (input_tx, input_rx) = mpsc::channel::<IncomingInput>(1024);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    spawn_capture_pump(
        Arc::clone(&capture),
        Arc::clone(&encoder),
        cfg.refresh_hz,
        shutdown_rx,
    );
    spawn_input_pump(
        input_rx,
        actual_w,
        actual_h,
        connector.clone(),
        name.clone(),
        key.clone(),
    );

    let id = next_id();
    Ok(Session {
        info: DisplayInfo {
            id,
            name: name.clone(),
            connector,
            width: actual_w,
            height: actual_h,
            encoder: encoder_name.to_string(),
        },
        client_name: name,
        client_key: key,
        bitrate_kbps,
        video_tx,
        idr_tx,
        input_tx,
        viewers: 0,
        ever_attached: false,
        shutdown: shutdown_tx,
        capture: Some(capture),
        encoder: Some(encoder),
    })
}

fn spawn_capture_pump(
    capture: Arc<KwinVirtualCapture>,
    encoder: Arc<Encoder>,
    refresh_hz: u32,
    mut shutdown: watch::Receiver<bool>,
) {
    tokio::spawn(async move {
        const KEEPALIVE: Duration = Duration::from_millis(100);
        let frame_interval_ns = 1_000_000_000u64 / refresh_hz.max(1) as u64;
        let started = std::time::Instant::now();
        let mut last_pts_ns: u64 = 0;
        let mut keepalive_frame: Option<(u32, u32, Vec<u8>)> = None;
        let mut next_frame_target_ns = started.elapsed().as_nanos() as u64;
        loop {
            if *shutdown.borrow() {
                break;
            }
            let outcome = tokio::select! {
                _ = shutdown.changed() => break,
                result = tokio::time::timeout(KEEPALIVE, capture.next_frame()) => result,
            };
            match outcome {
                Ok(Ok(frame)) => {
                    keepalive_frame = Some((frame.width, frame.height, frame.data.to_vec()));
                    let now_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
                    let pts_ns = now_ns.max(last_pts_ns.saturating_add(1_000));
                    last_pts_ns = pts_ns;
                    if let Err(e) =
                        encoder.push_frame_owned(frame.data, frame.width, frame.height, pts_ns)
                    {
                        match e {
                            orbiscreen_encode::EncodeError::Flushing
                            | orbiscreen_encode::EncodeError::Eos => break,
                            _ => warn!("frame push rejected: {e}"),
                        }
                    }
                    next_frame_target_ns += frame_interval_ns;
                    if next_frame_target_ns > now_ns {
                        let sleep_ns = next_frame_target_ns - now_ns;
                        if sleep_ns > 1_000_000 {
                            tokio::time::sleep(std::time::Duration::from_nanos(sleep_ns)).await;
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!("client display capture ended: {e}");
                    break;
                }
                Err(_elapsed) => {
                    let Some((width, height, data)) = &keepalive_frame else {
                        continue;
                    };
                    let now_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
                    let pts_ns = now_ns.max(last_pts_ns.saturating_add(1_000));
                    last_pts_ns = pts_ns;
                    if let Err(
                        orbiscreen_encode::EncodeError::Flushing
                        | orbiscreen_encode::EncodeError::Eos,
                    ) = encoder.push_frame(data, *width, *height, pts_ns)
                    {
                        break;
                    }
                    next_frame_target_ns += frame_interval_ns;
                    if next_frame_target_ns > now_ns {
                        let sleep_ns = next_frame_target_ns - now_ns;
                        if sleep_ns > 1_000_000 {
                            tokio::time::sleep(std::time::Duration::from_nanos(sleep_ns)).await;
                        }
                    }
                }
            }
        }
    });
}

fn spawn_input_pump(
    mut input_rx: mpsc::Receiver<IncomingInput>,
    width: u32,
    height: u32,
    connector: String,
    client_name: String,
    client_key: Option<String>,
) {
    tokio::spawn(async move {
        let label = client_key
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(client_name.as_str());
        let prefix = format!(
            "OrbiScreen-{}",
            orbiscreen_capture::kwin_virtual::sanitize_client_name(label)
        );
        let spec = VirtualTouchscreenSpec {
            width,
            height,
            output_name: Some(connector.clone()),
            device_label: Some(prefix.clone()),
        };
        let mut injector = match InputInjector::open_async(spec).await {
            Ok(inj) => {
                info!(connector = %connector, "input injector open for client display");
                bind_inputs(&connector, &prefix).await;
                Some(inj)
            }
            Err(e) => {
                warn!("input injector unavailable for {connector}: {e}");
                None
            }
        };
        while let Some(event) = input_rx.recv().await {
            let Some(inj) = injector.as_mut() else {
                continue;
            };
            match event {
                IncomingInput::Pointer(p) => {
                    let _ = inj.inject_pointer(p).await;
                }
                IncomingInput::Key(k) => {
                    let _ = inj.inject_key(k).await;
                }
                IncomingInput::Stylus(s) => {
                    let _ = inj.inject_stylus(s).await;
                }
                IncomingInput::Touch(t) => {
                    let _ = inj.inject_touch(t).await;
                }
                IncomingInput::RawPointer { x, y } => {
                    let _ = inj.inject_pointer(PointerEvent::Move { x, y }).await;
                }
                IncomingInput::Resize { width, height } => {
                    inj.resize(width, height);
                }
            }
        }
    });
}

fn close_session(sessions: &mut HashMap<String, Session>, id: &str) {
    if let Some(session) = sessions.remove(id) {
        close_session_inner(session);
    }
}

fn close_session_inner(mut session: Session) {
    let _ = session.shutdown.send(true);
    if let Some(encoder) = session.encoder.take() {
        encoder.stop();
    }
    if let Some(capture) = session.capture.take() {
        info!(
            connector = %session.info.connector,
            "closed client virtual output"
        );
        drop(capture);
    }
}

pub(crate) fn event_paths_from_introspect(xml: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut rest = xml;
    while let Some(idx) = rest.find(r#"name="event"#) {
        let start = idx + r#"name=""#.len();
        let after = &rest[start..];
        let Some(end) = after.find('"') else {
            break;
        };
        let name = &after[..end];
        if name.starts_with("event") && name[5..].bytes().all(|b| b.is_ascii_digit()) {
            paths.push(format!("/org/kde/KWin/InputDevice/{name}"));
        }
        rest = &after[end..];
    }
    paths
}

pub(crate) async fn list_kwin_input_device_paths(conn: &zbus::Connection) -> Vec<String> {
    if let Ok(proxy) = zbus::Proxy::new(
        conn,
        "org.kde.KWin",
        "/org/kde/KWin/InputDevice",
        "org.freedesktop.DBus.Introspectable",
    )
    .await
    {
        if let Ok(xml) = proxy.call::<_, _, String>("Introspect", &()).await {
            let paths = event_paths_from_introspect(&xml);
            if !paths.is_empty() {
                return paths;
            }
        }
    }
    (0..=512)
        .map(|idx| format!("/org/kde/KWin/InputDevice/event{idx}"))
        .collect()
}

async fn bind_inputs(target_output: &str, device_prefix: &str) {
    let mut last_bound = 0;
    let mut last_resolved = target_output.to_string();
    for delay_ms in [250, 500, 1000, 2000] {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        let Some(resolved) =
            orbiscreen_capture::kwin_virtual::preferred_tablet_output(Some(target_output))
        else {
            continue;
        };
        last_resolved = resolved.clone();
        if let Ok(conn) = zbus::Connection::session().await {
            last_bound =
                bind_named_kwin_devices(&conn, &resolved, |name| name.starts_with(device_prefix))
                    .await;
            if last_bound >= 3 {
                return;
            }
        }
    }
    if last_bound < 3 {
        warn!(
            bound = last_bound,
            prefix = device_prefix,
            output = %last_resolved,
            "KWin input bind found fewer than 3 OrbiScreen devices"
        );
    }
}

pub(crate) async fn bind_named_kwin_devices<F>(
    conn: &zbus::Connection,
    resolved: &str,
    mut name_matches: F,
) -> usize
where
    F: FnMut(&str) -> bool,
{
    let uuid = orbiscreen_capture::kwin_virtual::output_uuid(resolved);
    let mut bound = 0;
    for path in list_kwin_input_device_paths(conn).await {
        let Ok(proxy) = zbus::Proxy::new(
            conn,
            "org.kde.KWin",
            path.as_str(),
            "org.kde.KWin.InputDevice",
        )
        .await
        else {
            continue;
        };
        let Ok(name) = proxy.get_property::<String>("name").await else {
            continue;
        };
        if !name_matches(&name) {
            continue;
        }
        let is_pointer = name.ends_with("Mouse") || name.contains("Mouse and Keyboard");
        if is_pointer {
            let _ = proxy.set_property::<bool>("mapToWorkspace", true).await;
        } else {
            if let Err(e) = proxy.set_property::<&str>("outputName", resolved).await {
                warn!("could not set outputName={resolved} on {path} ({name}): {e}");
                continue;
            }
            if let Some(uuid) = uuid.as_deref() {
                let _ = proxy.set_property::<&str>("outputUuid", uuid).await;
            }
            let _ = proxy.set_property::<bool>("mapToWorkspace", false).await;
        }
        info!("bound KWin input device {path} ({name}) to output {resolved}");
        bound += 1;
    }
    bound
}

#[cfg(test)]
mod tests {
    use super::{
        adopt_resize_state, event_paths_from_introspect, should_reap_idle_session, ResizeCarry,
    };
    use crate::client_display::Session;
    use orbiscreen_transport::IncomingInput;
    use std::collections::HashMap;

    fn carry(name: &str, viewers: usize, ever_attached: bool) -> ResizeCarry {
        ResizeCarry {
            name: name.to_string(),
            key: None,
            bitrate_kbps: None,
            viewers,
            ever_attached,
        }
    }

    fn session_with(id: &str, ever_attached: bool) -> Session {
        use tokio::sync::{broadcast, watch};
        let (video_tx, _) = broadcast::channel(1);
        let (input_tx, _) = tokio::sync::mpsc::channel::<IncomingInput>(1);
        let (idr_tx, _) = tokio::sync::mpsc::channel::<()>(1);
        let (shutdown, _) = watch::channel(false);
        Session {
            info: orbiscreen_transport::DisplayInfo {
                id: id.to_string(),
                name: "test".into(),
                connector: "Virtual-TEST".into(),
                width: 1280,
                height: 720,
                encoder: "auto".into(),
            },
            client_name: "test".into(),
            client_key: None,
            bitrate_kbps: None,
            video_tx,
            idr_tx,
            input_tx,
            viewers: 0,
            ever_attached,
            shutdown,
            capture: None,
            encoder: None,
        }
    }

    #[test]
    fn resize_swap_preserves_viewers_and_deadline() {
        let mut idle_at = HashMap::new();
        let fresh = adopt_resize_state(
            session_with("a", false),
            carry("test", 0, false),
            &mut idle_at,
        );
        assert_eq!(fresh.viewers, 0);
        assert!(!fresh.ever_attached);
        let waiting = idle_at.get("a").copied().unwrap();
        let mut idle_at2 = HashMap::new();
        let viewed = adopt_resize_state(
            session_with("b", true),
            carry("test", 2, true),
            &mut idle_at2,
        );
        assert_eq!(viewed.viewers, 2);
        assert!(viewed.ever_attached);
        let after = idle_at2.get("b").copied().unwrap();
        assert!(after > waiting);
    }

    #[test]
    fn subscribed_video_is_not_reaped_at_the_idle_timeout() {
        use std::time::Duration;
        assert!(!should_reap_idle_session(
            0,
            1,
            true,
            Duration::from_secs(120),
        ));
        assert!(!should_reap_idle_session(
            1,
            0,
            true,
            Duration::from_secs(120),
        ));
        assert!(should_reap_idle_session(
            0,
            0,
            true,
            Duration::from_secs(120),
        ));
        assert!(!should_reap_idle_session(
            0,
            0,
            true,
            Duration::from_secs(119),
        ));
        assert!(should_reap_idle_session(
            0,
            0,
            false,
            Duration::from_secs(30),
        ));
        assert!(!should_reap_idle_session(
            0,
            0,
            false,
            Duration::from_secs(29),
        ));
    }

    #[test]
    fn resize_swap_initializes_idle_deadline_for_never_attached() {
        let mut idle_at = HashMap::new();
        adopt_resize_state(
            session_with("c", false),
            carry("test", 0, false),
            &mut idle_at,
        );
        assert!(idle_at.contains_key("c"));
    }

    #[test]
    fn resolve_id_does_not_steal_the_only_session_when_id_is_unknown() {
        use super::resolve_session_id;
        let one = ["web"];
        assert_eq!(resolve_session_id(one, None).as_deref(), Some("web"));
        assert_eq!(resolve_session_id(one, Some("")).as_deref(), Some("web"));
        assert_eq!(resolve_session_id(one, Some("web")).as_deref(), Some("web"));
        assert_eq!(resolve_session_id(one, Some("stale")), None);
        let two = ["web", "android"];
        assert_eq!(resolve_session_id(two, None), None);
        assert_eq!(
            resolve_session_id(two, Some("android")).as_deref(),
            Some("android")
        );
    }

    #[test]
    fn resolve_attach_session_id_prefers_key_unattached_and_newest() {
        use super::resolve_attach_session_id;
        let mut sessions = HashMap::new();
        let mut s1 = session_with("1", true);
        s1.client_key = Some("device-a".into());
        s1.viewers = 1;
        sessions.insert("1".into(), s1);

        let mut s2 = session_with("2", false);
        s2.client_key = Some("device-b".into());
        s2.viewers = 0;
        sessions.insert("2".into(), s2);

        assert_eq!(
            resolve_attach_session_id(&sessions, Some("1"), None).as_deref(),
            Some("1")
        );
        assert_eq!(
            resolve_attach_session_id(&sessions, None, Some("device-a")).as_deref(),
            Some("1")
        );
        assert_eq!(
            resolve_attach_session_id(&sessions, None, Some("device-b")).as_deref(),
            Some("2")
        );
        assert_eq!(
            resolve_attach_session_id(&sessions, None, None).as_deref(),
            Some("2")
        );

        if let Some(s) = sessions.get_mut("2") {
            s.viewers = 1;
        }
        assert_eq!(
            resolve_attach_session_id(&sessions, None, None).as_deref(),
            Some("2")
        );
    }

    #[test]
    fn introspect_xml_includes_high_event_nodes() {
        let xml = r#"<!DOCTYPE node PUBLIC "-
"http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node>
  <interface name="org.freedesktop.DBus.Introspectable"/>
  <node name="event0"/>
  <node name="event31"/>
  <node name="event256"/>
  <node name="event260"/>
  <node name="not-an-event"/>
</node>"#;
        let paths = event_paths_from_introspect(xml);
        assert_eq!(
            paths,
            vec![
                "/org/kde/KWin/InputDevice/event0",
                "/org/kde/KWin/InputDevice/event31",
                "/org/kde/KWin/InputDevice/event256",
                "/org/kde/KWin/InputDevice/event260",
            ]
        );
    }

    #[test]
    fn target_resolution_prioritizes_configured_resolution() {
        let cfg = super::HubConfig {
            encode_kind: orbiscreen_encode::EncoderKind::Auto,
            bitrate_kbps: 8000,
            refresh_hz: 90,
            default_width: 1920,
            default_height: 1152,
            has_explicit_override: false,
        };
        assert_eq!(super::target_resolution(&cfg, 2560, 1536), (1920, 1152));
    }

    #[test]
    fn target_resolution_adopts_client_native_when_auto() {
        let cfg = super::HubConfig {
            encode_kind: orbiscreen_encode::EncoderKind::Auto,
            bitrate_kbps: 8000,
            refresh_hz: 60,
            default_width: 0,
            default_height: 0,
            has_explicit_override: false,
        };
        assert_eq!(super::target_resolution(&cfg, 2560, 1536), (2560, 1536));
        assert_eq!(super::target_resolution(&cfg, 0, 0), (1920, 1080));
    }
}
