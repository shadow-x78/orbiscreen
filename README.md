<div align="center">
  <img src="data/orbiscreen.svg" alt="Orbiscreen" width="180" style="display: block; margin: 0 auto;" />
  <h1 style="margin-top: 0px; margin-bottom: 4px; border-bottom: none;">Orbiscreen</h1>
  <p>Real virtual secondary displays for Linux, streamed to Android - over Wi-Fi or USB</p>

  <p>
    <a href="CHANGELOG.md"><img src="https://img.shields.io/badge/version-v0.10.1-blue?style=flat-square" alt="Version" /></a>
    <a href="https://github.com/shadow-x78/orbiscreen/actions/workflows/ci.yml"><img src="https://github.com/shadow-x78/orbiscreen/actions/workflows/ci.yml/badge.svg?style=flat-square" alt="CI" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-blue?style=flat-square" alt="License" /></a>
    <a href="https://github.com/shadow-x78/orbiscreen/stargazers"><img src="https://img.shields.io/github/stars/shadow-x78/orbiscreen?style=flat-square" alt="Stars" /></a>
  </p>
</div>

---

## 📋 Table of Contents

- [What is Orbiscreen?](#what-is-orbiscreen)
- [Why Orbiscreen Exists](#why-orbiscreen-exists)
- [Highlights](#highlights)
- [Status](#status)
- [Quick Start & Packaging](#quick-start)
- [Commands](#commands)
- [Android App](#android-app)
- [Architecture](#architecture)
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
| No Linux host support | spacedesk refuses officially | Real kernel-level virtual display |
| X11-only workaround | VirtScreen unmaintained since 2018 | X11 **and** Wayland via evdi/DRM |
| Wayland second screen missing | Weylus caps it to X11 | Full Wayland path via ashpd + PipeWire |
| Manual IP configuration | Most projects | mDNS discovery + live network scan + manual add |
| Single-purpose client | Spacedesk only | Native Android screen + host control panel |

---

<a id="highlights"></a>
## ✨ Highlights

- Real virtual display via `evdi` (X11 *and* Wayland).
- **Material 3 Android client** — Jetpack Compose, dynamic colour, light/dark theme.
- **Live discovery** — NSD scan of nearby hosts, manual `host:port` entry, optional subnet scanner.
- **Native streaming** — ExoPlayer with `OkHttpDataSource` + `DefaultLoadControl` for low-latency MPEG-TS / H.264.
- **Reverse touch** — absolute pointer / keyboard / stylus / wheel events flow Android → host.
- **Host control panel** — keyboard, lock, blank, Ctrl+Alt+Del, file manager, and retry actions.
- **USB transport** via `adb reverse`, no special drivers.
- **Hardware encoding** — VAAPI, NVENC, x264 software fallback.
- **Cryptographic signing** of every Linux and Android artifact.

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
| 5 | Material 3 UI + live discovery + control panel | ✅ Completed (v0.10.1) |

> See `CHANGELOG.md` for the complete release history.

---

<a id="quick-start"></a>
## 🚀 Quick Start & Multi-Distro Installation

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
  cd release-bundle && ./install.sh
  ```

- **Android (`.apk`):**
  Install `orbiscreen-android-release.apk` (cryptographically signed release build to bypass Play Protect warnings).

### 2. Building from Source

```bash
# Clone the repository
git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
cd ~/Orbiscreen

# One-command installation for Linux
./scripts/install.sh

# Probe local capture, input, and display backends
orbiscreen probe

# Start the Orbiscreen daemon (EVDI DRM or Wayland Portal auto-fallback)
orbiscreen start
```

---

<a id="commands"></a>
## ⌨️ Commands

| Command | Description |
|---------|-------------|
| `orbiscreen start` | Create the virtual display and start streaming |
| `orbiscreen start --no-mdns` | Start without mDNS advertising |
| `orbiscreen list-displays` | List configured virtual displays |
| `orbiscreen probe` | Report capture / input / display backends |
| `orbiscreen print-config` | Print the resolved configuration |
| `orbiscreen uninstall` | Remove the daemon, systemd service, and desktop entries |

```bash
orbiscreen --config orbiscreen.toml --verbose probe
```

---

<a id="android-app"></a>
## 📱 Android App (v0.10.1)

The Android client is a **Material 3 + Jetpack Compose** single-Activity app. Three screens are wired through Compose Navigation:

### Discovery screen

- Status banner that reports the active scan and the number of hosts found.
- **Live list** of `_orbiscreen._tcp.` services discovered via `NsdManager`, with name, IP, and port.
- **Quick chips** for each host: tap to connect, or long-press for details.
- **Add manually** card — expand to reveal an `OutlinedTextField` that validates `host:port` with a regex.
- **USB mode** card — pre-fills `127.0.0.1:8788` for `adb reverse tcp:8788 tcp:8788`.
- Refresh button to restart the scan.
- **Recent host** persisted in `SharedPreferences` and surfaced at the top of the list with a "Recent" chip.

### Stream screen

- Background-black `Scaffold` with a centred top app bar and a `ControlToolbar` rail.
- **ExoPlayer** is wrapped in a `PlayerView` (via `AndroidView`) with `useController = false` so the in-app bar is the single source of UI.
- A `Player.Listener` reports `Buffering`, `Playing`, and `Error` events; errors show a retry card with a clear root-cause message.
- `StreamUrl` always builds a `.ts` URL with `setMimeType(MimeTypes.VIDEO_MP2T)` so MPEG-TS over HTTP is decoded without sniffing.
- `OkHttpDataSource` uses zero read-timeout for live streams and a tuned `DefaultLoadControl` for stable buffering.

### Control toolbar

A floating action rail over the player surface with:

| Action | Effect |
|--------|--------|
| Keyboard | Show soft keyboard overlay + system-IME handoff |
| Lock | `POST /api/control {action:"lock"}` |
| Blank | Toggle `POST /api/control {action:"blank", state:"on"|"off"}` |
| Ctrl+Alt+Del | `POST /api/control {action:"ctrl_alt_del"}` |
| Files | `POST /api/control {action:"open", target:"files"}` |
| Retry | Re-prepare the player |

### Settings screen

- Theme: System / Light / Dark (System default).
- Streaming: force software decoder; enable advanced subnet scanner.
- Recent host: view and forget the last successful connection.
- About: app version, SDK level, and copy-to-clipboard version info.

### Input model

`InputDispatcher` translates Android touch events into absolute host coordinates using the host's reported display dimensions:

```kotlin
fun move(localX: Float, localY: Float, containerW: Int, containerH: Int) {
    val (x, y) = map(localX, localY, containerW, containerH)
    moves.tryEmit(JSONObject().apply {
        put("Move", JSONObject().apply { put("x", x); put("y", y) })
    })
}
```

Pointer events are debounced through a `MutableSharedFlow` with `BufferOverflow.DROP_OLDEST` so the network never backs up during a fast drag.

### Host control endpoints (Rust side)

The Android client uses three lightweight JSON endpoints in addition to the existing `/stream` and `/input`:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/info` | GET | Returns display resolution, encoder, and version |
| `/api/control` | POST | Host-side actions (blank, lock, ctrl-alt-del, open) |
| `/health` | GET | Liveness probe |

---

<a id="architecture"></a>
## 🏗️ Architecture

```
orbiscreen/
├── crates/
│   ├── orbiscreen-core/        # types, config, errors
│   ├── orbiscreen-display/     # evdi-backed virtual displays
│   ├── orbiscreen-capture/     # X11 (x11rb) + Wayland (ashpd + PipeWire)
│   ├── orbiscreen-encode/      # GStreamer pipeline (VAAPI / NVENC / x264)
│   ├── orbiscreen-input/       # evdevil + ashpd RemoteDesktop
│   ├── orbiscreen-transport/   # axum + mDNS + adb reverse + /api/info + /api/control
│   └── orbiscreen-daemon/      # CLI binary wiring every layer together
├── clients/
│   ├── web/                    # browser WebRTC client (HTML / CSS / JS)
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
├── packaging/{flatpak,appimage,debian}/
├── scripts/{setup-dev-env.sh,test-evdi.sh,install.sh,uninstall.sh}
├── .github/{workflows/,ISSUE_TEMPLATE/,dependabot.yml}
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
│  │ evdevil/ashpd│  │ axum + webrtc-rs + mdns-sd + adb     │  │
│  │              │  │ + /api/info + /api/control + /health  │  │
│  └──────────────┘  └──────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐    │
│  │ core: shared types, config, errors                   │    │
│  └──────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
       │                  │                    │
       ▼                  ▼                    ▼
   /dev/dri/...     X11 / Wayland         Network (mDNS + UDP)
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System topology, zero-copy pipeline & D-Bus architecture |
| [docs/PACKAGING.md](docs/PACKAGING.md) | Multi-distro packaging specifications (.deb, .rpm, AppImage, Flatpak) |
| [docs/DBUS_SPEC.md](docs/DBUS_SPEC.md) | D-Bus Session Bus IPC interface specifications |
| [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) | Common issues, diagnostics & hardware acceleration fixes |
| [SECURITY.md](SECURITY.md) | Security model, WebRTC transport safety & network policies |
| [CHANGELOG.md](CHANGELOG.md) | Full release history and commit style |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Guidelines for contributing and building from source |

---

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
orbiscreen | v0.10.2 | release: hotfix for blank screen on Pixel 7
```

---

## 📜 License

Distributed under the [GPL-3.0 License](LICENSE).

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Changelog](CHANGELOG.md) ·
[Security](SECURITY.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
