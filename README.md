<div align="center">

<a href="https://github.com/shadow-x78/orbiscreen">
  <img src="https://raw.githubusercontent.com/shadow-x78/orbiscreen/main/assets/logo/orbiscreen-banner.png" alt="Orbiscreen banner" width="100%" />
</a>

# Orbiscreen

Secondary monitor for Linux, powered by an Android tablet or phone.

[![Version](https://img.shields.io/badge/version-0.33.2-2563eb?style=for-the-badge)](CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=for-the-badge)](LICENSE)
![Rust](https://img.shields.io/badge/rust-1.92%2B-16a34a?style=for-the-badge)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=for-the-badge)

</div>

---

## Language

<a href="README.md">🇬🇧 English</a> · <a href="README_AR.md">🇸🇦 العربية</a>

---

## Table of Contents

- [Overview](#overview)
- [Comparison](#comparison)
- [Use cases](#use-cases)
- [Features](#features)
- [Desktop support](#desktop-support)
- [Installation](#installation)
- [Usage](#usage)
- [Architecture](#architecture)
- [Project structure](#project-structure)
- [FAQ](#faq)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

---

<a id="overview"></a>
## Overview

Orbiscreen turns a spare Android tablet or phone into a secondary display for a Linux desktop. The host creates a virtual monitor, either as a kernel display through DisplayLink's `evdi` (COSMIC, GNOME, X11) or as a compositor-native output on KDE Plasma and wlroots compositors, with no root and no share dialog. The display is encoded as H.264 and streamed to Android or to a web browser, with reverse control from touch, mouse, keyboard, and a pressure-sensitive stylus.

<a id="comparison"></a>
## Comparison

| Capability | Spacedesk | Deskreen | Weylus | Sidecar | **Orbiscreen** |
| :--- | :---: | :---: | :---: | :---: | :---: |
| Linux host | ❌ Windows only | ✅ Web based | ✅ Web based | ❌ macOS only | ✅ |
| Wayland and X11 | ❌ | ⚠️ dummy plug | ⚠️ mirror only | ❌ | ✅ native |
| Extended display | ✅ Windows only | ❌ HDMI dummy | ❌ mirror only | ✅ Apple only | ✅ virtual monitor |
| Native Android client | ✅ | ❌ browser | ❌ browser | ❌ iPad | ✅ Compose |
| Hardware encoding | ✅ | ❌ | ⚠️ | ✅ | ✅ NVENC, VA-API |
| Measured latency | ~50-80 ms | ~150-300 ms | ~80-120 ms | ~30 ms | ~25-40 ms |
| Stylus pressure and tilt | ❌ | ❌ | ⚠️ pen only | ✅ | ✅ 4095 levels |
| Reverse touch and mouse | ✅ | ❌ | ⚠️ pen only | ✅ | ✅ multi-touch |
| Rootless on KDE, wlroots | N/A | ✅ | ❌ | N/A | ✅ |
| Open source | ❌ | ✅ GPL-3.0 | ✅ AGPL-3.0 | ❌ | ✅ GPL-3.0 |

<a id="use-cases"></a>
## Use cases

- Second screen for an Android tablet (Samsung Galaxy Tab, Xiaomi Pad, Lenovo Tab) without buying a portable monitor.
- Drawing tablet with pressure and tilt in Krita, GIMP, Blender, and Inkscape.
- Portrait monitor, rotated with the device, for terminals, documentation, and chat.
- Wired operation: USB carries the stream over Android Open Accessory, with no Wi-Fi and no `adb reverse`.

<a id="features"></a>
## Features

- Virtual display creation through the XDG Desktop Portal on GNOME 46+ and KDE Plasma 6+, through compositor IPC on wlroots, and through EVDI on X11 and COSMIC
- Stylus support: 4095 pressure levels, tilt, and in-air hover, exposed as a Linux input tablet through `uinput`
- Touchpad mode with double-tap drag, constrained to the virtual display bounds
- Low-latency encoding: H.264 via NVENC, VA-API, or x264, with GOP and rate control tuned for LAN play
- Transports: UDP Annex-B with Reed-Solomon FEC, WebTransport for browsers, USB AOA bulk, and HTTP MPEG-TS as fallback
- Discovery with mDNS, manual entry, and an optional subnet scanner
- Web client: no install, runs in Chromium browsers over WebTransport and WebCodecs
- Material 3 Android client with light and dark themes
- Host control over an authenticated API: lock, blank, unblank, Ctrl+Alt+Del
- D-Bus service and CLI (`orbiscreen start`, `stop`, `doctor`) with a systemd user unit
- Cryptographic signing of Linux and Android release artifacts

<a id="desktop-support"></a>
## Desktop support

| Environment | Virtual second display | Capture | Input |
|-------------|------------------------|---------|-------|
| KDE Plasma (Wayland) | ✅ native, Portal Virtual or zkde-screencast, no root | ✅ PipeWire | ✅ RemoteDesktop portal, uinput |
| COSMIC (Wayland) | ⚠️ via EVDI (`doctor --fix`) | ✅ Portal ScreenCast (PipeWire) | ✅ uinput, RemoteDesktop |
| Sway / Hyprland / wlroots | ✅ headless output via compositor IPC, no root | ✅ wlr-screencopy | ✅ virtual-pointer, virtual-keyboard |
| GNOME (Wayland) | ✅ Portal Virtual (GNOME 46+) or EVDI | ✅ Portal ScreenCast (PipeWire) | ✅ RemoteDesktop portal, persisted |
| XFCE / MATE / LXQt / Cinnamon (X11) | ✅ via EVDI | ✅ XShm root mirror | ✅ XTEST, uinput |
| Anything else | ✅ via EVDI | best available | best available |

`orbiscreen doctor` prints the detected compositor, the capture plan, and anything missing. `orbiscreen doctor --fix` installs the EVDI kernel module on detected distributions. Details: [Desktop Environment Support](docs/DE_SUPPORT.md).

<a id="installation"></a>
## Installation

**Ubuntu, Pop!_OS, Linux Mint (PPA):**

```bash
sudo add-apt-repository ppa:shadow-x78/ppa -y
sudo apt update && sudo apt install orbiscreen -y
```

**Fedora (COPR):**

```bash
sudo dnf copr enable shadow-x78/orbiscreen -y
sudo dnf install orbiscreen -y
```

**Arch Linux, Manjaro (PKGBUILD):**

```bash
git clone https://github.com/shadow-x78/orbiscreen.git
cd orbiscreen
makepkg -si
```

**AppImage:** download `orbiscreen-x86_64.AppImage` from [GitHub Releases](https://github.com/shadow-x78/orbiscreen/releases) and run it.

**From source:**

```bash
git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
cd ~/Orbiscreen && ./scripts/install.sh
```

**Android:** install `orbiscreen-android-release.apk` from [GitHub Releases](https://github.com/shadow-x78/orbiscreen/releases).

<a id="usage"></a>
## Usage

Start the daemon in the foreground, or enable the systemd user service:

```bash
orbiscreen start
systemctl --user enable --now orbiscreen
```

Commands:

```bash
orbiscreen start                              # run with auto environment detection
orbiscreen start --width 1920 --height 1080 --fps 60
orbiscreen display set 1920x1080@60            # persist a display size
orbiscreen doctor                             # diagnostics
orbiscreen doctor --fix                       # install missing dependencies
orbiscreen stop                               # stop a running daemon
```

The encoder is chosen in the config file rather than on the command line:

```toml
[encode]
preferred_encoder = "auto"   # auto, nvenc, vaapi, or x264
```

Open the Android app, pick the discovered host, and enter the session token (delivered over mDNS or read with `orbiscreen doctor`).

<a id="architecture"></a>
## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      orbiscreen-daemon                       │
│  ┌────────────────────┐  ┌────────────────────────────────┐  │
│  │ orbiscreen-display │  │ orbiscreen-capture             │  │
│  │ (evdi kernel/DRM)  │  │ (kwin-virtual / wlr / portal)  │  │
│  └────────────────────┘  └────────────────────────────────┘  │
│             │                            │                   │
│             ▼                            ▼                   │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ orbiscreen-encode (GStreamer NVENC/VAAPI/x264)         │  │
│  └────────────────────────────────────────────────────────┘  │
│                              │                               │
│                              ▼                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ orbiscreen-transport: HTTP, UDP, WebTransport, AOA     │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

Reverse input returns through `orbiscreen-input` (uinput, XTEST, wlroots virtual devices, or the RemoteDesktop portal). The full specification is in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

<a id="project-structure"></a>
## Project structure

```
orbiscreen/
├── crates/
│   ├── orbiscreen-core/        # shared types, config, errors
│   ├── orbiscreen-display/     # evdi-backed virtual displays
│   ├── orbiscreen-capture/     # KWin/wlroots virtual outputs, portal/X11 capture
│   ├── orbiscreen-encode/      # GStreamer pipeline (VAAPI / NVENC / x264)
│   ├── orbiscreen-input/       # uinput tablet and touch, RemoteDesktop portal
│   ├── orbiscreen-transport/   # axum, UDP/WebTransport, AOA USB, mDNS
│   ├── orbiscreen-daemon/      # daemon binary, D-Bus service, CLI
│   └── orbiscreen-gui/         # Tauri v2 desktop control center
├── clients/
│   ├── web/                    # browser client (WebTransport + WebCodecs)
│   └── android/                # Material 3 Compose app
├── assets/
│   └── logo/                   # SVG and PNG icons, banners
├── data/                       # desktop entry, RPM spec, systemd service
├── scripts/                    # install, packaging, dev tooling
└── docs/                       # bilingual guides (EN + AR)
```

<a id="faq"></a>
## FAQ

<details>
<summary><b>Is this a true extended display or a mirror?</b></summary>
<br>
It is an independent virtual monitor placed next to your physical displays. Windows can be moved onto it, and the resolution is configurable up to 7680x4320 (8K).
</details>

<details>
<summary><b>Does it work on Wayland without root?</b></summary>
<br>
On KDE Plasma and wlroots compositors the virtual monitor is created by the compositor itself, with no root and no kernel module. On GNOME, COSMIC, and X11 the EVDI kernel module provides the virtual display.
</details>

<details>
<summary><b>Can I draw in Krita or GIMP with a stylus?</b></summary>
<br>
Stylus pressure (up to 4095 levels), tilt, and hover are injected as a Linux input tablet through `uinput`, so applications see a real drawing device.
</details>

<details>
<summary><b>Can I connect over USB instead of Wi-Fi?</b></summary>
<br>
Plug in the cable and accept the accessory permission dialog. The host switches the device to Android Open Accessory mode and streams H.264 over USB bulk endpoints. USB debugging and `adb reverse` are not required.
</details>

<details>
<summary><b>What latency should I expect?</b></summary>
<br>
On a 5 GHz network with hardware encoding, transport delay measures 2 to 4 ms between host send and client assemble. End to end (glass to glass) is typically 25 to 40 ms on a tuned Wi-Fi connection.
</details>

<a id="documentation"></a>
## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System topology, frame pipeline, crate layout |
| [UDP_TRANSPORT.md](docs/UDP_TRANSPORT.md) | UDP Annex-B video path, packet types, DPLPMTUD |
| [FRAME_TRANSPORT.md](docs/FRAME_TRANSPORT.md) | I/P/IDR/GOP terms, AU flow, loss recovery |
| [DE_SUPPORT.md](docs/DE_SUPPORT.md) | Per-desktop support matrix and capture plans |
| [PACKAGING.md](docs/PACKAGING.md) | Packaging specifications (deb, rpm, AppImage) |
| [DBUS_SPEC.md](docs/DBUS_SPEC.md) | D-Bus session interface specification |
| [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) | Common issues and fixes |

<a id="contributing"></a>
## Contributing

1. Fork the repository and create a branch (`feature/`, `fix/`, `docs/`, `chore/`).
2. Keep `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and the test suite green.
3. Add a `CHANGELOG.md` entry.
4. Commit as `orbiscreen | <type>: <description>` and open a pull request against `main`.

Read the [Contributing Guidelines](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md). Report problems through [GitHub Issues](https://github.com/shadow-x78/orbiscreen/issues).

<a id="license"></a>
## License

Distributed under the [GPL-3.0 License](LICENSE).

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Changelog](CHANGELOG.md) ·
[Security](SECURITY.md)

<sub>&copy; 2026 Orbiscreen</sub>

</div>
