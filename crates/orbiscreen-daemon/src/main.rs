// Orbiscreen - main.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

pub mod dbus;
pub mod ui;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use orbiscreen_capture::capabilities::{Capabilities, CaptureStep};
use orbiscreen_capture::wlr_virtual_output::{VirtualOutputSpec, WlrootsVirtualOutput};
use orbiscreen_capture::{CaptureBackend, CapturePreference, CaptureSession};
use orbiscreen_core::{dump_config, load_config, Config};
use orbiscreen_display::{DisplayStatus, EvdiFramePump, VirtualDisplaySpec};
use orbiscreen_encode::{EncodeParams, Encoder, EncoderKind};
use orbiscreen_input::{InputInjector, VirtualTouchscreenSpec};
use orbiscreen_transport::{H264Packet, ServerConfig, Stats, Transport};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "orbiscreen",
    author = "shadow-x78 <https://github.com/shadow-x78/orbiscreen>",
    version = concat!(env!("CARGO_PKG_VERSION"), " (by shadow-x78)"),
    about = "Turn any Android device into a second monitor for Linux",
    long_about = "Orbiscreen turns Android tablets and phones into an extended secondary display\n\
                  for Linux (Wayland & X11) with ultra-low latency, hardware encoding, and\n\
                  reverse touch and stylus control.",
    after_help = "Written by shadow-x78 <https://github.com/shadow-x78/orbiscreen>",
    after_long_help = "Written by shadow-x78 <https://github.com/shadow-x78/orbiscreen>",
    styles = ui::clap_styles(),
)]
struct Cli {
    #[arg(
        short,
        long,
        global = true,
        value_name = "FILE",
        help = "Path to a custom configuration TOML file"
    )]
    config: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count, global = true, help = "Increase logging verbosity (-v for debug, -vv for trace)")]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(
        about = "Start the Orbiscreen daemon and begin streaming",
        alias = "up"
    )]
    Start {
        #[arg(long, help = "Disable mDNS network service broadcasting")]
        no_mdns: bool,

        #[arg(short, long, help = "Run in background as a systemd user service")]
        daemon: bool,
    },
    #[command(about = "Stop the running Orbiscreen daemon", alias = "down")]
    Stop,
    #[command(about = "Query the live status dashboard of the daemon")]
    Status {
        #[arg(long, help = "Output status information in raw JSON format")]
        json: bool,
    },
    #[command(about = "Inspect and configure virtual display properties")]
    Display {
        #[command(subcommand)]
        action: Option<DisplayAction>,
    },
    #[command(
        about = "List connected Android USB ADB devices and active clients",
        alias = "clients"
    )]
    Devices {
        #[arg(long, help = "Output devices list in JSON format")]
        json: bool,
    },
    #[command(about = "Run comprehensive system diagnostics and check compatibility")]
    Doctor {
        #[arg(long, help = "Output diagnostic report in JSON format")]
        json: bool,
        #[arg(
            long,
            help = "Attempt automatic installation of missing drivers and tools"
        )]
        fix: bool,
        #[arg(long, help = "Answer yes to all interactive prompts during fix")]
        yes: bool,
    },
    #[command(about = "Manage background systemd service lifecycle")]
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
}

#[derive(Subcommand, Debug)]
enum DisplayAction {
    #[command(about = "Show current virtual display properties")]
    Show,
    #[command(about = "Set virtual display resolution and refresh rate (e.g. 1920x1080@60)")]
    Set {
        #[arg(
            value_name = "SPEC",
            help = "Resolution spec (e.g. 1920x1080@60 or 2560x1440@120)"
        )]
        spec: Option<String>,
        #[arg(long, help = "Display width in pixels")]
        width: Option<u32>,
        #[arg(long, help = "Display height in pixels")]
        height: Option<u32>,
        #[arg(long, help = "Refresh rate in Hz")]
        fps: Option<u32>,
    },
}

#[derive(Subcommand, Debug)]
enum ServiceAction {
    #[command(about = "Show status of systemd user service")]
    Status,
    #[command(about = "Start daemon in background as systemd service")]
    Start,
    #[command(about = "Stop background systemd service")]
    Stop,
    #[command(about = "Restart background systemd service")]
    Restart,
    #[command(about = "Enable systemd service to start at login")]
    Enable,
    #[command(about = "Disable systemd service from starting at login")]
    Disable,
    #[command(about = "Install systemd user service file")]
    Install,
    #[command(about = "Cleanly uninstall systemd service and data")]
    Uninstall,
    #[command(about = "View or tail live logs from background service")]
    Logs {
        #[arg(short, long, help = "Follow log output in real time")]
        follow: bool,
        #[arg(
            short = 'n',
            long,
            default_value = "50",
            help = "Number of log lines to show"
        )]
        lines: usize,
    },
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

fn load_or_default_config(path: &Path) -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(load_config(&s)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(e.into()),
    }
}

struct Frame {
    width: u32,
    height: u32,
    data: orbiscreen_core::frame_pool::PooledFrameBuffer,
}

enum FrameSource {
    Evdi(EvdiFramePump, Arc<orbiscreen_core::frame_pool::FramePool>),
    Capture(CaptureSession),
    WlrVirtual {
        session: CaptureSession,
        output: WlrootsVirtualOutput,
    },
}

enum SourceOutcome {
    Frame(Frame),
    Retryable(String),
    Ended,
}

