# Changelog

All notable changes to **Orbiscreen** are documented here.

## [v0.4.9] - 2026-07-23

### 🚀 Added
- **Official Keystore Tracking & Release Build Fallback:** Included `orbiscreen-release.keystore` directly in the release pipeline and added fallback checks in `build.gradle.kts` to ensure Android release builds compile cleanly.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.4.8` → `0.4.9`.

---

## [v0.4.8] - 2026-07-23

### 🚀 Added
- **Official Keystore Release Signing (`orbiscreen-release.keystore`):** Created a dedicated Android Release Keystore and configured `signingConfigs` in `build.gradle.kts` so `orbiscreen-android-release.apk` is signed with a standalone release key to bypass Play Protect warnings.
- **Transparent Android Launcher Icons:** Updated `data/orbiscreen-app.svg` and all Android mipmap launcher icons (`mdpi` through `xxxhdpi`) to be 100% transparent without background boxes.

### 🔧 Fixed
- **README Header Proximity & Zero-Gap Alignment:** Restructured README HTML header in both `README.md` and `README_AR.md` to eliminate vertical paragraph gap above `# Orbiscreen` title and set logo width to `180px`.
- **Arabic Documentation Index Alignment:** Cleaned up `README_AR.md` documentation index table to feature Arabic-localized documentation resources exclusively.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.4.7` → `0.4.8`.

---

## [v0.4.7] - 2026-07-23

### 🚀 Added
- **Automated Multi-Distro Release Packaging Matrix:** Added packaging build scripts (`scripts/package-deb.sh`, `scripts/package-rpm.sh`, `scripts/package-appimage.sh`) and updated `.github/workflows/release.yml` to automatically build and attach `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL/openSUSE), `.AppImage`, `.tar.gz` (standalone Linux tarball), `orbiscreen-android-release.apk` (signed release build), and `orbiscreen-android-debug.apk` directly to GitHub Releases.
- **Global Documentation Index & Roadmap Navigation:** Added comprehensive documentation index tables and distribution-specific installation commands to both `README.md` and `README_AR.md`, linking directly to `ARCHITECTURE.md`, `PACKAGING.md`, `DBUS_SPEC.md`, `TROUBLESHOOTING.md`, and `SECURITY.md` (English & Arabic).

### 🔧 Fixed
- **Status Standardization & Compact Branding:** Standardized all phase status labels across `README.md`, `README_AR.md`, `implementation_plan.md`, and `walkthrough.md` to use consistent `✅ Completed` terminology (eliminating mixed `Closed` / `Completed` labels). Reduced README logo width to `140px` and styled bottom margin to remove excessive whitespace gap.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.4.6` → `0.4.7`.

---

## [v0.4.6] - 2026-07-23

### 🚀 Added
- **Signed Android Release Artifact (`orbiscreen-android-release.apk`):** Configured `signingConfig` in `build.gradle.kts` and automated release APK packaging in `.github/workflows/release.yml` so Android releases are signed to bypass Google Play Protect installation warnings.

### 🔧 Fixed
- **Android Cleartext HTTP Connection (`android:usesCleartextTraffic="true"`):** Added cleartext traffic permission to `AndroidManifest.xml` so WebRTC/HTTP streams to local Orbiscreen Linux host IP addresses connect without Android 9+ network security blocks.

### 🔄 Updated
- **Logo Breathing Room & Padding:** Balanced `viewBox="0 0 512 480"` for 24px breathing room margins around the original SVG logo so it displays comfortably without clipping or suffocated borders.
- **Version Bump:** All version sources bumped from `0.4.5` → `0.4.6`.

---

## [v0.4.5] - 2026-07-23

### 🚀 Added
- **Ultra-crisp Vector SVG Logo & Brand Identity:** Designed a clean, high-grade vector SVG logo (`data/orbiscreen.svg` and `data/orbiscreen-app.svg`) representing the Linux Host Display, Android Secondary Display, and glowing wireless WebRTC/Axum stream bridge using Catppuccin Blue (`#89b4fa`), Mauve (`#cba6f7`), Rosewater (`#f5e0dc`), and Surface palette (`#11111b`, `#1e1e2e`).
- **Crisp Multi-Density Icons:** Replaced raster image with crisp vector SVG renderings for README headers and Android launcher icons across all mipmap densities.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.4.4` → `0.4.5`.

---

## [v0.4.4] - 2026-07-23

### 🚀 Added
- **Professional Icon Design:** New Catppuccin-themed icon with monitor and orbital arcs, applied to README headers, Android launcher icons (all densities), and `.desktop` entry.
- **Android Launcher Icons:** Generated `ic_launcher.png` for all Android mipmap densities (`mdpi` through `xxxhdpi`) and configured `AndroidManifest.xml`.

### 🔧 Fixed
- **GitHub Release Workflow:** Fixed `permissions: contents: write` in `.github/workflows/release.yml` to resolve the 403 "Resource not accessible by integration" error when creating releases.

### 🔄 Updated
- **README & README_AR:** Replaced ASCII art header with the new icon image.
- **Version Bump:** All version sources bumped from `0.4.3` → `0.4.4`.

---

## [v0.4.3] - 2026-07-23

### 🚀 Added
- **World-Class System Architecture Specifications:** `docs/ARCHITECTURE.md` & `docs/ARCHITECTURE_AR.md` — published bilingual system topology specifications, workspace crate breakdown, and zero-copy streaming pipeline diagrams.
- **Bilingual Documentation Pairing:** Added English/Arabic `_AR` bilingual documentation pairs for all documentation (`SECURITY_AR.md`, `DBUS_SPEC_AR.md`, `PACKAGING_AR.md`, `ARCHITECTURE_AR.md`).

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.4.2` → `0.4.3` (`Cargo.toml` workspace version, README + README_AR badges).

