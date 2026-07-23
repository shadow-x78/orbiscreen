# Changelog

All notable changes to **Orbiscreen** are recorded here.

## [0.1.0] - 2026-07-22 (Initial Public Release)

First public release. The codebase is structured as a single Rust workspace containing seven crates, with a CLI daemon binary that wires every layer together. This release ships **scaffolded** layers for all five phases (display, capture, encode, input, transport) plus an Android WebView client, packaging manifests for Flatpak / AppImage / .deb, and a browser WebRTC client. See `SECURITY.md` for the current threat model.

### Workspace
- 7 crates under `crates/`: `orbiscreen-{core,display,capture,encode,input,transport,daemon}`
- Single workspace version (currently `0.1.0`)
- Binary daemon: `cargo run -p orbiscreen-daemon -- start|probe|list-displays|print-config`
- 38 unit tests passing on `cargo test --workspace`

### 🚀 Highlight: All 5 layers scaffolded end-to-end

| Layer | Backend | Crate | Notes |
|-------|---------|-------|-------|
| **Display** | `evdi` kernel module + libevdi | `orbiscreen-display` | Synthesized EDID 1.4 for non-1080p; checksum-validated |
| **Capture** (X11) | `x11rb` Z Pixmap GetImage | `orbiscreen-capture` | Wrapped in `tokio::task::spawn_blocking` |
| **Capture** (Wayland) | `ashpd::Screencast` + PipeWire fd | `orbiscreen-capture` | Scaffolded; DMA-BUF frame path returns placeholder black frames |
| **Encode** | GStreamer `appsrc → videoconvert → <enc> → appsink` | `orbiscreen-encode` | Auto-discovers VAAPI, x264, NVENC at runtime |
| **Input** (X11) | `evdevil` (uinput) | `orbiscreen-input` | Stylus pressure (`ABS_PRESSURE` 0–1024) + tilt axes (`ABS_TILT_*` ±90°) |
| **Input** (Wayland) | `ashpd::RemoteDesktop` + libei | `orbiscreen-input` | Scaffolded for Phase 3 portal integration |
| **Transport** | `axum` HTTP + WebSocket | `orbiscreen-transport` | Static client serving + mDNS (`_orbiscreen._tcp.local.`) + `adb reverse` bootstrap |

### 🎒 Clients

- **Browser**: `clients/web/` - HTML + CSS + JS WebRTC client with DataChannel input (pointer/keyboard/stylus pressure + tilt)
- **Android**: `clients/android/` - Kotlin WebView host (`com.orbiscreen.android`); bundles the web client at build time

### 📦 Packaging

- **Flatpak** manifest (`io.github.shadow-x78.orbiscreen.json`)
- **AppImage** build script (no ImageMagick dependency)
- **Debian** `.deb` package (`packaging/debian/control` + `build-deb.sh`)

### 📋 GitHub Infrastructure

- `.github/workflows/{ci,android}.yml` - CI for fmt + clippy + build + test on every PR; Android APK build verification
- `.github/dependabot.yml` - automated dependency updates (cargo + github-actions)
- `.github/ISSUE_TEMPLATE/{bug,feature}.yml` - pre-formatted reports
- `.github/PULL_REQUEST_TEMPLATE.md` - submission checklist
- `deny.toml` - cargo-deny license allowlist (includes LGPL-2.1 for `libevdi`, GPL-3.0 for `evdi` kernel module references)

### 🔐 Security & Privacy

- Local network only in v1 (no cloud relay, no TURN server, no telemetry)
- `evdi` kernel module loading is the operator's responsibility (DKMS + Secure Boot signing per distro)
- See `SECURITY.md` for full threat model

### ⚠️ Not Yet Implemented (Honest Stubs)

The crates, HTTP routes, and types are all present and the workspace compiles + tests pass, but two integrations are explicitly stubbed pending a `webrtc-rs` pre-release upgrade:

- **WebRTC peer connection** (`Transport::sdp_post` returns `503`) - `webrtc-rs` 0.20 is still RC-only
- **Wayland capture DMA-BUF frame path** - returns placeholder black frames; the PipeWire node ID + fd are correctly acquired via `OpenPipeWireRemote`, but piping the DMA-BUF to GStreamer `pipewiresrc` requires a future phase
- **Native Android WebRTC client** - current Android client uses a WebView wrapping the browser client; a native `org.webrtc` client is the next milestone

### ❌ Explicitly Out of Scope

- **iOS / iPhone / Safari** support is NOT in this project - full stop. No PRs adding iOS clients or Safari workarounds will be accepted.

### 🧹 Cleanup & GitHub Readiness (delivered as part of v0.1.0)

- Renamed `client-web/` → `clients/web/` and `client-android/` → `clients/android/`; Android Gradle `syncWebClient` task copies the web client at build time (single source of truth)
- Stripped all inline `//` non-doc comments and verbose `///`/`//!` rustdoc from Rust sources; replaced with 2-line UMO-style file headers (no inline commentary - code is self-documenting)
- Markdown (`README.md`, `README_AR.md`, `CHANGELOG.md`, `SECURITY.md`) rewritten in the UMO reference style: centered badge blocks (flat-square multi-color palette), emoji TOC with `<a id>` anchors, `---` separators, centered `<sub>&copy;</sub>` footer
- Owner / authorship unified to `shadow-x78` (matching the UMO reference project convention of a single named owner)
- `.gitattributes`, `.editorconfig` slimmed to the UMO pattern (no extraneous section comments, no unrelated indentation overrides)
- Reduced `.gitignore` to the runtime / build artifact categories that matter

---

## [Unreleased]

Nothing yet. Next commit lands here.

[0.1.0]: https://github.com/shadow-x78/orbiscreen/releases/tag/v0.1.0

