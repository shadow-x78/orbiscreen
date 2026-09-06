<div align="center">

<a href="https://github.com/shadow-x78/orbiscreen">
  <img src="https://raw.githubusercontent.com/shadow-x78/orbiscreen/main/assets/logo/orbiscreen-banner.png" alt="Orbiscreen - Turn any Android tablet or phone into a low-latency second monitor for Linux" width="100%" />
</a>

<br><br>

# Orbiscreen: Turn Android into a Second Monitor for Linux

**High-performance, ultra-low latency virtual secondary display for Linux (Wayland &amp; X11) streamed to Android tablets and phones.**

[![Version](https://img.shields.io/badge/version-0.23.4-2563eb?style=for-the-badge&logo=semver)](CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=for-the-badge)](LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=for-the-badge&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=for-the-badge&logo=linux)
[![Stars](https://img.shields.io/github/stars/shadow-x78/orbiscreen?style=for-the-badge&color=eab308&logo=github)](https://github.com/shadow-x78/orbiscreen/stargazers)

<br>

<!-- 1-Click Viral Sharing Badges -->
[![Share on Reddit](https://img.shields.io/badge/Share-Reddit-FF4500?style=flat-square&logo=reddit&logoColor=white)](https://www.reddit.com/submit?url=https%3A%2F%2Fgithub.com%2Fshadow-x78%2Forbiscreen&title=Orbiscreen%20-%20Turn%20any%20Android%20device%20into%20a%20low-latency%20second%20monitor%20for%20Linux)
[![Share on X](https://img.shields.io/badge/Share-X%2FTwitter-000000?style=flat-square&logo=x&logoColor=white)](https://twitter.com/intent/tweet?url=https%3A%2F%2Fgithub.com%2Fshadow-x78%2Forbiscreen&text=Turn%20any%20Android%20tablet%20or%20phone%20into%20a%20low-latency%20second%20monitor%20for%20Linux%20with%20Orbiscreen!%20%23Linux%20%23Rust%20%23OpenSource)
[![Share on Hacker News](https://img.shields.io/badge/Share-Hacker%20News-FF6600?style=flat-square&logo=ycombinator&logoColor=white)](https://news.ycombinator.com/submitlink?u=https%3A%2F%2Fgithub.com%2Fshadow-x78%2Forbiscreen&t=Orbiscreen%20-%20Turn%20Android%20into%20a%20low-latency%20second%20monitor%20for%20Linux)

</div>


---


## 🌐 Language

<a href="README.md">🇬🇧 English</a> · <a href="README_AR.md">🇸🇦 العربية</a>

---

## 📋 Table of Contents

- [What is Orbiscreen?](#what-is-orbiscreen)
- [Why Orbiscreen vs Alternatives?](#comparison)
- [Popular Use Cases](#use-cases)
- [Highlights](#highlights)
- [Desktop Support](#desktop-support)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [Android App](#android-app)
- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [Frequently Asked Questions (FAQ)](#faq)
- [Documentation](#documentation)
- [Support &amp; Community](#support)
- [Contributing](#contributing)
- [License](#license)

---

<a id="what-is-orbiscreen"></a>
## 🤔 What is Orbiscreen?

**Orbiscreen** turns a spare Android tablet or phone into a genuine secondary monitor for your Linux desktop. It creates an independent **kernel-level virtual display** via DisplayLink's `evdi` (COSMIC, GNOME, X11), or a **compositor-native virtual monitor** on KDE Plasma and wlroots with zero root required and no screen-share dialogs, streamed as **MPEG-TS/H.264** with reverse multi-touch, mouse, keyboard, and pressure-sensitive stylus control natively on Android.

<a id="comparison"></a>
### 🆚 How Does Orbiscreen Compare to Alternatives?

| Feature / Capability | Spacedesk | Deskreen | Weylus | Apple Sidecar | **Orbiscreen** |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Linux Host Support** | ❌ (Windows only) | ✅ (Web based) | ✅ (Web based) | ❌ (macOS only) | **✅ Linux-first** |
| **Wayland &amp; X11 Support** | ❌ | ⚠️ (Requires dummy plug) | ⚠️ (Mirror only) | ❌ | **✅ Native Wayland &amp; X11** |
| **True Extended Display** | ✅ (Windows only) | ❌ (Needs HDMI dummy) | ❌ (Screen mirror only) | ✅ (Apple only) | **✅ Real Virtual Monitor** |
| **Native Android Client** | ✅ | ❌ (Browser only) | ❌ (Browser only) | ❌ (iPad only) | **✅ Native Jetpack Compose** |
| **Hardware Encoding** | ✅ | ❌ | ⚠️ | ✅ | **✅ NVENC &amp; VA-API** |
| **Ultra-Low Latency** | ~50-80ms | ~150-300ms | ~80-120ms | ~30ms | **⚡ ~25-40ms** |
| **Stylus Pressure &amp; Tilt** | ❌ | ❌ | ⚠️ (Stylus only) | ✅ | **✅ 4095 Levels (Krita/GIMP)** |
| **Reverse Touch &amp; Mouse** | ✅ | ❌ | ⚠️ (Stylus only) | ✅ | **✅ Multi-touch, Mouse &amp; Keys** |
| **Zero Root (KDE &amp; wlroots)**| N/A | ✅ | ❌ | N/A | **✅ Rootless (KWin / wlroots)** |
| **Open Source** | ❌ (Proprietary) | ✅ (GPL-3.0) | ✅ (AGPL-3.0) | ❌ (Closed source) | **✅ GPL-3.0 (100% Free)** |

---

<a id="use-cases"></a>
## 🎯 Popular Use Cases

- 📱 **Repurpose Spare Tablets &amp; Phones**: Don't let your older Android tablet (Samsung Galaxy Tab, Xiaomi Pad, Lenovo Tab) gather dust. Turn it into a high-utility second screen.
- 🎨 **Digital Drawing Tablet (Graphic Digitizer)**: Connect your tablet and stylus (S-Pen, capacitive, active stylus) to paint with native **pressure sensitivity and tilt** in Linux creative software like **Krita, GIMP, Blender, and Inkscape**.
- 💻 **Dual Monitor On the Go**: Travel light without carrying fragile external monitors. Extend your Linux laptop screen at coffee shops, flights, coworking spaces, or hotel desks.
- 🖥️ **Auto-Rotating Portrait Monitor**: Rotate your device to vertical orientation for reading code, API documentation, terminal logs, or keeping Discord/Slack visible.
- ⚡ **Zero Wi-Fi Latency via USB**: Plug in a USB cable; Orbiscreen's automatic ADB reverse tunnel delivers interference-free, ultra-low latency performance.

---

<a id="highlights"></a>
## ✨ Highlights

- **XDG Desktop Portal Virtual Display API**: Rootless virtual display creation on GNOME 46+ and KDE Plasma 6+ via ScreenCast `SourceType::Virtual`, with automatic EVDI fallback
- **Graphic Digitizer with Stylus Hover &amp; Pressure**: 4095 pressure levels, tilt tracking, and in-air hover cursor tracking mapped to Linux `uinput` for Krita, GIMP, and Blender
- **Touchpad Drag-and-Drop Gesture**: Double-tap and drag in Touchpad mode for seamless window moving and file selection, with strict multi-monitor boundary confinement
- **Ultra-Low Latency 5GHz Wi-Fi Tuning**: 6-frame GOP (100ms keyframe interval) with sub-50ms buffer pipeline for instantaneous recovery and zero-lag responsiveness
- **Chromebook CM3001 &amp; ChromeOS Support**: Auto-probing internal ADB route (`100.115.92.2:5555`) for plug-and-play USB streaming on ChromeOS ARC++
- **Session Token Security & Bootstrap**: Session token generation with strict `0o600` permissions and automated discovery delivery, supporting `#token=` URL hash for direct links
- **Connection Lifecycle &amp; Fast Recovery**: Explicit disconnect detection, 500ms `/health` probe, and capped reconnection attempts
- **Real virtual display via `evdi`** (X11 and Wayland), **or zero root on KDE Plasma / wlroots**: KWin virtual monitor or compositor headless output without kernel modules
- **Material 3 Android client**: Jetpack Compose, Catppuccin Mocha / Latte brand palette, light/dark theme
- **Auto-Orientation Resolution Adaptation**: Automatically swaps width and height when your phone/tablet rotates between landscape and portrait
- **3-Row Spacious Keyboard Overlay**: Top-docked layout ensuring bottom taskbars, terminals, and prompts stay 100% visible
- **Built-in web client**: Watch from any browser at `http://<host>:8788/` (MSE via locally bundled `mpegts.js`, no CDN)
- **Live discovery**: NSD scan of nearby hosts, manual `host:port` entry, optional subnet scanner
- **Native streaming**: ExoPlayer with `OkHttpDataSource` + tuned load control for ultra-low latency MPEG-TS / H.264
- **Host control panel**: Keyboard, lock, blank, Ctrl+Alt+Del, and retry actions
- **USB transport via `adb reverse`**: Hot-plug detection picking up newly connected devices within two seconds
- **Hardware encoding**: NVIDIA NVENC, Intel/AMD VA-API, and x264 software fallback
- **Cryptographic signing** of every Linux and Android artifact

---

<a id="desktop-support"></a>
## 🖥️ Desktop Support

| Environment | Virtual second display | Capture | Input |
|-------------|------------------------|---------|-------|
| KDE Plasma (Wayland) | ✅ Native (Portal Virtual / zkde-screencast, no root) | ✅ PipeWire | ✅ RemoteDesktop portal / uinput |
| COSMIC (Wayland) | ⚠️ Via EVDI (guided by `doctor --fix`) | ✅ Portal ScreenCast (PipeWire) | ✅ uinput (multi-touch & stylus) / RemoteDesktop |
| Sway / Hyprland / wlroots | ✅ Headless output via compositor IPC (no root) | ✅ wlr-screencopy (no dialog) | ✅ virtual-pointer / virtual-keyboard (no portal) |
| GNOME (Wayland) | ✅ Portal Virtual (GNOME 46+) / EVDI | ✅ Portal ScreenCast (PipeWire) | ✅ RemoteDesktop portal: persisted |
| XFCE / MATE / LXQt / Cinnamon (X11) | ✅ Via EVDI | ✅ XShm mirrored root (pooled, duplicate frames skipped) | ✅ XTEST (rootless), uinput fallback |
| Anything else | ✅ Via EVDI (guided by `orbiscreen doctor --fix`) | Best available backend | Best available backend |

`orbiscreen doctor` reports the detected compositor, the exact capture plan `auto` will follow, and what is missing on the system; `orbiscreen doctor --fix` installs the EVDI kernel module on detected distros. Full details in [Desktop Environment Support](docs/DE_SUPPORT.md).

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

- **Universal AppImage (`.AppImage`):**
  Download `orbiscreen-x86_64.AppImage` from [GitHub Releases](https://github.com/shadow-x78/orbiscreen/releases):
  ```bash
  chmod +x orbiscreen-x86_64.AppImage
  ./orbiscreen-x86_64.AppImage
  ```

- **One-Command Automated Installer:**
  ```bash
  git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
  cd ~/Orbiscreen
  ./scripts/install.sh
  ```

- **Android App (`.apk`):**
  Download `orbiscreen-android-release.apk` from [GitHub Releases](https://github.com/shadow-x78/orbiscreen/releases).

### 2. Running Orbiscreen

- **From Terminal (Foreground Daemon):**
  ```bash
  orbiscreen start
  ```

- **As Background Service (systemd user unit):**
  ```bash
  systemctl --user start orbiscreen
  # To enable automatically on login:
  systemctl --user enable orbiscreen
  ```

---

<a id="commands"></a>
## ⚙️ Commands

```bash
# Start virtual display daemon with auto environment detection
orbiscreen start

# Start with custom resolution and framerate
orbiscreen start --width 1920 --height 1080 --fps 60

# Force specific hardware encoder (nvenc, vaapi, x264)
orbiscreen start --encoder nvenc

# Run diagnostics and view capture pipeline details
orbiscreen doctor

# Automatically install missing kernel modules or dependencies
orbiscreen doctor --fix

# Gracefully stop running daemon
orbiscreen stop
```

---

<a id="android-app"></a>
## 📱 Android App Features

- **Instant mDNS Discovery**: Automatically discovers Linux hosts on your Wi-Fi network.
- **USB Cable Hot-Plug**: Connect with a single USB cable for zero-latency, zero-interference streaming via ADB reverse.
- **Chromebook &amp; ChromeOS ARC++ Ready**: Auto-probing internal ADB route on `100.115.92.2:5555` for Chromebook CM3001 and ChromeOS devices.
- **Graphic Digitizer Support**: Stylus in-air hover cursor tracking, pressure sensitivity (4095 levels), and tilt angle support for drawing in Krita and GIMP.
- **Touchpad Drag-and-Drop**: Double-tap and drag gesture for smooth window dragging, file movement, and text selection.
- **Ultra-Low Latency Wi-Fi Tuning**: 40-120ms tuned buffer profile paired with 6-frame GOP encoding for instant live edge tracking on 5GHz networks.
- **Fast Disconnect Detection**: Instant 500ms health checking and capped reconnection preventing infinite retry loops.
- **Dynamic Auto-Rotation**: Rotating your device automatically synchronizes virtual screen aspect ratio.
- **Top-Docked Keyboard Overlay**: 3-row layout with function keys, shortcuts, and navigation that never obscures your workspace.
- **Streamlined Display Settings**: Dedicated pointer speed slider and instant resolution picker.

---

<a id="architecture"></a>
## 🏛️ Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      orbiscreen-daemon                       │
│  ┌────────────────────┐  ┌────────────────────────────────┐  │
│  │ orbiscreen-display │  │ orbiscreen-capture             │  │
│  │ (evdi kernel/DRM)  │  │ (zkde-screencast/wlr/ashpd)    │  │
│  └────────────────────┘  └────────────────────────────────┘  │
│             │                            │                   │
│             ▼                            ▼                   │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ orbiscreen-encode (GStreamer NVENC/VAAPI/x264)         │  │
│  └────────────────────────────────────────────────────────┘  │
│                              │                               │
│                              ▼                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ orbiscreen-transport (HTTP MPEG-TS + mDNS + ADB USB)   │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

---

<a id="project-structure"></a>
## 🏗️ Project Structure

```
orbiscreen/
├── crates/
│   ├── orbiscreen-core/        # shared types, config, errors
│   ├── orbiscreen-display/     # evdi-backed virtual displays
│   ├── orbiscreen-capture/     # X11 (x11rb) + Wayland (zkde-screencast / ashpd PipeWire)
│   ├── orbiscreen-encode/      # GStreamer pipeline (VAAPI / NVENC / x264)
│   ├── orbiscreen-input/       # uinput tablet & touch + ashpd RemoteDesktop
│   ├── orbiscreen-transport/   # axum + mDNS + ADB reverse USB
│   └── orbiscreen-daemon/      # CLI binary wiring every layer together
├── clients/
│   ├── web/                    # browser MPEG-TS client (HTML / CSS / JS)
│   └── android/                # Material 3 Compose app
├── assets/
│   └── logo/                  # Vector SVG and PNG icon & banner set
├── data/                      # desktop entry, RPM spec, systemd service
├── scripts/                   # install, packaging (deb / rpm / AppImage), dev tooling
└── docs/                      # bilingual guides (EN + AR)
```

---

<a id="faq"></a>
## ❓ Frequently Asked Questions (FAQ)

<details>
<summary><b>Can I use my Android tablet as a true extended display, not just screen mirroring?</b></summary>
<br>
<b>Yes!</b> Unlike screen-mirroring apps, Orbiscreen creates an independent, native virtual monitor on your Linux desktop (via KDE Plasma KWin virtual display, wlroots headless output, or EVDI kernel driver). You can position it anywhere relative to your physical monitors, drag application windows to it, and adjust resolution up to 4K.
</details>

<details>
<summary><b>Does Orbiscreen work on Wayland without root permissions?</b></summary>
<br>
<b>Yes!</b> On modern Linux desktop environments like KDE Plasma (Wayland) and wlroots compositors (Sway, Hyprland), Orbiscreen creates native virtual monitors rootlessly without any kernel modules or screen-share authorization dialogs. On COSMIC, GNOME, and X11, Orbiscreen uses EVDI for kernel-level display support.
</details>

<details>
<summary><b>Can I use my tablet as a drawing tablet with pressure sensitivity in Krita or GIMP?</b></summary>
<br>
<b>Yes!</b> Orbiscreen intercepts stylus pressure (up to 4095 levels) and tilt angles from active styluses (such as Samsung S-Pen or capacitive styluses) and injects them as a genuine Linux tablet digitizer. You can sketch and paint naturally in Krita, GIMP, Blender, and Inkscape.
</details>

<details>
<summary><b>How does Orbiscreen compare to other solutions?</b></summary>
<br>
Unlike proprietary solutions restricted to specific operating systems and ecosystems, Orbiscreen delivers a high-performance, open-source extended monitor experience built specifically for Linux hosts and compatible with any Android tablet, phone, or web browser.
</details>

<details>
<summary><b>Can I connect via USB cable instead of Wi-Fi?</b></summary>
<br>
<b>Yes!</b> Orbiscreen features built-in automatic ADB reverse tunneling over USB. Simply enable USB Debugging on your Android device and plug it in; Orbiscreen automatically establishes a zero-interference USB connection within two seconds.
</details>

<details>
<summary><b>What is the latency during streaming?</b></summary>
<br>
With hardware encoding enabled (NVIDIA NVENC or Intel/AMD VA-API on Linux, and MediaCodec hardware decoding on Android), latency is imperceptible and near-zero, making it snappy and responsive for coding, reading, browsing, and media consumption.
</details>

---

<a id="documentation"></a>
## 📚 Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System topology, frame pipeline &amp; D-Bus architecture |
| [DE_SUPPORT.md](docs/DE_SUPPORT.md) | Per-desktop support matrix, capture plans &amp; troubleshooting |
| [PACKAGING.md](docs/PACKAGING.md) | Multi-distro packaging specs (.deb, .rpm, AppImage) |
| [DBUS_SPEC.md](docs/DBUS_SPEC.md) | D-Bus Session Bus IPC interface specifications |
| [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) | Common issues, diagnostics &amp; hardware acceleration fixes |

---

<a id="support"></a>
## ⭐ Support the Project &amp; Spread the Word

If Orbiscreen has made your workflow smoother or saved you from buying a costly portable monitor:

- ⭐ **Star this repository** on GitHub - every star boosts discoverability in GitHub Search and Google!
- 📢 **Share with the community** on Reddit ([r/linux](https://reddit.com/r/linux), [r/android](https://reddit.com/r/android), [r/kde](https://reddit.com/r/kde)), Mastodon, or X/Twitter.
- 🐛 **Report issues or suggest features** via [GitHub Issues](https://github.com/shadow-x78/orbiscreen/issues).
- 💡 **Contribute code or translations** via [Pull Requests](https://github.com/shadow-x78/orbiscreen/pulls).

---

<a id="contributing"></a>
## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit your changes: `orbiscreen | <type>: <description>`
4. Push to the branch
5. Open a Pull Request

Please read our [Contributing Guidelines](CONTRIBUTING.md) and <span id="coc-ov-file"></span>[Code of Conduct](CODE_OF_CONDUCT.md) for community standards, development workflow, and the release process.

---

<a id="license"></a>
## 📜 License

Distributed under the [GPL-3.0 License](LICENSE).

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Changelog](CHANGELOG.md) ·
[Code of Conduct](CODE_OF_CONDUCT.md) ([Web Tab](https://github.com/shadow-x78/orbiscreen?tab=coc-ov-file#readme)) ·
[Security](SECURITY.md)

<sub>&copy; 2026 Orbiscreen</sub>

</div>
