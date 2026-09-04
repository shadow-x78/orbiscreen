// Orbiscreen - ui.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::io::{stdout, IsTerminal};

pub fn colors_enabled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }
    stdout().is_terminal()
}

pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const UNDERLINE: &str = "\x1b[4m";

    pub const BLUE: &str = "\x1b[38;2;137;180;250m";
    pub const SAPPHIRE: &str = "\x1b[38;2;116;199;236m";
    pub const LAVENDER: &str = "\x1b[38;2;180;190;254m";
    pub const GREEN: &str = "\x1b[38;2;166;227;161m";
    pub const YELLOW: &str = "\x1b[38;2;249;226;175m";
    pub const PEACH: &str = "\x1b[38;2;250;179;135m";
    pub const RED: &str = "\x1b[38;2;243;139;168m";
    pub const TEXT: &str = "\x1b[38;2;205;214;244m";
    pub const SUBTEXT: &str = "\x1b[38;2;166;173;200m";
}

pub fn clap_styles() -> clap::builder::styling::Styles {
    use clap::builder::styling::{AnsiColor, Color, Style, Styles};

    Styles::styled()
        .header(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Blue))),
        )
        .usage(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
        )
        .literal(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Green))),
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))))
        .valid(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Magenta))))
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red))),
        )
}

pub fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    if colors_enabled() {
        use colors::*;
        println!();
        println!(
            r#"    {BLUE}▄▄████▄▄{RESET}
  {BLUE}▄██▀    ▀██▄{RESET}
  {BLUE}██        ██{RESET}    {BOLD}{BLUE}Orbiscreen{RESET}  {LAVENDER}v{version}{RESET}  {DIM}(by shadow-x78){RESET}
  {BLUE}██        ██{RESET}    {SUBTEXT}Virtual secondary display for Linux{RESET}
  {BLUE}▀██▄    {SAPPHIRE}▄███{RESET}
    {BLUE}▀▀████{SAPPHIRE}██▀{RESET}"#
        );
    } else {
        println!();
        println!(
            r#"    ▄▄████▄▄
  ▄██▀    ▀██▄
  ██        ██    Orbiscreen  v{version}  (by shadow-x78)
  ██        ██    Virtual secondary display for Linux
  ▀██▄    ▄███
    ▀▀██████▀"#
        );
    }
}

pub fn badge_ok() -> &'static str {
    if colors_enabled() {
        "\x1b[38;2;166;227;161m✔\x1b[0m"
    } else {
        "[OK]"
    }
}

pub fn badge_warn() -> &'static str {
    if colors_enabled() {
        "\x1b[38;2;249;226;175m⚠\x1b[0m"
    } else {
        "[WARN]"
    }
}

pub fn badge_err() -> &'static str {
    if colors_enabled() {
        "\x1b[38;2;243;139;168m✖\x1b[0m"
    } else {
        "[FAIL]"
    }
}

pub fn badge_info() -> &'static str {
    if colors_enabled() {
        "\x1b[38;2;137;180;250mℹ\x1b[0m"
    } else {
        "[INFO]"
    }
}

pub fn print_welcome_card() {
    if colors_enabled() {
        use colors::*;
        println!();
        println!("{RED}●{RESET} {BOLD}{RED}Daemon is stopped{RESET}");
        println!();
        println!("{SAPPHIRE}╭── Quick Start ────────────────────────────────────────────────────────{RESET}");
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{GREEN}orbiscreen start{RESET}         {DIM}Start streaming in foreground{RESET}"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{BLUE}orbiscreen start -d{RESET}      {DIM}Start in background as a service{RESET}"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{PEACH}orbiscreen doctor{RESET}        {DIM}Check system & auto-install drivers{RESET}"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}orbiscreen --help{RESET}        {DIM}Show all commands & options{RESET}"
        );
        println!("{SAPPHIRE}╰───────────────────────────────────────────────────────────────────────{RESET}");
        println!();
    } else {
        println!("\n[Daemon is stopped]\n");
        println!("+-- Quick Start --------------------------------------------------------");
        println!("|  orbiscreen start         Start streaming in foreground");
        println!("|  orbiscreen start -d      Start in background as a service");
        println!("|  orbiscreen doctor        Check system & auto-install drivers");
        println!("|  orbiscreen --help        Show all commands & options");
        println!("+-----------------------------------------------------------------------\n");
    }
}

