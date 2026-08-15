# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.11.0] - 2026-08-16

### 💥 Breaking
- **Token auth for media & control endpoints:** `/stream`, `/input`, and `/api/control` now require a per-session access token (32 random bytes, regenerated on every daemon start). Clients present it via `Authorization: Bearer <token>` or `?token=<token>`. Android clients get it from mDNS discovery TXT / `/client/config.json`; the bundled web client bootstraps from `/client/config.json`. `/health`, `/api/info`, `/client/*` remain public.
- **Keystore removed from the repository:** the Android release signing key is no longer shipped with the source (it remains in git history for anyone who pulled earlier). Builders must supply their own signing key; APKs signed with the old key have a different signature, so uninstall the old app before installing newly signed releases.
- **Dead transport paths removed:** the legacy `/ws` WebSocket and WebRTC `/sdp` signaling endpoints are gone. WebRTC is fully replaced by MPEG-TS over HTTP; `webrtc_port_range` is kept only as a compatibility placeholder.
- **`orbiscreen stop` is now real:** instead of a no-op, it asks the running daemon to shut itself down gracefully through the D-Bus session service (`Stop()`), reporting "daemon is not running" when the service is absent.
- **Android Stream toolbar:** the dead "Open files" action (no backend since the WebRTC era) was removed from the control toolbar.

### ✨ Added
- **evdi is now the primary frame source:** `orbiscreen-display` reads the real virtual-display framebuffer through the evdi DRM interface, so clients see the actual second monitor the compositor draws on (previously the capture fallback was always used). On hosts without the evdi kernel module the daemon degrades gracefully to Wayland portal / X11 capture of the primary desktop and logs a clear warning.
- **Web client:** a browser client is bundled and served by the daemon at `http://<host>:8788/`, playing MPEG-TS live via MediaSource Extensions using the locally vendored `mpegts.js` (no CDN dependency, works offline on the LAN).
- **D-Bus live status:** `GetStatus()` returns a JSON object with live fields (`running`, `frames_forwarded`, `active_clients`, `total_clients`, `encoder`, `capture_backend`); `ListClients()` reports real counts from transport stats; `GetConfig()` dumps the sanitized TOML config; `Stop()` triggers a graceful shutdown.
- **Client statistics:** the transport tracks active/total stream clients and forwarded frames, surfaced via D-Bus and used by the GTK panel.
- **`/api/control` real host tools:** lock (`loginctl`/`xdg-screensaver`), blank/unblank (DPMS via sway/hyprland/xset), and `ctrl_alt_del` injection; arbitrary URL `open` is explicitly rejected.
- **GTK panel rewrite:** the GTK4 app now talks to the daemon over zbus with live status polling, honest Stop-only behavior with toast feedback, and real settings from the daemon config.

### 🐛 Fixed
- **Stream/input dimension alignment:** pointer coordinates are scaled through the capture region reported to clients, so touch and mouse land in the right place regardless of display scaling.
- **Bounded channels:** the per-client MPEG-TS path uses a bounded channel and drops chunks for stalled consumers instead of growing daemon memory without limit; stream lag tolerates `Lagged` broadcast errors until the next keyframe instead of disconnecting.
- **Android translation breakage:** removed the dangling `onOpenFiles` reference in StreamScreen/ControlToolbar/strings.xml that kept Kotlin resources out of sync.
- **Packaging:** deb/rpm/AppImage/Flatpak specs and `install.sh` updated to match the current binary layout, client directory, and service unit.

### 🔒 Security
- Corrected SECURITY.md: the daemon binds `0.0.0.0:8788` (not `127.0.0.1`), endpoints are now token-protected (not "unauthenticated by design"), and the token model + keystore rotation are documented, including limitations against a determined LAN attacker.

### 📚 Docs
- Rewrote `docs/DBUS_SPEC.md` to match the real interface (JSON GetStatus, TOML GetConfig, systemd-only Start); added streaming/client troubleshooting guides (wrong screen via missing evdi, no picture without MSE, missing x264 plugin, 401 token flow, D-Bus service absent); refreshed READMEs for the web client, token model, and evdi requirement.

## [v0.10.7] - 2026-08-12