---

## [v0.4.2] - 2026-07-23

### 🚀 Added
- **Multi-Distro Release Matrix Workflow:** `.github/workflows/release.yml` — automated release packaging workflow triggered on `push: tags: ['v*']` that compiles and attaches Linux tarballs (`orbiscreen-linux-x86_64.tar.gz`) and Android APKs (`orbiscreen-android-debug.apk`) directly to GitHub Releases.
- **Multi-Distro Packaging Documentation:** `docs/PACKAGING.md` & `docs/PACKAGING_AR.md` — published complete packaging guides for AppImage, Flatpak, Debian/Ubuntu (.deb), Fedora/RHEL (.rpm), and Android.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.4.1` → `0.4.2` (`Cargo.toml` workspace version, README + README_AR badges).

---

## [v0.4.1] - 2026-07-23

### 🚀 Added
- **GTK4 / Libadwaita Desktop Control Panel GUI:** `crates/orbiscreen-gtk` — created native Linux GTK4 control panel application providing desktop GUI for daemon status, resolution selection, and transport monitoring.
- **Desktop Entry & Application Icon:** `data/com.orbiscreen.OrbiscreenGtk.desktop` & `data/orbiscreen.svg` — added native desktop integration files.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.4.0` → `0.4.1` (`Cargo.toml` workspace version, README + README_AR badges).

---

## [v0.4.0] - 2026-07-23

### 🚀 Added
- **D-Bus Session Service Interface:** `orbiscreen-daemon` — implemented `com.orbiscreen.Daemon` D-Bus session service via `zbus` crate (`Start`, `Stop`, `GetStatus`, `ListClients`, `GetConfig`), providing background IPC for GTK4 GUI and system tray indicators.
- **D-Bus Specification Document:** `docs/DBUS_SPEC.md` — published detailed D-Bus API specification and `busctl` command usage examples.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.3.2` → `0.4.0` (`Cargo.toml` workspace version, README + README_AR badges).

---

## [v0.3.2] - 2026-07-23

### 🚀 Added
- **Touch Calibration & Scaling:** `orbiscreen-input` — added `TouchCalibration` helper for coordinate clamping and scaling across High-DPI Android screen displays.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.3.1` → `0.3.2` (`Cargo.toml` workspace version, README + README_AR badges).

---

## [v0.3.1] - 2026-07-23

### 🚀 Added
- **Android Release Artifact Publishing:** `.github/workflows/android.yml` — updated workflow to build and publish the Android APK artifact (`orbiscreen-android-debug`) on every `main` push and manual dispatch.
- **Standalone Linux Install Script:** `scripts/install.sh` — added one-command installation script for Linux daemon binary and `orbiscreen.service` systemd user unit.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.3.0` → `0.3.1` (`Cargo.toml` workspace version, README + README_AR badges).

---

## [v0.3.0] - 2026-07-23

### 🚀 Added
- **Wayland Desktop Portal Auto-Fallback:** `orbiscreen-daemon` — gracefully falls back to Wayland/X11 portal display capture when the EVDI kernel module is absent, enabling instant execution on any Wayland desktop (GNOME, KDE, Sway) without requiring custom kernel driver compilation.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.2.3` → `0.3.0` (`Cargo.toml` workspace version, README + README_AR badges).

---

## [v0.2.3] - 2026-07-23

### 🚀 Added
- **Auto ADB Reverse Port Forwarding:** `orbiscreen-transport` — automatically configures `adb reverse tcp:8788 tcp:8788` for connected USB Android devices during transport startup.

