// Orbiscreen - orbiscreen-daemon daemon binary (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen
pub mod dbus;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use orbiscreen_capture::{CapturePreference, CaptureSession};
use orbiscreen_core::{dump_config, load_config, Config};
use orbiscreen_display::{DisplayStatus, EvdiFramePump, VirtualDisplaySpec};
use orbiscreen_encode::{EncodeParams, Encoder, EncoderKind};
use orbiscreen_input::{InputInjector, VirtualTouchscreenSpec};
use orbiscreen_transport::{H264Packet, ServerConfig, Stats, Transport};
use tokio::sync::mpsc;
use tracing::{error, info, warn, Level};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "orbiscreen",
    version,
    about = "Virtual secondary displays for Linux, streamed to Android",
    long_about = "Orbiscreen creates a real virtual display via evdi and streams it to \
                  Android devices as MPEG-TS/H.264 over Wi-Fi or USB."
)]
struct Cli {
    #[arg(short, long, global = true, default_value = "orbiscreen.toml")]
    config: String,

    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Start {
        #[arg(long)]
        no_mdns: bool,
    },
    Stop,
    Uninstall,
    ListDisplays,
    Probe,
    PrintConfig,
}

fn init_tracing(verbose: u8) {
    let level = match verbose {
        0 => Level::INFO,
        1 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let filter_str = format!("{},zbus=error,ashpd=error", level.as_str());
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter_str));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

fn load_or_default_config(path: &str) -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    if std::path::Path::new(path).exists() {
        let s = std::fs::read_to_string(path)?;
        Ok(load_config(&s)?)
    } else {
        Ok(Config::default())
    }
}

fn probe() {
    println!(
        "capture backend: {:?}",
        orbiscreen_capture::detect_backend()
    );
    println!("input backend:   {:?}", orbiscreen_input::detect_backend());
    match orbiscreen_display::probe() {
        DisplayStatus::Compatible => {
            println!("display backend: Compatible (kernel + libevdi OK)");
        }
        DisplayStatus::Outdated => {
            println!("display backend: Outdated (kernel evdi older than libevdi requires)");
        }
        DisplayStatus::KernelModuleMissing => {
            println!("display backend: kernel module missing");
        }
        DisplayStatus::NoDeviceNode => {
            println!(
                "display backend: kernel OK, but no evdi device node yet (run \
                 `orbiscreen start` as root to add one)",
            );
        }
    }
}

fn list_displays(path: &str) {
    match load_or_default_config(path) {
        Ok(cfg) => {
            println!(
                "configured virtual display: {}x{} @ {} Hz (count = {})",
                cfg.display.width,
                cfg.display.height,
                cfg.display.refresh_rate_hz,
                cfg.display.count,
            );
        }
        Err(e) => eprintln!("config error: {e}"),
    }
    println!("display backend: {:?}", orbiscreen_display::probe());
}

#[derive(Clone)]
struct Frame {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

enum FrameSource {
    Evdi(EvdiFramePump),
    Capture(CaptureSession),
}

enum SourceOutcome {
    Frame(Frame),
    Retryable(String),
    Ended,
}

impl FrameSource {
    async fn next_frame(&mut self) -> SourceOutcome {
        match self {
            FrameSource::Evdi(pump) => match pump.next_frame().await {
                Some(frame) => SourceOutcome::Frame(Frame {
                    width: frame.width,
                    height: frame.height,
                    data: frame.data,
                }),
                None => SourceOutcome::Ended,
            },
            FrameSource::Capture(capture) => match capture.next_frame().await {
                Ok(frame) => SourceOutcome::Frame(Frame {
                    width: frame.width,
                    height: frame.height,
                    data: frame.data,
                }),
                Err(e) => SourceOutcome::Retryable(e.to_string()),
            },
        }
    }

    fn backend_name(&self) -> &'static str {
        match self {
            FrameSource::Evdi(_) => "evdi",
            FrameSource::Capture(c) => match c.backend() {
                orbiscreen_capture::CaptureBackend::X11 => "x11",
                orbiscreen_capture::CaptureBackend::Wayland => "wayland-portal",
                orbiscreen_capture::CaptureBackend::KwinVirtual => "kwin-virtual",
            },
        }
    }

    /// True when a retryable-looking capture error is actually terminal (the
    /// source will not produce more frames); callers should stop or reopen.
    fn is_ended(&self) -> bool {
        match self {
            FrameSource::Evdi(_) => false,
            FrameSource::Capture(capture) => capture.is_ended(),
        }
    }