impl FrameSource {
    async fn next_frame(&mut self) -> SourceOutcome {
        match self {
            FrameSource::Evdi(pump, pool) => match pump.next_frame().await {
                Some(frame) => SourceOutcome::Frame(Frame {
                    width: frame.width,
                    height: frame.height,
                    data: pool.wrap(frame.data),
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
            FrameSource::WlrVirtual { session, .. } => match session.next_frame().await {
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
            FrameSource::Evdi(_, _) => "evdi",
            FrameSource::WlrVirtual { .. } => "wlr-virtual",
            FrameSource::Capture(c) => match c.backend() {
                orbiscreen_capture::CaptureBackend::X11 => "x11",
                orbiscreen_capture::CaptureBackend::Wayland => "wayland-portal",
                orbiscreen_capture::CaptureBackend::KwinVirtual => "kwin-virtual",
                orbiscreen_capture::CaptureBackend::WlrScreencopy => "wlr-screencopy",
            },
        }
    }

    fn is_ended(&self) -> bool {
        match self {
            FrameSource::Evdi(_, _) => false,
            FrameSource::Capture(capture) => capture.is_ended(),
            FrameSource::WlrVirtual { session, .. } => session.is_ended(),
        }
    }

    fn actual_dimensions(&self) -> (u32, u32) {
        match self {
            FrameSource::Evdi(pump, _) => pump.actual_dimensions(),
            FrameSource::Capture(capture) => (capture.width(), capture.height()),
            FrameSource::WlrVirtual { session, .. } => (session.width(), session.height()),
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(orbiscreen_core::default_config_path);
    let cfg = match load_or_default_config(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("config error: {e}");
            return ExitCode::from(2);
        }
    };

    match cli.command {
        Some(Command::Start { no_mdns, daemon }) => {
            if daemon {
                run_service_action(ServiceAction::Start).await
            } else {
                if is_daemon_already_running(cfg.transport.signaling_port).await {
                    ui::print_banner();
                    println!();
                    println!(
                        "{} Orbiscreen is already running in the background",
                        ui::badge_warn()
                    );
                    println!(
                        "   Run 'orbiscreen status' to view live status, or 'orbiscreen stop' to stop it.\n"
                    );
                    return ExitCode::SUCCESS;
                }
                match run_start(cfg, no_mdns).await {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        let err_str = e.to_string();
                        if err_str.contains("Address already in use") {
                            ui::print_banner();
                            println!();
                            println!(
                                "{} Orbiscreen is already running in the background (port in use)",
                                ui::badge_warn()
                            );
                            println!(
                                "   Run 'orbiscreen status' to view live status, or 'orbiscreen stop' to stop it.\n"
                            );
                        } else {
                            error!("orbiscreen start failed: {e}");
                        }
                        ExitCode::from(1)
                    }
                }
            }
        }
        Some(Command::Stop) => match dbus::request_stop().await {
            Ok(reply) => {
                println!("{} {reply}", ui::badge_ok());
                ExitCode::SUCCESS
            }
            Err(zbus::Error::MethodError(name, _, _))
                if name.to_string().contains("ServiceUnknown") =>
            {
                let systemd_status = std::process::Command::new("systemctl")
                    .args(["--user", "is-active", "orbiscreen"])
                    .output()
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .unwrap_or_default();
                if systemd_status == "active" {
                    run_service_action(ServiceAction::Stop).await
                } else {
                    println!(
                        "{} Daemon is not running (no com.orbiscreen.Daemon on the session bus)",
                        ui::badge_info()
                    );
                    ExitCode::from(1)
                }
            }
            Err(e) => {
                let systemd_status = std::process::Command::new("systemctl")
                    .args(["--user", "is-active", "orbiscreen"])
                    .output()
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .unwrap_or_default();
                if systemd_status == "active" {
                    run_service_action(ServiceAction::Stop).await
                } else {
                    eprintln!("{} Stop failed: {e}", ui::badge_err());
                    ExitCode::from(1)
                }
            }
        },
        Some(Command::Status { json }) => run_status(json, cfg.transport.signaling_port).await,
        Some(Command::Display { action }) => run_display(&config_path, action).await,
        Some(Command::Devices { json }) => run_devices(json).await,
        Some(Command::Doctor { json, fix, yes }) => {
            if fix {
                run_doctor_fix(yes).await
            } else {
                run_doctor(json).await
            }
        }
        Some(Command::Service { action }) => run_service_action(action).await,
        None => match dbus::request_status().await {
            Ok(_) => run_status(false, cfg.transport.signaling_port).await,
            Err(_) => {
                if is_daemon_already_running(cfg.transport.signaling_port).await {
                    run_status(false, cfg.transport.signaling_port).await
                } else {
                    ui::print_banner();
                    ui::print_welcome_card();
                    println!();
                    ExitCode::SUCCESS
                }
            }
        },
    }
}

async fn query_local_endpoint(port: u16, path: &str) -> Option<serde_json::Value> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::time::timeout(
        std::time::Duration::from_millis(400),
        tokio::net::TcpStream::connect(("127.0.0.1", port)),
    )
    .await
    .ok()?
    .ok()?;

    let req = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).await.ok()?;

    let mut buf = Vec::with_capacity(1024);
    tokio::time::timeout(
        std::time::Duration::from_millis(400),
        stream.read_to_end(&mut buf),
    )
    .await
    .ok()?
    .ok()?;

    let resp = String::from_utf8_lossy(&buf);
    let body = resp.split("\r\n\r\n").nth(1)?;
    serde_json::from_str(body).ok()
}

async fn is_daemon_already_running(port: u16) -> bool {
    if dbus::request_status().await.is_ok() {
        return true;
    }
    if query_local_endpoint(port, "/health").await.is_some() {
        return true;
    }
    if std::net::TcpListener::bind(("0.0.0.0", port)).is_err() {
        return true;
    }
    false
}

async fn run_status(json: bool, port: u16) -> ExitCode {
    match dbus::request_status().await {
        Ok(reply) => {
            if json {
                println!("{reply}");
                return ExitCode::SUCCESS;
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&reply) {
                let is_running = val
                    .get("running")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let encoder = val
                    .get("encoder")
                    .and_then(|v| v.as_str())
                    .unwrap_or("auto");
                let capture = val
                    .get("capture_backend")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                let active = val
                    .get("active_clients")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let usb = val.get("usb_devices").and_then(|v| v.as_u64()).unwrap_or(0);
                let w = val
                    .get("display_width")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1920);
                let h = val
                    .get("display_height")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1080);
                let fps = val
                    .get("display_fps")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(60);
                let port = val
                    .get("signaling_port")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(u64::from(port)) as u16;

                let token = std::fs::read_to_string(orbiscreen_core::default_token_path())
                    .unwrap_or_else(|_| "token-active".to_string())
                    .trim()
                    .to_string();

                ui::print_banner();
                ui::print_status_dashboard(
                    is_running, w, h, fps, encoder, capture, port, active, usb, &token,
                );
                ExitCode::SUCCESS
            } else {
                println!("{reply}");
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            if let Some(health) = query_local_endpoint(port, "/health").await {
                let info = query_local_endpoint(port, "/api/info")
                    .await
                    .unwrap_or_default();
                let is_running = health.get("status").and_then(|v| v.as_str()) == Some("ok");
                let encoder = health
                    .get("encoder")
                    .and_then(|v| v.as_str())
                    .or_else(|| info.get("encoder").and_then(|v| v.as_str()))
                    .unwrap_or("auto");
                let active = health
                    .get("active_clients")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let usb = health
                    .get("usb_devices")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let w = info
                    .get("display_width")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1920);
                let h = info
                    .get("display_height")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1080);
                let fps = info
                    .get("refresh_hz")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(60);
                let capture = "Virtual";
                let token = std::fs::read_to_string(orbiscreen_core::default_token_path())
                    .unwrap_or_else(|_| "token-active".to_string())
                    .trim()
                    .to_string();

                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "running": is_running,
                            "encoder": encoder,
                            "capture_backend": capture,
                            "active_clients": active,
                            "usb_devices": usb,
                            "display_width": w,
                            "display_height": h,
                            "display_fps": fps,
                            "signaling_port": port,
                        })
                    );
                    return ExitCode::SUCCESS;
                }

                ui::print_banner();
                ui::print_status_dashboard(
                    is_running, w, h, fps, encoder, capture, port, active, usb, &token,
                );
                return ExitCode::SUCCESS;
            }

            let systemd_status = std::process::Command::new("systemctl")
                .args(["--user", "is-active", "orbiscreen"])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|| "inactive".to_string());

            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "running": false,
                        "systemd_service": systemd_status,
                        "error": e.to_string()
                    })
                );
                return ExitCode::from(1);
            }

            ui::print_banner();
            println!();
            println!(
                "{} {}Orbiscreen Daemon is STOPPED{}",
                ui::badge_err(),
                if ui::colors_enabled() {
                    ui::colors::BOLD
                } else {
                    ""
                },
                if ui::colors_enabled() {
                    ui::colors::RESET
                } else {
                    ""
                }
            );
            println!();
            let rows = vec![
                (
                    "Daemon Process",
                    "Not running (No com.orbiscreen.Daemon on D-Bus)".to_string(),
                ),
                (
                    "Systemd Service",
                    format!("orbiscreen.service ({systemd_status})"),
                ),
                (
                    "Start Foreground",
                    "orbiscreen start (or: orbiscreen up)".to_string(),
                ),
                (
                    "Start Background",
                    "orbiscreen start -d (or: orbiscreen service start)".to_string(),
                ),
                ("Health Check", "orbiscreen doctor".to_string()),
            ];
            ui::print_card("Orbiscreen Daemon Status", &rows);
            println!();
            ExitCode::from(1)
        }
    }
}