pub fn print_card(title: &str, rows: &[(&str, String)]) {
    let dashes_len = 72_usize.saturating_sub(5 + title.chars().count());
    if colors_enabled() {
        use colors::*;
        let dashes = "─".repeat(dashes_len);
        println!("{SAPPHIRE}╭── {BOLD}{BLUE}{title}{RESET} {SAPPHIRE}{dashes}{RESET}");
        for (k, v) in rows {
            println!("{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}{k:<18}{RESET} {TEXT}{v}{RESET}");
        }
        println!("{SAPPHIRE}╰───────────────────────────────────────────────────────────────────────{RESET}");
    } else {
        let dashes = "-".repeat(dashes_len);
        println!("+-- {title} {dashes}");
        for (k, v) in rows {
            println!("|  {k:<18} {v}");
        }
        println!("+-----------------------------------------------------------------------");
    }
}

pub fn format_encoder_name(encoder: &str) -> &'static str {
    match encoder.to_lowercase().as_str() {
        "nvenc" => "NVENC",
        "vaapi" => "VA-API",
        "software" | "x264" => "Software",
        _ => "Hardware",
    }
}

pub fn format_backend_name(backend: &str) -> &'static str {
    match backend.to_lowercase().as_str() {
        "kwin-virtual" | "kwin" => "KWin Virtual",
        "wlr-virtual" | "wlroots" => "wlroots",
        "x11" => "X11",
        "portal" => "XDG Portal",
        _ => "Virtual",
    }
}

pub fn print_startup_card(
    display_info: &str,
    encoder_name: &str,
    backend_name: &str,
    port: u16,
    token: &str,
    usb_active: bool,
) {
    let lan_ip = get_lan_ip().unwrap_or_else(|| "localhost".to_string());
    let lan_url = format!("http://{lan_ip}:{port}/?token={token}");
    let enc = format_encoder_name(encoder_name);
    let back = format_backend_name(backend_name);

    if colors_enabled() {
        use colors::*;
        println!();
        println!("{GREEN}●{RESET} {BOLD}{GREEN}Orbiscreen is streaming{RESET}");
        println!();
        println!("{SAPPHIRE}╭── Stream Details ─────────────────────────────────────────────────────{RESET}");
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}Display{RESET}   {TEXT}{display_info}{RESET}  {DIM}· {enc} · {back}{RESET}"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}USB ADB{RESET}   {TEXT}{}{RESET}",
            if usb_active {
                format!("{GREEN}✔ Active on port {port}{RESET}")
            } else {
                format!(
                    "{YELLOW}Hot-plug ready{RESET}  {DIM}(Plug in Android with USB Debugging){RESET}"
                )
            }
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}Stream{RESET}    {UNDERLINE}{PEACH}{lan_url}{RESET}"
        );
        println!("{SAPPHIRE}╰───────────────────────────────────────────────────────────────────────{RESET}");
        println!(
            "{DIM}Press {BOLD}Ctrl+C{RESET}{DIM} to stop, or run {BOLD}orbiscreen stop{RESET}{DIM} from another terminal.{RESET}\n"
        );
    } else {
        println!("\n[Orbiscreen is streaming]\n");
        println!("+-- Stream Details -----------------------------------------------------");
        println!("|  Display   {display_info} · {enc} · {back}");
        println!(
            "|  USB ADB   {}",
            if usb_active {
                "Active"
            } else {
                "Hot-plug ready"
            }
        );
        println!("|  Stream    {lan_url}");
        println!("+-----------------------------------------------------------------------");
        println!("Press Ctrl+C to stop, or run 'orbiscreen stop'.\n");
    }
}