### 🐛 Fixed
- **Wayland portal stall (1-5 fps):** Encoder `appsrc` was missing `is-live=true` and `do-timestamp=true`. With Wayland's bursty deliverable from `pipewiresrc` and x264's `tune=zerolatency` assumptions, frames backed up in the encoder queue and never reached the appsink. Watchdog showed constant "N frames pushed" with no growth. Now the appsrc timestamps each buffer on arrival, so the encoder treats live input as a stream rather than one giant blob.
- **Stream PTS jumping:** The transport's appsrc previously had `do-timestamp=true` which overrode the encoder's real PTS values, causing mpegtsmux to see wildly out-of-order timestamps and emit nothing. Now we use the real PTS emitted by `h264parse` (after `config-interval=1` ensures SPS/PPS lands on each IDR).
- **NAL garbage filter:** Added a strict H264 start-code validator on the streaming push path so out-of-band packets never reach `gst_base_parse_handle_buffer`, eliminating the `SIGSEGV` seen when a client disconnected mid-frame.

## [v0.10.6] - 2026-08-11

### 🐛 Fixed
- **`/stream` returning 0 bytes:** Streamed MPEG-TS pipeline lacked explicit `video/x-h264` caps on `appsrc`. Without caps, `mpegtsmux` never classified incoming NAL units as keyframe-anchored video and withheld output indefinitely. Clients saw `200 OK` with an empty body.
- **Slow frame pacing (1-2 fps instead of 60):** Wayland capture pipeline lacked explicit frame size caps. The `videoscale video/x-raw,format=BGRA,width=WIDTH,height=HEIGHT` chain was missing so `pipewiresrc` negotiated its own resolution and `videoconvert` overhead ballooned. Added explicit caps plus stride-mismatch detection that drops frames whose buffer size doesn't match `W*H*4`.
- **`/api/info` and `/api/control` 404:** Routes weren't registered with the axum `Router` at all. Added fully wired handlers backed by `AppState` so the Android client can discover resolution / encoder / version and trigger host control actions.
- **SPS/PPS visibility:** Added `h264parse config-interval=1` in the encoder pipeline so parameter sets are reattached to every keyframe, letting clients that join mid-stream sync immediately.
- **Encoder push starvation:** Reduced x264 `key-int-max` to 30 (0.5 s at 60 fps) so clients joining mid-stream get a keyframe within half a second instead of waiting 8+ s for the next IDR.
- **Android launcher icon (legacy PNGs):** Regenerated all five density PNGs from the freshly restyled vector so the install screen and pre-Oreo launchers see the same shape as Android 8+ adaptive icons.
- **Android launcher icon (adaptive):** Artwork group scaled by 1.08 in `ic_launcher_foreground.xml` around the canvas centre so the brand mark reads bigger at small sizes without clipping the safe zone.

## [v0.10.5] - 2026-08-11