async fn run_display(config_path: &Path, action: Option<DisplayAction>) -> ExitCode {
    match action {
        None | Some(DisplayAction::Show) => {
            ui::print_banner();
            println!();
            match load_or_default_config(config_path) {
                Ok(cfg) => {
                    let caps = Capabilities::from_env();
                    let chain = caps.auto_chain();
                    let display_backend = if caps.compositor
                        == orbiscreen_capture::capabilities::Compositor::Kde
                        || chain.iter().any(|s| matches!(s, CaptureStep::KwinVirtual))
                    {
                        "KWin Virtual (Native)".to_string()
                    } else if orbiscreen_capture::wlr_virtual_output::detect_ipc_kind().is_some() {
                        "wlroots (Native)".to_string()
                    } else {
                        format!("{:?}", orbiscreen_display::probe())
                    };
                    let rows = vec![
                        (
                            "Resolution",
                            format!("{}x{}", cfg.display.width, cfg.display.height),
                        ),
                        (
                            "Refresh Rate",
                            format!("{} Hz", cfg.display.refresh_rate_hz),
                        ),
                        (
                            "Auto-Orientation",
                            "Supported (Swaps W/H dynamically)".to_string(),
                        ),
                        ("Capture Preferred", cfg.capture.preferred.clone()),
                        ("Display Backend", display_backend),
                    ];
                    ui::print_card("Virtual Display Specification", &rows);
                    println!();
                    println!(
                        "{} {}To change resolution:{} orbiscreen display set <WIDTHxHEIGHT@FPS>",
                        ui::badge_info(),
                        if ui::colors_enabled() {
                            ui::colors::BOLD
                        } else {
                            ""
                        },
                        if ui::colors_enabled() {
                            ui::colors::RESET
                        } else {
                            ""
                        }
                    );
                    println!();
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{} Config error: {e}", ui::badge_err());
                    ExitCode::from(1)
                }
            }
        }
        Some(DisplayAction::Set {
            spec,
            width,
            height,
            fps,
        }) => {
            let mut cfg = load_or_default_config(config_path).unwrap_or_default();
            let mut parsed_w = width;
            let mut parsed_h = height;
            let mut parsed_fps = fps;

            if let Some(s) = spec {
                let s = s.trim();
                let (res_part, rate_part) = match s.split_once('@') {
                    Some((res, r)) => (res, Some(r)),
                    None => (s, None),
                };
                if let Some((w_str, h_str)) = res_part.split_once('x') {
                    if let Ok(w) = w_str.parse::<u32>() {
                        parsed_w = Some(w);
                    }
                    if let Ok(h) = h_str.parse::<u32>() {
                        parsed_h = Some(h);
                    }
                }
                if let Some(r_str) = rate_part {
                    if let Ok(r) = r_str
                        .trim_end_matches("Hz")
                        .trim_end_matches("hz")
                        .parse::<u32>()
                    {
                        parsed_fps = Some(r);
                    }
                }
            }

            if let Some(w) = parsed_w {
                cfg.display.width = w;
            }
            if let Some(h) = parsed_h {
                cfg.display.height = h;
            }
            if let Some(f) = parsed_fps {
                cfg.display.refresh_rate_hz = f;
            }

            match dump_config(&cfg) {
                Ok(content) => {
                    if let Some(parent) = config_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if let Err(e) = std::fs::write(config_path, content) {
                        eprintln!("{} Failed to save config: {e}", ui::badge_err());
                        return ExitCode::from(1);
                    }
                    println!(
                        "{} Virtual display set to {}x{} @ {} Hz (saved to {})",
                        ui::badge_ok(),
                        cfg.display.width,
                        cfg.display.height,
                        cfg.display.refresh_rate_hz,
                        config_path.display()
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{} Failed to serialize config: {e}", ui::badge_err());
                    ExitCode::from(1)
                }
            }
        }
    }
}

async fn run_devices(json: bool) -> ExitCode {
    let report = tokio::task::spawn_blocking(usb_doctor_report)
        .await
        .unwrap_or_default();

    if json {
        println!(
            "{}",
            serde_json::json!({
                "adb_installed": report.adb_installed,
                "devices": report.devices,
                "reverse_tunnels": report.reverse_tunnels,
            })
        );
        return ExitCode::SUCCESS;
    }

    ui::print_banner();
    println!();
    let dev_str = if !report.adb_installed {
        format!(
            "{} ADB not installed (Wi-Fi streaming unaffected)",
            ui::badge_warn()
        )
    } else if report.devices.is_empty() {
        format!("{} No devices connected (Hot-plug ready)", ui::badge_warn())
    } else {
        format!(
            "{} {} device(s) connected ({})",
            ui::badge_ok(),
            report.devices.len(),
            report.devices.join(", ")
        )
    };

    let tunnel_str = if report.reverse_tunnels > 0 {
        format!("{} Active (Port 8788 reversed)", ui::badge_ok())
    } else if !report.devices.is_empty() {
        format!("{} Active on port 8788", ui::badge_ok())
    } else {
        format!("{} Waiting for device", ui::badge_info())
    };

    let rows = vec![
        ("USB ADB Devices", dev_str),
        ("Reverse Tunnel", tunnel_str),
        (
            "Auto-Discovery",
            "Active via mDNS on local subnet".to_string(),
        ),
        (
            "Connection Tip",
            "Enable USB Debugging on Android for zero-config wired streaming".to_string(),
        ),
    ];
    ui::print_card("Connected Devices & Clients", &rows);
    println!();
    ExitCode::SUCCESS
}

async fn run_service_action(action: ServiceAction) -> ExitCode {
    match action {
        ServiceAction::Status => {
            let status = std::process::Command::new("systemctl")
                .args(["--user", "status", "orbiscreen"])
                .status();
            match status {
                Ok(s) if s.success() => ExitCode::SUCCESS,
                _ => ExitCode::from(1),
            }
        }
        ServiceAction::Start => {
            let status = std::process::Command::new("systemctl")
                .args(["--user", "start", "orbiscreen"])
                .status();
            match status {
                Ok(s) if s.success() => {
                    println!("{} Orbiscreen background service started", ui::badge_ok());
                    println!(
                        "{} Run 'orbiscreen status' to view live dashboard",
                        ui::badge_info()
                    );
                    ExitCode::SUCCESS
                }
                _ => {
                    eprintln!(
                        "{} Failed to start orbiscreen.service via systemctl",
                        ui::badge_err()
                    );
                    ExitCode::from(1)
                }
            }
        }
        ServiceAction::Stop => {
            let status = std::process::Command::new("systemctl")
                .args(["--user", "stop", "orbiscreen"])
                .status();
            match status {
                Ok(s) if s.success() => {
                    println!("{} Orbiscreen background service stopped", ui::badge_ok());
                    ExitCode::SUCCESS
                }
                _ => {
                    eprintln!(
                        "{} Failed to stop orbiscreen.service via systemctl",
                        ui::badge_err()
                    );
                    ExitCode::from(1)
                }
            }
        }
        ServiceAction::Restart => {
            let status = std::process::Command::new("systemctl")
                .args(["--user", "restart", "orbiscreen"])
                .status();
            match status {
                Ok(s) if s.success() => {
                    println!("{} Orbiscreen background service restarted", ui::badge_ok());
                    ExitCode::SUCCESS
                }
                _ => {
                    eprintln!(
                        "{} Failed to restart orbiscreen.service via systemctl",
                        ui::badge_err()
                    );
                    ExitCode::from(1)
                }
            }
        }
        ServiceAction::Enable => {
            let status = std::process::Command::new("systemctl")
                .args(["--user", "enable", "orbiscreen"])
                .status();
            match status {
                Ok(s) if s.success() => {
                    println!(
                        "{} Orbiscreen background service enabled at login",
                        ui::badge_ok()
                    );
                    ExitCode::SUCCESS
                }
                _ => {
                    eprintln!("{} Failed to enable orbiscreen.service", ui::badge_err());
                    ExitCode::from(1)
                }
            }
        }
        ServiceAction::Disable => {
            let status = std::process::Command::new("systemctl")
                .args(["--user", "disable", "orbiscreen"])
                .status();
            match status {
                Ok(s) if s.success() => {
                    println!(
                        "{} Orbiscreen background service disabled from login",
                        ui::badge_ok()
                    );
                    ExitCode::SUCCESS
                }
                _ => {
                    eprintln!("{} Failed to disable orbiscreen.service", ui::badge_err());
                    ExitCode::from(1)
                }
            }
        }
        ServiceAction::Install => {
            let unit_dir = if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME")
                .filter(|v| !v.is_empty() && Path::new(v).is_absolute())
            {
                PathBuf::from(xdg).join("systemd/user")
            } else if let Some(home) = std::env::var_os("HOME").filter(|v| !v.is_empty()) {
                PathBuf::from(home).join(".config/systemd/user")
            } else {
                PathBuf::from(".config/systemd/user")
            };
            let _ = std::fs::create_dir_all(&unit_dir);
            let unit_path = unit_dir.join("orbiscreen.service");
            let unit_content = r#"[Unit]
Description=Orbiscreen Virtual Display Daemon
After=graphical-session.target
PartOf=graphical-session.target

[Service]
ExecStart=/usr/bin/orbiscreen start
Restart=on-failure
RestartSec=2s

[Install]
WantedBy=graphical-session.target
"#;
            if let Err(e) = std::fs::write(&unit_path, unit_content) {
                eprintln!("{} Failed to write unit file: {e}", ui::badge_err());
                return ExitCode::from(1);
            }
            let _ = std::process::Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .status();
            println!(
                "{} Installed systemd user service to {}",
                ui::badge_ok(),
                unit_path.display()
            );
            ExitCode::SUCCESS
        }
        ServiceAction::Uninstall => tokio::task::spawn_blocking(run_uninstall)
            .await
            .unwrap_or(ExitCode::from(1)),
        ServiceAction::Logs { follow, lines } => {
            let mut cmd = std::process::Command::new("journalctl");
            cmd.args(["--user", "-u", "orbiscreen", "-n", &lines.to_string()]);
            if follow {
                cmd.arg("-f");
            }
            let status = cmd.status();
            match status {
                Ok(s) if s.success() => ExitCode::SUCCESS,
                _ => ExitCode::from(1),
            }
        }
    }
}

fn run_uninstall() -> ExitCode {
    println!("[Orbiscreen] Uninstalling...");
    let mut failures = 0u32;

    for (program, args) in [
        ("systemctl", ["--user", "stop", "orbiscreen"].as_slice()),
        ("systemctl", ["--user", "disable", "orbiscreen"].as_slice()),
        ("systemctl", ["--user", "daemon-reload"].as_slice()),
    ] {
        match std::process::Command::new(program).args(args).status() {
            Ok(status) if status.success() => {}
            Ok(status) => {
                warn!("{program} {args:?} exited with {status} (may be already removed)");
            }
            Err(e) => {
                warn!("failed to run {program}: {e}");
            }
        }
    }

    let remove_file = |path: &Path, failures: &mut u32| match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            warn!("failed to remove {}: {e}", path.display());
            *failures += 1;
        }
    };
    let remove_dir = |path: &Path, failures: &mut u32| match std::fs::remove_dir_all(path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            warn!("failed to remove {}: {e}", path.display());
            *failures += 1;
        }
    };

    match std::env::var_os("HOME").map(PathBuf::from) {
        Some(home) if home.is_absolute() => {
            remove_file(&home.join(".local/bin/orbiscreen"), &mut failures);
            remove_file(
                &home.join(".config/systemd/user/orbiscreen.service"),
                &mut failures,
            );
            remove_file(
                &home.join(".local/share/applications/com.orbiscreen.OrbiscreenGtk.desktop"),
                &mut failures,
            );
            remove_file(
                &home.join(
                    ".local/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg",
                ),
                &mut failures,
            );
            remove_dir(&home.join(".local/share/orbiscreen"), &mut failures);
        }
        _ => warn!("HOME is not set or not absolute; skipping user-level removal"),
    }

    remove_file(Path::new("/usr/bin/orbiscreen"), &mut failures);
    remove_file(
        Path::new("/usr/share/applications/com.orbiscreen.OrbiscreenGtk.desktop"),
        &mut failures,
    );
    remove_file(
        Path::new("/usr/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg"),
        &mut failures,
    );
    remove_dir(Path::new("/usr/share/orbiscreen"), &mut failures);

    if failures == 0 {
        println!("[Orbiscreen] Uninstallation complete.");
        ExitCode::SUCCESS
    } else {
        println!(
            "[Orbiscreen] Uninstallation finished with {failures} error(s); see warnings above."
        );
        ExitCode::from(1)
    }
}