#[allow(clippy::too_many_arguments)]
pub fn print_status_dashboard(
    is_running: bool,
    w: u64,
    h: u64,
    fps: u64,
    encoder: &str,
    capture: &str,
    port: u16,
    active_clients: u64,
    usb_count: u64,
    token: &str,
) {
    let lan_ip = get_lan_ip().unwrap_or_else(|| "localhost".to_string());
    let stream_url = format!("http://{lan_ip}:{port}/?token={token}");
    let enc = format_encoder_name(encoder);
    let back = format_backend_name(capture);

    if colors_enabled() {
        use colors::*;
        println!();
        if is_running {
            println!("{GREEN}●{RESET} {BOLD}{GREEN}Daemon is active & streaming{RESET}");
        } else {
            println!("{RED}●{RESET} {BOLD}{RED}Daemon is stopped{RESET}");
        }
        println!();

        let dev_str = if usb_count > 0 {
            format!("{GREEN}✔ {usb_count} Android device connected{RESET}  {DIM}(USB ADB){RESET}")
        } else if active_clients > 0 {
            format!("{GREEN}✔ {active_clients} client(s) streaming{RESET}")
        } else {
            format!(
                "{YELLOW}Waiting for device{RESET}  {DIM}(Connect Android via USB or Wi-Fi){RESET}"
            )
        };

        println!("{SAPPHIRE}╭── Status ─────────────────────────────────────────────────────────────{RESET}");
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}Display{RESET}   {TEXT}{w}x{h} @ {fps}Hz{RESET}  {DIM}· {enc} · {back}{RESET}"
        );
        println!("{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}Devices{RESET}   {dev_str}");
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}Stream{RESET}    {UNDERLINE}{PEACH}{stream_url}{RESET}"
        );
        println!("{SAPPHIRE}╰───────────────────────────────────────────────────────────────────────{RESET}");
        println!();
    } else {
        println!(
            "\n[Daemon is {}]\n",
            if is_running {
                "active & streaming"
            } else {
                "stopped"
            }
        );
        println!("+-- Status -------------------------------------------------------------");
        println!("|  Display   {w}x{h} @ {fps}Hz  · {enc} · {back}");
        println!(
            "|  Devices   {} device(s) connected",
            usb_count + active_clients
        );
        println!("|  Stream    {stream_url}");
        println!("+-----------------------------------------------------------------------\n");
    }
}

pub fn get_lan_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

pub fn print_version_card(json: bool) {
    let version = env!("CARGO_PKG_VERSION");
    let target = format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH);
    let caps = orbiscreen_capture::capabilities::Capabilities::from_env();
    let desktop_str = match (caps.compositor, caps.session) {
        (
            orbiscreen_capture::capabilities::Compositor::Unknown,
            orbiscreen_capture::capabilities::SessionType::Unknown,
        ) => "Linux Session".to_string(),
        (c, s) => format!("{c} ({s})"),
    };

    if json {
        println!(
            "{}",
            serde_json::json!({
                "name": "orbiscreen",
                "version": version,
                "developer": "shadow-x78",
                "repository": "https://github.com/shadow-x78/orbiscreen",
                "license": "GPL-3.0-or-later",
                "target": target,
                "compositor": caps.compositor.to_string(),
                "session": caps.session.to_string(),
                "dbus_service": "org.shadow_x7.Orbiscreen",
            })
        );
        return;
    }

    print_banner();
    println!();

    if colors_enabled() {
        use colors::*;
        println!("{SAPPHIRE}╭── Developer & System Details ─────────────────────────────────────────{RESET}");
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}{:<16}{RESET} {BOLD}{GREEN}shadow-x78{RESET}  {DIM}(https://github.com/shadow-x78){RESET}",
            "Developer"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}{:<16}{RESET} {UNDERLINE}{PEACH}https://github.com/shadow-x78/orbiscreen{RESET}",
            "Repository"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}{:<16}{RESET} {TEXT}v{version}{RESET}  {DIM}· {target} · GPL-3.0-or-later{RESET}",
            "Version"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}{:<16}{RESET} {TEXT}{desktop_str}{RESET}",
            "Environment"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}{:<16}{RESET} {TEXT}Virtual Display · Hardware Encoding · Stylus & Touch · USB ADB{RESET}",
            "Features"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}{:<16}{RESET} {TEXT}org.shadow_x7.Orbiscreen{RESET}  {DIM}(Session Bus){RESET}",
            "D-Bus Service"
        );
        println!("{SAPPHIRE}╰───────────────────────────────────────────────────────────────────────{RESET}");
        println!();
    } else {
        println!("+-- Developer & System Details -----------------------------------------");
        println!("|  Developer        shadow-x78 (https://github.com/shadow-x78)");
        println!("|  Repository       https://github.com/shadow-x78/orbiscreen");
        println!("|  Version          v{version} · {target} · GPL-3.0-or-later");
        println!("|  Environment      {desktop_str}");
        println!(
            "|  Features         Virtual Display · Hardware Encoding · Stylus & Touch · USB ADB"
        );
        println!("|  D-Bus Service    org.shadow_x7.Orbiscreen (Session Bus)");
        println!("+-----------------------------------------------------------------------\n");
    }
}