    fn actual_dimensions(&self) -> (u32, u32) {
        match self {
            FrameSource::Evdi(pump) => pump.actual_dimensions(),
            FrameSource::Capture(capture) => (capture.width(), capture.height()),
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let cfg = match load_or_default_config(&cli.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("config error: {e}");
            return ExitCode::from(2);
        }
    };

    match cli.command {
        Command::Start { no_mdns } => match run_start(cfg, no_mdns).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                error!("orbiscreen start failed: {e}");
                ExitCode::from(1)
            }
        },
        Command::Stop => match dbus::request_stop().await {
            Ok(reply) => {
                println!("daemon: {reply}");
                ExitCode::SUCCESS
            }
            Err(zbus::Error::MethodError(name, _, _))
                if name.to_string().contains("ServiceUnknown") =>
            {
                println!("daemon is not running (no com.orbiscreen.Daemon on the session bus)");
                ExitCode::from(1)
            }
            Err(e) => {
                eprintln!("stop failed: {e}");
                eprintln!("hint: use 'systemctl --user stop orbiscreen' if it runs as a service");
                ExitCode::from(1)
            }
        },
        Command::Uninstall => {
            run_uninstall();
            ExitCode::SUCCESS
        }
        Command::ListDisplays => {
            list_displays(&cli.config);
            ExitCode::SUCCESS
        }
        Command::Probe => {
            probe();
            ExitCode::SUCCESS
        }
        Command::PrintConfig => match dump_config(&cfg) {
            Ok(s) => {
                println!("{s}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("config serialize error: {e}");
                ExitCode::from(1)
            }
        },
    }
}

fn run_uninstall() {
    println!("[Orbiscreen] Uninstalling...");

    if let Err(e) = std::process::Command::new("systemctl")
        .args(["--user", "stop", "orbiscreen"])
        .status()
    {
        warn!("Failed to stop service: {e}");
    }
    if let Err(e) = std::process::Command::new("systemctl")
        .args(["--user", "disable", "orbiscreen"])
        .status()
    {
        warn!("Failed to disable service: {e}");
    }
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    if let Ok(home) = std::env::var("HOME") {
        let home = PathBuf::from(home);
        let _ = std::fs::remove_file(home.join(".local/bin/orbiscreen"));
        let _ = std::fs::remove_file(home.join(".config/systemd/user/orbiscreen.service"));
        let _ = std::fs::remove_file(
            home.join(".local/share/applications/com.orbiscreen.OrbiscreenGtk.desktop"),
        );
        let _ = std::fs::remove_file(
            home.join(".local/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg"),
        );
        let _ = std::fs::remove_dir_all(home.join(".local/share/orbiscreen"));
    }

    let _ = std::fs::remove_file("/usr/bin/orbiscreen");
    let _ = std::fs::remove_file("/usr/share/applications/com.orbiscreen.OrbiscreenGtk.desktop");
    let _ = std::fs::remove_file(
        "/usr/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg",
    );
    let _ = std::fs::remove_dir_all("/usr/share/orbiscreen");

    println!("[Orbiscreen] Uninstallation complete.");
}

