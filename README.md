<div align="center">

<img src="assets/logo/orbiscreen-logo.svg" alt="Orbiscreen logo - the O of Orbiscreen as a display ring with the device screen, a solid dot, riding its path" width="180" />

# Orbiscreen

Real virtual secondary displays for Linux, streamed to Android - one command, zero hassle

[![Version](https://img.shields.io/badge/version-0.17.0-2563eb?style=flat-square&logo=semver)](CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)
[![Stars](https://img.shields.io/github/stars/shadow-x78/orbiscreen?style=flat-square&color=eab308&logo=github)](https://github.com/shadow-x78/orbiscreen/stargazers)

</div>

---

## 🌐 Language

<a href="README.md">🇬🇧 English</a> · <a href="README_AR.md">🇸🇦 العربية</a>

---

## 📋 Table of Contents

- [What is Orbiscreen?](#what-is-orbiscreen)
- [Highlights](#highlights)
- [Desktop Support](#desktop-support)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [Android App](#android-app)
- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

---

<a id="what-is-orbiscreen"></a>
## 🤔 What is Orbiscreen?

**Orbiscreen** turns a spare Android tablet or phone into a real second monitor for your Linux desktop. It creates a **kernel-level virtual display** via DisplayLink's `evdi`, or a **compositor-native virtual monitor** on KDE Plasma and wlroots - no root, no share dialog - then streams it as **MPEG-TS/H.264** with reverse touch input natively on Android.

| Problem | Other Projects | Orbiscreen |
|---------|---------------|------------|
| No Linux host support | ❌ Windows-only tools | ✅ Built for Linux first |
| X11-only workarounds | ❌ Break on Wayland | ✅ X11 **and** Wayland via evdi/DRM + compositor IPC |
| Browser-only streaming | ❌ High latency, no touch | ✅ Native Android client + reverse touch |
| Manual IP configuration | ❌ Type addresses by hand | ✅ mDNS discovery + live network scan + manual add |
| Root required everywhere | ❌ Kernel hacks on the client side | ✅ Rootless on wlroots and KDE; `doctor --fix` guides the rest |

---

<a id="highlights"></a>
## ✨ Highlights

- **Real virtual display via `evdi`** (X11 *and* Wayland), **or zero root on KDE Plasma**: a KWin virtual monitor through `zkde-screencast` (no kernel module, no portal dialog), with portal capture fallback elsewhere
- **Material 3 Android client** - Jetpack Compose, Catppuccin Mocha / Latte brand palette, light/dark theme
- **Built-in web client** - watch from any browser at `http://<host>:8788/` (MSE via the locally bundled `mpegts.js`, no CDN)
- **Live discovery** - NSD scan of nearby hosts, manual `host:port` entry, optional subnet scanner
- **Native streaming** - ExoPlayer with `OkHttpDataSource` + `DefaultLoadControl` for low-latency MPEG-TS / H.264
- **Token protection** - `/stream`, `/input` and `/api/control` require a per-session token (mDNS TXT / `/client/config.json`), rotated on every daemon start
- **Reverse touch** - absolute pointer / keyboard / stylus / wheel events flow Android → host
- **Host control panel** - keyboard, lock, blank, Ctrl+Alt+Del, and retry actions
- **USB transport** via `adb reverse` with hot-plug (a device plugged in later is picked up within two seconds), clean teardown on stop, and a live "tunnel ready" card state in the Android app
- **Hardware encoding** - VAAPI, NVENC, x264 software fallback
- **Cryptographic signing** of every Linux and Android artifact

---

<a id="desktop-support"></a>
## 🖥️ Desktop Support

| Environment | Virtual second display | Capture | Input |
|-------------|------------------------|---------|-------|
| KDE Plasma (Wayland) | ✅ Native (zkde-screencast, no root / no dialog) | ✅ PipeWire | ✅ RemoteDesktop portal |
| Sway / Hyprland / wlroots | ✅ Headless output via compositor IPC (no root) | ✅ wlr-screencopy (no dialog) | ✅ virtual-pointer / virtual-keyboard (no portal) |
| GNOME (Wayland) | ⚠️ Via EVDI | ✅ Portal: dialog only on the first run (saved restore token) | ✅ RemoteDesktop portal: likewise persisted |
| XFCE / MATE / LXQt / Cinnamon (X11) | ✅ Via EVDI | ✅ XShm mirrored root (pooled, duplicate frames skipped) | ✅ XTEST (rootless), uinput fallback |
| Anything else | ✅ Via EVDI (guided by `orbiscreen doctor --fix`) | Best available backend | Best available backend |

`orbiscreen doctor` reports the detected compositor, the exact capture plan `auto` will follow, and what is missing on the system; `orbiscreen doctor --fix` installs the EVDI kernel module on detected distros.

---

<a id="quick-start"></a>
## 🚀 Quick Start

### 1. Installation

- **Ubuntu / Pop!_OS / Linux Mint (Launchpad PPA):**
  ```bash
  sudo add-apt-repository ppa:shadow-x78/ppa -y
  sudo apt update
  sudo apt install orbiscreen -y
  ```

- **Fedora (COPR):**
  ```bash
  sudo dnf copr enable shadow-x78/orbiscreen -y
  sudo dnf install orbiscreen -y
  ```

- **Arch Linux (AUR):**
  ```bash
  paru -S orbiscreen
  ```

- **Universal AppImage (`.AppImage`):**
  Download `orbiscreen-x86_64.AppImage` from [GitHub Releases](https://github.com/shadow-x78/orbiscreen/releases):
  ```bash
  chmod +x orbiscreen-x86_64.AppImage
  ./orbiscreen-x86_64.AppImage
  ```

- **Standalone Tarball (`.tar.gz`):**
  ```bash
  tar -xzvf orbiscreen-linux-x86_64.tar.gz
  ./bin/orbiscreen start
  ```
  Put the files in `bin/` on your `PATH` (for example `~/.local/bin`) to run
  `orbiscreen` from anywhere. The bundle contains prebuilt binaries only; for the
  systemd unit, desktop entry and web-client files, use the DEB/RPM/AppImage
  packages or install from source (below).

- **Android (`.apk`):**
  Install `orbiscreen-android-release.apk` (cryptographically signed release build to bypass Play Protect warnings).

### 2. Building from Source (contributors)

```bash
git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
cd ~/Orbiscreen

# One-command installation for Linux
./scripts/install.sh

# evdi kernel module (DKMS) - required for a real second monitor on X11 and
# GNOME desktops. On KDE Plasma Wayland and wlroots compositors (Sway,
# Hyprland) no kernel module is needed: the daemon creates a compositor-native
# virtual monitor on its own. Run `orbiscreen doctor` to see what applies.
sudo modprobe evdi

# Diagnose the environment: detected compositor, capture plan, missing pieces
orbiscreen doctor

# Start the daemon (EVDI DRM, KWin/wlroots virtual display, or portal
# auto-fallback)
orbiscreen start
```

### 3. Connect

- **Android:** tap the discovered host (mDNS) or add it manually.
- **Web browser:** open `http://<host-ip>:8788/` - the daemon serves the MPEG-TS client directly (MSE + bundled `mpegts.js`).
- **Token:** every daemon start generates a session token. Android gets it automatically from mDNS discovery; the web client fetches it from `/client/config.json`. If a client gets `401 Unauthorized`, re-discover the host or restart the client - the token may have rotated.

---

<a id="commands"></a>
## ⌨️ Commands

| Command | Description |
|---------|-------------|
| `orbiscreen start` | Create the virtual display and start streaming |
| `orbiscreen start --no-mdns` | Start without mDNS advertising |
| `orbiscreen stop` | Gracefully stop a running daemon via D-Bus |
| `orbiscreen list-displays` | List configured virtual displays |
| `orbiscreen probe` | Report capture / input / display backends |
| `orbiscreen doctor` | Diagnose the environment: compositor, capture plan, missing permissions/tools |
| `orbiscreen doctor --json` | Machine-readable doctor report (consumed by the GTK panel) |
| `orbiscreen doctor --fix` | Detect the distro and offer to install/load the EVDI kernel module (`--yes` to skip the prompt) |
| `orbiscreen print-config` | Print the resolved configuration |
| `orbiscreen uninstall` | Remove the daemon, systemd service, and desktop entries |

---

<a id="android-app"></a>
## 📱 Android App

The Android client is a **Material 3 + Jetpack Compose** single-Activity app with three screens wired through Compose Navigation:

| Screen | What it does |
|--------|--------------|
| **Discovery** | Live NSD scan of `_orbiscreen._tcp.` hosts, quick-connect chips, manual `host:port` entry, USB mode via `adb reverse`, recent host pinned on top |
| **Stream** | Full-screen ExoPlayer (MPEG-TS over HTTP) with a floating control toolbar: keyboard, lock, blank, Ctrl+Alt+Del, retry |
| **Settings** | Theme (System / Light / Dark), force software decoder, advanced subnet scanner, recent host, about |

Reverse touch works out of the box: `InputDispatcher` maps Android touch to absolute host coordinates over the `/input` endpoint, debounced so the network never backs up during a fast drag.

The client talks to the daemon through three lightweight JSON endpoints in addition to `/stream` and `/input`:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/info` | GET | Display resolution, encoder, and version |
| `/api/control` | POST | Host-side actions (blank, unblank, lock, ctrl-alt-del); token required |
| `/health` | GET | Liveness probe |

---

<a id="architecture"></a>
## 🏗️ Architecture

```
┌──────────────────────────────────────────────────────────────┐
│  orbiscreen-daemon (CLI, clap)                               │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────┐   │
│  │ display      │  │ capture      │  │ encode            │   │
│  │  evdi crate  │  │ x11rb/ashpd  │  │ gstreamer-rs      │   │
│  └──────────────┘  └──────────────┘  └───────────────────┘   │
│  ┌──────────────┐  ┌──────────────────────────────────────┐  │
│  │ input        │  │ transport                            │  │
│  │ evdevil/ashpd│  │ axum + mdns-sd + adb                 │  │
│  │              │  │ + /api/info + /api/control + /health  │  │
│  └──────────────┘  └──────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐    │
│  │ core: shared types, config, errors                   │    │
│  └──────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
       │                  │                    │
       ▼                  ▼                    ▼
   /dev/dri/...     X11 / Wayland         Network (mDNS + HTTP)
```

---

<a id="project-structure"></a>
## 🏗️ Project Structure

```
orbiscreen/
├── crates/
│   ├── orbiscreen-core/        # shared types, config, errors
│   ├── orbiscreen-display/     # evdi-backed virtual displays
│   ├── orbiscreen-capture/     # X11 (x11rb) + Wayland (KWin zkde-screencast / ashpd portal + PipeWire)
│   ├── orbiscreen-encode/      # GStreamer pipeline (VAAPI / NVENC / x264)
│   ├── orbiscreen-input/       # evdevil + ashpd RemoteDesktop
│   ├── orbiscreen-transport/   # axum + mDNS + /api/info + /api/control
│   └── orbiscreen-daemon/      # CLI binary wiring every layer together
├── clients/
│   ├── web/                    # browser MPEG-TS client (HTML / CSS / JS)
│   └── android/                # Material 3 Compose app
│       └── app/src/main/java/com/orbiscreen/android/
│           ├── MainActivity.kt
│           ├── data/           # PrefsStore (recent host + settings)
│           ├── net/            # DiscoveryService, SubnetScanner, HostApi
│           ├── player/        # PlayerHolder, StreamUrl
│           ├── input/         # InputDispatcher
│           └── ui/            # theme, nav, discovery, stream, settings
├── assets/
│   └── logo/                  # The project mark (SVG + PNG set)
├── data/                      # desktop entry, RPM spec, master SVG
├── scripts/                   # install, packaging (deb / rpm / AppImage), dev tooling
├── docs/                      # bilingual guides (EN + AR)
├── .github/{workflows/,ISSUE_TEMPLATE/,PULL_REQUEST_TEMPLATE.md}
└── .editorconfig, .gitignore, .gitattributes, deny.toml, rustfmt.toml
```

---

<a id="documentation"></a>
## 📚 Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System topology, frame pipeline & D-Bus architecture |
| [DE_SUPPORT.md](docs/DE_SUPPORT.md) | Per-desktop support matrix, capture plans & troubleshooting |
| [PACKAGING.md](docs/PACKAGING.md) | Multi-distro packaging specs (.deb, .rpm, AppImage) |
| [DBUS_SPEC.md](docs/DBUS_SPEC.md) | D-Bus Session Bus IPC interface specifications |
| [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) | Common issues, diagnostics & hardware acceleration fixes |

---

<a id="contributing"></a>
## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit your changes: `orbiscreen | <type>: <description>`
4. Push to the branch
5. Open a Pull Request

Please read our [Contributing Guidelines](CONTRIBUTING.md) for the development environment, code style, and the release process.

---

<a id="license"></a>
## 📜 License

Distributed under the [GPL-3.0 License](LICENSE).

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Changelog](CHANGELOG.md) ·
[Security](SECURITY.md)

<sub>&copy; 2026 Orbiscreen</sub>

</div>