### ✨ Android Client & CI
- **APK CI Artifact:** `clients/android` — verified full debug APK assembly and GitHub Actions release artifact publication (`orbiscreen-android-debug`).

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.2.2` → `0.2.3` (`Cargo.toml` workspace version, README + README_AR badges).

---

## [v0.2.2] - 2026-07-23

### 🚀 Added
- **WebRTC SDP Signaling:** `orbiscreen-transport` — updated `/sdp` POST handler for RTC SDP offer/answer exchange and video session initialization.

### ⚙️ CI & Workflow
- **Node 20 Warning Fix:** `.github/workflows/android.yml` — added `FORCE_JAVASCRIPT_ACTIONS_TO_NODE20: true` environment variable to suppress runner deprecation warnings.
- **Android Gradle Assets:** `clients/android` — fixed asset merger duplication in `app/build.gradle.kts` and `strings.xml`.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.2.1` → `0.2.2` (`Cargo.toml` workspace version, README + README_AR badges).

---

## [v0.2.1] - 2026-07-23

### 🚀 Added
- **Direct HTTP Stream Endpoint:** `orbiscreen-transport` — `/stream` GET handler streaming real-time H.264 video chunks (`tokio::sync::broadcast` + `axum::body::Body`).

### ✨ Features
- **Web Client Fallback:** `clients/web` — updated `app.js` with auto-fallback to `/stream` for zero-configuration playback on local networks.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.2.0` → `0.2.1` (`Cargo.toml` workspace version, README + README_AR badges).

---

## [v0.2.0] - 2026-07-23

### 🚀 Added
- **WebRTC SDP Signaling:** `orbiscreen-transport` — `/sdp` POST handler for WebRTC SDP offer/answer exchange with browser and Android clients.
- **WebSocket Input Stream:** `orbiscreen-transport` — `/ws` handler for bidirectional signaling and real-time touch event forwarding.
- **Input Endpoint:** `orbiscreen-transport` — `/input` POST route accepting JSON touch, pointer, and keyboard payloads.

### ✨ Features
- **End-to-End Input Pipeline:** `orbiscreen-input` & `orbiscreen-daemon` — derived `serde::Serialize` / `serde::Deserialize` on `PointerEvent`, `StylusEvent`, and `KeyEvent`. Wired `input_tx` channel in daemon main loop directly into `InputInjector` for `/dev/uinput` event dispatch.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.1.1` → `0.2.0` (`Cargo.toml` workspace version, README + README_AR badges).

---

## [v0.1.1] - 2026-07-23

### 🐛 Fixed
- **ASCII Logo Alignment:** `README.md` — removed the stray leading space on the top and bottom rows of the ASCII logo block that caused a visible right-shift when the `<pre>` block was centered.

### 📝 Documentation
- **README_AR.md Parity:** `README_AR.md` — restructured to mirror `README.md` end-to-end: identical badge layout, anchored TOC (`<a id>`), emoji section headers, table vs. bullet-list formats, `---` separators, and centered `<sub>&copy;</sub>` footer.
- **TROUBLESHOOTING Listed:** `README.md` / `README_AR.md` — added `docs/TROUBLESHOOTING.md` to the Documentation table (was orphaned, only referenced inside the file itself).

### 🗑️ Removed
- **Branch Protection Docs:** `docs/BRANCH_PROTECTION.md` and `docs/BRANCH_PROTECTION_AR.md` deleted. These documented maintainer-only GitHub repo admin workflows (solo-dev relax-and-restore, `restrictions: null` quirks) and were not useful to end users or external contributors.

### 🔄 Updated
- **Version Bump:** All version sources bumped from `0.1.0` → `0.1.1` (`Cargo.toml` workspace version, README + README_AR badges, TROUBLESHOOTING + TROUBLESHOOTING_AR badges).

---

## [v0.1.0] - 2026-07-22

First public release. The codebase is structured as a single Rust workspace containing seven crates, with a CLI daemon binary that wires every layer together. This release ships **scaffolded** layers for all five phases (display, capture, encode, input, transport) plus an Android WebView client, packaging manifests for Flatpak / AppImage / .deb, and a browser WebRTC client. See `SECURITY.md` for the current threat model.

