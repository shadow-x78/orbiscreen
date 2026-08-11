# Troubleshooting - Orbiscreen

## 🌐 Language

<a href="TROUBLESHOOTING.md">🇬🇧 English</a> · <a href="TROUBLESHOOTING_AR.md">🇸🇦 العربية</a>

---

> Applies to **v0.10.3** and later.

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
- [Runtime: capture backend unavailable on Wayland](#runtime-wayland)
- [Runtime: `unsafe_op_in_unsafe_fn` / `missing_debug_implementations` lint warnings](#runtime-lints)

### Android

- [Android: app crashes or process dies when tapping Connect](#android-connect-crash)
- [Android: black screen after Connect](#android-black-screen)
- [Android: discovery list is empty even though hosts are on the same Wi-Fi](#android-no-hosts)
- [Android: touch is rotated / misaligned](#android-touch-offset)
- [Android: control toolbar actions return 404](#android-control-404)
- [Android: app crashes immediately on launch](#android-crash)
- [Android: USB connection shows "Looking for host…"](#android-usb)

### Daemon

- [Daemon: 100% CPU usage or freeze](#daemon-cpu)

### Still Stuck?

- [Build still failing? Check the action logs](#still-stuck)
- [Re-run a single CI job](#re-run-job)

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

<a id="runtime-wayland"></a>
## 🚀 Runtime: capture backend unavailable on Wayland

Use `CaptureSession::open_async()` (the daemon already does so).

---

<a id="runtime-lints"></a>
## 🚀 Runtime: `unsafe_op_in_unsafe_fn` / `missing_debug_implementations`

Use `#[allow(missing_debug_implementations)]` or `#[allow(unsafe_code)]` on the offending type/function.

---

## 📱 Android (v0.10.3)

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
Orbiscreen automatically configures `adb reverse tcp:8788 tcp:8788` when started. Ensure:
1. **USB Debugging** is enabled in Android Developer Options.
2. The host device is authorized on your Android phone/tablet prompt.
3. Verify manually:
   ```bash
   adb devices
   adb reverse tcp:8788 tcp:8788
   ```
4. Tap the **USB mode** card on the Discovery screen.

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

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Back to README](../README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
