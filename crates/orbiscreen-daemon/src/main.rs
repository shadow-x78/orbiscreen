// Orbiscreen - orbiscreen-daemon daemon binary (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

pub mod dbus;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use orbiscreen_capture::CaptureSession;
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

/// One captured frame from either source, normalized to (width, height, data).
struct Frame {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

/// Frame source abstraction: evdi framebuffer (primary) or portal/X11 capture
/// fallback when the evdi kernel module is unavailable.
enum FrameSource {
    /// Reads the evdi virtual display framebuffer directly. This is the real
    /// secondary screen the compositor draws on. Runs on a dedicated thread
    /// because the evdi handle is `!Send`.
    Evdi(EvdiFramePump),
    /// Primary-desktop capture (portal or X11 root window) — a degraded mode
    /// that streams whatever the host shows on its main display.
    Capture(CaptureSession),
}

/// Outcome of one frame read.
enum SourceOutcome {
    /// A frame is ready.
    Frame(Frame),
    /// Transient failure — the pump keeps working; retry shortly.
    Retryable(String),
    /// The source is gone (evdi pump thread ended); stop streaming.
    Ended,
}

impl FrameSource {
    /// Reads the next frame. Blocks until a frame is ready or the source
    /// ends; there is no "no new content yet" state at this level because
    /// the pump/capture internals already wait on updates.
    async fn next_frame(&mut self) -> SourceOutcome {
        match self {
            FrameSource::Evdi(pump) => match pump.next_frame().await {
                Some(frame) => SourceOutcome::Frame(Frame {
                    width: frame.width,
                    height: frame.height,
                    data: frame.data,
                }),
                // Pump thread ended: the channel closed.
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
                orbiscreen_capture::CaptureBackend::X11 => "x11-portal-fallback",
                orbiscreen_capture::CaptureBackend::Wayland => "wayland-portal-fallback",
            },
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
        Command::Stop => {
            // Ask the running daemon to shut itself down gracefully through
            // the D-Bus session service.
            match dbus::request_stop().await {
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
                    eprintln!(
                        "hint: use 'systemctl --user stop orbiscreen' if it runs as a service"
                    );
                    ExitCode::from(1)
                }
            }
        }
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

    // Stop and disable service
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

    // Remove user files
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

    // Attempt system-wide cleanup (silently fail if no permission)
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

    // Primary frame source: read the evdi virtual display framebuffer. This
    // streams the actual secondary screen the compositor draws on. The
    // portal/X11 capture path is only a degraded fallback for hosts without
    // the evdi kernel module (it streams the primary desktop instead).
    let mut source = match EvdiFramePump::spawn(spec) {
        Ok(pump) => {
            info!(
                connector = ?pump.info().connector,
                device_index = pump.info().device_index,
                "Virtual display is open (EVDI DRM active); streaming the evdi framebuffer",
            );
            FrameSource::Evdi(pump)
        }
        Err(e) => {
            warn!(
                "EVDI kernel module missing/inactive ({e}). Falling back to primary-desktop \
                 capture via Wayland/X11 portal — clients will see the host's main display."
            );
            let capture = CaptureSession::open_async(spec.width, spec.height).await?;
            info!(backend = ?capture.backend(), "Capture backend open (fallback)");
            FrameSource::Capture(capture)
        }
    };

    // Actual negotiated dimensions of the stream source; may differ from the
    // requested spec. The encoder, `/api/info` and input mapping all use
    // these so buffer sizes always match the pushed frames.
    let actual_dims = source.actual_dimensions();
    info!(
        stream_width = actual_dims.0,
        stream_height = actual_dims.1,
        "stream dimensions established from source"
    );

    let injector = InputInjector::open_async(VirtualTouchscreenSpec {
        width: spec.width,
        height: spec.height,
    })
    .await?;
    info!(backend = ?injector.backend(), "Input injector open");

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
    // Shared between the capture pump (push) and shutdown (stop/EOS).
    let encoder = Arc::new(encoder);

    // Live stats shared with the transport and the D-Bus interface.
    let stats = std::sync::Arc::new(Stats::default());

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
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

    let (video_tx, video_rx) = mpsc::unbounded_channel::<H264Packet>();
    let frame_pump = tokio::spawn(async move {
        let mut n = 0u64;
        while let Some(chunk) = encoded_rx.recv().await {
            n += 1;
            if n <= 5 || n % 300 == 0 {
                info!(
                    "frame_pump: chunk #{n} ({} B, kf={}, pts={})",
                    chunk.bytes.len(),
                    chunk.is_keyframe,
                    chunk.pts_ns
                );
            }
            let pkt = H264Packet {
                bytes: chunk.bytes,
                is_keyframe: chunk.is_keyframe,
                pts_ns: chunk.pts_ns,
            };
            if video_tx.send(pkt).is_err() {
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
        loop {
            match source.next_frame().await {
                SourceOutcome::Frame(frame) => {
                    // PTS is assigned by the encoder's live appsrc
                    // (do-timestamp); the caller's pts is intentionally 0.
                    if let Err(e) = encoder.push_frame(&frame.data, frame.width, frame.height, 0) {
                        warn!(
                            "frame push rejected ({}x{}, {} B): {e}",
                            frame.width,
                            frame.height,
                            frame.data.len()
                        );
                    }
                    let n = fc.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    if n % 300 == 0 || n == 1 {
                        info!(
                            "source frame #{n} pushed ({}x{}, {} B)",
                            frame.width,
                            frame.height,
                            frame.data.len()
                        );
                    }
                    // Rate cap: never push faster than the configured refresh,
                    // even if the source delivers bursts (X11 fallback).
                    tokio::time::sleep(std::time::Duration::from_nanos(frame_dur)).await;
                }
                SourceOutcome::Retryable(e) => {
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

    // Watchdog: log if the pipeline stops producing frames.
    let enc_check = frame_count.clone();
    let _watchdog = tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let n = enc_check.load(std::sync::atomic::Ordering::Relaxed);
            if n == 0 {
                warn!(
                    "no frames captured yet - compositor may not be drawing on the virtual \
                     display (evdi) or the portal is not delivering buffers"
                );
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
    let transport = Transport::new(ServerConfig {
        signaling_port: cfg.transport.signaling_port,
        client_web_dir: client_dir,
    })
    .with_input_sender(input_tx);
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

    // USB transport bootstrap: reverse the signaling port on every
    // adb-authorized device so clients can reach the daemon via localhost.
    match orbiscreen_transport::adb::setup_reverse_for_all(
        orbiscreen_transport::adb::default_adb_path(),
        cfg.transport.signaling_port,
    ) {
        Ok(devs) => info!(
            "adb reverse set up for {} device(s) on port {}",
            devs.len(),
            cfg.transport.signaling_port,
        ),
        Err(orbiscreen_transport::adb::AdbError::NoDevice) => {
            info!("No USB-attached Android device with adb-authorized USB debugging");
        }
        Err(orbiscreen_transport::adb::AdbError::NotInstalled) => {
            info!("`adb` not in $PATH; skipping USB transport bootstrap");
        }
        Err(e) => warn!("adb reverse setup error: {e}"),
    }

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