type DynError = Box<dyn std::error::Error + Send + Sync>;

async fn try_capture_step(
    step: CaptureStep,
    spec: VirtualDisplaySpec,
    frame_pool: &Arc<orbiscreen_core::frame_pool::FramePool>,
) -> Result<FrameSource, DynError> {
    match step {
        CaptureStep::Evdi => {
            let pump = EvdiFramePump::spawn(spec)?;
            info!(
                connector = ?pump.info().connector,
                device_index = pump.info().device_index,
                "Virtual display is open (EVDI DRM active); streaming the evdi framebuffer",
            );
            Ok(FrameSource::Evdi(pump, Arc::clone(frame_pool)))
        }
        CaptureStep::X11Root => {
            let capture = CaptureSession::open_with_preference(
                spec.width,
                spec.height,
                CapturePreference::Auto,
            )
            .await?;
            info!(backend = ?capture.backend(), "Capture backend open");
            Ok(FrameSource::Capture(capture))
        }
        CaptureStep::KwinVirtual => {
            let capture = CaptureSession::open_with_preference(
                spec.width,
                spec.height,
                CapturePreference::KwinVirtual,
            )
            .await?;
            info!(backend = ?capture.backend(), "Capture backend open");
            Ok(FrameSource::Capture(capture))
        }
        CaptureStep::Portal => {
            let capture = CaptureSession::open_with_preference(
                spec.width,
                spec.height,
                CapturePreference::Portal,
            )
            .await?;
            info!(backend = ?capture.backend(), "Capture backend open");
            Ok(FrameSource::Capture(capture))
        }
        CaptureStep::WlrootsVirtual => {
            let vspec = VirtualOutputSpec {
                width: spec.width,
                height: spec.height,
                refresh_rate_hz: spec.refresh_rate_hz,
            };
            let output = tokio::task::spawn_blocking(move || WlrootsVirtualOutput::create(vspec))
                .await
                .map_err(|e| format!("wlroots virtual output task: {e}"))??;
            let output_name = output.name().to_string();
            let session = CaptureSession::open_screencopy(Some(output_name)).await?;
            info!(backend = ?session.backend(), "Capture backend open");
            Ok(FrameSource::WlrVirtual { session, output })
        }
        CaptureStep::WlrScreencopy => {
            let capture = CaptureSession::open_with_preference(
                spec.width,
                spec.height,
                CapturePreference::Screencopy,
            )
            .await?;
            info!(backend = ?capture.backend(), "Capture backend open");
            Ok(FrameSource::Capture(capture))
        }
    }
}

async fn resolve_frame_source(
    preferred: &str,
    caps: &Capabilities,
    spec: VirtualDisplaySpec,
    frame_pool: &Arc<orbiscreen_core::frame_pool::FramePool>,
) -> Result<FrameSource, DynError> {
    let chain = match preferred {
        "auto" => caps.auto_chain(),
        "evdi" => vec![CaptureStep::Evdi],
        "kwin-virtual" => vec![CaptureStep::KwinVirtual],
        "screencopy" => vec![CaptureStep::WlrScreencopy],
        "portal" | "mirror" => vec![CaptureStep::Portal],
        _ => vec![CaptureStep::X11Root],
    };
    let explicit = preferred != "auto";
    info!(
        session = %caps.session,
        compositor = %caps.compositor,
        preferred = preferred,
        chain = ?chain,
        "capture plan resolved from environment capabilities",
    );
    let mut last_err: Option<DynError> = None;
    for step in chain {
        match try_capture_step(step, spec, frame_pool).await {
            Ok(source) => {
                if let Some(e) = last_err {
                    info!(step = %step, "capture step succeeded after earlier failure: {e}");
                }
                return Ok(source);
            }
            Err(e) => {
                if explicit {
                    return Err(e);
                }
                warn!(step = %step, "capture step failed ({e}); trying the next step in the chain");
                last_err = Some(e);
            }
        }
    }
    match last_err {
        Some(e) => {
            eprintln!("\n[Orbiscreen] Capture pipeline failed to initialize automatically: {e}");
            eprintln!("[Orbiscreen] Run 'orbiscreen doctor --fix' to auto-install missing kernel drivers or dependencies.\n");
            Err(e)
        }
        None => {
            eprintln!("\n[Orbiscreen] No capture step is available for this environment.");
            eprintln!("[Orbiscreen] Run 'orbiscreen doctor --fix' to auto-install missing kernel drivers or dependencies.\n");
            Err("no capture step is available for this environment".into())
        }
    }
}

