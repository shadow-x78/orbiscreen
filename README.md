<div align="center">
  <img src="data/orbiscreen.svg" alt="Orbiscreen" width="180" style="margin-bottom: 0px;" />
  <h1 style="margin-top: 4px; margin-bottom: 8px;">Orbiscreen</h1>
  <p>Real virtual secondary displays for Linux, streamed to Android - over Wi-Fi or USB</p>

  <p>
    <a href="CHANGELOG.md"><img src="https://img.shields.io/badge/version-v0.4.8-blue?style=flat-square" alt="Version" /></a>
    <a href="https://github.com/shadow-x78/orbiscreen/actions/workflows/ci.yml"><img src="https://github.com/shadow-x78/orbiscreen/actions/workflows/ci.yml/badge.svg?style=flat-square" alt="CI" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-blue?style=flat-square" alt="License" /></a>
    <a href="https://github.com/shadow-x78/orbiscreen/stargazers"><img src="https://img.shields.io/github/stars/shadow-x78/orbiscreen?style=flat-square" alt="Stars" /></a>
  </p>
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
- [Documentation Index](#documentation-index)
- [Quick Start & Packaging](#quick-start)
- [Commands](#commands)
- [Architecture](#architecture)
- [License](#license)

---

<a id="what-is-orbiscreen"></a>
## 🤔 What is Orbiscreen?

**Orbiscreen** turns a spare Android tablet or phone into a real second monitor for your Linux desktop. Unlike X11-only or browser-only workarounds, Orbiscreen creates a **kernel-level virtual display** via DisplayLink's `evdi`, which appears as a genuine monitor to both X11 and Wayland compositors, and streams it over **WebRTC** with reverse touch input.

---

<a id="why-orbiscreen-exists"></a>
## 🧭 Why Orbiscreen Exists

| Problem | Other Projects | Orbiscreen |
|---------|---------------|------------|
| No Linux host support | spacedesk refuses officially | Real kernel-level virtual display |
| X11-only workaround | VirtScreen unmaintained since 2018 | X11 **and** Wayland via evdi/DRM |
| Wayland second screen missing | Weylus caps it to X11 | Full Wayland path via ashpd + PipeWire |
| Manual IP configuration | Most projects | mDNS discovery + `adb reverse` USB |

---

<a id="highlights"></a>
## ✨ Highlights

- Real virtual display via `evdi` (X11 *and* Wayland)
- WebRTC streaming - opens in any modern browser, no app install needed
- Reverse touch - pointer / keyboard / stylus events flow Android → host
- mDNS discovery - Android clients find the host automatically
- USB transport via `adb reverse`, no special drivers
- Hardware encoding - VAAPI, NVENC, x264 software fallback

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

> See `CHANGELOG.md` for the complete release history.

---

<a id="documentation-index"></a>
## 📚 Documentation Index

| Topic | English Guide | Arabic Guide (العربية) |
| :--- | :--- | :--- |
| **System Architecture** | [ARCHITECTURE.md](docs/ARCHITECTURE.md) | [ARCHITECTURE_AR.md](docs/ARCHITECTURE_AR.md) |
| **Multi-Distro Packaging** | [PACKAGING.md](docs/PACKAGING.md) | [PACKAGING_AR.md](docs/PACKAGING_AR.md) |
| **D-Bus Session Spec** | [DBUS_SPEC.md](docs/DBUS_SPEC.md) | [DBUS_SPEC_AR.md](docs/DBUS_SPEC_AR.md) |
| **Troubleshooting & Fixes** | [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) | [TROUBLESHOOTING_AR.md](docs/TROUBLESHOOTING_AR.md) |
| **Security Architecture** | [SECURITY.md](docs/SECURITY.md) | [SECURITY_AR.md](docs/SECURITY_AR.md) |

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
  ```bash
  sudo dnf install ./orbiscreen-1.x86_64.rpm
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

- **Android App (`.apk`):**
  Install `orbiscreen-android-release.apk` (signed release build to bypass Play Protect warnings) or `orbiscreen-android-debug.apk`.

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

```bash
orbiscreen --config orbiscreen.toml --verbose probe
```

---

<a id="project-structure"></a>
## 🏗️ Project Structure

```
orbiscreen/
├── crates/
│   ├── orbiscreen-core/        # types, config, errors
│   ├── orbiscreen-display/     # evdi-backed virtual displays
│   ├── orbiscreen-capture/     # X11 (x11rb) + Wayland (ashpd + PipeWire)
│   ├── orbiscreen-encode/      # GStreamer pipeline (VAAPI / NVENC / x264)
│   ├── orbiscreen-input/       # evdevil + ashpd RemoteDesktop
│   ├── orbiscreen-transport/   # axum + mDNS + adb reverse + signaling scaffold
│   └── orbiscreen-daemon/      # CLI binary wiring every layer together
├── clients/
│   ├── web/                    # browser WebRTC client (HTML / CSS / JS)
│   └── android/                # Kotlin Android WebView host
├── packaging/{flatpak,appimage,debian}/
├── scripts/{setup-dev-env.sh,test-evdi.sh}
├── .github/{workflows/,ISSUE_TEMPLATE/,dependabot.yml}
└── .editorconfig, .gitignore, .gitattributes, deny.toml, rustfmt.toml
```

---

<a id="architecture"></a>
## 🧩 Architecture

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

<a id="documentation"></a>
## 📚 Documentation

| Document | Description |
|----------|-------------|
| [CHANGELOG.md](CHANGELOG.md) | Release history |
| [SECURITY.md](SECURITY.md) | Security policy and reporting |
| [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) | Common issues and debugging |

---

<a id="contributing"></a>
## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit your changes
4. Push to the branch
5. Open a Pull Request

---

<a id="license"></a>
## 📜 License

Distributed under the [GPL-3.0 License](LICENSE).

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Changelog](CHANGELOG.md) ·
[Security](SECURITY.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
