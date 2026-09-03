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
    pub const MAUVE: &str = "\x1b[38;2;203;166;247m";
    pub const GREEN: &str = "\x1b[38;2;166;227;161m";
    pub const YELLOW: &str = "\x1b[38;2;249;226;175m";
    pub const PEACH: &str = "\x1b[38;2;250;179;135m";
    pub const RED: &str = "\x1b[38;2;243;139;168m";
    pub const TEXT: &str = "\x1b[38;2;205;214;244m";
    pub const SUBTEXT: &str = "\x1b[38;2;166;173;200m";
    pub const SURFACE: &str = "\x1b[38;2;49;50;68m";
    pub const CRUST: &str = "\x1b[38;2;17;17;27m";

    pub const BG_BLUE: &str = "\x1b[48;2;137;180;250m";
    pub const BG_GREEN: &str = "\x1b[48;2;166;227;161m";
    pub const BG_RED: &str = "\x1b[48;2;243;139;168m";
    pub const BG_SURFACE: &str = "\x1b[48;2;49;50;68m";
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
        println!(
            r#"{BLUE}       ▄▄▄▄▄▄▄       
    ▄██▀▀     ▀▀██▄    {BOLD}ORBI{SAPPHIRE}SCREEN{RESET} {SUBTEXT}v{version}{RESET}
{BLUE}  ▄██             ██▄  {TEXT}Turn any Android device into a second monitor for Linux{RESET}
{BLUE} ▐█▌               ▐█▌ {SUBTEXT}Pure CLI Daemon · Zero Root on KDE · Hardware Accelerated{RESET}
{BLUE} ▐█▌               ▐█▌ 
  ▀██             ██▀  {MAUVE}Wayland & X11{SUBTEXT} · {SAPPHIRE}NVENC/VA-API{SUBTEXT} · {GREEN}Stylus & Touch{SUBTEXT} · {PEACH}USB ADB{RESET}
{BLUE}    ▀██▄▄     ▄▄██▀  {SAPPHIRE}▄▄{RESET}
{BLUE}       ▀▀▀▀▀▀▀      {SAPPHIRE}████{RESET}
{SAPPHIRE}                     ▀▀{RESET}"#
        );
    } else {
        println!(
            r#"       ▄▄▄▄▄▄▄       
    ▄██▀▀     ▀▀██▄    ORBISCREEN v{version}
  ▄██             ██▄  Turn any Android device into a second monitor for Linux
 ▐█▌               ▐█▌ Pure CLI Daemon · Zero Root on KDE · Hardware Accelerated
 ▐█▌               ▐█▌ 
  ▀██             ██▀  Wayland & X11 · NVENC/VA-API · Stylus & Touch · USB ADB
    ▀██▄▄     ▄▄██▀  ▄▄
       ▀▀▀▀▀▀▀      ████
                     ▀▀"#
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

pub fn print_card(title: &str, rows: &[(&str, String)]) {
    if colors_enabled() {
        use colors::*;
        println!(
            "{SAPPHIRE}╭─{RESET} {BOLD}{BLUE}{title}{RESET} {SAPPHIRE}{line}╮{RESET}",
            line = "─".repeat(54usize.saturating_sub(title.len()))
        );
        for (k, v) in rows {
            println!(
                "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}{k:<18}{RESET} {TEXT}{v}{RESET}",
                k = k,
                v = v
            );
        }
        println!("{SAPPHIRE}╰{line}╯{RESET}", line = "─".repeat(56));
    } else {
        println!(
            "+-- {title} {}",
            "-".repeat(54usize.saturating_sub(title.len()))
        );
        for (k, v) in rows {
            println!("|  {k:<18} {v}");
        }
        println!("+{}", "-".repeat(56));
    }
}

pub fn print_startup_card(
    display_info: &str,
    encoder_name: &str,
    backend_name: &str,
    port: u16,
    token: &str,
    usb_active: bool,
    mdns_active: bool,
) {
    let local_url = format!("http://localhost:{port}/?token={token}");
    let lan_ip = get_lan_ip().unwrap_or_else(|| "YOUR-IP".to_string());
    let lan_url = format!("http://{lan_ip}:{port}/?token={token}");

    if colors_enabled() {
        use colors::*;
        println!();
        println!("{GREEN}●{RESET} {BOLD}{GREEN}Orbiscreen Daemon is ACTIVE and streaming{RESET}");
        println!();
        println!("{SAPPHIRE}╭── Stream Access & Clients ──────────────────────────────────────────╮{RESET}");
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{PEACH}🌐 Web Client (Local){RESET}   {UNDERLINE}{TEXT}{local_url}{RESET}"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{PEACH}🌐 Web Client (LAN){RESET}     {UNDERLINE}{TEXT}{lan_url}{RESET}"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{MAUVE}📱 Android App{RESET}          {TEXT}Ready (Auto-discovery via mDNS or USB ADB){RESET}"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{BLUE}🔌 USB ADB Tunnel{RESET}       {TEXT}{}{RESET}",
            if usb_active {
                "Reverse tunnel active on port 8788"
            } else {
                "Waiting for device (hot-plug ready)"
            }
        );
        println!(
            "{SAPPHIRE}│{RESET}  {BOLD}{LAVENDER}📡 Discovery (mDNS){RESET}     {TEXT}{}{RESET}",
            if mdns_active {
                "Broadcasting Orbiscreen service"
            } else {
                "Disabled"
            }
        );
        println!("{SAPPHIRE}├─────────────────────────────────────────────────────────────────────┤{RESET}");
        println!(
            "{SAPPHIRE}│{RESET}  {DIM}Display:{RESET} {TEXT}{display_info:<22}{RESET} {DIM}Encoder:{RESET} {TEXT}{encoder_name:<10}{RESET} {DIM}Capture:{RESET} {TEXT}{backend_name}{RESET}"
        );
        println!(
            "{SAPPHIRE}│{RESET}  {DIM}Token:{RESET}   {TEXT}{token:<30}{RESET} {DIM}D-Bus:{RESET}   {TEXT}com.orbiscreen.Daemon{RESET}"
        );
        println!("{SAPPHIRE}╰─────────────────────────────────────────────────────────────────────╯{RESET}");
        println!(
            "{DIM}Press {BOLD}Ctrl+C{RESET}{DIM} to stop, or run {BOLD}orbiscreen stop{RESET}{DIM} from another terminal.{RESET}\n"
        );
    } else {
        println!("\n[Orbiscreen Daemon is ACTIVE and streaming]\n");
        println!("+-- Stream Access & Clients ------------------------------------------+");
        println!("|  Web Client (Local)   {local_url}");
        println!("|  Web Client (LAN)     {lan_url}");
        println!("|  Android App          Ready (Auto-discovery via mDNS or USB ADB)");
        println!(
            "|  USB ADB Tunnel       {}",
            if usb_active {
                "Active on port 8788"
            } else {
                "Hot-plug ready"
            }
        );
        println!(
            "|  Discovery (mDNS)     {}",
            if mdns_active {
                "Broadcasting"
            } else {
                "Disabled"
            }
        );
        println!("+---------------------------------------------------------------------+");
        println!(
            "|  Display: {display_info:<22} Encoder: {encoder_name:<10} Capture: {backend_name}"
        );
        println!("|  Token:   {token:<30} D-Bus:   com.orbiscreen.Daemon");
        println!("+---------------------------------------------------------------------+");
        println!("Press Ctrl+C to stop, or run 'orbiscreen stop' from another terminal.\n");
    }
}

fn get_lan_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}