### 🚀 Added
- **Workspace:** 7 crates under `crates/` — `orbiscreen-{core,display,capture,encode,input,transport,daemon}`. Binary daemon: `cargo run -p orbiscreen-daemon -- start|probe|list-displays|print-config`. 38 unit tests passing on `cargo test --workspace`.
- **Display Layer:** `orbiscreen-display` — `evdi` kernel module + libevdi backend, synthesized EDID 1.4 for non-1080p modes with checksum validation.
- **Capture Layer (X11):** `orbiscreen-capture` — `x11rb` Z Pixmap GetImage wrapped in `tokio::task::spawn_blocking`.
- **Capture Layer (Wayland):** `orbiscreen-capture` — `ashpd::Screencast` + PipeWire fd scaffold; DMA-BUF frame path returns placeholder black frames pending Phase 3.
- **Encode Layer:** `orbiscreen-encode` — GStreamer pipeline `appsrc → videoconvert → <enc> → appsink` with auto-discovery of VAAPI, x264, NVENC at runtime.
- **Input Layer (X11):** `orbiscreen-input` — `evdevil` (uinput) with stylus pressure (`ABS_PRESSURE` 0–1024) and tilt axes (`ABS_TILT_*` ±90°).
- **Input Layer (Wayland):** `orbiscreen-input` — `ashpd::RemoteDesktop` + libei scaffolded for Phase 3 portal integration.
- **Transport Layer:** `orbiscreen-transport` — `axum` HTTP + WebSocket, static client serving, mDNS (`_orbiscreen._tcp.local.`), `adb reverse` bootstrap.
- **Browser Client:** `clients/web/` — HTML + CSS + JS WebRTC client with DataChannel input (pointer/keyboard/stylus pressure + tilt).
- **Android Client:** `clients/android/` — Kotlin WebView host (`com.orbiscreen.android`); bundles the web client at build time via Gradle `syncWebClient` task.
- **Packaging:** Flatpak manifest (`io.github.shadow-x78.orbiscreen.json`), AppImage build script (no ImageMagick dependency), Debian `.deb` package (`packaging/debian/control` + `build-deb.sh`).
- **CI Workflows:** `.github/workflows/{ci,android}.yml` — fmt + clippy + build + test on every PR; Android APK build verification.
- **Dependabot:** `.github/dependabot.yml` — automated cargo + github-actions dependency updates.
- **Issue Templates:** `.github/ISSUE_TEMPLATE/{bug,feature}.yml` — pre-formatted reports.
- **PR Template:** `.github/PULL_REQUEST_TEMPLATE.md` — submission checklist.
- **License Allowlist:** `deny.toml` — cargo-deny license allowlist (includes LGPL-2.1 for `libevdi`, GPL-3.0 for `evdi` kernel module references).
- **Security Policy:** `SECURITY.md` — threat model + local-network-only disclosure scope.

### 📝 Documentation
- **README.md:** Centered badge block (flat-square multi-color palette), emoji TOC with `<a id>` anchors, `---` separators, centered `<sub>&copy;</sub>` footer.
- **README_AR.md:** Full Arabic translation mirroring the English README.
- **CHANGELOG.md:** UMO-style — emoji section headers (`✨`, `🚀`, `📝`, `🐛`, `🔄`, `🗑️`), `[vX.Y.Z] - YYYY-MM-DD` versioning, no horizontal rules between versions.
- **TROUBLESHOOTING.md / TROUBLESHOOTING_AR.md:** Common issues, evdi setup, capture failures, debugging.

### 🔐 Security
- Local network only in v1 — no cloud relay, no TURN server, no telemetry.
- `evdi` kernel module loading is the operator's responsibility (DKMS + Secure Boot signing per distro).

### 🧹 Cleanup
- **Renamed:** `client-web/` → `clients/web/`, `client-android/` → `clients/android/`; Android Gradle `syncWebClient` task copies the web client at build time (single source of truth).
- **Rust Sources:** Stripped all inline `//` non-doc comments and verbose `///`/`//!` rustdoc; replaced with 2-line UMO-style file headers.
- **Authorship:** Owner / authorship unified to `shadow-x78`.
- **`.gitattributes`, `.editorconfig`:** Slimmed to the UMO pattern (no extraneous section comments, no unrelated indentation overrides).
- **`.gitignore`:** Reduced to the runtime / build artifact categories that matter.

### ⚠️ Not Yet Implemented (Honest Stubs)
The crates, HTTP routes, and types are all present and the workspace compiles + tests pass, but two integrations are explicitly stubbed pending a `webrtc-rs` pre-release upgrade:
- **WebRTC Peer Connection:** `Transport::sdp_post` returns `503` — `webrtc-rs` 0.20 is still RC-only.
- **Wayland Capture DMA-BUF Frame Path:** Returns placeholder black frames; the PipeWire node ID + fd are correctly acquired via `OpenPipeWireRemote`, but piping the DMA-BUF to GStreamer `pipewiresrc` requires a future phase.
- **Native Android WebRTC Client:** Current Android client uses a WebView wrapping the browser client; a native `org.webrtc` client is the next milestone.

### ❌ Out of Scope
- **iOS / iPhone / Safari:** No PRs adding iOS clients or Safari workarounds will be accepted.