fn display_status_text(status: DisplayStatus) -> &'static str {
    match status {
        DisplayStatus::Compatible => "Compatible (kernel + libevdi OK)",
        DisplayStatus::Outdated => "Outdated (kernel evdi older than libevdi requires)",
        DisplayStatus::KernelModuleMissing => "kernel module missing",
        DisplayStatus::NoDeviceNode => {
            "kernel OK, no evdi device node yet (added by `orbiscreen start` or evdi_ctl)"
        }
    }
}

fn has_binary(name: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| {
        std::fs::metadata(dir.join(name))
            .map(|m| {
                use std::os::unix::fs::PermissionsExt as _;
                m.is_file() && m.permissions().mode() & 0o111 != 0
            })
            .unwrap_or(false)
    })
}

async fn portal_available() -> Option<bool> {
    let conn = zbus::Connection::session().await.ok()?;
    let proxy = zbus::Proxy::new(
        &conn,
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
    )
    .await
    .ok()?;
    let owned: bool = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        proxy.call::<_, &str, bool>("NameHasOwner", &"org.freedesktop.portal.Desktop"),
    )
    .await
    .ok()?
    .ok()?;
    Some(owned)
}

async fn bind_kwin_virtual_inputs(target_output: &str) {
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    if let Ok(conn) = zbus::Connection::session().await {
        for idx in 0..64 {
            let path = format!("/org/kde/KWin/InputDevice/event{idx}");
            if let Ok(proxy) = zbus::Proxy::new(
                &conn,
                "org.kde.KWin",
                path.as_str(),
                "org.kde.KWin.InputDevice",
            )
            .await
            {
                if let Ok(name) = proxy.get_property::<String>("name").await {
                    if name.starts_with("Orbiscreen") {
                        let _ = proxy
                            .set_property::<&str>("outputName", target_output)
                            .await;
                        info!("bound KWin input device {path} ({name}) to output {target_output}");
                    }
                }
            }
        }
    }
}

async fn run_doctor(json: bool) -> ExitCode {
    let caps = Capabilities::from_env();
    let chain = caps.auto_chain();
    let display_status = orbiscreen_display::probe();
    let input = orbiscreen_input::detect_backend();
    let uinput_writable = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/uinput")
        .is_ok();
    let swaymsg = has_binary("swaymsg");
    let hyprctl = has_binary("hyprctl");
    let cosmic_comp = has_binary("cosmic-comp");
    let cosmic_randr = has_binary("cosmic-randr");
    let wlr_virtual_ipc = orbiscreen_capture::wlr_virtual_output::detect_ipc_kind();
    let usb = usb_doctor_report();
    let portal_state = orbiscreen_core::portal_state::load_portal_state();
    let screencast_saved = portal_state.screencast_restore_token.is_some();
    let input_saved = portal_state.remote_desktop_restore_token.is_some();
    let portal = match caps.session {
        orbiscreen_capture::capabilities::SessionType::Wayland => portal_available().await,
        _ => None,
    };

    let is_kwin_native = caps.compositor == orbiscreen_capture::capabilities::Compositor::Kde
        || chain.iter().any(|s| matches!(s, CaptureStep::KwinVirtual));
    let is_wlr_native = wlr_virtual_ipc.is_some()
        || chain
            .iter()
            .any(|s| matches!(s, CaptureStep::WlrootsVirtual));

    let display_backend_name = if is_kwin_native {
        "Native (KWin Virtual)"
    } else if is_wlr_native {
        "Native (wlroots)"
    } else {
        display_status_text(display_status)
    };

    if json {
        let report = serde_json::json!({
            "session": caps.session.to_string(),
            "compositor": caps.compositor.to_string(),
            "current_desktop": caps.current_desktop,
            "capture_plan": chain.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            "display_backend": display_backend_name,
            "input_backend": format!("{input:?}"),
            "uinput_writable": uinput_writable,
            "portal_on_session_bus": portal,
            "screencast_saved_token": screencast_saved,
            "remote_desktop_saved_token": input_saved,
            "swaymsg": swaymsg,
            "hyprctl": hyprctl,
            "cosmic_comp": cosmic_comp,
            "cosmic_randr": cosmic_randr,
            "wlroots_virtual_output_ipc": wlr_virtual_ipc.map(|k| k.to_string()),
            "usb": {
                "adb_installed": usb.adb_installed,
                "devices": usb.devices,
                "reverse_tunnels": usb.reverse_tunnels,
            },
        });
        println!("{report}");
        return ExitCode::SUCCESS;
    }

    ui::print_banner();
    println!();

    let session_name = match caps.session {
        orbiscreen_capture::capabilities::SessionType::Wayland => "Wayland",
        orbiscreen_capture::capabilities::SessionType::X11 => "X11",
        orbiscreen_capture::capabilities::SessionType::Unknown => "Unknown",
    };
    let session_str = format!(
        "{}{}",
        session_name,
        caps.current_desktop
            .as_deref()
            .map(|d| format!(" (XDG_CURRENT_DESKTOP={d})"))
            .unwrap_or_default(),
    );

    let plan_str = chain
        .iter()
        .map(|s| match s {
            CaptureStep::KwinVirtual => "KWin Virtual",
            CaptureStep::WlrootsVirtual => "wlroots Virtual",
            CaptureStep::WlrScreencopy => "wlroots Screencopy",
            CaptureStep::Portal => "Portal",
            CaptureStep::Evdi => "EVDI",
            CaptureStep::X11Root => "X11 Root",
        })
        .collect::<Vec<_>>()
        .join(" ➔ ");

    let display_str = if is_kwin_native {
        format!("{} Native (KWin Virtual)", ui::badge_ok())
    } else if is_wlr_native {
        let ipc_label = wlr_virtual_ipc
            .map(|k| k.to_string())
            .unwrap_or_else(|| "wlroots".to_string());
        format!("{} Native ({ipc_label})", ui::badge_ok())
    } else {
        match display_status {
            DisplayStatus::Compatible => {
                format!("{} Compatible (Kernel + libevdi)", ui::badge_ok())
            }
            DisplayStatus::Outdated => format!(
                "{} Outdated (Kernel evdi older than libevdi)",
                ui::badge_warn()
            ),
            DisplayStatus::KernelModuleMissing => {
                format!("{} Kernel module missing", ui::badge_warn())
            }
            DisplayStatus::NoDeviceNode => {
                format!("{} Kernel OK, no evdi device node yet", ui::badge_info())
            }
        }
    };

    let uinput_str = if uinput_writable {
        format!("{} Writable (/dev/uinput)", ui::badge_ok())
    } else {
        format!("{} Needs root or uinput group", ui::badge_warn())
    };

    let portal_str = match portal {
        Some(true) => format!("{} Active (org.freedesktop.portal.Desktop)", ui::badge_ok()),
        Some(false) => format!("{} Missing (Install xdg-desktop-portal)", ui::badge_warn()),
        None => {
            if caps.session == orbiscreen_capture::capabilities::SessionType::Wayland {
                format!("{} Unchecked (Session bus not connected)", ui::badge_info())
            } else {
                format!("{} Not applicable (X11)", ui::badge_info())
            }
        }
    };

    let grants_str = format!(
        "Screencast: {} · Remote Input: {}",
        if screencast_saved {
            "Saved (Reused)"
        } else {
            "Prompt on start"
        },
        if input_saved {
            "Saved (Reused)"
        } else {
            "Prompt on start"
        },
    );

    let tools_str = match caps.compositor {
        orbiscreen_capture::capabilities::Compositor::Kde => "KWin D-Bus (Native)".to_string(),
        orbiscreen_capture::capabilities::Compositor::Gnome => "Mutter Portal (Native)".to_string(),
        orbiscreen_capture::capabilities::Compositor::Cosmic => format!(
            "cosmic-comp: {} · cosmic-randr: {}",
            if cosmic_comp { "Found" } else { "No" },
            if cosmic_randr { "Found" } else { "No" },
        ),
        _ => format!(
            "swaymsg: {} · hyprctl: {}",
            if swaymsg { "Found" } else { "No" },
            if hyprctl { "Found" } else { "No" },
        ),
    };

    let usb_str = match usb.adb_installed {
        true => match usb.devices.as_slice() {
            [] => format!("{} Ready (No devices connected)", ui::badge_ok()),
            [only] => format!("{} Device connected ({only})", ui::badge_ok()),
            many => format!("{} Devices connected ({})", ui::badge_ok(), many.join(", ")),
        },
        false => format!(
            "{} ADB not installed (Wi-Fi streaming unaffected)",
            ui::badge_warn()
        ),
    };

    let rows = vec![
        ("Session / Desktop", session_str),
        ("Compositor", caps.compositor.to_string()),
        ("Capture Pipeline", plan_str),
        ("Virtual Display", display_str),
        ("Input Injection", uinput_str),
        ("Desktop Portal", portal_str),
        ("Saved Grants", grants_str),
        ("Compositor Tools", tools_str),
        ("USB ADB Tunnel", usb_str),
    ];

    ui::print_card("System Diagnostics & Compatibility", &rows);
    println!();
    ExitCode::SUCCESS
}

