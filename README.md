<div align="center">

<img src="data/orbiscreen.svg" alt="Orbiscreen" width="160" />

# Orbiscreen

Real virtual secondary displays for Linux, streamed to Android - one command, zero hassle

[![Version](https://img.shields.io/badge/version-0.13.2-2563eb?style=flat-square&logo=semver)](CHANGELOG.md)
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
- [Why Orbiscreen Exists](#why-orbiscreen-exists)
- [Highlights](#highlights)
- [Status](#status)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [Android App](#android-app)
- [Architecture](#architecture)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

---

<a id="what-is-orbiscreen"></a>
## 🤔 What is Orbiscreen?

**Orbiscreen** turns a spare Android tablet or phone into a real second monitor for your Linux desktop. Unlike X11-only or browser-only workarounds, Orbiscreen creates a **kernel-level virtual display** via DisplayLink's `evdi`, which appears as a genuine monitor to both X11 and Wayland compositors, and streams it using **MPEG-TS/H.264** with reverse touch input natively on Android.

---

<a id="why-orbiscreen-exists"></a>
## 🧭 Why Orbiscreen Exists

| Problem | Other Projects | Orbiscreen |
|---------|---------------|------------|
| No Linux host support | ❌ spacedesk refuses officially | ✅ Real kernel-level virtual display |
| X11-only workaround | ❌ VirtScreen unmaintained since 2018 | ✅ X11 **and** Wayland via evdi/DRM |
| Wayland second screen missing | ❌ Weylus caps it to X11 | ✅ Full Wayland path via ashpd + PipeWire |
| Manual IP configuration | ❌ Most projects | ✅ mDNS discovery + live network scan + manual add |
| Single-purpose client | ❌ spacedesk only | ✅ Native Android screen + host control panel |

---

<a id="highlights"></a>
## ✨ Highlights

- Real virtual display via `evdi` (X11 *and* Wayland), **or with zero root on KDE Plasma**: a KWin virtual monitor created through `zkde-screencast` (no kernel module, no portal dialog), with portal capture fallback elsewhere
- **Material 3 Android client** - Jetpack Compose, Catppuccin Mocha / Latte brand palette, light/dark theme
- **Built-in web client** - watch from any browser at `http://<host>:8788/` (MSE via the locally bundled `mpegts.js`, no CDN)
- **Live discovery** - NSD scan of nearby hosts, manual `host:port` entry, optional subnet scanner
- **Native streaming** - ExoPlayer with `OkHttpDataSource` + `DefaultLoadControl` for low-latency MPEG-TS / H.264
- **Token protection** - `/stream`, `/input` and `/api/control` require a per-session token (mDNS TXT / `/client/config.json`), rotated on every daemon start
- **Reverse touch** - absolute pointer / keyboard / stylus / wheel events flow Android → host
- **Host control panel** - keyboard, lock, blank, Ctrl+Alt+Del, and retry actions
- **USB transport** via `adb reverse`, no special drivers
- **Hardware encoding** - VAAPI, NVENC, x264 software fallback
- **Cryptographic signing** of every Linux and Android artifact

---

<a id="status"></a>
## 📊 Status

| Phase | Goal | State |
|-------|------|-------|
| 0 | Workspace scaffolding + evdi feasibility | ✅ Completed |
| 1 | Display + capture + encode + input (X11) | ✅ Completed |
| 2 | Android client + USB transport + mDNS | ✅ Completed |
| 3 | Wayland capture + portal fallback + input | ✅ Completed |
| 4 | Packaging + GTK4 GUI + D-Bus service + Standalone installation | ✅ Completed |
| 5 | Material 3 UI + live discovery + control panel | ✅ Completed |
| 6 | Desktop-environment parity: compositor virtual outputs, wlr-screencopy, rootless input, `doctor` | ✅ Completed |

> See `CHANGELOG.md` for the complete release history.

### Desktop support matrix

| Environment | Virtual second display | Capture | Input |
|-------------|------------------------|---------|-------|
| KDE Plasma (Wayland) | ✅ Native (zkde-screencast, no root / no dialog) | ✅ PipeWire | ✅ RemoteDesktop portal |
| Sway / wlroots general | ✅ Headless output via compositor IPC (no root) | ✅ wlr-screencopy (no dialog) | ✅ virtual-pointer / virtual-keyboard (no portal) |
| Hyprland | ✅ Headless output via compositor IPC (no root) | ✅ wlr-screencopy (no dialog) | ✅ virtual-pointer / virtual-keyboard (no portal) |
| GNOME (Wayland) | ⚠️ Via EVDI | ✅ Portal: dialog only on the first run (saved restore token) | ✅ RemoteDesktop portal: likewise persisted |
| XFCE / MATE / LXQt / Cinnamon (X11) | ✅ Via EVDI | ✅ XShm mirrored root (pooled, duplicate frames skipped) | ✅ XTEST (rootless), uinput fallback |
| Anything else | ✅ Via EVDI (guided by `orbiscreen doctor --fix`) | Best available backend | Best available backend |

`orbiscreen doctor` reports the detected compositor, the exact capture plan `auto` will follow, and what is missing on the system; `orbiscreen doctor --fix` installs the EVDI kernel module on detected distros. Full details in [`docs/DE_SUPPORT.md`](docs/DE_SUPPORT.md).

---

<a id="quick-start"></a>
## 🚀 Quick Start

### 1. Official Packages & Pre-built Artifacts (GitHub Releases)

Download pre-built packages from [GitHub Releases](https://github.com/shadow-x78/orbiscreen/releases):

- **Debian / Ubuntu (`.deb`):**
  ```bash
  sudo dpkg -i orbiscreen_amd64.deb || sudo apt-get install -f
  ```

- **Fedora / RHEL (`.rpm`):**
  > **Note:** Our RPM packages are cryptographically signed. You must import the public key first.
  ```bash
  sudo rpm --import https://raw.githubusercontent.com/shadow-x78/orbiscreen/main/orbiscreen.asc
  sudo dnf install ./orbiscreen_x86_64.rpm
  ```

- **Universal AppImage (`.AppImage`):**
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

### 2. Building from Source

```bash
# Clone the repository
git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
cd ~/Orbiscreen

# One-command installation for Linux
./scripts/install.sh

# evdi kernel module (DKMS) - required for a real second monitor on most
# desktops. On KDE Plasma Wayland and on wlroots compositors (Sway, Hyprland)
# no kernel module is needed: the daemon creates a compositor-native virtual
# monitor on its own (no root, no share dialog). Without any of those paths
# Orbiscreen streams a display picked in the portal share dialog.
# Run `orbiscreen doctor` to see what applies to your system.
# See docs/TROUBLESHOOTING.md for per-distro steps. Then:
sudo modprobe evdi

# Diagnose the environment: detected compositor, capture plan, missing pieces
orbiscreen doctor

# Start the Orbiscreen daemon (EVDI DRM, KWin/wlroots virtual display, or
# portal auto-fallback)
orbiscreen start
```

#### Capture backend preference (`orbiscreen.toml`)

By default the daemon reads `$XDG_CONFIG_HOME/orbiscreen/orbiscreen.toml`
(`~/.config/orbiscreen/orbiscreen.toml` when `XDG_CONFIG_HOME` is unset) -
the same path used by the systemd user service. Create the file there, or
point at another location with `--config /path/to/orbiscreen.toml`.

```toml
[capture]
preferred = "auto"   # auto (default) | kwin-virtual | screencopy | evdi | portal | mirror
```

| Value | Behaviour |
|-------|-----------|
| `auto` | KDE Plasma Wayland: KWin virtual display. Sway/Hyprland/wlroots: compositor virtual output via IPC, else wlr-screencopy mirror, else portal. X11: EVDI when its module is loaded, else root capture. |
| `kwin-virtual` | Always the KWin virtual monitor (fails on non-KDE compositors). |
| `screencopy` | Always wlroots screencopy capture (needs a wlroots-based compositor). |
| `evdi` | Always the EVDI DRM virtual display (opt-in, needs the root-installed kernel module). |
| `portal` | Always the portal share dialog; pick any screen. |
| `mirror` | Show your **real** desktop instead of a second monitor: pick the screen to mirror in the share dialog. |

> A virtual display starts **empty** (desktop wallpaper only); that is what a second monitor is. Drag windows onto `Virtual-ORBISCREEN`, or use `mirror` to stream your actual screen.

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

```bash
orbiscreen --config orbiscreen.toml --verbose probe
```

To remove everything, including saved config and evdi module state:

```bash
orbiscreen uninstall && ./scripts/uninstall.sh
```

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
orbiscreen/
├── crates/
│   ├── orbiscreen-core/        # types, config, errors
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
│           ├── player/         # PlayerHolder, StreamUrl
│           ├── input/          # InputDispatcher
│           └── ui/
│               ├── theme/      # Material 3 colour, typography, theme
│               ├── nav/        # Compose Navigation graph
│               ├── discovery/  # DiscoveryScreen + ViewModel
│               ├── stream/     # StreamScreen, PlayerSurface, ControlToolbar
│               └── settings/   # SettingsScreen
├── scripts/                    # install, packaging (deb / rpm / AppImage), dev tooling
├── .github/{workflows/,ISSUE_TEMPLATE/,PULL_REQUEST_TEMPLATE.md}
└── .editorconfig, .gitignore, .gitattributes, deny.toml, rustfmt.toml
```

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

<a id="documentation"></a>
## 📚 Documentation

| Document | Description |
|----------|-------------|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) · [AR](docs/ARCHITECTURE_AR.md) | System topology, frame pipeline & D-Bus architecture |
| [docs/DE_SUPPORT.md](docs/DE_SUPPORT.md) · [AR](docs/DE_SUPPORT_AR.md) | Per-desktop support matrix, capture plans & troubleshooting |
| [docs/PACKAGING.md](docs/PACKAGING.md) · [AR](docs/PACKAGING_AR.md) | Multi-distro packaging specs (.deb, .rpm, AppImage, Flatpak) |
| [docs/DBUS_SPEC.md](docs/DBUS_SPEC.md) · [AR](docs/DBUS_SPEC_AR.md) | D-Bus Session Bus IPC interface specifications |
| [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) · [AR](docs/TROUBLESHOOTING_AR.md) | Common issues, diagnostics & hardware acceleration fixes |
| [SECURITY.md](SECURITY.md) | Security model, transport safety & network policies |
| [CHANGELOG.md](CHANGELOG.md) | Full release history |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Guidelines for contributing and building from source |

---

<a id="contributing"></a>
## 🤝 Contributing

Please read our [Contributing Guidelines](CONTRIBUTING.md) for details on how to set up the development environment, format your code, and submit Pull Requests.

When committing, follow the convention:

```text
orbiscreen | <scope>: <message>
```

For example:

```text
orbiscreen | android | player: retry on transient network errors
orbiscreen | docs | readme: clarify mDNS discovery flow
orbiscreen | v0.11.0 | release: token auth + evdi primary frame source
```

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
