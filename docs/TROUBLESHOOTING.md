<div align="center">

# Troubleshooting - Orbiscreen

[![Version](https://img.shields.io/badge/version-0.20.0-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 Language

<a href="TROUBLESHOOTING.md">🇬🇧 English</a> · <a href="TROUBLESHOOTING_AR.md">🇸🇦 العربية</a>

---

## 📋 Table of Contents

### CI Workflow Actions (`.github/workflows/ci.yml`)

- [Action: `Check formatting` (`cargo fmt --all -- --check`)](#ci-fmt)
- [Action: `Clippy (deny warnings)` (`cargo clippy --workspace --all-targets --locked -- -D warnings`)](#ci-clippy)
- [Action: `Build` (`cargo build --workspace --locked`)](#ci-build)
- [Action: `Test` (`cargo test --workspace --locked`)](#ci-test)
- [Action: `Run cargo-deny` (`cargo deny check`)](#ci-deny)
- [Action: `Android assembleDebug` + `lintDebug`](#ci-android)

### Runtime

- [Runtime: `orbiscreen start` fails - `kernel module is not installed`](#runtime-evdi)
- [Runtime: KDE Plasma (virtual display without evdi or root)](#runtime-kwin)
- [Runtime: capture backend unavailable on Wayland](#runtime-wayland)
- [Runtime: `unsafe_op_in_unsafe_fn` / `missing_debug_implementations` lint warnings](#runtime-lints)

### Android

- [Android / ChromeOS: ADB connection fails on ASUS Chromebook CM3001](#android-chromebook-adb)
- [Android: Stylus / Pen not drawing, incorrect pressure, or app crash on Lenovo Tab](#android-stylus)
- [Android: Dragging windows or selecting files in Touchpad mode](#android-touchpad-drag)
- [Android: app crashes or process dies when tapping Connect](#android-connect-crash)
- [Android: black screen after Connect](#android-black-screen)
- [Android: discovery list is empty even though hosts are on the same Wi-Fi](#android-no-hosts)
- [Android: touch is rotated / misaligned](#android-touch-offset)
- [Android: control toolbar actions return 404](#android-control-404)
- [Android: app crashes immediately on launch](#android-crash)
- [Android: USB connection shows "Looking for host…"](#android-usb)

### Streaming & Clients

- [Streaming: High latency, stutter, or slow mouse movement on 5GHz Wi-Fi](#streaming-wifi-latency)
- [Streaming: Stream error causes infinite reconnect flicker instead of detecting disconnect](#stream-disconnect-retry)
- [Multi-Monitor / X11: Mouse cursor escapes virtual display to other physical screens](#cursor-clamping)
- [Client shows the wrong screen (primary desktop instead of virtual display)](#wrong-screen)
- [Web client loads but shows no picture](#web-no-picture)
- [No encoder available - stream starts but errors out (x264 missing)](#no-encoder)
- [401 Unauthorized from `/stream`, `/input` or `/api/control` (token)](#token-401)
- [Daemon not found on D-Bus](#dbus-missing)

### Daemon

- [Daemon: 100% CPU usage or freeze](#daemon-cpu)

### Still Stuck?

- [Build still failing? Check the action logs](#still-stuck)
- [Re-run a single CI job](#re-run-job)
- [Verify a live stream end to end (`scripts/verify-stream.sh`)](#verify-stream)
- [Set up a development environment (`scripts/setup-dev-env.sh`)](#setup-dev-env)

---

<a id="ci-fmt"></a>
## 🧪 CI Action: `Check formatting` (`cargo fmt --all -- --check`)

**Symptom:**
```
Diff in /path/to/file.rs:
   println!("x");
-  println!("y");
+  println!("z");
```

**Cause:**
Rust source files don't match `cargo fmt`'s formatting.

**Fix:**
```bash
cargo fmt --all
git add -A
git commit -m "orbiscreen | v0.10.3 | style: cargo fmt --all"
```

**Prevention:**
Run `./gradlew :app:lintDebug` and `cargo fmt --all` locally before pushing.

---

<a id="ci-clippy"></a>
## 🧪 CI Action: `Clippy (deny warnings)`

**Symptom:**
```
error: this operation is not supported for derived errors
  --> src/lib.rs:42:5
```

**Cause:**
`cargo clippy -D warnings` treats every clippy warning as an error.

**Fix:**
```bash
cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 | head -50
cargo clippy --workspace --all-targets --locked --fix
git add -A
git commit -m "orbiscreen | v0.10.3 | fix: resolve clippy warnings"
```

**Prevention:**
Run `cargo clippy` locally before pushing.

---

<a id="ci-build"></a>
## 🧪 CI Action: `Build` (`cargo build --workspace --locked`)

**Symptom:**
```
error[E0463]: can't find crate for `gstreamer`
```

**Fix:**
```bash
cargo update -p gstreamer
cargo build --workspace --locked
git add Cargo.lock
git commit -m "orbiscreen | v0.10.3 | chore: refresh Cargo.lock"
```

---

<a id="ci-test"></a>
## 🧪 CI Action: `Test` (`cargo test --workspace --locked`)

Tests assume the host has GStreamer plugins (`x264enc`, `vaapih264enc`, `nvh264enc`). Install them locally:
```bash
sudo dnf install gstreamer1.0-plugins-{good,bad,ugly,libav}
```

---

<a id="ci-deny"></a>
## 🧪 CI Action: `Run cargo-deny`

This is a **non-blocking** informational check. See `deny.toml` for the allowlist.

---

<a id="ci-android"></a>
## 🧪 CI Action: `Android assembleDebug` + `lintDebug`

The Android workflow runs `./gradlew :app:assembleDebug :app:lintDebug`. Common failures:

- **UnstableApi lint error.** `clients/android/app/lint.xml` opts in to `androidx.media3.common.util.UnstableApi`. If you call a new Media3 API, make sure the surrounding class is annotated with `@OptIn(UnstableApi::class)`.
- **Compose imports.** Run `./gradlew :app:compileDebugKotlin` to localise the error first; lint is slower.

---

<a id="runtime-evdi"></a>
## 🚀 Runtime: `orbiscreen start` fails - `kernel module is not installed`

**Symptom:**
```
Error: evdi kernel module is not installed
```

**Fix:**
1. Install `evdi` (DKMS build) on the host:
   ```bash
   # Fedora / Nobara
   sudo dnf install dkms gcc make kernel-devel-$(uname -r) displaylink
   sudo modprobe evdi
   ```
   ```bash
   # Ubuntu / Pop!_OS
   sudo apt install dkms
   git clone https://github.com/DisplayLink/evdi.git
   cd evdi && sudo make dkms-install
   sudo modprobe evdi
   ```
2. Verify:
   ```bash
   lsmod | grep evdi
   ls /dev/dri/card*
   ```

---

<a id="runtime-kwin"></a>
## 🚀 Runtime: KDE Plasma (virtual display without evdi or root)

**Symptom:** `orbiscreen start` logs `EVDI kernel module not active` and you do not want to build a kernel module.

**Fix:** on KDE Plasma Wayland nothing else is needed. With the default `[capture] preferred = "auto"`, the daemon asks KWin to create a virtual monitor (`Virtual-ORBISCREEN`, visible in System Settings → Display Configuration) through the `zkde_screencast_unstable_v1` Wayland protocol and streams it over PipeWire, no root, no share dialog. KWin only exposes that protocol to allow-listed executables, so the daemon maintains `~/.local/share/applications/orbiscreen.kwin.desktop` (user-writable) and refreshes the KService cache automatically; the first run may take a few extra seconds while the grant becomes visible.

Notes:
- Force the path with `[capture] preferred = "kwin-virtual"` (fail loudly if unavailable) or `"portal"` (always show the share dialog).
- The virtual output disappears when the daemon stops; that is expected.
- **You see only the desktop wallpaper in the stream?** That is correct: the virtual monitor is a *second, empty* screen. Drag windows onto `Virtual-ORBISCREEN`, or set `[capture] preferred = "mirror"` to stream your real screen instead.
- On GNOME / wlroots compositors the protocol does not exist and `auto` falls back to the portal share dialog.
- EVDI is now opt-in (`preferred = "evdi"`); `auto` on Wayland never touches it, so the old `EVDI kernel module not active` line no longer appears on KDE.

---

<a id="runtime-wayland"></a>
## 🚀 Runtime: capture backend unavailable on Wayland

Use `CaptureSession::open_with_preference()` (the daemon already does so).

---

<a id="runtime-lints"></a>
## 🚀 Runtime: `unsafe_op_in_unsafe_fn` / `missing_debug_implementations`

Use `#[allow(missing_debug_implementations)]` or `#[allow(unsafe_code)]` on the offending type/function.

---

## 📱 Android Client & Devices

<a id="android-chromebook-adb"></a>
### Android / ChromeOS: ADB connection fails on ASUS Chromebook CM3001

**Symptom:**
Running the Orbiscreen Android app on ASUS Chromebook CM3001 (or other ChromeOS devices) shows "Looking for host" in USB mode and fails to find the Linux daemon.

**Cause:**
ChromeOS isolates Android apps inside an ARC++ container with its own virtual network namespace (`100.115.92.0/28`). Standard ADB reverse commands mapped to `127.0.0.1` inside the Crostini Linux container do not reach ARC++ without routing.

**Fix:**
- Orbiscreen v0.20.0 automatically probes the internal ARC++ gateway address `100.115.92.2:5555` alongside `localhost:5555`.
- In ChromeOS Settings, navigate to **Advanced** -> **Developers** -> **Develop Android apps** and turn on **Enable ADB debugging**.
- Restart the Chromebook if prompted, then launch `orbiscreen start` in the Linux container. USB mode connects instantly.

---

<a id="android-stylus"></a>
### Android: Stylus / Pen not drawing, incorrect pressure, or app crash on Lenovo Tab

**Symptom:**
Using a stylus on a Lenovo Tab (IdeaTab) or Chromebook causes either:
1. The app freezes or crashes with `NetworkOnMainThreadException` when moving the stylus.
2. In-air hover cursor is missing.
3. Tilt angle is inverted or pressure remains stuck.

**Cause:**
Early implementations dispatched stylus network packets synchronously on the Android main UI thread. In addition, generic motion hover listeners (`ACTION_HOVER_MOVE`) were not hooked, and pen release was not emitted on zero pressure.

**Fix:**
- Update to `orbiscreen-android-release.apk` **v0.20.0** or later.
- v0.20.0 moves stylus dispatch to background coroutines (`Dispatchers.IO`) with `latestStylus` coalescing, preventing UI freezes and thread exceptions.
- Implements `setOnGenericMotionListener` for in-air hover cursor tracking.
- Calibrates tilt math (`-altitudeDeg * cos(orientationRad)`) and ensures `BTN_TOOL_PEN: RELEASED` is cleanly emitted when lifting the pen.

---

<a id="android-touchpad-drag"></a>
### Android: Dragging windows or selecting files in Touchpad mode

**Symptom:**
In Touchpad mode, tapping and dragging on the tablet moves the mouse pointer, but does not drag windows or select text.

**Cause:**
Prior to v0.20.0, Touchpad mode only sent hover pointer movements, lacking a drag gesture.

**Fix:**
- Update to **v0.20.0**.
- **Double-tap and drag:** Double-tap on the touch surface and keep your finger held down on the second tap. As you move your finger, mouse button 1 remains pressed, smoothly dragging the window, file, or selection.
- Lifting your finger releases mouse button 1.

---

<a id="android-connect-crash"></a>
### Android: app crashes or process dies when tapping Connect

**Symptom:**
Tapping a host on the Discovery screen immediately kills the app process or crashes back to the launcher.

**Cause (fixed in v0.10.3):**
`PlayerHolder.build()` was executed inside `withContext(Dispatchers.IO)`. ExoPlayer requires main-thread construction; creating player components on IO threads throws thread access exceptions that terminate the process.

**Fix:**
- Upgrade to `orbiscreen-android-release.apk` **v0.10.3** or later.
- `StreamViewModel` moves ExoPlayer construction to `withContext(Dispatchers.Main)` and hardens `build()` with a try-catch block so construction errors surface as `StreamEvent.Error` retry cards instead of crashing.

---

<a id="android-black-screen"></a>
### Android: black screen after Connect

**Symptom:**
Tapping a discovered host shows a black surface; no video; the control toolbar does not appear.

**Cause (fixed in v0.10.3):**
The pre-v0.10.3 client relied on ExoPlayer's MIME sniffing for the `/stream` response and fell back to a black surface when it failed to detect MPEG-TS.

**Fix:**
- Upgrade to `orbiscreen-android-release.apk` **v0.10.3** or later.
- The new `PlayerHolder` builds `MediaItem` with `setMimeType(MimeTypes.VIDEO_MP2T)` and forces a `.ts` URL, so the stream is decoded without sniffing.
- Errors are surfaced as a retry card instead of a black surface.

If the issue persists after upgrade:
1. Confirm the host is reachable with `curl http://host:8788/health` from the same Wi-Fi.
2. Confirm `/api/info` responds: `curl http://host:8788/api/info`.
3. Check `adb logcat -s OrbiPlayer:*` for `player error:` lines.

---

<a id="android-no-hosts"></a>
### Android: discovery list is empty even though hosts are on the same Wi-Fi

**Cause:**
mDNS is blocked on the network (corporate Wi-Fi, Apple Bonjour filter, etc.).

**Fix:**
1. Open the **Add manually** card and enter `host:port` (e.g. `192.168.1.50:8788`).
2. Optional: enable the **Scan subnet for hosts** toggle in **Settings**. The scanner probes the /24 around the current gateway over TCP and adds any host that responds on port 8788.

---

<a id="android-touch-offset"></a>
### Android: touch is rotated / misaligned

**Cause:**
The pointer-to-host mapping uses the host's reported resolution from `/api/info`. If the host is rotated (e.g. portrait virtual display) but the JSON still reports landscape, mapping will be off.

**Fix:**
Rotate the host rather than the Android screen. The `PlayerView` letterboxes automatically to preserve the host's reported aspect ratio.

---

<a id="android-control-404"></a>
### Android: control toolbar actions return 404

**Cause:**
The host is running an older daemon that does not implement `/api/control`.

**Fix:**
Restart the daemon on the host to pick up the v0.10.3 transport binary. From the host:
```bash
orbiscreen stop
sudo orbiscreen start
```

---

<a id="android-crash"></a>
### Android: app crashes immediately on launch

**Symptom:**
You open the Orbiscreen app on Android and it immediately crashes back to the home screen.

**Cause (fixed in v0.7.1):**
Malformed `index.html` (stray `-->`) crashed the WebView.

**Fix:**
v0.10.3 uses Compose + `PlayerView` exclusively - there is no WebView. If the new APK still crashes, capture a logcat with `adb logcat *:E | grep orbiscreen` and open an issue.

---

<a id="android-usb"></a>
### Android: USB connection shows "Looking for host…"

**Fix:**
Orbiscreen manages the whole `adb reverse` lifecycle on its own: it creates the tunnel on every connected device when the daemon starts, picks up a newly plugged device within two seconds (hot-plug), recreates a tunnel that died with an unclean daemon exit (the setup is idempotent), and removes all tunnels on graceful shutdown. Ensure:
1. **USB Debugging** is enabled in Android Developer Options.
2. The host device is authorized on your Android phone/tablet prompt.
3. Verify what the daemon sees:
   ```bash
   orbiscreen doctor          # prints the usb: line: adb present? devices? active tunnels?
   adb devices
   adb reverse --list
   ```
4. Tap the **USB mode** card on the Discovery screen. The card probes `http://127.0.0.1:8788/health` and shows the live state: **tunnel ready** (green check) or **no tunnel** (start the daemon on the host, or reconnect the cable).

The daemon's tunnel count is also visible at any time in `GET /health` (`usb_devices`) and the D-Bus `GetStatus` payload.

<a id="streaming-wifi-latency"></a>
## ⚡ Streaming: High latency, stutter, or slow mouse movement on 5GHz Wi-Fi

**Symptom:**
When connected over 5GHz Wi-Fi (e.g. from a Lenovo Tab or phone), mouse movements feel sluggish or delayed by hundreds of milliseconds, or the stream lags behind the host.

**Cause:**
1. Traditional video pipelines buffer multiple seconds of video to smooth playback, introducing noticeable display latency.
2. Infrequent keyframes (GOP) force clients to wait for the next keyframe if any network packet drops over Wi-Fi.
3. Mouse input batching was queued at long intervals.

**Fix:**
- Orbiscreen v0.20.0 tunes the entire pipeline for ultra-low latency:
  - **6-frame GOP:** Hardware encoders emit a keyframe every 100ms, enabling instant recovery from Wi-Fi packet loss without buffering.
  - **40-120ms load control:** ExoPlayer buffers are tuned down to 40ms minimum and 120ms maximum.
  - **8ms input loop:** Mouse movement batching interval is reduced to 8ms for 120Hz-class responsiveness.
- Ensure your Wi-Fi router uses 5GHz with an 80MHz channel width and low channel congestion.

---

<a id="stream-disconnect-retry"></a>
## 🔁 Streaming: Stream error causes infinite reconnect flicker instead of detecting disconnect

**Symptom:**
When the Linux daemon stops or the network drops, the Android app repeatedly flickers and attempts to reconnect indefinitely instead of showing a clean disconnected state.

**Cause:**
Older versions lacked explicit lifecycle states for server disconnection and retried without bounds.

**Fix:**
- In v0.20.0, `PlayerHolder` introduces `StreamEvent.Disconnected`.
- On network errors, an immediate 500ms `/health` probe checks if the daemon is alive.
- Reconnection attempts are capped at 3 retries. If the host is unreachable, the app cleanly shows the disconnected card with manual retry options.

---

<a id="cursor-clamping"></a>
## 🖥 Multi-Monitor / X11: Mouse cursor escapes virtual display to other physical screens

**Symptom:**
When moving the mouse or stylus on the tablet, the cursor jumps outside the virtual display area onto your physical laptop/desktop monitors.

**Cause:**
XTEST coordinate injection without output geometry boundaries spans the entire X11 root desktop dimensions.

**Fix:**
- In v0.20.0, Orbiscreen queries XRandR output geometry and clamps cursor and stylus coordinates strictly within the virtual output rectangle (`InputProp::DIRECT`).
- The cursor is strictly confined to the tablet's virtual display screen.

---

<a id="wrong-screen"></a>
## 🖥 Client shows the wrong screen (primary desktop instead of virtual display)

**Symptom:**
The Android/web client connects and displays video, but it mirrors the host's main desktop instead of a clean second monitor. Dragging windows "ac" onto a second screen does nothing.

**Cause:**
The `evdi` kernel module is not loaded, so Orbiscreen falls back to primary-desktop capture (Wayland portal or X11 root window). This degraded mode is intentional: `GetStatus.capture_backend` reports `wayland-portal-fallback` or `x11-portal-fallback` instead of `evdi`, and the daemon logs a `EVDI kernel module missing/inactive ... Falling back` warning at start.

**Fix:**
1. Install and load `evdi` (DKMS) - see [Runtime: `orbiscreen start` fails](#runtime-evdi), then:
   ```bash
   sudo modprobe evdi && lsmod | grep evdi
   ```
2. Restart the daemon (`orbiscreen stop && orbiscreen start`) and verify:
   ```bash
   busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetStatus
   # "capture_backend":"evdi"
   ```
3. Move a window onto the Orbiscreen output (`EVDI-0`) in your compositor's display settings.

---

<a id="web-no-picture"></a>
## 🌐 Web client loads but shows no picture

**Symptom:**
`http://<host>:8788/` loads, the status bar keeps saying "Connecting to stream…" or immediately reports "This browser does not support MSE playback".

**Cause:**
The web client demuxes MPEG-TS with the locally vendored `mpegts.js` and feeds H.264 to MediaSource Extensions (MSE). Browsers without MSE or with autoplay blocked never decode the stream. There is no WebRTC path.

**Fix:**
1. Use a browser with MSE live playback: desktop Chrome, Firefox, or Edge. iOS Safari does not support MSE, and Firefox on mobile has no MSE either.
2. If autoplay was blocked, click/tap the video once to start playback.
3. Confirm the tab was served by the daemon (vendor mpegts.js loads from `/client/vendor/mpegts.js`) - not a stale copy cached from an older deployment.
4. Check DevTools console/network: a 401 on `/stream` means the token flow failed - see [401 Unauthorized](#token-401).

---

<a id="no-encoder"></a>
## 🎞 No encoder available - stream starts but errors out (x264 missing)

**Symptom:**
The daemon starts, clients connect, but video never arrives or the log shows GStreamer element link errors mentioning `x264enc` / `no element found`.

**Cause:**
Encoding goes through GStreamer. The software fallback element `x264enc` ships in the `ugly` plugin set; hardware encoders need `vaapih264enc` (`bad`) or `nvh264enc` (`bad`). Without them, no H.264 is produced.

**Fix:**
```bash
# Fedora / Nobara
sudo dnf install gstreamer1-plugins-ugly gstreamer1-plugins-bad-free gstreamer1-plugins-good

# Ubuntu / Debian
sudo apt install gstreamer1.0-plugins-ugly gstreamer1.0-plugins-bad gstreamer1.0-plugins-good

# Verify the encoder element exists
gst-inspect-1.0 x264enc
```
Then restart the daemon; `GetStatus.encoder` reports which encoder is actually in use.

---

<a id="token-401"></a>
## 🔑 401 Unauthorized from `/stream`, `/input` or `/api/control` (token)

**Symptom:**
Clients (Android, web, or hand-written scripts) get `401 Unauthorized`. `curl http://host:8788/health` works fine, but `/stream`, `/input` and `/api/control` all reject the request.

**Cause:**
These routes require the per-session access token generated when the daemon starts.
- **Token Validation:** Make sure the client is providing the correct session token matching the daemon's active session. If connecting via browser, `/client/config.json` automatically bootstraps the token, or you can supply it via `#token=` in the URL.
- Remote browsers connecting across the local network must provide the token in the URL.

**Fix:**
1. For remote web browsers, append the token via URL hash or query string:
   ```
   http://<host-ip>:8788/#token=<SECRET_TOKEN>
   ```
   Or:
   ```
   http://<host-ip>:8788/?token=<SECRET_TOKEN>
   ```
2. Retrieve the session token from the host machine:
   ```bash
   orbiscreen doctor
   # Or read the token file directly (stored with 0o600 permissions):
   cat ~/.config/orbiscreen/stream_token
   ```
3. Android clients receive the token automatically via mDNS discovery (`token=...` TXT record). If adding a host manually, enter the token in the host connection settings.
4. Pass the token via Authorization header or query parameter in custom scripts:
   ```bash
   curl -H "Authorization: Bearer $TOKEN" http://host:8788/stream --output - | head -c 1000
   # or: curl "http://host:8788/stream?token=***"
   ```

---

<a id="dbus-missing"></a>
## 🚌 Daemon not found on D-Bus

**Symptom:**
`orbiscreen stop` prints `daemon is not running (no com.orbiscreen.Daemon on the session bus)`

**Cause:**
The D-Bus service (`com.orbiscreen.Daemon`) is registered on the **user session bus** by the daemon process only while it runs. Common reasons it is absent:
- The daemon was never started (or crashed) in the current user session.
- `orbiscreen start` was started as a different user or with `sudo` - the system/other user's bus is not your session bus.
- `DBUS_SESSION_BUS_ADDRESS` is unset/overridden in the shell where you run `orbiscreen stop`.

**Fix:**
1. Check the service and status:
   ```bash
   busctl --user status com.orbiscreen.Daemon 2>&1 || echo "not on the bus"
   systemctl --user status orbiscreen
   ```
2. Start it as your normal user: `orbiscreen start` (without `sudo`) or `systemctl --user start orbiscreen`.
3. If started under systemd, prefer `systemctl --user stop orbiscreen` to stop it (`orbiscreen stop` also works and falls back to the D-Bus `Stop` method).

---

<a id="daemon-cpu"></a>
## 🚀 Daemon: 100% CPU usage or freeze

**Cause (fixed in v0.7.2):**
Capture loop ran without yielding.

**Fix:**
Update to v0.10.3 (workspace).

---

<a id="still-stuck"></a>
## 🛟 Still Stuck?

<a id="re-run-job"></a>
### Re-run a single CI job

On the failed PR page:
1. Open the **Checks** section.
2. Click the failed check name.
3. Click **Re-run jobs** → **Re-run failed jobs**.

### Check the action logs

The **Run logs** section shows the exact `cargo` / `gradlew` output. Cross-reference with the sections above.

### Open an issue

Use `.github/ISSUE_TEMPLATE/bug.yml`. Include:
- The exact `cargo` / `gradlew` error output.
- The CI run URL.
- The OS / compositor of the host (if runtime-related).
- `adb logcat *:E` output (if Android-related).

<a id="verify-stream"></a>
### Verify a live stream end to end

```bash
./scripts/verify-stream.sh [port] [duration_seconds]
```

Records a few seconds of `/stream`, checks that the payload decodes as H.264, and measures frame brightness (YAVG) to catch black/empty-stream regressions automatically. Requires `curl`, `python3`, and `ffmpeg` on the host.

<a id="setup-dev-env"></a>
### Set up a development environment

```bash
./scripts/setup-dev-env.sh
```

Installs the Rust toolchain and the build dependencies (GStreamer, Wayland/X11, libevdev) for Fedora-, Debian-, and Arch-based distros from the detected `/etc/os-release`.

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Back to README](../README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