async fn run_start(
    cfg: Config,
    no_mdns: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let spec = VirtualDisplaySpec {
        width: cfg.display.width,
        height: cfg.display.height,
        refresh_rate_hz: cfg.display.refresh_rate_hz,
    };

    let is_running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

    info!(
        "Orbiscreen starting - display {w}x{h}@{hz}Hz, encoder preferred = {enc}",
        w = spec.width,
        h = spec.height,
        hz = spec.refresh_rate_hz,
        enc = cfg.encode.preferred_encoder,
    );

    // Source selection. EVDI is opt-in: it needs a root-installed kernel
    // module, so it is only attempted when explicitly requested — or on X11,
    // where it is the only real-second-monitor path and a loaded module
    // already signals deliberate setup. On Wayland, `auto` means the KWin
    // virtual display first, then the portal share dialog.
    let preferred = cfg.capture.preferred.as_str();
    let on_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let mut source = match preferred {
        "evdi" => match EvdiFramePump::spawn(spec) {
            Ok(pump) => {
                info!(
                    connector = ?pump.info().connector,
                    device_index = pump.info().device_index,
                    "Virtual display is open (EVDI DRM active); streaming the evdi framebuffer",
                );
                FrameSource::Evdi(pump)
            }
            Err(e) => {
                error!(
                    "capture preference 'evdi' failed: {e}. Install/load the evdi kernel module \
                     or set [capture] preferred = \"auto\" / \"kwin-virtual\" / \"portal\"."
                );
                return Err(e.into());
            }
        },
        "auto" if !on_wayland => match EvdiFramePump::spawn(spec) {
            Ok(pump) => {
                info!(
                    connector = ?pump.info().connector,
                    device_index = pump.info().device_index,
                    "Virtual display is open (EVDI DRM active); streaming the evdi framebuffer",
                );
                FrameSource::Evdi(pump)
            }
            Err(e) => {
                info!("EVDI not in use ({e}); capturing the X11 root screen");
                let capture = CaptureSession::open_with_preference(
                    spec.width,
                    spec.height,
                    CapturePreference::Auto,
                )
                .await?;
                info!(backend = ?capture.backend(), "Capture backend open");
                FrameSource::Capture(capture)
            }
        },
        _ => {
            let capture = CaptureSession::open_with_preference(
                spec.width,
                spec.height,
                CapturePreference::parse(preferred),
            )
            .await?;
            info!(backend = ?capture.backend(), "Capture backend open");
            FrameSource::Capture(capture)
        }
    };

    let actual_dims = source.actual_dimensions();
    info!(
        stream_width = actual_dims.0,
        stream_height = actual_dims.1,
        "stream dimensions established from source"
    );

    // Input injection is optional: never block streaming startup on the
    // RemoteDesktop portal — it can hang, need interactive approval, or be
    // wedged. Streaming starts regardless; remote control comes online only
    // if the injector opens within the timeout.
    const INPUT_OPEN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);
    let injector = match tokio::time::timeout(
        INPUT_OPEN_TIMEOUT,
        InputInjector::open_async(VirtualTouchscreenSpec {
            width: spec.width,
            height: spec.height,
        }),
    )
    .await
    {
        Ok(Ok(inj)) => {
            info!(backend = ?inj.backend(), "Input injector open");
            Some(inj)
        }
        Ok(Err(e)) => {
            warn!("input injection unavailable ({e}); streaming continues without remote control");
            None
        }
        Err(_) => {
            warn!(
                "input injection portal did not respond within {}s; streaming continues \
                 without remote control (approve the portal dialog or restart to retry)",
                INPUT_OPEN_TIMEOUT.as_secs()
            );
            None
        }
    };

    let encoder_kind = match EncoderKind::parse(&cfg.encode.preferred_encoder) {
        Some(kind) => kind,
        None => {
            warn!(
                requested = %cfg.encode.preferred_encoder,
                "Unknown encoder; falling back to software x264"
            );
            EncoderKind::X264
        }
    };
    let mut encoder = Encoder::new(EncodeParams {
        kind: encoder_kind,
        bitrate_kbps: cfg.encode.bitrate_kbps,
        width: actual_dims.0,
        height: actual_dims.1,
        framerate: spec.refresh_rate_hz,
    })?;
    let encoder_name = match encoder.kind() {
        EncoderKind::Vaapi => "vaapi",
        EncoderKind::Nvenc => "nvenc",
        EncoderKind::X264 => "x264",
    };
    let mut encoded_rx = encoder.subscribe().ok_or("encoder returned no rx")?;
    let encoder = Arc::new(encoder);

    let stats = std::sync::Arc::new(Stats::default());

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
    let _shutdown_keepalive = shutdown_tx.clone();
    let dbus_handles = std::sync::Arc::new(dbus::DaemonHandles {
        is_running: is_running.clone(),
        stats: stats.clone(),
        config: cfg.clone(),
        encoder: encoder_name,
        capture_backend: source.backend_name(),
        shutdown_tx,
    });
    tokio::spawn(async move {
        if let Err(e) = dbus::run_dbus_server(dbus_handles).await {
            warn!("D-Bus session service init failed (is D-Bus running?): {e}");
        }
    });
    info!("D-Bus session service registered: com.orbiscreen.Daemon");

    // Bounded so a stalled transport applies backpressure through the
    // encoder into the capture pump instead of growing without limit; the
    // transport-side broadcast then drops for lagging clients as designed.
    let (video_tx, video_rx) = mpsc::channel::<H264Packet>(64);
    let frame_pump = tokio::spawn(async move {
        let mut n = 0u64;
        let mut ts_base: Option<u64> = None;
        while let Some(chunk) = encoded_rx.recv().await {
            n += 1;
            let base = *ts_base.get_or_insert(chunk.pts_ns);
            let pts_ns = chunk.pts_ns.saturating_sub(base);
            if n <= 5 || n % 300 == 0 {
                info!(
                    "frame_pump: chunk #{n} ({} B, kf={}, pts={})",
                    chunk.bytes.len(),
                    chunk.is_keyframe,
                    pts_ns
                );
            }
            let pkt = H264Packet {
                bytes: chunk.bytes,
                is_keyframe: chunk.is_keyframe,
                pts_ns,
            };
            if let Ok(path) = std::env::var("ORBISCREEN_ENCODER_DUMP") {
                use std::io::Write;
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                {
                    let _ = f.write_all(&pkt.bytes);
                }
            }
            if video_tx.send(pkt).await.is_err() {
                break;
            }
        }
        info!("frame_pump exited");
    });

    let cap_dims = actual_dims;
    let frame_count = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let fc = frame_count.clone();
    let encoder_for_pump = Arc::clone(&encoder);
    let cap_pump = tokio::spawn(async move {
        let encoder = encoder_for_pump;
        let frame_dur = Encoder::frame_duration_ns(spec.refresh_rate_hz);
        // PTS follows the wall clock so live players (mpegts.js, ExoPlayer)
        // stay latency-synced: keepalives at 2 fps stamp 500 ms apart instead
        // of advancing one frame duration per push, which made stream time
        // run many times slower than real time during idle periods.
        // Compositors deliver virtual-display frames on damage only: an idle
        // desktop stops producing frames entirely, which would leave new
        // clients waiting forever without even a keyframe (black screen).
        // Re-push the last frame as a keepalive when the source goes quiet.
        const KEEPALIVE: std::time::Duration = std::time::Duration::from_millis(500);
        let started = std::time::Instant::now();
        let mut last_pts_ns: u64 = frame_dur;
        let mut last_frame: Option<Frame> = None;
        let mut last_snapshot = std::time::Instant::now() - KEEPALIVE;
        loop {
            let outcome = match tokio::time::timeout(KEEPALIVE, source.next_frame()).await {
                Ok(outcome) => outcome,
                Err(_elapsed) => {
                    let Some(frame) = &last_frame else {
                        continue;
                    };
                    let now_ns = u64::try_from(started.elapsed().as_nanos())
                        .unwrap_or(u64::MAX);
                    last_pts_ns = now_ns.max(last_pts_ns.saturating_add(frame_dur));
                    let pts_ns = last_pts_ns;
                    if let Err(e) =
                        encoder.push_frame(&frame.data, frame.width, frame.height, pts_ns)
                    {
                        warn!(
                            "keepalive frame push rejected ({}x{}, {} B): {e}",
                            frame.width,
                            frame.height,
                            frame.data.len()
                        );
                    }
                    continue;
                }
            };
            match outcome {
                SourceOutcome::Frame(frame) => {
                    let n = fc.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    // Snapshot for keepalive at most once per interval, so
                    // steady-state streaming costs no extra frame copies.
                    if last_snapshot.elapsed() >= KEEPALIVE {
                        last_frame = Some(frame.clone());
                        last_snapshot = std::time::Instant::now();
                    }
                    let now_ns =
                        u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
                    last_pts_ns = now_ns.max(last_pts_ns.saturating_add(frame_dur));
                    let pts_ns = last_pts_ns;
                    if let Err(e) =
                        encoder.push_frame(&frame.data, frame.width, frame.height, pts_ns)
                    {
                        warn!(
                            "frame push rejected ({}x{}, {} B): {e}",
                            frame.width,
                            frame.height,
                            frame.data.len()
                        );
                    }
                    if n % 300 == 0 || n == 1 {
                        info!(
                            "source frame #{n} pushed ({}x{}, {} B)",
                            frame.width,
                            frame.height,
                            frame.data.len()
                        );
                    }
                    tokio::time::sleep(std::time::Duration::from_nanos(frame_dur)).await;
                }
                SourceOutcome::Retryable(e) => {
                    if source.is_ended() {
                        error!("capture source ended terminally ({e}); stopping capture pump");
                        break;
                    }
                    warn!("capture error: {e}");
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
                SourceOutcome::Ended => {
                    error!(
                        "frame source ended (evdi display disconnected?); stopping capture pump"
                    );
                    break;
                }
            }
        }
    });

    let enc_check = frame_count.clone();
    let _watchdog = tokio::spawn(async move {
        let mut warned = false;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let n = enc_check.load(std::sync::atomic::Ordering::Relaxed);
            if n == 0 && !warned {
                warn!(
                    "no frames captured yet - compositor may not be drawing on the virtual \
                     display (evdi) or the portal is not delivering buffers"
                );
                warned = true;
            }
        }
    });

    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<orbiscreen_transport::IncomingInput>();
    let mut injector = injector;
    let _input_pump = tokio::spawn(async move {
        use orbiscreen_input::PointerEvent;
        use orbiscreen_transport::IncomingInput;
        let (cap_w, cap_h) = cap_dims;
        let scale = |x: f64, y: f64| {
            let x = x * f64::from(spec.width) / f64::from(cap_w.max(1));
            let y = y * f64::from(spec.height) / f64::from(cap_h.max(1));
            (x, y)
        };
        while let Some(event) = input_rx.recv().await {
            let Some(injector) = injector.as_mut() else {
                continue;
            };
            match event {
                IncomingInput::Pointer(p) => {
                    let p = match p {
                        PointerEvent::Move { x, y } => {
                            let (x, y) = scale(x, y);
                            PointerEvent::Move { x, y }
                        }
                        other => other,
                    };
                    let _ = injector.inject_pointer(p).await;
                }
                IncomingInput::Key(k) => {
                    let _ = injector.inject_key(k).await;
                }
                IncomingInput::Stylus(s) => {
                    let _ = injector.inject_stylus(s).await;
                }
                IncomingInput::RawPointer { x, y } => {
                    let (x, y) = scale(x, y);
                    let _ = injector.inject_pointer(PointerEvent::Move { x, y }).await;
                }
            }
        }
    });

    let client_dir = std::env::var_os("ORBISCREEN_CLIENT_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            let mut paths = vec![
                std::env::current_dir()
                    .unwrap_or_default()
                    .join("clients")
                    .join("web"),
                PathBuf::from("/usr/share/orbiscreen/client"),
                PathBuf::from("/app/share/orbiscreen/client"),
            ];
            if let Ok(home) = std::env::var("HOME") {
                paths.insert(
                    1,
                    PathBuf::from(home).join(".local/share/orbiscreen/client"),
                );
            }
            paths.into_iter().find(|p| p.exists())
        })
        .unwrap_or_else(|| PathBuf::from("clients/web"));
    let transport = Transport::new(
        ServerConfig {
            signaling_port: cfg.transport.signaling_port,
            client_web_dir: client_dir,
        },
        input_tx,
    );
    let token = transport.token().to_owned();
    info!("stream access token generated ({len} chars); clients fetch it via mDNS TXT or /client/config.json",
        len = token.len());

    let _mdns = if !no_mdns && cfg.transport.mdns_advertise {
        match orbiscreen_transport::mdns::Advertiser::register(
            &orbiscreen_transport::ServiceDescriptor {
                instance: hostname::get()
                    .ok()
                    .and_then(|h| h.into_string().ok())
                    .unwrap_or_else(|| "orbiscreen-host".into()),
                port: cfg.transport.signaling_port,
                token: Some(token),
            },
        ) {
            Ok(a) => Some(a),
            Err(e) => {
                warn!("mDNS advertise failed (non-fatal): {e}");
                None
            }
        }
    } else {
        None
    };

    let serve_fut = transport.serve(
        video_rx,
        stats,
        actual_dims.0,
        actual_dims.1,
        spec.refresh_rate_hz,
        encoder_name,
    );

    let serve_res = tokio::select! {
        res = serve_fut => res,
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT (Ctrl-C), initiating graceful shutdown...");
            Ok(())
        }
        _ = shutdown_rx.changed() => {
            info!("D-Bus Stop received, initiating graceful shutdown...");
            Ok(())
        }
    };

    is_running.store(false, std::sync::atomic::Ordering::SeqCst);
    encoder.stop();
    cap_pump.abort();
    frame_pump.abort();
    serve_res.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_loads_when_file_absent() {
        let cfg = load_or_default_config("/tmp/orbiscreen-nonexistent-config.toml").unwrap();
        assert_eq!(cfg.display.width, 1920);
    }
}