#[derive(Default)]
struct UsbDoctorReport {
    adb_installed: bool,
    devices: Vec<String>,
    reverse_tunnels: usize,
}

fn usb_doctor_report() -> UsbDoctorReport {
    let adb_path = orbiscreen_transport::adb::default_adb_path();
    let probe = tokio::task::block_in_place(|| {
        orbiscreen_transport::adb::setup_reverse_for_all(adb_path, cfg_default_signaling_port())
    });
    let devices = match probe {
        Ok(devices) => devices,
        Err(_) => return UsbDoctorReport::default(),
    };
    let mut tunnels = 0usize;
    for serial in &devices {
        if let Ok(n) = orbiscreen_transport::adb::reverse_tunnel_count(
            adb_path,
            serial,
            cfg_default_signaling_port(),
        ) {
            tunnels += n;
        }
    }
    UsbDoctorReport {
        adb_installed: true,
        devices,
        reverse_tunnels: tunnels,
    }
}

fn cfg_default_signaling_port() -> u16 {
    orbiscreen_core::TransportConfig::default().signaling_port
}

struct EvdiFixPlan {
    package_manager: &'static str,
    install_cmd: Vec<String>,
}

fn detect_evdi_fix_plan() -> Option<EvdiFixPlan> {
    let os_release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    evdi_fix_plan_for_os_release(&os_release)
}

