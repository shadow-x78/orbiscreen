// Orbiscreen - orbiscreen-daemon daemon binary (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

pub mod dbus;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use orbiscreen_capture::CaptureSession;
use orbiscreen_core::{dump_config, load_config, Config};
use orbiscreen_display::{DisplayStatus, VirtualDisplay, VirtualDisplaySpec};
use orbiscreen_encode::{EncodeParams, Encoder, EncoderKind};
use orbiscreen_input::{InputInjector, VirtualTouchscreenSpec};
use orbiscreen_transport::{H264Packet, ServerConfig, Transport};
use tokio::sync::mpsc;
use tracing::{error, info, warn, Level};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "orbiscreen",
    version,
    about = "Virtual secondary displays for Linux, streamed to Android",
    long_about = "Orbiscreen creates a real virtual display via evdi and streams it to \
                  Android devices over WebRTC (Wi-Fi or USB)."
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
    // try_init fails only if a subscriber is already set; ignored deliberately.
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

fn list_displays() {
    let cfg = Config::default();
    println!(
        "configured virtual display: {}x{} @ {} Hz (count = {})",
        cfg.display.width, cfg.display.height, cfg.display.refresh_rate_hz, cfg.display.count,
    );
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
            // There is no control socket yet, so the graceful stop path is to delegate
            // to systemd (or kill the PID). The D-Bus stop method only flips the
            // reported status flag for clients; the capture/transport loop is tied to
            // SIGINT.
            println!(
                "Use 'systemctl --user stop orbiscreen' or kill the process to stop the daemon."
            );
            ExitCode::SUCCESS
        }
        Command::Uninstall => {
            run_uninstall();
            ExitCode::SUCCESS
        }
        Command::ListDisplays => {
            list_displays();
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

    // The daemon's actual lifecycle is tied to the tokio runtime; the atomic here
    // reflects the D-Bus view of "serve is up". It is true from startup because
    // clients can connect as soon as the listener binds.
    let is_running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let dbus_running = is_running.clone();
    tokio::spawn(async move {
        if let Err(e) = dbus::run_dbus_server(dbus_running).await {
            warn!("D-Bus session service init failed (is D-Bus running?): {e}");
        }
    });
    info!("D-Bus session service registered: com.orbiscreen.Daemon");

    info!(
        "Orbiscreen starting - display {w}x{h}@{hz}Hz, encoder preferred = {enc}",
        w = spec.width,
        h = spec.height,
        hz = spec.refresh_rate_hz,
        enc = cfg.encode.preferred_encoder,
    );

    let _handle = match VirtualDisplay::open(spec).await {
        Ok(handle) => {
            info!(
                connector = ?handle.drm_connector_name(),
                "Virtual display is open (EVDI DRM active)",
            );
            Some(handle)
        }
        Err(e) => {
            warn!(
                "EVDI kernel module missing/inactive ({e}). Falling back to Wayland/X11 portal capture.",
            );
            None
        }
    };

    let capture = CaptureSession::open_async(spec.width, spec.height).await?;
    info!(backend = ?capture.backend(), "Capture backend open");

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
        width: spec.width,
        height: spec.height,
        framerate: spec.refresh_rate_hz,
    })?;
    let mut encoded_rx = encoder.subscribe().ok_or("encoder returned no rx")?;

    // Pumps encoded chunks out of the encoder into the transport. The encoder
    // timestamps buffers internally at push time, so H264Packet's pts_ns is
    // informational only here; 0 means "use the encoder's clock".
    let (video_tx, video_rx) = mpsc::unbounded_channel::<H264Packet>();
    let frame_pump = tokio::spawn(async move {
        while let Some(chunk) = encoded_rx.recv().await {
            let pkt = H264Packet {
                bytes: chunk.bytes,
                is_keyframe: chunk.is_keyframe,
                pts_ns: 0,
            };
            if video_tx.send(pkt).is_err() {
                break;
            }
        }
    });

    // Input scaling needs the capture region after `capture` moves into the
    // pump below, so snapshot it here.
    let cap_dims = (capture.width(), capture.height());
    let cap_pump = tokio::spawn(async move {
        let mut pts: u64 = 0;
        loop {
            match capture.next_frame().await {
                Ok(frame) => {
                    let _ = encoder.push_frame(&frame.data, frame.width, frame.height, pts);
                    let delay = std::time::Duration::from_nanos(Encoder::frame_duration_ns(
                        spec.refresh_rate_hz,
                    ));
                    pts = pts.saturating_add(Encoder::frame_duration_ns(spec.refresh_rate_hz));
                    tokio::time::sleep(delay).await;
                }
                Err(e) => {
                    warn!("capture error: {e}");
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            }
        }
    });

    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<orbiscreen_transport::IncomingInput>();
    let mut injector = injector;
    let _input_pump = tokio::spawn(async move {
        use orbiscreen_input::PointerEvent;
        use orbiscreen_transport::IncomingInput;
        // Clients report coordinates in the region actually shown on screen
        // (the evdi viewport if active, the native display otherwise), so
        // scale into display pixels before injecting. The capture region is
        // the stream's source of truth for the visible area.
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

    let _mdns = if !no_mdns && cfg.transport.mdns_advertise {
        match orbiscreen_transport::mdns::Advertiser::register(
            &orbiscreen_transport::ServiceDescriptor {
                instance: hostname::get()
                    .ok()
                    .and_then(|h| h.into_string().ok())
                    .unwrap_or_else(|| "orbiscreen-host".into()),
                port: cfg.transport.signaling_port,
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

    if !no_mdns {
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
    }

    let serve_fut = transport.serve(video_rx);

    // Dropping the mDNS advertiser before serve() returns defaults to unregistering
    // (Drop impl handles shutdown explicitly); keeping `_mdns` alive until here is
    // what stops the announcement disappearing before the HTTP listener is up.
    let serve_res = tokio::select! {
        res = serve_fut => res,
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT (Ctrl-C), initiating graceful shutdown...");
            Ok(())
        }
    };

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