### 🐛 Fixed
- **Android Launcher Icon Distortion:** The previous `ic_launcher_foreground.xml` used corner-radius arcs (`A24,24`) on the monitor outline which the VectorDrawable renderer stretched, making the icon look rubbery and off from the brand SVG. Rebuilt all pathData with explicit `h`/`v`/`a` commands matching `data/orbiscreen-app.svg` exactly (monitor fill `#1E1E2E` with blue stroke, inner screen bezel, white accent arc, blue stand neck, slate base). Art now scales to 0.50 within a centered `(128, 124)` translate so it sits perfectly inside the adaptive safe zone.
- **Legacy Launcher PNGs:** Regenerated all `mipmap-*dpi/ic_launcher.png` from the corrected vector so the install-time icon (used by launchers that don't support adaptive icons) renders sharp and non-stretched.
- **x264 `repeat-headers` Crash:** Setting the property unconditionally panicked on newer GStreamer builds where `GstX264Enc` no longer exposes it. Wrapped in `find_property(...).is_some()` since `tune=zerolatency` already enables repeat headers internally.

## [v0.10.4] - 2026-08-11

### 🐛 Fixed
- **Android Launcher Icon Background:** Adaptive icons (`ic_launcher.xml`, `ic_launcher_round.xml`) used `@android:color/transparent` so the launcher showed a black square around the artwork on many OEM launchers - switched to a real `@color/ic_launcher_background` (#FFFFFF) resource.
- **Android Launcher Icon Oversized:** Foreground scale reduced from `0.66` to `0.56` inside `ic_launcher_foreground.xml`, keeping the artwork inside the adaptive safe zone so it renders smaller and non-intrusive after install.

### 🔧 Changed
- **Legacy Launcher PNGs:** Regenerated every `mipmap-*dpi/ic_launcher.png` from `data/orbiscreen-app.svg` on a white background with properly scaled artwork (was full-bleed on transparent).
- **README Restyle:** Reworked both `README.md` and `README_AR.md` to the concise project pattern (comparison table, highlights list, screen-summary table for the Android app instead of long prose, full uninstall command documented).
- **Docs Typography:** Replaced all em dashes (`—`) with standard hyphens across `README*.md`, `CHANGELOG.md`, `SECURITY.md`, and `docs/*.md` for a cleaner, tool-agnostic documentation style.

## [v0.10.3] - 2026-08-10

### 🐛 Fixed
- **EDID Range-Limits Descriptor:** Write the `0xFD` tag at byte 72 (descriptor start) instead of byte 75, so virtual-monitor EDID blocks validate against strict DRM parsers.
- **Mouse Buttons Dropped:** Register `BTN_LEFT..=BTN_TASK` with the uinput device - clicks were silently discarded because only keyboard codes were declared.
- **Wayland Session Leak:** Close the `RemoteDesktop` portal session on drop so the compositor's screen-sharing indicator disappears with the daemon.
- **D-Bus Service Exit:** Keep the D-Bus task alive with a parked future instead of returning `Ok(())` immediately and dropping the connection.
- **Stream Pipeline Panics:** Return `503 SERVICE_UNAVAILABLE` from `/stream` when the GStreamer pipeline can't be built, instead of panicking the axum worker.
- **MPEG-TS Packet Panics:** Replace `unwrap()` calls inside the per-client broadcast task with graceful `warn + break`, so a lagging client can't crash the daemon.
- **mDNS Fullname:** Build the real service fullname (`instance._orbiscreen._tcp.local.`) for `unregister()` so the record is actually removed when the advertiser drops.
- **ADB Scan Robustness:** Skip malformed `adb devices` lines instead of aborting the scan on the first one.
- **Config Hardening:** Add `#[serde(default)]` to all config sections and clamp degenerate values (`fps=0` divide-by-zero, `bitrate` overflow, inverted port ranges, `count=0`).
- **Encoder Bitrate Units:** Pass kbit/s verbatim to VAAPI/NVENC instead of multiplying by 1000 (hardware encoders silently received a 1000× too-high target).
- **Encoder Resource Leaks:** Send EOS before `State::Null` so tail frames flush, cap the appsrc queue (~8 MB), and `Drop`-shutdown the pipeline cleanly.
- **Wheel/Key/Input Bounds:** Round wheel deltas instead of truncating, clamp stylus tilt to ±90°, reject `u16`-overflowing key codes, and `saturating_sub` in `clamp_point` for zero-size specs.
- **Coordinate Scaling:** Scale raw `{x,y}` pointer payloads through the capture region (the stream's source of truth) so hand-written clients land in the right place.

### 📱 Android
- **Wire Protocol Alignment:** `InputDispatcher` now sends the tagged envelopes the daemon actually deserializes (`{"Pointer":{...}}`, `{"Key":{...}}`, `{"Stylus":{"Tilt":{...}}}`) - previously wheel/stylus/key events failed to parse.
- **Linux Key Codes:** The soft keyboard emits evdev codes (`a`=30, space=57, enter=28, backspace=14) instead of Android `KeyEvent` codes, which the host rejected.
- **Stream URL:** Point ExoPlayer at `/stream` (the served route) instead of the nonexistent `/stream.ts?fmt=mp2t`.
- **NSD Lifecycle:** `DiscoveryService.stop()` actually cancels the coroutine scope so `stopServiceDiscovery` runs; refresh no longer leaks discovery sessions.
- **Host:Port Validation:** Reject out-of-range ports and IPv4 octets > 255 in the manual-entry field.
- **Launcher Icon:** White-background adaptive launcher for SDK < 26 (PNG fallbacks), vector foreground scaled into the safe zone for SDK 26+.

### 🌐 Web Client
- **Protocol:** Rewrite `app.js` to send the same tagged `Pointer`/`Key`/`Stylus` envelopes as Android; delete the dead WebRTC `/sdp` negotiation path (`/sdp` always 503s).
- **Pointer Mapping:** Correct for `object-fit: contain` letterboxing so taps land where the video renders, not where the element is.
- **Pointer Capture:** Add `setPointerCapture` + `pointercancel` so drags past the video edge don't leave host buttons stuck.
- **Key Translation:** Map DOM `KeyboardEvent.code` to Linux evdev codes (1:1 table) instead of the broken `keyCode` numerical passthrough.

### 🔧 Changed
- **Icons:** `data/orbiscreen.svg` and `data/orbiscreen-app.svg` now ship a white rounded-rect background; all raster icons (Linux `data/*.png`, every Android `mipmap-*/ic_launcher.png`) are regenerated from the SVG with a white background.
- **AppImage Builder:** Replace the hand-painted placeholder PNG with a `magick` rasterization of the real logo (white background).
- **Docs:** English docs and new `README_AR.md` / `docs/*_AR.md` now follow the reference style (ASCII banner, hex Tailwind badges, `Language` switcher, anchored sections, centered footer).

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