fn evdi_fix_plan_for_os_release(os_release: &str) -> Option<EvdiFixPlan> {
    let os_release = os_release.to_ascii_lowercase();
    let id = os_release
        .lines()
        .find_map(|line| line.strip_prefix("id="))
        .unwrap_or("")
        .trim_matches('"');
    let id_like = os_release
        .lines()
        .find_map(|line| line.strip_prefix("id_like="))
        .unwrap_or("")
        .trim_matches('"');
    let tokens: Vec<&str> = std::iter::once(id)
        .chain(id_like.split_ascii_whitespace())
        .collect();

    if tokens
        .iter()
        .any(|t| matches!(*t, "fedora" | "rhel" | "centos"))
    {
        return Some(EvdiFixPlan {
            package_manager: "dnf",
            install_cmd: ["sudo", "dnf", "install", "-y", "evdi"]
                .into_iter()
                .map(String::from)
                .collect(),
        });
    }
    if tokens.iter().any(|t| matches!(*t, "opensuse" | "suse")) {
        return Some(EvdiFixPlan {
            package_manager: "zypper",
            install_cmd: ["sudo", "zypper", "install", "-y", "evdi"]
                .into_iter()
                .map(String::from)
                .collect(),
        });
    }
    if tokens
        .iter()
        .any(|t| matches!(*t, "arch" | "endeavouros" | "manjaro"))
    {
        return Some(EvdiFixPlan {
            package_manager: "pacman",
            install_cmd: ["sudo", "pacman", "-S", "--noconfirm", "evdi"]
                .into_iter()
                .map(String::from)
                .collect(),
        });
    }
    if tokens.iter().any(|t| {
        matches!(
            *t,
            "debian" | "ubuntu" | "linuxmint" | "pop" | "elementary" | "zorin" | "cosmic"
        )
    }) {
        return Some(EvdiFixPlan {
            package_manager: "apt",
            install_cmd: [
                "sudo",
                "apt-get",
                "install",
                "-y",
                "evdi-dkms",
                "dkms",
                "linux-headers-generic",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        });
    }
    None
}

fn confirm_with_user() -> bool {
    use std::io::{BufRead as _, Write as _};
    print!("proceed? [y/N] ");
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    if std::io::stdin().lock().read_line(&mut line).is_err() {
        return false;
    }
    matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

async fn run_doctor_fix(assume_yes: bool) -> ExitCode {
    let caps = Capabilities::from_env();
    let chain = caps.auto_chain();
    if caps.compositor == orbiscreen_capture::capabilities::Compositor::Kde
        || chain.iter().any(|s| matches!(s, CaptureStep::KwinVirtual))
    {
        println!("[doctor --fix] KWin Virtual display is native on this system; no kernel module required.");
        return ExitCode::SUCCESS;
    }
    if orbiscreen_capture::wlr_virtual_output::detect_ipc_kind().is_some()
        || chain
            .iter()
            .any(|s| matches!(s, CaptureStep::WlrootsVirtual))
    {
        println!("[doctor --fix] wlroots virtual display is native on this system; no kernel module required.");
        return ExitCode::SUCCESS;
    }
    let display_status = orbiscreen_display::probe();
    match display_status {
        DisplayStatus::Compatible => {
            println!("[doctor --fix] evdi is already available, nothing to do");
            return ExitCode::SUCCESS;
        }
        DisplayStatus::NoDeviceNode => {
            println!(
                "[doctor --fix] evdi kernel + library are OK; a device node appears once the \
                 daemon starts one (`orbiscreen start`). nothing to do"
            );
            return ExitCode::SUCCESS;
        }
        DisplayStatus::Outdated => {
            println!(
                "[doctor --fix] the loaded evdi module is older than libevdi requires; \
                 update/rebuild evdi (see docs/PACKAGING.md), then reboot"
            );
            return ExitCode::from(1);
        }
        DisplayStatus::KernelModuleMissing => {}
    }

    let Some(plan) = detect_evdi_fix_plan() else {
        println!(
            "[doctor --fix] could not detect a supported distribution (/etc/os-release); \
             build evdi from source instead:\n    bash scripts/install-evdi-module.sh"
        );
        return ExitCode::from(1);
    };
    println!(
        "[doctor --fix] evdi kernel module is missing; distro detected ({})",
        plan.package_manager
    );
    println!("[doctor --fix] will run: {}", plan.install_cmd.join(" "));
    if !assume_yes && !confirm_with_user() {
        println!("[doctor --fix] aborted by the user");
        return ExitCode::from(1);
    }

    let Some((program, args)) = plan.install_cmd.split_first() else {
        eprintln!("[doctor --fix] the detected fix plan has an empty install command");
        return ExitCode::from(1);
    };
    let program = program.to_string();
    let args = args.to_vec();
    let program_label = program.clone();
    let status = match tokio::task::spawn_blocking(move || {
        std::process::Command::new(&program).args(&args).status()
    })
    .await
    {
        Ok(Ok(status)) => status,
        Ok(Err(e)) => {
            eprintln!("[doctor --fix] failed to run {program_label}: {e}");
            return ExitCode::from(1);
        }
        Err(_) => {
            eprintln!("[doctor --fix] install task aborted");
            return ExitCode::from(1);
        }
    };
    if !status.success() {
        eprintln!(
            "[doctor --fix] {program_label} exited with {status}; the package may not exist \
             for this distro; try: bash scripts/install-evdi-module.sh"
        );
        return ExitCode::from(1);
    }

    let status = tokio::task::spawn_blocking(|| {
        std::process::Command::new("sudo")
            .args(["modprobe", "evdi"])
            .status()
    })
    .await;
    match status {
        Ok(Ok(s)) if s.success() => {}
        Ok(Ok(s)) => {
            eprintln!("[doctor --fix] modprobe evdi failed: {s}");
            return ExitCode::from(1);
        }
        Ok(Err(e)) => {
            eprintln!("[doctor --fix] failed to run modprobe: {e}");
            return ExitCode::from(1);
        }
        Err(_) => {
            eprintln!("[doctor --fix] modprobe task aborted");
            return ExitCode::from(1);
        }
    }

    match orbiscreen_display::probe() {
        DisplayStatus::Compatible | DisplayStatus::NoDeviceNode => {
            println!("[doctor --fix] evdi module loaded successfully");
            ExitCode::SUCCESS
        }
        other => {
            eprintln!(
                "[doctor --fix] module still not ready after install ({}); a reboot may be \
                 required",
                display_status_text(other)
            );
            ExitCode::from(1)
        }
    }
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

    let preferred = cfg.capture.preferred.as_str();
    let caps = Capabilities::from_env();
    let frame_pool = orbiscreen_core::frame_pool::FramePool::new();
    let mut source = resolve_frame_source(preferred, &caps, spec, &frame_pool).await?;

    let actual_dims = source.actual_dimensions();
    info!(
        stream_width = actual_dims.0,
        stream_height = actual_dims.1,
        "stream dimensions established from source"
    );

    let captured_output_name = match &source {
        FrameSource::WlrVirtual { output, .. } => Some(output.name().to_string()),
        FrameSource::Capture(c) => match c.backend() {
            CaptureBackend::KwinVirtual => Some("Virtual-ORBISCREEN".to_string()),
            _ => None,
        },
        _ => None,
    };
    let target_kwin_output = captured_output_name.clone();
    let (injector_tx, injector_rx) = tokio::sync::oneshot::channel::<InputInjector>();
    let input_spec = VirtualTouchscreenSpec {
        width: spec.width,
        height: spec.height,
        output_name: captured_output_name,
    };
    tokio::spawn(async move {
        match InputInjector::open_async(input_spec).await {
            Ok(inj) => {
                info!(backend = ?inj.backend(), "Input injector open");
                let _ = injector_tx.send(inj);
                if let Some(out_name) = target_kwin_output {
                    bind_kwin_virtual_inputs(&out_name).await;
                }
            }
            Err(e) => {
                warn!(
                    "input injection unavailable ({e}); streaming continues without remote control"
                );
            }
        }
    });

    let encoder_kind = match EncoderKind::parse(&cfg.encode.preferred_encoder) {
        Some(kind) => kind,
        None => EncoderKind::Auto,
    };
    let mut encoder = Encoder::new(EncodeParams {
        kind: encoder_kind,
        bitrate_kbps: cfg.encode.bitrate_kbps,
        width: actual_dims.0,
        height: actual_dims.1,
        framerate: spec.refresh_rate_hz,
    })?;
    let encoder_name = match encoder.kind() {
        EncoderKind::Auto => "auto",
        EncoderKind::Vaapi => "vaapi",
        EncoderKind::Nvenc => "nvenc",
        EncoderKind::X264 => "x264",
    };
    let mut encoded_rx = encoder.subscribe().ok_or("encoder returned no rx")?;
    let encoder = Arc::new(encoder);

    let stats = std::sync::Arc::new(Stats::default());

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
    let shutdown_keepalive = shutdown_tx.clone();
    let _shutdown_keepalive = shutdown_keepalive.clone();
    let backend_name = source.backend_name();
    let dbus_handles = std::sync::Arc::new(dbus::DaemonHandles {
        is_running: is_running.clone(),
        stats: stats.clone(),
        config: cfg.clone(),
        encoder: encoder_name,
        capture_backend: backend_name,
        shutdown_tx,
    });
    tokio::spawn(async move {
        if let Err(e) = dbus::run_dbus_server(dbus_handles).await {
            warn!("D-Bus session service init failed (is D-Bus running?): {e}");
        }
    });
    info!("D-Bus session service registered: com.orbiscreen.Daemon");

    let (video_tx, video_rx) = mpsc::channel::<H264Packet>(64);
    let encoder_dump = match std::env::var("ORBISCREEN_ENCODER_DUMP") {
        Ok(path) => match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            Ok(file) => Some(file),
            Err(e) => {
                warn!("ORBISCREEN_ENCODER_DUMP={path} could not be opened: {e}");
                None
            }
        },
        Err(_) => None,
    };
    let frame_pump = tokio::spawn(async move {
        let mut ts_base: Option<u64> = None;
        let mut dump_file = encoder_dump;
        while let Some(chunk) = encoded_rx.recv().await {
            let base = *ts_base.get_or_insert(chunk.pts_ns);
            let pts_ns = chunk.pts_ns.saturating_sub(base);
            let pkt = H264Packet {
                bytes: chunk.bytes,
                is_keyframe: chunk.is_keyframe,
                pts_ns,
            };
            if let Some(file) = dump_file.as_mut() {
                use std::io::Write;
                let _ = file.write_all(&pkt.bytes);
            }
            if video_tx.send(pkt).await.is_err() {
                break;
            }
        }
    });

    let cap_dims = actual_dims;
    let frame_count = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let fc = frame_count.clone();
    let encoder_for_pump = Arc::clone(&encoder);
    let cap_pump = tokio::spawn(async move {
        let encoder = encoder_for_pump;
        let frame_dur = Encoder::frame_duration_ns(spec.refresh_rate_hz);
        const KEEPALIVE: std::time::Duration = std::time::Duration::from_millis(100);
        let started = std::time::Instant::now();
        let mut last_pts_ns: u64 = frame_dur;
        let mut keepalive_frame: Option<(u32, u32, Vec<u8>)> = None;
        let mut last_snapshot: Option<std::time::Instant> = None;
        loop {
            let outcome = match tokio::time::timeout(KEEPALIVE, source.next_frame()).await {
                Ok(outcome) => outcome,
                Err(_elapsed) => {
                    let Some((width, height, data)) = &keepalive_frame else {
                        continue;
                    };
                    let now_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
                    last_pts_ns = now_ns.max(last_pts_ns.saturating_add(frame_dur));
                    let pts_ns = last_pts_ns;
                    if let Err(e) = encoder.push_frame(data, *width, *height, pts_ns) {
                        match e {
                            orbiscreen_encode::EncodeError::Flushing
                            | orbiscreen_encode::EncodeError::Eos => {
                                debug!(
                                    "keepalive frame push ignored during pipeline shutdown ({e})"
                                );
                                break;
                            }
                            _ => {
                                warn!(
                                    "keepalive frame push rejected ({}x{}, {} B): {e}",
                                    width,
                                    height,
                                    data.len()
                                );
                            }
                        }
                    }
                    continue;
                }
            };
            match outcome {
                SourceOutcome::Frame(frame) => {
                    let _ = fc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let (width, height) = (frame.width, frame.height);
                    if last_snapshot.map_or(true, |t| t.elapsed() >= KEEPALIVE) {
                        keepalive_frame = Some((width, height, frame.data.to_vec()));
                        last_snapshot = Some(std::time::Instant::now());
                    }
                    let now_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
                    last_pts_ns = now_ns.max(last_pts_ns.saturating_add(frame_dur));
                    let pts_ns = last_pts_ns;
                    let data_len = frame.data.len();
                    if let Err(e) = encoder.push_frame_owned(frame.data, width, height, pts_ns) {
                        match e {
                            orbiscreen_encode::EncodeError::Flushing
                            | orbiscreen_encode::EncodeError::Eos => {
                                debug!("frame push ignored during pipeline shutdown ({e})");
                                break;
                            }
                            _ => {
                                warn!(
                                    "frame push rejected ({}x{}, {} B): {e}",
                                    width, height, data_len
                                );
                            }
                        }
                    }
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
    let watchdog = tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let n = enc_check.load(std::sync::atomic::Ordering::Relaxed);
            if n == 0 {
                warn!(
                    "no frames captured yet - compositor may not be drawing on the virtual \
                     display (evdi) or the portal is not delivering buffers"
                );
            } else {
                break;
            }
        }
    });

    let (input_tx, mut input_rx) = mpsc::channel::<orbiscreen_transport::IncomingInput>(1024);
    let input_pump = tokio::spawn(async move {
        use orbiscreen_input::PointerEvent;
        use orbiscreen_transport::IncomingInput;
        let (cap_w, cap_h) = cap_dims;
        let scale = |x: f64, y: f64| {
            let x = x * f64::from(spec.width) / f64::from(cap_w.max(1));
            let y = y * f64::from(spec.height) / f64::from(cap_h.max(1));
            (x, y)
        };
        let mut injector: Option<InputInjector> = None;
        let mut pending_rx = Some(injector_rx);
        let mut warned_no_injector = false;
        while let Some(event) = input_rx.recv().await {
            if injector.is_none() {
                if let Some(rx) = pending_rx.as_mut() {
                    match rx.try_recv() {
                        Ok(inj) => {
                            info!("input injector is now active");
                            injector = Some(inj);
                            pending_rx = None;
                            warned_no_injector = false;
                        }
                        Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                            pending_rx = None;
                        }
                        Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
                    }
                }
            }
            let Some(inj) = injector.as_mut() else {
                if !warned_no_injector {
                    warn!("input event received but no injector is available yet; dropping events until the portal responds");
                    warned_no_injector = true;
                }
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
                    let _ = inj.inject_pointer(p).await;
                }
                IncomingInput::Key(k) => {
                    let _ = inj.inject_key(k).await;
                }
                IncomingInput::Stylus(s) => {
                    let s = match s {
                        orbiscreen_input::StylusEvent::Pressure { x, y, pressure } => {
                            let (x, y) = scale(x, y);
                            orbiscreen_input::StylusEvent::Pressure { x, y, pressure }
                        }
                        orbiscreen_input::StylusEvent::Tilt {
                            x,
                            y,
                            pressure,
                            tilt_x_deg,
                            tilt_y_deg,
                        } => {
                            let (x, y) = scale(x, y);
                            orbiscreen_input::StylusEvent::Tilt {
                                x,
                                y,
                                pressure,
                                tilt_x_deg,
                                tilt_y_deg,
                            }
                        }
                        other => other,
                    };
                    let _ = inj.inject_stylus(s).await;
                }
                IncomingInput::RawPointer { x, y } => {
                    let (x, y) = scale(x, y);
                    let _ = inj.inject_pointer(PointerEvent::Move { x, y }).await;
                }
            }
        }
    });

    let client_dir = std::env::var_os("ORBISCREEN_CLIENT_DIR")
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .or_else(|| {
            if std::env::var_os("ORBISCREEN_CLIENT_DIR").is_some() {
                warn!("ORBISCREEN_CLIENT_DIR does not exist; falling back to defaults");
            }
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
    let token_path = orbiscreen_core::default_token_path();
    let saved_token = std::fs::read_to_string(&token_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| s.len() >= 32);
    let token_to_use = saved_token.unwrap_or_else(|| {
        let t = orbiscreen_transport::generate_token();
        if let Some(parent) = token_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&token_path, &t);
        t
    });

    let transport = Transport::with_token(
        ServerConfig {
            signaling_port: cfg.transport.signaling_port,
            client_web_dir: client_dir,
        },
        input_tx,
        Some(token_to_use),
    );
    let token = transport.token().to_owned();
    info!("stream access token active ({len} chars, prefix={prefix}); clients fetch it via mDNS TXT or /client/config.json",
        len = token.len(), prefix = token.get(..4).unwrap_or(""));

    let _mdns = if !no_mdns && cfg.transport.mdns_advertise {
        match orbiscreen_transport::mdns::Advertiser::register(
            &orbiscreen_transport::ServiceDescriptor {
                instance: hostname::get()
                    .ok()
                    .and_then(|h| h.into_string().ok())
                    .unwrap_or_else(|| "orbiscreen-host".into()),
                port: cfg.transport.signaling_port,
                token: Some(token.clone()),
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

    ui::print_banner();
    let display_info = format!(
        "{}x{} @ {}Hz",
        actual_dims.0, actual_dims.1, spec.refresh_rate_hz
    );
    ui::print_startup_card(
        &display_info,
        encoder_name,
        backend_name,
        cfg.transport.signaling_port,
        &token,
        true,
    );

    let mut serve_fut = std::pin::pin!(transport.serve(
        video_rx,
        stats,
        actual_dims.0,
        actual_dims.1,
        spec.refresh_rate_hz,
        encoder_name,
        shutdown_rx.clone(),
    ));

    tokio::select! {
        res = &mut serve_fut => {
            res.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received SIGINT (Ctrl-C), initiating graceful shutdown...");
            _ = shutdown_keepalive.send(true);
            let _ = (&mut serve_fut).await;
        }
        _ = shutdown_rx.changed() => {
            info!("D-Bus Stop received, initiating graceful shutdown...");
            _ = shutdown_keepalive.send(true);
            let _ = (&mut serve_fut).await;
        }
    }
    is_running.store(false, std::sync::atomic::Ordering::SeqCst);
    encoder.stop();
    cap_pump.abort();
    frame_pump.abort();
    input_pump.abort();
    watchdog.abort();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_loads_when_file_absent() {
        let cfg =
            load_or_default_config(Path::new("/tmp/orbiscreen-nonexistent-config.toml")).unwrap();
        assert_eq!(cfg.display.width, 1920);
    }

    fn plan_pm(os_release: &str) -> Option<&'static str> {
        evdi_fix_plan_for_os_release(os_release).map(|p| p.package_manager)
    }

    #[test]
    fn fedora_uses_dnf() {
        let os = "NAME=\"Fedora Linux\"\nID=fedora\nVERSION_ID=\"41\"\n";
        assert_eq!(plan_pm(os), Some("dnf"));
    }

    #[test]
    fn rhel_derivative_uses_dnf_via_id_like() {
        let os = "NAME=\"Rocky Linux\"\nID=\"rocky\"\nID_LIKE=\"rhel centos fedora\"\n";
        assert_eq!(plan_pm(os), Some("dnf"));
    }

    #[test]
    fn ubuntu_uses_apt() {
        let os = "NAME=\"Ubuntu\"\nID=ubuntu\nID_LIKE=debian\n";
        assert_eq!(plan_pm(os), Some("apt"));
    }

    #[test]
    fn debian_derivative_uses_apt_via_id_like() {
        let os = "NAME=\"Linux Mint\"\nID=linuxmint\nID_LIKE=\"ubuntu debian\"\n";
        assert_eq!(plan_pm(os), Some("apt"));
    }

    #[test]
    fn arch_uses_pacman() {
        let os = "NAME=\"Arch Linux\"\nID=arch\n";
        assert_eq!(plan_pm(os), Some("pacman"));
    }

    #[test]
    fn opensuse_uses_zypper() {
        let os =
            "NAME=\"openSUSE Tumbleweed\"\nID=\"opensuse-tumbleweed\"\nID_LIKE=\"opensuse suse\"\n";
        assert_eq!(plan_pm(os), Some("zypper"));
    }

    #[test]
    fn unknown_distro_has_no_plan() {
        let os = "NAME=\"Gentoo\"\nID=gentoo\n";
        assert_eq!(plan_pm(os), None);
    }

    #[test]
    fn empty_os_release_has_no_plan() {
        assert_eq!(plan_pm(""), None);
    }

    #[test]
    fn fedora_plan_installs_evdi_package() {
        let plan = evdi_fix_plan_for_os_release("ID=fedora\n").expect("fedora plan");
        assert!(plan.install_cmd.contains(&"evdi".to_string()));
        assert!(plan.install_cmd.iter().any(|c| c == "sudo"));
    }

    #[test]
    fn apt_plan_installs_evdi_dkms_with_headers() {
        let plan = evdi_fix_plan_for_os_release("ID=ubuntu\n").expect("apt plan");
        assert!(plan.install_cmd.contains(&"evdi-dkms".to_string()));
        assert!(plan
            .install_cmd
            .contains(&"linux-headers-generic".to_string()));
    }
}
