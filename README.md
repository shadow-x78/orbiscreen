<div align="center">

<pre align="center">
   ___  ___   _   _____ _____
  / _ \/ _ \ / | / __  / ___/
 / /_/ / /_/ // |/ / / / __ \
/ _, _/ _, _/ /|_/ /_/ / /_/ /
/_/ |_/_/ |_/_/  |\___/\____/
</pre>

# Orbiscreen

Real virtual secondary displays for Linux, streamed to Android — over Wi-Fi or USB

[![Version](https://img.shields.io/badge/version-0.1.0-2563eb?style=flat-square&logo=semver)](CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](LICENSE)
![Language](https://img.shields.io/badge/rust-edition_2021-16a34a?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux-9333ea?style=flat-square&logo=linux)
![Client](https://img.shields.io/badge/client-Android-eab308?style=flat-square&logo=android)
[![CI](https://github.com/shadow-x78/orbiscreen/actions/workflows/ci.yml/badge.svg?style=flat-square)](https://github.com/shadow-x78/orbiscreen/actions/workflows/ci.yml)
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
- [Project Structure](#project-structure)
- [Architecture](#architecture)
- [Documentation](#documentation)
- [Contributing](#contributing)
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
- WebRTC streaming — opens in any modern browser, no app install needed
- Reverse touch — pointer / keyboard / stylus events flow Android → host
- mDNS discovery — Android clients find the host automatically
- USB transport via `adb reverse`, no special drivers
- Hardware encoding — VAAPI, NVENC, x264 software fallback

---

<a id="status"></a>
## 📊 Status

| Phase | Goal | State |
|-------|------|-------|
| 0 | Workspace scaffolding + evdi feasibility | ✅ Closed |
| 1 | Display + capture + encode + input (X11) | ⚠️ Scaffolded — WebRTC streaming pending |
| 2 | Android client + USB transport + mDNS | ⚠️ Scaffolded |
| 3 | Wayland capture + input | ⚠️ Scaffolded — PipeWire DMA-BUF pending |
| 4 | Packaging + advanced features | ⚠️ Scaffolded |

> See `CHANGELOG.md` for the complete release history.

---

<a id="quick-start"></a>
## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
cd ~/Orbiscreen

# Build the workspace
. "$HOME/.cargo/env"
cargo build --workspace

# Probe the local session
cargo run -p orbiscreen-daemon -- probe

# Start the daemon (needs evdi kernel module + /dev/dri permissions)
cargo run -p orbiscreen-daemon -- start
```

A full host-side walk-through lives in `scripts/setup-dev-env.sh`, and the evdi feasibility probe in `scripts/test-evdi.sh`.

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
