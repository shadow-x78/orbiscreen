# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.10.2] - 2026-07-27

### 📱 Android
- **Brand Palette:** Replace Material You dynamic color with Catppuccin Mocha / Latte palettes matching `data/orbiscreen-app.svg` so app chrome (status/nav bars, splash, launcher icon background) matches the logo.
- **Splash Screen & Adaptive Launcher:** Add a SplashScreen with brand background and a vector adaptive launcher foreground mirroring the SVG.
- **Connect-time Crash Fix:** Move ExoPlayer construction to the Main thread (previously `playerHolder.build()` ran inside `withContext(Dispatchers.IO)`, which killed the process the moment the user tapped Connect).
- **Error Hardening:** Harden `build()` against construction failures so they surface as `StreamEvent.Error` instead of crashing.

### 🔄 Updated
- **Version Bump:** Bump Cargo workspace version to `0.10.2` and Android `versionName` to `0.10.2` (`versionCode = 10`).
- **Documentation:** Update all documentation files (`README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `docs/*`) to `v0.10.2`.

## [v0.10.1] - 2026-07-27

### 🔄 Updated
- **Workspace Bump:** Bump workspace `Cargo.toml` version to `0.10.1`.
- **Android Bump:** Bump Android `versionName` to `0.10.1`.
- **Cargo.lock:** Refresh `Cargo.lock` path-crate entries for version `0.10.1`.

## [v0.10.0] - 2026-07-27

### ✨ Added
- **Material 3 UI:** Migrate Android client to Material 3 + Jetpack Compose.
- **Live Discovery:** Add Spacedesk-style live NSD scan with manual `host:port` entry.
- **Subnet Scanner:** Add optional subnet scanner for networks without mDNS.
- **Recent Host:** Persist and surface recent host with a Material chip.
- **Absolute Input:** Replace delta-based `TouchInjector` with absolute `InputDispatcher` supporting multi-touch, wheel, stylus, and keyboard events.
- **Control Toolbar:** Add host control toolbar overlay (lock, blank, ctrl-alt-del, files, retry).
- **Soft Keyboard:** Add soft keyboard overlay with system IME handoff.
- **Settings Screen:** Add settings screen with theme, decoder, scanner, and recent host options.

### 🐛 Fixed
- **Black Screen:** Fix black screen on connect by setting explicit MPEG-TS MIME type and using `OkHttpDataSource` with zero read-timeout.
- **Error Handling:** Surface ExoPlayer errors through `StreamEvent.Error` instead of silent black surface.

### 🔄 Updated
- **Build & Dependencies:** Add Compose BOM, `navigation-compose`, `material-icons-extended`, proguard rules for Media3/OkHttp/Compose/NSD, and opt-in lint rules.
- **Documentation:** Comprehensive documentation update across `README.md`, `docs/ARCHITECTURE.md`, `DBUS_SPEC.md`, `INSTALL.md`, `PACKAGING.md`, `TROUBLESHOOTING.md`, and `SECURITY.md`.

## [v0.9.0] - 2026-07-26

### ✨ Added
- **Artifact Signing:** Implement universal artifact signing for Android release APK (production keystore) and Linux packages (RPM, DEB, AppImage GPG keys).

### 📝 Documentation
- Document signing process in `SECURITY.md`.

## [v0.8.7] - 2026-07-26

### ⚡ Optimized
- **Native Player Pipeline:** Replace WebView/WebRTC path with native GStreamer + ExoPlayer.
- **Zero-Config Discovery:** Add mDNS auto-discovery on Android.
- **Touch Injection:** Stream touch events into Linux Wayland compositor via `uinput` and `evdev`.

### 🐛 Fixed
- **Portal Capture Fallback:** Drop failing EVDI dependency and use XDG Desktop Portal for capture.
- **Build & Formatting:** Resolve `cargo fmt`, `clippy`, and Android Kotlin compilation errors.

## [v0.7.4] - 2026-07-25

### ✨ Added
- **Uninstall Command:** Add `orbiscreen uninstall` command for clean removal.

### 🐛 Fixed
- **Web Client Assets:** Bundle web client files (`index.html`, `app.js`, `style.css`) in packages and resolve fallback paths across `~/.local/share`, `/usr/share`, `/app/share`.

## [v0.7.3] - 2026-07-25

### ⚡ CI & Packaging
- Inject dynamic packaging versions and generate SHA256 checksums for all release artifacts.

### 📝 Documentation
- Reformat `README.md`, `ARCHITECTURE.md`, `DBUS_SPEC.md`, `PACKAGING.md`, `TROUBLESHOOTING.md`.

## [v0.7.2] - 2026-07-24

### 🐛 Fixed
- **CPU Loop:** Rate-limit capture frame pump with `tokio::time::sleep` to prevent 100% CPU usage.

## [v0.7.1] - 2026-07-24

### 🐛 Fixed
- **Android Launch Crash:** Wrap `MainActivity` init in try/catch and fix malformed HTML in WebView fallback.
