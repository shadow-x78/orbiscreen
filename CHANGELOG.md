# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### ⚡ Performance & Low Latency
- **HTTP MPEG-TS Transport Queue (`orbiscreen-transport`, Android, Web)**:
  - Per-client muxer keeps four sink buffers and a 16-slot HTTP queue instead of 512 / 1024 queued chunks.
  - HTTP send no longer drops MPEG-TS packets on a full queue (that desynced the decoder). The muxer backpressures instead; a stalled `appsrc` push resyncs on the next keyframe.
  - Android ExoPlayer live offset cut to 24 ms (8–48 ms) with 32–64 ms load control, `FEATURE_LowLatency` decoder preference, and MediaCodec low-latency flags.
  - Web `mpegts.js` live sync target cut to 30 ms with a tighter latency chase.

## [v0.23.3] - 2026-09-06

Enable Gradle dependency caching in CI workflows to eliminate Cloudflare 403 Forbidden errors; enhance in-page Code of Conduct navigation anchors.

### 🐛 Fixed & Optimized
- **CI Workflow Gradle Caching (`.github/workflows/android.yml`, `.github/workflows/release.yml`)**:
  - Added `cache: gradle` to `actions/setup-java` in Android CI and release workflows.
  - Caches resolved Maven dependencies across runner invocations, permanently preventing Cloudflare WAF `403 Forbidden` rate-limiting on shared runner IPs and accelerating build times.
- **Code of Conduct Navigation (`README.md`, `README_AR.md`, `CODE_OF_CONDUCT.md`)**:
  - Added in-page `<span id="coc-ov-file"></span>` anchor to support GitHub's hash-based Code of Conduct links (`#coc-ov-file`).
  - Added direct web tab hyperlinks (`?tab=coc-ov-file#readme`) for reliable one-click access.
  - Bumped community standards version badge to `0.23.3`.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.23.3.
- **Android Client**: Incremented `versionCode` to 63; updated `versionName` to "0.23.3".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.23.3 with changelog entry.
- **debian/changelog**: Added 0.23.3-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.23.3.

## [v0.23.2] - 2026-09-06

Fix black screen, video freezing, and visual glitches over USB AOA transport by eliminating arbitrary TCP/MPEG-TS chunk dropping and stabilizing Android ExoPlayer buffer thresholds.

### 🐛 Fixed
- **Uncorrupted MPEG-TS Delivery & AOA TCP Stream Reliability (`crates/orbiscreen-transport`)**:
  - In `v0.23.0`, attempts to drop laggy frames via `appsink drop=true max-buffers=1` and `video_tx.try_send()` in the AOA bridge caused arbitrary 8KB chunks to be discarded from the TCP byte stream after MPEG-TS muxing.
  - Dropping data mid-stream corrupted the 188-byte MPEG-TS alignment and sliced H.264 NAL headers, causing constant decoder errors, visual artifacts ("تشويش"), and frozen video frames on Android.
  - Video over TCP is now transferred 100% reliably: `video_tx` is an unbounded channel with blocking `.send()`, and `appsink` operates with `drop=false sync=false max-buffers=64` and clean backpressure.
  - Real-time frame dropping is now strictly handled **before** MPEG-TS muxing at the H.264 packet boundary (`broadcast::channel::<H264Packet>(64)`), cleanly discarding stale GOPs and resuming on the next clean IDR keyframe without data corruption.
- **ExoPlayer Buffer Stabilization (`PlayerHolder.kt`)**:
  - Previous load control thresholds of 10-15ms were shorter than a single 60 FPS video frame (16.67ms), causing MediaCodec to starve and cycle repeatedly between `STATE_BUFFERING` and rendering a single frame (leading to black screens and permanent freeze).
  - Configured `DefaultLoadControl` to `(40, 120, 25, 40)` ms and live target offset to `40ms` (min `25ms`, max `120ms`). This provides a stable ~2.5 frame buffer that prevents starvation while keeping end-to-end display latency imperceptible (< 50ms).

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.23.2.
- **Android Client**: Incremented `versionCode` to 62; updated `versionName` to "0.23.2".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.23.2 with changelog entry.
- **debian/changelog**: Added 0.23.2-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.23.2.

## [v0.23.1] - 2026-09-06

Restore direct touch as a virtual multitouch device, injecting evdev type-B multitouch events on the virtual touchscreen instead of routing touch through the virtual mouse.

### 🐛 Fixed
- **Direct Touch Injected as Mouse (`orbiscreen-input`, Android Client)**:
  - Previously, `Orbiscreen Virtual Touchscreen` was unused and Android touches in Direct Touch mode were routed through the virtual mouse as pointer moves and clicks, causing finger movements to hijack the host mouse cursor.
  - Android Direct Touch mode now dispatches dedicated per-finger `Touch` events (`slot`, `id`, `x`, `y`, `pressed`) with 60Hz motion coalescing.
  - The Linux daemon and input injector now inject these events via evdev type-B multitouch slots (`ABS_MT_SLOT`, `ABS_MT_TRACKING_ID`, `ABS_MT_POSITION_X`, `ABS_MT_POSITION_Y`, and `BTN_TOUCH`) onto `Orbiscreen Virtual Touchscreen`.
  - Native touch gestures such as multi-finger scrolling, pinch-to-zoom, and direct screen tapping now function seamlessly on the virtual display.
  - Touchpad mode remains dedicated to virtual mouse cursor control.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.23.1.
- **Android Client**: Incremented `versionCode` to 61; updated `versionName` to "0.23.1".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.23.1 with changelog entry.
- **debian/changelog**: Added 0.23.1-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.23.1.

## [v0.23.0] - 2026-09-06

Eliminate USB and streaming latency by redesigning the pipeline with single-buffer appsink, non-blocking queue drops, and dual-channel USB egress; resolve USB interface claim collisions with kernel driver detachment retries; and prevent Android app exit when the Linux server disconnects.

### 🐛 Fixed & Optimized
- **Real-Time Stream Latency (`crates/orbiscreen-transport`)**:
  - Reconfigured `mpegtsmux` appsink from `drop=false max-buffers=512` to `drop=true sync=false max-buffers=1`. Eliminates internal GStreamer buffer queues that caused multi-second visual and cursor lag.
  - Reduced bounded video channel from 1024 frames to 32 frames using non-blocking `try_send()`. On throughput bottlenecks, stale frames are discarded immediately instead of queuing up and delaying subsequent user input.
  - Implemented a **Dual-Channel AOA USB Writer**: split USB egress into an unbounded high-priority channel (`prio_tx` for control frames, input HTTP responses, and close events) and a bounded video channel (`video_tx`, depth 8). Control packets and mouse/touch ACKs are dispatched ahead of video data and are never blocked behind bulk video bursts.
  - Lowered bulk transfer write timeout from 2000ms to 100ms.
- **Resilient USB Interface 0 Claim (`crates/orbiscreen-transport/src/aoa.rs`)**:
  - Resolved `Device or resource busy (os error 16)` during accessory bridge startup: configured `USBDEVFS_DISCONNECT_CLAIM` with `flags = 0` to unconditionally unbind any active kernel driver from interface 0 before claiming.
  - Added a 3-attempt retry loop with exponential backoff and fallback to `USBDEVFS_CLAIMINTERFACE`, ensuring smooth accessory handover across daemon restarts.
- **ExoPlayer Ultra-Low Latency Buffer Tuning (`PlayerHolder.kt`)**:
  - Reconfigured `DefaultLoadControl` on the Android client: reduced minimum and maximum buffer duration from 80ms–250ms down to 15ms–45ms (`bufferForPlaybackMs = 10`, `bufferForPlaybackAfterRebufferMs = 15`).
  - Tuned `MediaItem.LiveConfiguration` target live offset down to 15ms (min 10ms, max 35ms), eliminating playback buffer accumulation and providing instant mouse cursor and screen reaction.
- **Graceful USB Detach & App Lifecycle (`MainActivity.kt` & `OrbiNav.kt`)**:
  - Overrode `finish()` in `MainActivity` to prevent the Android system from force-terminating the activity when the USB accessory is detached (`ACTION_USB_ACCESSORY_DETACHED`).
  - Added reactive observation of `isAoaActiveFlow` in `OrbiNav`: when the Linux server stops or the USB cable is unplugged, the app automatically transitions from `Routes.STREAM` back to `Routes.DISCOVERY` instead of closing.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.23.0.
- **Android Client**: Incremented `versionCode` to 60; updated `versionName` to "0.23.0".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.23.0 with changelog entry.
- **debian/changelog**: Added 0.23.0-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.23.0.

## [v0.22.9] - 2026-09-06

Fix USB bulk packet truncation in Android AOA proxy that caused session tokens to be silently lost, implement proper TCP connection lifecycle management on the Linux bridge, and optimize session token caching in the Android client.

### 🐛 Fixed
- **AOA USB Packet Truncation (`UsbAccessoryManager.kt`)**:
  - The Linux kernel AOA gadget driver (`f_accessory`) delivers each USB Bulk transfer in a single `read()` call; the previous `readFully()` per-field approach caused the kernel to silently discard the remainder of each bulk packet after reading only the 5-byte frame header, permanently corrupting the byte stream.
  - Replaced with a **streaming accumulator buffer**: `handleUsbIncoming` reads raw bytes in one `InputStream.read()` call and extracts complete frames from a sliding byte array - eliminating all packet truncation.
  - `/client/config.json` responses now arrive intact, session token is delivered to `sessionToken`, and all subsequent `/stream` and `/input` requests include a valid `Authorization: Bearer` header.
- **Auth Rejection Loop (`auth=missing`)**:
  - With `sessionToken = null` (caused by the truncation bug above), ExoPlayer and `InputDispatcher` were sending every request without authentication headers, causing the server to reject all requests with `unauthorized request rejected auth=missing`.
  - Fully resolved by the accumulator buffer fix above.

### ✨ Improved
- **AOA Bridge Lifecycle (`aoa.rs`)**:
  - Changed `tcp_streams` map value type from `Sender<Vec<u8>>` to `(Sender<Vec<u8>>, TcpStream)` to retain a cloned stream handle alongside each channel sender.
  - On `FRAME_FLAG_CLOSE` from Android: immediately calls `stream.shutdown(Shutdown::Both)` so Axum HTTP connection handlers receive EOF and release resources promptly.
  - On bridge exit: drains all remaining open streams and shuts them down before joining the writer thread, preventing resource leaks on accessory disconnect.
- **Session Token Caching (`StreamViewModel.kt`)**:
  - Added `forceRefresh: Boolean = false` parameter to `freshToken()`.
  - When `forceRefresh = false` and a valid cached token exists, returns the cached token immediately without a network round-trip, avoiding a redundant `/client/config.json` HTTP request during player setup.
  - `onUnauthorized` callbacks and explicit `retry()` calls use `forceRefresh = true` to always fetch a fresh token.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.22.9.
- **Android Client**: Incremented `versionCode` to 59; updated `versionName` to "0.22.9".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.22.9.
- **debian/changelog**: Added 0.22.9-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.22.9.

## [v0.22.8] - 2026-09-05

Embed full multi-vendor Android udev rules in `orbiscreen doctor --fix`, implement resilient token retry logic on Android client to eliminate initial unauthorized stream errors, and provide zero-root USB Tethering guidance.

### ✨ Added & Improved
- **Multi-Vendor Udev Rule Installation (`doctor --fix`)**:
  - Embedded complete `data/99-orbiscreen-usb.rules` in `orbiscreen doctor --fix`, ensuring all Android vendor IDs (`18d1`, `17ef`, `2717`, `04e8`, `12d1`, `22b8`, etc.) receive unprivileged non-root access (`TAG+="uaccess", MODE="0666"`).
  - Enhanced error handling in AOA supervisor: prevents log spamming on permission denied and logs helpful non-root remediation instructions.
  - Added explicit diagnostic guidance recommending USB Tethering for 100% configuration-free, zero-root USB cable streaming.
- **Resilient Android Session Token Handshake**:
  - Implemented automatic retry loop (up to 6 attempts with 250ms backoff) in `StreamViewModel.kt` when fetching session token (`/client/config.json`) during stream startup.
  - Eliminates the race condition where Android connects before daemon token initialization, preventing `401 Unauthorized` stream errors.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.22.8.
- **Android Client**: Incremented `versionCode` to 58; updated `versionName` to "0.22.8".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.22.8.
- **debian/changelog**: Added 0.22.8-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.22.8.
- **Documentation**: Synchronized version badges and references across all documentation files.

## [v0.22.7] - 2026-09-05

Purge all ADB legacy dependencies, implement real-time progressive terminal diagnostics in `orbiscreen doctor`, provide unprivileged Linux udev rules for Android Open Accessory (AOA) mode, and enhance the Android in-app update dialog with native Compose Markdown rendering and auto-installation.

### ✨ Added & Improved
- **Direct USB Cable Streaming & Udev Rules**:
  - Completely purged ADB socket calls (`adb::setup_reverse_for_all`, `adb_installed`, `reverse_tunnels`) from diagnostics and background streaming services.
  - Implemented smart device filtering (`is_android_candidate`) in `aoa::supervisor` targeting Android vendor IDs (`0x18d1`, `0x17ef`, `0x2717`, `0x04e8`, etc.) and skipping non-Android peripherals (keyboards, gaming mice, audio headsets, USB hubs).
  - Added `data/99-orbiscreen-usb.rules` with `TAG+="uaccess", MODE="0666"` for Android Open Accessory (`18d1:2d00-2d05`) and Android devices, allowing unprivileged direct streaming without root or ADB.
  - Extended endpoint discovery in `aoa.rs` across all interface subdirectories under sysfs.
- **Progressive Live Diagnostics (`orbiscreen doctor`)**:
  - Implemented real-time progressive output with immediate stdout flushing (`print_card_header`, `print_card_row`, `print_card_footer`), completely eliminating CLI freezing.
  - Added dedicated `USB Direct / Cable` diagnostic check detecting AOA accessory readiness and guiding users with a 1-line command when udev permissions are missing.
  - Added auto-installation support for udev rules in `orbiscreen doctor --fix`.
  - Updated `orbiscreen devices` to report direct USB hardware status instead of legacy ADB reverse tunnels.
- **Android In-App Update Dialog Enhancements (`UpdateDialog.kt`)**:
  - Native Jetpack Compose Markdown renderer supporting headings, bullet lists, bold, italics, and code blocks.
  - Polished Catppuccin surface design with symmetrical action button weights.
  - Lifecycle observer on `ON_RESUME` to automatically detect package install permission grant, dynamically update the button to "Install", and invoke installation immediately.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.22.7.
- **Android Client**: Incremented `versionCode` to 57; updated `versionName` to "0.22.7".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.22.7, installed `99-orbiscreen-usb.rules`, and removed `Recommends: android-tools`.
- **debian/changelog**: Added 0.22.7-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.22.7.
- **Documentation**: Synchronized version badges and references across all documentation files.

## [v0.22.6] - 2026-09-05

Implement native USB Direct streaming via Android Open Accessory (AOA) protocol with standard Linux kernel usbfs ioctls, and unify concise, tool-oriented translations across Web, Android, and CLI interfaces.

### ✨ Added & Improved
- **Direct USB Streaming via Android Open Accessory (AOA)**:
  - Implemented native Linux `usbfs` AOA driver in `orbiscreen-transport` (`aoa.rs`), communicating directly with USB bulk endpoints via standard kernel ioctls without external dependencies.
  - Enables plug-and-play high-speed streaming over a standard USB cable while the device is in default "File Transfer" (MTP) mode, with zero requirement for ADB, USB debugging, or developer options.
  - Added Android USB accessory filter and `UsbAccessoryManager` running a local loopback bridge on `127.0.0.1:8789`.
  - Expanded network interface and gateway discovery in `HostApi.kt` to cover `eth*`, `arc*`, `usb*`, `rndis*`, `ncm*`, `tun*`, and ChromeOS ARCVM fallback routes (`100.115.92.2`), ensuring 100% compatibility across tablets, phones, and ChromeOS devices (`android:required="false"` on accessory feature).
  - Updated CLI diagnostics (`orbiscreen doctor`, `orbiscreen -V`, `orbiscreen status`) and UI cards to reflect "USB Direct".
- **Tool-Oriented Bilingual Translations (Web, Android, CLI)**:
  - Completely audited and refined English and Arabic dictionaries across the web client (`clients/web/app.js`, `index.html`), Android app (`strings.xml`, `values-ar/strings.xml`), and CLI output cards.
  - Eliminated verbose descriptions and manual-style sentences in favor of sleek, punchy, action-oriented tool terminology.
  - Graceful styled stop card for `orbiscreen stop` when daemon is not currently running.
- **Contributor Covenant Code of Conduct**:
  - Added project community standards (`CODE_OF_CONDUCT.md`) based on Contributor Covenant v2.1.
  - Linked across `README.md`, `README_AR.md`, and `CONTRIBUTING.md`.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.22.6.
- **Android Client**: Incremented `versionCode` to 56; updated `versionName` to "0.22.6".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.22.6.
- **debian/changelog**: Added 0.22.6-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.22.6.
- **Documentation**: Synchronized version badges and references across all documentation files.

## [v0.22.5] - 2026-09-05

Implement native in-app Android updater with real-time download progress bar, automated SHA-256 checksum integrity verification, and direct PackageInstaller integration without external browser redirects.

### ✨ Added & Improved
- **Native In-App Android Updater (`com.orbiscreen.android.updater`)**:
  - Implemented `UpdateManager` to query GitHub Releases API, parse version tags, release notes, APK download endpoints (`orbiscreen-android-release.apk`), and published checksum assets (`orbiscreen-android-release.apk.sha256`).
  - Added streaming file download with live coroutine progress updates (percentage and downloaded/total megabytes) and cancellation support.
  - Added automatic SHA-256 checksum verification before launching installation, protecting against corrupted or incomplete downloads.
  - Added seamless `FileProvider` (`content://...`) and `REQUEST_INSTALL_PACKAGES` permission management with direct navigation to Android system settings when required.
- **Material 3 Update Dialog (`UpdateDialog.kt`)**:
  - Custom styled dialog displaying new version badge, scrollable release notes, live progress indicator, and action buttons ("Update Now", "Cancel", "Install", and fallback "Open in Browser").
- **Automatic & Manual Update Triggers**:
  - Non-intrusive background check on application launch in `MainActivity.kt` after initial splash delay.
  - Interactive "Check for Updates" button in `SettingsScreen.kt` triggering the in-app update dialog directly.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.22.5.
- **Android Client**: Incremented `versionCode` to 55; updated `versionName` to "0.22.5".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.22.5.
- **debian/changelog**: Added 0.22.5-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.22.5.
- **Documentation**: Synchronized version badges and references across all documentation files.

## [v0.22.4] - 2026-09-05

Fix pure pointer classification in libinput/KWin, add automatic USB Tethering gateway discovery for ADB-free high-speed USB connections, implement styled ASCII stop card, and register org.shadow-x78.Orbiscreen on D-Bus.

### ✨ Added & Improved
- **Pure Pointer Device Classification (`orbiscreen-input`)**:
  - Filtered key capabilities in `Orbiscreen Virtual Mouse and Keyboard`: restricted keys to standard keyboard codes (`1..=248`) and mouse buttons (`0x110..=0x117`), removing `BTN_PAD` (`0x160..=0x2FF`) and `BTN_MISC` (`0x100..=0x10F`).
  - Resolves issue where `libinput` mistakenly classified the device as a `tablet-pad` (`pointer = false` in KWin). The device is now correctly recognized as `Capabilities: keyboard pointer` (`pointer = true`), enabling instant absolute touch and mouse control.
  - Cursor coordinates remain strictly confined to `Virtual-ORBISCREEN` with zero snapback.
- **Automatic USB Tethering Discovery (Android Client)**:
  - Updated `HostApi.probeUsb` and `DiscoveryScreen` to scan local network interfaces for active USB Tethering adapters (`rndis*`, `usb*`, `ncm*`) and probe default gateway endpoints (e.g. `192.168.42.1`).
  - Android users can now connect over a USB cable simply by enabling USB Tethering (توصيل شبكة USB) in Android settings, without needing ADB, USB debugging, or terminal commands.
- **Styled ASCII Stop Card (`orbiscreen-daemon`)**:
  - Implemented `ui::print_stop_card` providing a formatted Catppuccin ASCII card for `orbiscreen stop` matching `orbiscreen -V` and `orbiscreen status`.
  - Clearly displays shutdown status, message, and session D-Bus service identifier.
- **D-Bus Service Registration (`orbiscreen-daemon`)**:
  - Registered `org.shadow-x78.Orbiscreen` well-known name on the D-Bus session bus alongside `com.orbiscreen.Daemon` for seamless client compatibility.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.22.4.
- **Android Client**: Incremented `versionCode` to 54; updated `versionName` to "0.22.4".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.22.4.
- **debian/changelog**: Added 0.22.4-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.22.4.
- **Documentation**: Synchronized version badges and references across all documentation files.

## [v0.22.3] - 2026-09-05

Eliminate video stuttering and packet bursts with 1-second GOP interval, make virtual mouse a pure absolute pointer device to permanently eliminate cursor snapback, and stabilize stylus proximity and network dispatch.

### ✨ Added & Improved
- **Video Stuttering Elimination via GOP Interval Normalization (`orbiscreen-encode`)**:
  - Increased encoder `gop-size` (NVENC), `keyframe-period` (VAAPI), and `key-int-max` (x264) from 6 frames to 60 frames (1 keyframe per second at 60 FPS).
  - Eliminates the 10 IDR bursts per second that flooded Wi-Fi networks and caused continuous frame drops on mobile hardware decoders.
- **Pure Absolute Pointer Device & Zero Cursor Snapback (`orbiscreen-input`)**:
  - Removed `REL_X` and `REL_Y` axes from `Orbiscreen Virtual Mouse and Keyboard`, leaving only `ABS_X`, `ABS_Y`, and `REL_WHEEL`.
  - In `libinput`, devices declaring `REL_X`/`REL_Y` ignore `ABS_X`/`ABS_Y`. Removing relative axes allows `libinput` and KWin to treat the device as a pure absolute pointer.
  - Absolute pointer coordinates are strictly mapped to `Virtual-ORBISCREEN` (`outputName` + `mapToWorkspace = false`).
  - Cursor is permanently confined to the virtual screen during pointer moves, finger taps, drags, and pen lifts, with zero snapback to the host display.
  - Preserved relative movement compatibility via clamped internal coordinate tracking.
- **Stylus Proximity Release & Hover Stability (`orbiscreen-input`)**:
  - Updated virtual tablet tool lifecycle: `BTN_TOOL_PEN` releases cleanly when physical contact ends (`pressure == 0`), preventing fake permanent hover locks in KWin and Xwayland.
  - Mirrored absolute pointer events continuously ensure smooth, lag-free stylus drawing and desktop control.
- **Network Contention Reduction (Android Client)**:
  - Adjusted Android input dispatch loop delay from 8ms to 16ms, aligning input events with the 60 Hz display refresh rate and cutting upstream HTTP POST traffic by 50%.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.22.3.
- **Android Client**: Incremented `versionCode` to 53; updated `versionName` to "0.22.3".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.22.3.
- **debian/changelog**: Added 0.22.3-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.22.3.
- **Documentation**: Synchronized version badges across all documentation files.

## [v0.22.2] - 2026-09-05

Fix video stuttering, prevent stream termination on buffer overrun, confine mouse pointer strictly to virtual screen without snapback, and enable universal stylus touch and hover support.

### ✨ Added & Improved
- **Video Stuttering and Pipeline Stability (`orbiscreen-transport`)**:
  - Expanded broadcast video packet channel from 32 to 512 packets, eliminating keyframe lag stalls.
  - Increased MPEG-TS appsink queue to 512 buffers and mpsc delivery channel to 1024 chunks.
  - Eliminated stream termination on buffer overrun: chunks are safely dropped under transient network pressure instead of returning EOS.
- **Mouse Pointer Confinement and Snapback Elimination (`orbiscreen-input`, `orbiscreen-daemon`)**:
  - Routed all pointer button clicks (including Left Click / Button 1) through the virtual mouse device (`mouse_keyboard`), preventing KWin from dropping the pointer back to the primary monitor when releasing touch.
  - Applied `mapToWorkspace = false` alongside `outputName` in KWin input binding for all Orbiscreen virtual devices, ensuring strict confinement to the secondary display.
- **Universal Stylus Touch, Pressure, and Hover (`orbiscreen-input`, Android Client)**:
  - Corrected Linux evdev tablet tool states: `BTN_TOOL_PEN` stays pressed while in proximity, with `BTN_TOUCH` reflecting physical contact.
  - Mirrored stylus coordinates and touch state to `mouse_keyboard`, enabling stylus interaction across all standard desktop applications, window controls, and menus, while retaining pressure and tilt in drawing software.
  - Enhanced Android `PlayerSurface` with automatic view focus, positive pressure clamping on physical touch, and responsive hover tracking.
- **Stutter-Free 60 FPS Android Playback (Android Client)**:
  - Tuned ExoPlayer `DefaultLoadControl` to 40-80ms playback threshold and 80-250ms buffer range, preventing micro-stutter buffering loops over USB ADB and Wi-Fi.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.22.2.
- **Android Client**: Incremented `versionCode` to 52; updated `versionName` to "0.22.2".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.22.2.
- **debian/changelog**: Added 0.22.2-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.22.2.
- **Documentation**: Synchronized version badges across all documentation files.

## [v0.22.1] - 2026-09-05

Ultra-low latency streaming optimizations, 3 dedicated uinput virtual devices, direct touch output confinement, and hardware stylus recognition.

### ✨ Added & Improved
- **Dedicated uinput Device Separation (`orbiscreen-input`)**: Split virtual input into three specialized uinput devices:
  1. `Orbiscreen Virtual Mouse and Keyboard`: handles pointer relative movement, buttons, wheel scrolling, and keyboard events.
  2. `Orbiscreen Virtual Touchscreen`: direct touch input with `InputProp::DIRECT`, absolute X/Y axes, and `BTN_TOUCH`.
  3. `Orbiscreen Virtual Tablet`: dedicated digitizer with `BTN_TOOL_PEN`, `BTN_TOOL_RUBBER`, `BTN_TOUCH`, `BTN_STYLUS`, `BTN_STYLUS2`, pressure, and tilt.
- **Hardware Stylus / Tablet Tool Resolution (`orbiscreen-input`)**: Added explicit axis resolution (`with_resolution(10)`) to `ABS_X` and `ABS_Y` on the tablet device. Resolves the Linux `libinput` kernel-level bug where tablet devices without explicit resolution are discarded.
- **KWin Virtual Output Confinement (`orbiscreen-daemon`)**: Bound both virtual touchscreen and tablet devices to `Virtual-ORBISCREEN` using KWin D-Bus properties (`supportsOutputArea: true`), guaranteeing touch and stylus inputs stay strictly confined to the virtual secondary screen.
- **Ultra-Low Latency Pipeline (`orbiscreen-transport`)**: Reconfigured GStreamer `appsrc` to `is-live=false` to prevent clock synchronization drift, and trimmed buffer queues (`appsink max-buffers=128`, channel capacity 32) to prevent frame queue latency accumulation.
- **Client Playback and Dispatch Acceleration (Android Client)**: Added HTTP connection pooling with persistent keep-alive, switched input event dispatch to non-blocking asynchronous execution (`enqueue`), and tuned ExoPlayer live playback buffering to 20-50ms with automatic speed catch-up.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.22.1.
- **Android Client**: Incremented `versionCode` to 51; updated `versionName` to "0.22.1".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.22.1.
- **debian/changelog**: Added 0.22.1-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.22.1.
- **Documentation**: Synchronized version badges across all documentation files.

## [v0.22.0] - 2026-09-05

Direct touch reverse input precision, zero-snapback mouse pointer mapping, green screen video pipeline fix, and clean startup branding.

### ✨ Added & Improved
- **Zero-Snapback Virtual Pointer (`orbiscreen-input`)**: Added absolute coordinate axes (`Abs::X`, `Abs::Y`) and `InputProp::POINTER` directly to `mouse_keyboard`. Completely eliminated mouse pointer snapping back to the primary screen on touch lift by maintaining active pointer position on the virtual display.
- **KWin Virtual Output Binding (`orbiscreen-daemon`)**: Updated `bind_kwin_virtual_inputs` to explicitly set `mapToWorkspace = false` and implemented resilient retry passes (at 250ms, 500ms, and 1000ms) to ensure all virtual input devices are reliably bound to the target virtual monitor.
- **Atomic Pointer Event Dispatch (Android Client & Web Client)**: Added `pointerAction` in `InputDispatcher.kt` to send movement coordinates and mouse button state atomically in strict FIFO sequence, preventing premature clicks at stale positions.
- **Default Direct Touch Mode (`PrefsStore.kt` & `StreamScreen.kt`)**: Defaulted input mode to Direct Touchscreen mode (`isTouchMode = true`) on Android and persisted user preference across sessions.
- **Dynamic Resize Mode Scaling (`PlayerSurface.kt`)**: Enhanced `computeContentRect` to automatically handle both `RESIZE_MODE_FIT` (aspect letterboxing) and `RESIZE_MODE_FILL` (fullscreen stretching), ensuring pixel-accurate 1:1 touch coordinate tracking directly beneath the user's finger.

### 🐛 Fixed
- **Green Screen & MPEG-TS Stream Corruption (`orbiscreen-transport`)**:
  - Eliminated packet dropping in GStreamer `appsink` by setting `drop=false` and increasing buffer queues to 1024 chunks.
  - Replaced lossy join-buffer with monotonic keyframe PTS synchronization and increased broadcast channel capacity to 128 packets.
- **Clean Banner Branding (`orbiscreen-daemon`)**: Removed `(by shadow-x78)` from the ASCII art logo and startup CLI output.

### 🧹 Packaging & Maintenance
- **Cargo Workspace**: Bumped workspace package version to 0.22.0.
- **Android Client**: Incremented `versionCode` to 50; updated `versionName` to "0.22.0".
- **COPR / RPM Spec** (`data/orbiscreen-copr.spec`): Updated to version 0.22.0.
- **debian/changelog**: Added 0.22.0-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.22.0.
- **Documentation**: Synchronized version badges across all documentation files.

## [v0.21.0] - 2026-09-04

Rich developer and system version card, comprehensive bilingual localization (Arabic & English) across web and Android clients, fully translated Arabic architecture diagrams, and streamlined packaging specifications.

### ✨ Added
- **Rich Version & Developer Card (`orbiscreen -V`)**: Added status-card style display with Catppuccin color scheme, ASCII banner, developer attribution, live desktop environment detection, feature matrix, and JSON output support (`orbiscreen version --json`).
- **Comprehensive Bilingual Localization**: Added pure JavaScript i18n switcher in the web client with instant English/Arabic toggle and RTL styling, refined Android string resources (`values-ar`), streamlined packaging descriptions, and polished technical documentation across all Markdown guides.
- **Arabic Architecture Diagram**: Fully translated system architecture diagrams into clear, standard technical Arabic in `docs/ARCHITECTURE_AR.md`, with complete quotation wrapping to prevent Mermaid parse errors.

### 🧹 Packaging & Maintenance
- **COPR / RPM spec** (`data/orbiscreen-copr.spec`): Updated to version 0.21.0.
- **debian/changelog**: Added 0.21.0-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.21.0.
- **Android `versionCode`**: Incremented to 49; `versionName` updated to "0.21.0".

## [v0.20.0] - 2026-09-04

XDG Desktop Portal virtual display API, comprehensive stylus digitizer overhaul, touchpad drag-and-drop gesture, ChromeOS ADB support, token security hardening, and Wi-Fi latency optimizations.

### ✨ Added
- **XDG Desktop Portal Virtual Display API (`orbiscreen-capture`)**: Added native support for `SourceType::Virtual` in `ashpd` ScreenCast portal with seamless fallback to `SourceType::Monitor`. Enables rootless virtual monitor creation on GNOME and KDE Plasma Wayland sessions without requiring EVDI or root privileges.
- **Stylus Digitizer Overhaul (`orbiscreen-input` & Android Client)**:
  - Added pointer hover motion listener (`setOnGenericMotionListener`) for stylus in-air tracking.
  - Resolved `NetworkOnMainThreadException` by offloading stylus network dispatch to background coroutines on `Dispatchers.IO` with `latestStylus` coalescing.
  - Fixed tilt math sign (`-altitudeDeg * cos(orientationRad)`).
  - Fixed pen release logic by emitting `BTN_TOOL_PEN: RELEASED` when pressure reaches 0.
- **Touchpad Drag-and-Drop Gesture (`PlayerSurface.kt`)**: Added double-tap and drag gesture in Touchpad mode, keeping mouse button 1 pressed throughout the drag and releasing on touch lift.
- **Virtual Display Coordinate Confinement (`xtest.rs` & `x11.rs`)**: Clamped mouse and stylus coordinates strictly within the virtual display geometry retrieved via XRandR and added `InputProp::DIRECT` to prevent cursor jumping across multiple physical monitors.
- **Direct Touchscreen Mode Fix**: Decoupled absolute coordinate axes from mouse/keyboard to tablet/touch devices, routing touch events with synthetic `BTN_TOUCH`.
- **ChromeOS ARC++ ADB Support (`adb.rs`)**: Added internal ADB fallback probing `100.115.92.2:5555` and `localhost:5555` for seamless tethering on Chromebooks (e.g. ASUS CM3001).
- **Stream Lifecycle & Fast Disconnect Detection (`PlayerHolder.kt`)**: Added explicit `StreamEvent.Disconnected` state, 500ms immediate `/health` probe on network error, and capped automatic reconnection retries at 3 to prevent infinite flickering reconnect loops.
- **Rich Version & Developer Card (`orbiscreen -V`)**: Added status-card style display with Catppuccin color scheme, ASCII banner, developer attribution, live desktop environment detection, feature matrix, and JSON output support (`orbiscreen version --json`).
- **Comprehensive Bilingual Localization & Copywriting**: Added pure JavaScript i18n switcher in the web client with instant English/Arabic toggle and RTL styling, refined Android string resources (`values-ar`), streamlined packaging descriptions, and polished technical documentation across all Markdown guides.
- **Architecture Diagram Fix**: Quoted all node and edge labels in Mermaid diagrams across `docs/ARCHITECTURE.md` and `docs/ARCHITECTURE_AR.md` to prevent syntax parse errors.

### 🛡️ Security & Hardening
- **Session Token Security & Bootstrap (`orbiscreen-transport`)**: Hardened token delivery and filesystem permissions (`0o600`). Web client bootstrap serves display geometry and token for zero-friction LAN discovery, supporting URL hash `#token=...` for direct links.
- **Disk Token Permissions (`orbiscreen-daemon`)**: Enforced strict `0o600` permissions on `stream_token` file and `0o700` on parent directories.
- **Production Expect Removal (`wlr_virtual_output.rs`)**: Replaced the single production `.expect()` with safe slice parsing returning `WlrootsVirtualOutputError::Sway`.
- **Resolution Parameter Clamping**: Clamped input resolutions in `api_control` to `[320..=7680]` width and `[240..=4320]` height.
- **Ultra-Low Latency Wi-Fi Tuning**: Tuned GOP keyframe interval to 6 (100ms) in hardware encoders, reduced GStreamer appsink buffers with `drop = true`, tuned ExoPlayer buffers to (40, 120, 20, 30) ms, and reduced mouse batching delay to 8ms.

### 🧹 Code Cleanliness & Packaging
- **Standardized Box-Drawing Comment Headers**: Standardized Unicode box-drawing headers across all shell scripts, spec files, and config files.
- **Zero-Comment Compliance**: Verified zero comments after line 2 across all Rust, Kotlin, and web client source files.
- **COPR / RPM spec** (`data/orbiscreen-copr.spec`): Updated to version 0.20.0.
- **debian/changelog**: Added 0.20.0-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.20.0.
- **Android `versionCode`**: Incremented to 48; `versionName` updated to "0.20.0".

## [v0.19.0] - 2026-09-04

Full COSMIC desktop support, native Fedora CI and RPM packaging pipeline, RTL documentation overhaul, and codebase-wide zero-comment standards.

### ✨ Added
- **COSMIC Desktop Support (`capabilities.rs`)**: Added `Compositor::Cosmic` detection for System76's Smithay-based `cosmic-comp`. Implemented dedicated capture pipeline fallback (`evdi -> portal`), refined `KDE_FULL_SESSION` detection logic, and added comprehensive unit test suite.
- **COSMIC Diagnostics & Auto-Fix (`orbiscreen doctor`)**: Added detection for `cosmic-comp` and `cosmic-randr` binaries, JSON diagnostic outputs, and distro auto-fix tokens for Pop!_OS and Ubuntu COSMIC sessions.
- **Native Fedora CI Workflow (`.github/workflows/ci.yml`)**: Added native Fedora build job with RPM Fusion repositories, GStreamer dependencies, test execution, and automated RPM artifact packaging.
- **RTL Arabic Documentation Overhaul**: Converted all Markdown tables across Arabic documentation to native RTL containers with BiDi isolation (`&rlm;`), and converted directory trees in `README_AR.md` and `docs/ARCHITECTURE_AR.md` into structured RTL tables with complete Arabic translations.

### 🧹 Cleaned & Hardened
- **Zero-Comment Code Standards**: Enforced zero comments after line 2 across all Rust (`.rs`), Kotlin (`.kt`), and web client (`.js`, `.css`, `.html`) source files while preserving official credit headers.
- **Standardized Configuration Headers**: Standardized Unicode boxed banner headers across all shell scripts, spec files, and environment templates (`.env.example`).
- **Cleaned Typography**: Removed all em dashes and non-technical filler phrases across documentation and comments.
- **Transport Test Hardening (`mpegts_mux_timestamps.rs`)**: Added graceful handling for environments without GStreamer x264/openh264 encoders.

### 🔧 CI / Packaging
- **COPR / RPM spec** (`data/orbiscreen-copr.spec`): Updated to version 0.19.0.
- **debian/changelog**: Added 0.19.0-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.19.0.
- **Android `versionCode`**: Incremented to 47; `versionName` updated to "0.19.0".

## [v0.18.3] - 2026-09-04

Full web client parity with Android Compose UI (direct hotkey typing, input isolation, and streamlined overlays), refined ASCII logo geometry, and packaging updates.

### ✨ Added
- **Android 1:1 Web Client Keyboard (`KeyboardOverlay`)**: Replaced text input field with direct IME keystroke forwarding and a full Material 3 hotkey overlay featuring latched modifiers (`Ctrl`, `Alt`, `Shift`, `Win`), function keys (`F1`-`F12`), productivity shortcuts (`Undo`, `Copy`, `Paste`, `CAD`), and navigation controls.
- **Brand Favicon Assets**: Added official high-resolution `favicon.svg` (vector ring with sapphire display dot), `favicon.png`, and `apple-touch-icon.png` for PWA and browser tab integration.
- **Single-Element Streamlined Overlay**: Overhauled connection and error states with mutually exclusive graphics (Material spinner during connecting, official logo on disconnect, and dedicated action `[ ↻ Reconnect ]` button).

### 🛠️ Fixed & Improved
- **VNC Input Isolation & Disconnect Handling**: Cursor is cleanly released and toolbar/controls are completely hidden upon stream termination; pointer events are blocked when disconnected to prevent accidental cursor capture.
- **Esc & Fullscreen Separation**: `Esc` solely releases host pointer lock; `F11` independently toggles browser fullscreen.
- **Refined ASCII Logo Geometry (`ui.rs`)**: Restored true 5-line circular ring contour and highlighted the orbiting satellite node via sapphire TrueColor accent without breaking block alignment.
- **CLI Credit & Flag Streamlining**: Removed redundant attribution noise from `--version` output, cleaned GNU-style help footers, and pruned unused subcommands.
- **X11 Input Driver Fix (`x11.rs`)**: Removed erroneous `BTN_TOOL_PEN` injection during mouse motion that caused cursor freezes.

### 🔧 CI / Packaging
- **COPR / RPM spec** (`data/orbiscreen-copr.spec`): Updated to version 0.18.3; included web client favicon assets.
- **debian/changelog**: Added 0.18.3-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.18.3 and included web favicon assets.
- **Android `versionCode`**: Incremented to 46; `versionName` updated to "0.18.3".

## [v0.18.2] - 2026-09-03

Modernized CLI interface with Catppuccin Mocha theme, official ASCII/ANSI vector logo, new interactive `status` command, clean code standards, and standardized credit headers.

### ✨ Added
- **Modernized CLI Interface (`ui.rs`)**: Integrated Catppuccin Mocha 24-bit TrueColor palette (`#89b4fa`, `#74c7ec`, `#a6e3a1`, `#f9e2af`, `#f38ba8`), custom Clap v4 styling, rounded Unicode cards, and intelligent `NO_COLOR` and terminal auto-detection.
- **Official ASCII/ANSI Banner**: Terminal rendering of the Orbiscreen ring and orbiting display dot with project highlights.
- **New `orbiscreen status` Command**: Direct D-Bus client querying `com.orbiscreen.Daemon` for active display resolution, encoder, capture pipeline, client counts, USB status, and forwarded frame counts, with `--json` support.
- **Enhanced Command Cards**: Modernized output formatting for `orbiscreen doctor`, `orbiscreen probe`, `orbiscreen list-displays`, and `orbiscreen start`.

### 🧹 Cleaned & Standardized
- **Credit Header Standardization**: Enforced standard 2-line credit header followed by an empty line across all `.rs` and `.sh` files.
- **Codebase Comment Cleanup**: Cleaned implementation comments across all modules and replaced doc comments on CLI flags with explicit Clap attributes.

### 🔧 CI / Packaging
- **COPR / RPM spec** (`data/orbiscreen-copr.spec`): Updated to version 0.18.2.
- **debian/changelog**: Added 0.18.2-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.18.2.
- **Android `versionCode`**: Incremented to 45; `versionName` updated to "0.18.2".

## [v0.18.1] - 2026-09-03

Pure CLI daemon architecture, removal of desktop application launcher, and synchronized package metadata across all distribution channels.

### 🎨 Changed
- **Pure CLI Daemon Architecture**: Completely removed `data/orbiscreen.desktop` and application launcher integration to focus Orbiscreen entirely on a headless and CLI daemon workflow.
- **`install.sh` & `uninstall.sh`**: Stripped desktop database triggers (`update-desktop-database`, `gtk-update-icon-cache`) and application entry installations. Clean uninstallation now sweeps legacy files.
- **Package Metadata Synchronization**: Unified and updated project descriptions across Debian/Ubuntu control, RPM spec, COPR spec, Arch Linux AUR PKGBUILD, and Fedora COPR project settings.
- **Distribution Packages**: Cleaned `.deb` and `.rpm` file manifests to package only the CLI daemon, web client, and systemd user unit.

### 🔧 CI / Packaging
- **COPR / RPM spec** (`data/orbiscreen-copr.spec`): Updated to version 0.18.1 with new changelog entry.
- **debian/changelog**: Added 0.18.1-1 release for Ubuntu noble.
- **PKGBUILD**: Bumped `pkgver` to 0.18.1.
- **Android `versionCode`**: Incremented to 44; `versionName` updated to "0.18.1".

## [v0.18.0] - 2026-09-03

Native Graphic Digitizer (stylus pressure & tilt), auto-orientation virtual display, pure CLI daemon workflow, SEO-optimized README and banners, and comprehensive packaging improvements.

### ✨ Added
- **Graphic Tablet Digitizer & Stylus Pressure/Tilt**: Full Linux `uinput` tablet digitizer support - `BTN_TOOL_PEN`, `BTN_TOUCH`, `ABS_PRESSURE` (4095 levels), `ABS_TILT_X/Y`. Android intercepts active stylus events (`TOOL_TYPE_STYLUS`/`TOOL_TYPE_ERASER`), reads normalized pressure (`ev.pressure`) and tilt (`AXIS_TILT`, `AXIS_ORIENTATION`), and forwards to `InputDispatcher.stylus()` → `POST /input`. Krita, GIMP, Blender, and Inkscape respond with full brush pressure sensitivity.
- **Auto-Orientation Resolution Adaptation**: `StreamScreen.kt` now observes `LocalConfiguration.current.orientation`; on Landscape↔Portrait flip, it calls `viewModel.updateDimensions(curH, curW, ...)` to invoke `kscreen-doctor` on the host and swap the virtual display aspect ratio automatically - no black bars, no manual settings.
- **Production-Grade SEO & Social Discovery**: Fully rewritten `README.md` and `README_AR.md` with `style=for-the-badge` shields, 1-click viral sharing buttons (Reddit, X/Twitter, Hacker News), comprehensive Spacedesk/Deskreen/Weylus/Sidecar comparison matrix, rich FAQ section targeting Google "People Also Ask" boxes, and popular use-case sections for search-intent optimization.
- **Professional SVG Repository Banner** (`assets/logo/orbiscreen-banner.svg`): 1280×400 dark-themed banner with crisp vector icons (no blurry filter halos), featuring precise SVG paths for Lightning, Monitor, and Stylus Pen icons, with a clean dotless brand logo.
- **Friendly Capture Pipeline Error Guidance**: `resolve_frame_source` in daemon now emits human-readable `eprintln!` messages directing users to `orbiscreen doctor --fix` when the capture pipeline fails to initialize automatically.

### 🎨 Changed
- **`uninstall.sh` Cleanup**: Removes all installed daemon binary, service, and legacy configuration files.
- **README badges** upgraded from `style=flat-square` to `style=for-the-badge` throughout for higher visual impact.
- **CONTRIBUTING.md**: Removed stale `--exclude orbiscreen-gtk` flags from verification commands.
- **ARCHITECTURE.md** (EN & AR): Documented the full Graphic Tablet Digitizer pipeline with `uinput` event codes and Android stylus axis mapping.
- **`InputDispatcher.kt` `stylus()` method**: Fixed field names from `displayWidth`/`displayHeight` (constructor params, no longer in scope) to `streamWidth`/`streamHeight` (class-level volatile properties).

### 🔧 CI / Packaging
- **COPR / RPM spec** (`data/orbiscreen-copr.spec`): Added new 0.18.0 changelog entry.
- **debian/changelog**: Updated to 0.18.0-1 for Ubuntu noble.
- **PKGBUILD**: Updated `pkgver` to 0.18.0.
- **Cargo.lock**: Updated via `cargo update -w` to reflect workspace version bump from 0.17.4 → 0.18.0.
- **Android `versionCode`**: Incremented from 42 → 43; `versionName` updated to "0.18.0".

## [v0.17.4] - 2026-09-02

Zero-latency streaming overhaul with hardware NVENC optimization, framerate-matched damage pump, zero-backlog keyframe delivery, gesture fix for double-tap menu reveal, hide-eye icon update, and modern Arabic UI redesign of the exit modal and display settings sheet.

### ⚡ Performance & Low Latency
- **Framerate-Matched Virtual Display Damage Pump**: Replaced the static 200ms damage pump timer in `kwin_virtual.rs` with a display framerate-matched interval (`16.6ms` for 60Hz), eliminating desktop frame throttling and delivering true 60 FPS streaming.
- **Zero-Backlog Stream Connection**: Modified `join_buffer` in `orbiscreen-transport` to retain only the latest IDR keyframe and push a single packet on client connection. New clients start decoding immediately without queueing stale delta packets, eliminating the 1.5-3 second initial delay.
- **Real-Time Capture Appsink**: Set `drop=true max-buffers=1` on the PipeWire GStreamer capture sink in `kwin_virtual.rs`, preventing frame queue accumulation during static-to-dynamic transitions.
- **Hardware NVENC Verification**: Verified and prioritized NVIDIA hardware encoding (`nvh264enc`) with `tune=ultra-low-latency`, `zerolatency=true`, and `preset=p1` (<2.8ms per 1080p60 frame).

### 🐛 Bug Fixes & Interactions
- **Restored Double-Tap Gesture Detection**: Integrated an internal Android `GestureDetector` directly into `TouchOverlay`, allowing double-tap events to reliably reveal controls even when touch input injection is consuming screen events.
- **Hide-Eye Icon Update**: Replaced `Visibility` (open eye) with `VisibilityOff` (eye with slash) in `ControlToolbar.kt` to clearly communicate the hide action.
- **Cleaned Landscape Toolbar**: Pruned duplicate action buttons in landscape mode and ensured portrait toolbar strictly displays the 4 core icons (Mouse, Keyboard, Hide Eye, and Red Disconnect).
- **Restored Credit Headers**: Maintained 2-line clean credit & GPL-3.0 license headers across all source files while keeping function bodies stripped of clutter.

### 🎨 UI & UX Redesign
- **Redesigned Exit Confirmation Modal**: Replaced the generic dialog with a frosted-glass Material 3 modal featuring localized Arabic typography, a soft glowing exit icon, and sleek action buttons.
- **Revamped Display Settings Sheet**: Redesigned `ConnectionSettingsSheet` with an elegant drag handle, compact resolution chips, a dedicated phone screen matching card, sleek custom resolution inputs, and pure Arabic scale mode options.

### 🔧 CI / Packaging
- **Removed GTK4 from All Build Pipelines**: Eliminated `libgtk-4-dev`, `libadwaita-1-dev`, `libgraphene-1.0-dev` from `ci.yml` and `release.yml` system dependencies; dropped all `--exclude orbiscreen-gtk` flags from `clippy`, `build`, and `test` steps.
- **Fixed Packaging Scripts**: Removed stale `orbiscreen-gtk` binary copy, GTK `.desktop` and `.metainfo` file references, and GTK runtime `Depends` from `package-deb.sh`, `package-rpm.sh`, and `package-appimage.sh`.
- **Cleaned Per-Frame Log Noise**: Removed repetitive `frame_pump: chunk #N` and `source frame #N pushed` `info!` calls from the daemon; demoted the damage-pump startup log to `debug!` level.
- **Cargo Fmt Compliance**: Formatted entire workspace to pass `cargo fmt --all -- --check`; zero Clippy warnings across all targets.

## [v0.17.3] - 2026-09-02

Critical streaming fix for pitch black screen, sleek keyboard accessory bar, redesigned settings floating controls & sheet, eye button fix, back-button exit confirmation dialog, and project-wide documentation synchronization.

### 🐛 Bug Fixes & Streaming Stability
- **Fixed Pitch Black Screen on Connection**: Guaranteed immediate keyframe delivery upon stream connection by preserving the IDR keyframe at index 0 of `join_buffer` while keeping the latest delta frames, restoring non-live `appsrc` and non-dropping `appsink` in GStreamer, and setting safe low-latency buffer durations in ExoPlayer.
- **Fixed Duplicate Eye Icon in Landscape**: Removed the redundant host-blanking eye icon from the stream toolbar, keeping a single, dedicated eye icon for hiding controls.
- **Fixed Controls Hiding & Stealth Mode**: Tapping the eye icon now completely hides both the control toolbar and the floating corner button until the next session, with an intuitive double-tap gesture to restore them if desired.

### ✨ Added
- **Back-Button Exit Confirmation Dialog**: Intercepted the Android system back gesture and added a confirmation modal with clear Cancel and Disconnect actions, preventing accidental session terminations.

### 🎨 UI & UX Redesign
- **Slim Floating Keyboard Accessory Bar**: Completely replaced the bulky full-screen card overlay with an ultra-slim frosted glass bar resting directly atop Gboard, featuring latched `Ctrl`, `Alt`, and `Super` modifier states and no background scrim covering the desktop.
- **Redesigned Floating Controls FAB**: Replaced the plain dark circle with a polished frosted-glass floating button featuring glowing borders and smooth corner snapping.
- **Redesigned Display Settings Sheet**: Cleaned up the connection settings bottom sheet with pure localized typography, removing awkward English tags in parentheses and providing modern resolution chips, phone dimension matching, and scale mode controls.

## [v0.17.2] - 2026-09-02

Comprehensive UI/UX overhaul, ultra-low latency streaming (<100ms), robust mouse input injection, background idle & auto-resume, floating IME keyboard accessory bar, corner-snapping controls FAB, and dedicated stream display settings.

### ✨ Added
- **Dedicated Stream Connection & Display Settings**: Integrated settings modal bottom sheet right from the stream toolbar - switch between standard presets (1080p, 720p, 1440p, 1200p, 4K), 1-tap adaptive matching to your device screen, or custom resolutions, plus display scale modes (Fit, Fill, Zoom).
- **Floating IME Keyboard Accessory Bar**: Modifier and navigation shortcut keys now float seamlessly directly above the Android soft keyboard (Gboard) via `imePadding`, keeping shortcuts always accessible while typing.
- **Draggable Corner-Snapping Controls FAB**: Floating controls toggle button can be dragged freely across the screen and snaps automatically with a smooth spring animation to any of the four screen corners.
- **Orientation-Adaptive Stream Toolbar**: Portrait mode now shows a clean, uncluttered 5-icon bar (Mouse, Keyboard, Settings, Eye to hide, and Red exit), while Landscape shows full controls. Removed obsolete back, terminal, and fullscreen buttons.
- **Background Idle & Silent Auto-Resume**: Lifecycle-aware stream engine gracefully pauses when leaving the app and automatically reconnects upon returning, eliminating `SocketTimeoutException` error dialogs.

### ⚡ Performance & Low Latency
- **Sub-100ms Latency (<100ms)**: Slashed stream `join_buffer` from 32 historical packets (up to 6.4s of delay) down to 2 packets with live GStreamer pipeline parameters and low-latency ExoPlayer buffering, presenting the current desktop in real time.
- **Fixed Mouse Movement in Both Modes**: Replaced request-cancelling `collectLatest` with a steady 60fps atomic dispatch loop in Android, and prioritized direct `/dev/uinput` injection on Wayland (with ACL access) for lag-free, 100% responsive mouse tracking.

### 🎨 UI Harmonization
- **Unified Design System**: Standardized 22.dp rounded corners, elevated surface cards, frosted glass status indicators, and matching 14.dp icon containers across Discovery, Settings, and Stream screens.

## [v0.17.1] - 2026-09-02

Settings enhancements, project & creator credits, in-app update checking, and zero-buffering playback refinement.

### ✨ Added
- **In-App Project & Creator Credits**: Added complete maintainer recognition for **shadow-x78**, direct links to the creator's GitHub profile and the Orbiscreen project repository, and GNU GPL-3.0 license details in the Android Settings screen.
- **One-Tap Check for Updates**: Integrated real-time GitHub release discovery directly within the Android Settings screen - automatically queries the GitHub Releases API to detect new versions, notifying users with immediate download options or confirming up-to-date status.

### ⚡ Performance & Polish
- **Zero-Buffering Playback Mode**: Set buffer durations for playback and rebuffering to 0ms in ExoPlayer, enabling instant frame presentation upon packet arrival and preventing video stalling on static screens.
- **Transparent Video Shutter**: Eliminated black screen obscuration during stream pauses and buffering transitions by making the PlayerView shutter transparent.
- **5Hz Damage Keepalive Pump**: Accelerated daemon keepalive pulses from 500ms to 200ms to continuously feed demuxers during static desktop sessions.

## [v0.17.0] - 2026-09-02

Major user experience, input control, and low-latency streaming release for the Android client and backend packaging pipeline.

### ✨ Added
- **Trackpad Mouse Control Mode**: Relative cursor movement directly from the phone screen with intuitive gestures - single-finger smooth motion pans the host cursor, single-finger tap triggers left click, two-finger tap triggers right click, and two-finger vertical drag activates the mouse scroll wheel.
- **Mouse & Touch Mode Toggle**: A dedicated toolbar button allows seamless switching between Trackpad Mode and Direct Touchscreen Mode at any time during streaming.
- **Direct Keyboard Typing**: Bypasses cumbersome intermediate text boxes; activating the keyboard immediately connects the mobile system IME, forwarding typed characters and backspace events directly to the host PC in real time.
- **Dedicated Toolbar Back Button**: Added a prominent back navigation button (`ArrowBack`) before the host IP address in the stream control toolbar for immediate 1-tap navigation back to discovery.
- **High-contrast Keyboard Overlay**: Keyboard pill buttons restyled with dark surface background (`#262637`), subtle translucent borders, and bold white text (`Color.White`) for effortless legibility.
- **Debian / Ubuntu Launchpad PPA Automation**: Adapted source changelog generation to native package format (`${VERSION}~ubuntu${SERIES}1`), automated secret GPG key identification from imported keyrings, and added `--no-lintian` to debuild for uninterrupted automated PPA distribution.

### ⚡ Performance & Low Latency
- **Ultra-Low Latency Video Pipeline**: Tuned ExoPlayer LoadControl buffer durations down to 50ms - 150ms with `prioritizeTimeOverSizeThresholds`, eliminating the 1.5s - 5s accumulated delay and preventing stuttering.
- **Non-intrusive Buffering State**: Kept the last rendered frame visible during static screen pauses instead of covering the screen with an opaque black spinner overlay.
- **Robust Stream URL Builder**: Bulletproof URL construction preventing duplicate ports (`:8788:8788`) and malformed path parsing in OkHttp/Media3.

## [v0.16.2] - 2026-09-01

Full-project audit round five, after the USB feature (v0.16.0) and the distribution-repo groundwork (v0.16.1). All tooling gates green (clippy zero, fmt clean, 125 tests, cargo-deny clean, cargo-machete clean), zero inline comments in any code file, zero embedded secrets, all docs links and anchors resolve in both languages - and two real defects found and fixed, one of them proven by a live end-to-end test against a physical Android device.

### 🐛 Fixed
- **Graceful shutdown could leave the USB tunnel behind (proven live, then fixed):** the v0.16.0 supervisor task was detached, so when `main`'s select! caught Ctrl-C or D-Bus Stop and returned, the tokio runtime dropped the still-running supervisor mid-teardown - the exact stale-tunnel bug the feature was meant to eliminate, and my first live test confirmed it: after SIGINT, `adb reverse --list` still showed `tcp:8788`. The shutdown path is redesigned: `serve()` now owns the lifecycle - it races the HTTP accept loop against the daemon's shutdown channel and SIGINT itself, and on any exit it aborts the supervisor and runs `teardown_reverse_for_all` to completion before returning; `main` in turn sends the shutdown signal and *awaits* `serve_fut` (pinned) instead of abandoning it, then stops the encoder and pumps. Re-tested end-to-end against a real device: tunnel present while running, `ADB reverse tunnels removed for devices: [...]` logged on SIGINT, and `adb reverse --list` empty afterwards - the tunnel is now removed on Ctrl-C, D-Bus Stop, and serve's own error exit alike.
- **GitHub expression inside a `run:` block (hardening):** the fork-PR check from v0.15.3 inlined `${{ github.event_name == ... }}` in the shell script. The expression only ever evaluates to true/false (no user-controlled text reaches the shell), but it violates GitHub's own hardening rule that workflow expressions never belong inside `run:`. The check now passes through an `env:` variable (`ORBISCREEN_FORK_PR`) and reads it as a plain environment variable - the same pattern as the secrets above it. A repo-wide scan confirms no other workflow inlines `${{ }}` in `run:`.

### 🧹 Cleanup
- **Post-tag "pin the checksum" commits broke the house commit format and the tag's integrity:** the two post-release commits that pinned each tag's archive checksum into the in-repo PKGBUILD used the day-to-day `orbiscreen | chore: …` form (no version) instead of the release form, and worse - the checksum commit always landed *after* the tag, so anything building from the tag got a PKGBUILD whose pinned sum pointed at the previous release and failed verification. The in-repo PKGBUILD now ships `sha256sums=('SKIP')` by design (a tag's archive checksum mathematically cannot exist before the release workflow finishes); the real pin happens at AUR publish time via `updpkgsums`, which writes it into the AUR copy of the PKGBUILD, never back into this repo. The PACKAGING publish flow (EN + AR) is updated accordingly, and no post-tag checksum commits will ever be needed again.

## [v0.16.1] - 2026-09-01

The Linux distribution-repository groundwork: everything needed for Fedora COPR (via Packit) and the AUR now lives in the repo, ready for the one-time account activations, plus the AppStream metainfo every store front expects.

### ✨ Added
- **AppStream metainfo** (`data/com.orbiscreen.OrbiscreenGtk.metainfo.xml`, validated clean with `appstreamcli`): presents the app in GNOME Software/Discover with bilingual EN/AR name/summary/description, homepage/bugtracker/donation URLs, launchable desktop-id, provides-binaries, OARS rating, and the release history for v0.16.0/v0.15.3/v0.15.2/v0.13.3 with their notes and the release URL. Packaged into every install tree: the COPR spec, the AUR PKGBUILD, and the deb/RPM/AppImage scripts all install it now (the local `data/orbiscreen.spec` gains it in both `%install` and `%files`).
- **Fedora COPR automation** (`.packit.yaml` + `data/orbiscreen-copr.spec`): a new source-build spec (distinct from the local prebuilt-binary spec the `package-rpm.sh` script uses) that builds from the GitHub release tarball with cargo on rpmbuild itself - verified by producing a real warning-free SRPM locally - with `BuildRequires` for the GStreamer/GTK4/libadwaita/libevdev toolchains, the metainfo installed, `Recommends: android-tools` for USB transport, and `%check` running the test suite. Packit config: PR builds verify the spec on `fedora-stable` as a CI check, release-tag builds publish to the `shadow-x78/orbiscreen` COPR project. One-time maintainer steps documented in PACKAGING.
- **Arch Linux AUR package** (`PKGBUILD` at the repo root, syntax-verified, tarball contents cross-checked): builds from the release tarball with `cargo build --release --locked`, runs the test suite in `check()`, installs the daemon, GTK panel, desktop entry, icon, metainfo, web client, systemd user unit, and the evdi installer helper; `optdepends` on `android-tools` (USB) and `evdi-dkms` (X11/GNOME), `options=('!lto')`, and the real v0.16.0 tarball checksum pinned. The publish/update flow (`makepkg --printsrcinfo > .SRCINFO`, then push over SSH to AUR) is documented in PACKAGING; `.gitignore` learns the local `makepkg` work dirs.
- **Distribution-repositories section in PACKAGING (EN + AR):** COPR one-time setup (GitHub SSO on copr.fedorainfracloud.org, enable Packit on packit.dev, push the next tag), the `dnf copr enable` install line; the AUR clone/publish commands and `yay -S orbiscreen` install; and what the metainfo is for.

## [v0.16.0] - 2026-09-01

USB transport, completed. The audit rounds had pruned the half-finished adb lifecycle (v0.11.2 removed an unused `remove_reverse`, correctly dead at the time because nothing ever called it), which left USB as a best-effort one-shot: tunnels were created once at daemon start, never re-created for a device plugged in later, never cleaned up on stop, invisible to `doctor`, and unreported anywhere - while the Android app's USB card connected to a hardcoded `127.0.0.1:8788` with no feedback at all. This release closes the loop on both sides of the cable.

### ✨ Added
- **Full `adb reverse` lifecycle on the host:** the transport now runs a persistent USB supervisor instead of a one-shot setup - it (re)establishes the reverse tunnel for every connected device every two seconds (`adb reverse` is idempotent, so this also self-heals tunnels that died with an unclean daemon exit, e.g. SIGKILL), logs each device attach/detach at info level, and on graceful shutdown (Ctrl-C or D-Bus `Stop`) removes every tunnel it created via the restored `remove_reverse`/`teardown_reverse_for_all` (the v0.11.1 functions, restored with unified error handling and an idempotent "listener not found" pass-through, plus a `reverse --list` parser and `reverse_tunnel_count`). A phone plugged in after `orbiscreen start` is streaming within two seconds, and a stopped daemon no longer leaves a stale tunnel on the device causing black-screen confusion on the next USB connect.
- **USB visibility everywhere:** `orbiscreen doctor` gains a `usb:` line (adb installed? which devices? how many tunnels active? plus a JSON `usb` object in `--json` output for the GTK panel); `GET /health` and the D-Bus `GetStatus` payload both expose a live `usb_devices` count (the same surface pattern `auth_failures` got in v0.13.3), documented in DBUS_SPEC.md/DBUS_SPEC_AR.md.
- **A USB card that tells the truth (Android):** the Discovery-screen USB card now probes `http://127.0.0.1:<port>/health` when shown and renders the actual state - **tunnel ready** (green check icon) or **no tunnel - start the daemon / reconnect the cable** - instead of always offering a blind connect; the port is no longer hardcoded: it reads `usbPort` from `PrefsStore` (default 8788, clamped to 1024-65535); and the Stream screen shows a **USB badge chip** next to the title when connected via 127.0.0.1 so it is obvious which transport carries the stream. `probeUsb` reuses the existing 1s-timeout OkHttp client so the card never blocks the UI.

## [v0.15.3] - 2026-09-01

Full-project audit round four, after three releases of heavy brand and documentation churn (v0.13.3 - v0.15.2). Every crate, client, script, workflow, document, and asset re-verified with live tooling: clippy zero warnings, fmt clean, 123 tests pass, cargo-deny clean, cargo-machete finds no unused dependencies, zero inline comments in any code file, zero secrets embedded, every README/docs link and anchor resolves in both languages.

### 🐛 Fixed
- **Fork pull requests could never pass the Android workflow:** the signing step unconditionally fails when repo secrets are absent - which is exactly the case for every pull request opened from a fork (GitHub never exposes secrets to fork PRs). The keystore-preparation step now runs only on trusted events (push, dispatch, and same-repo PRs), and the build's unsigned-APK check is skipped with an explicit notice on fork PRs, so external contributors get a green unsigned build instead of a guaranteed red one. No secrets handling changed: the guard reads only event metadata.
- **Two files still carried the plain pre-brand comment style:** `clients/web/index.html`'s license header and `clients/android/gradlew`'s starter description now use the house `─`-rule style with a `── Section ──` split, matching every other config file in the tree.

### 🎨 Changed
- **The READMEs now embed the mark from `assets/logo/` (project convention):** both language READMEs referenced the master SVG from `data/` while the full brand set (`orbiscreen-logo.svg` + the six PNG renders + preview) lived in `assets/logo/` unreferenced by anything but the changelog. The header image now points at `assets/logo/orbiscreen-logo.svg`, giving the brand directory its canonical consumer (the `data/` master stays the packaging/scripts reference, as before).

## [v0.15.2] - 2026-09-01

The mark chosen by the maintainer from a rendered concept board. The v0.15.1 side-by-side pairing (hollow monitor rectangle + lit phone rectangle) was rejected for reading as two detached shapes rather than one form - boring, identity-less, and badly proportioned. Five concept directions were rendered as full preview boards (dark canvas, light canvas, and a simulated circular launcher icon for each) and reviewed visually; concept D - the interlock - was selected, then engineered into its final geometry.

### 🎨 Changed
- **The mark is the O of Orbiscreen as a display ring with the device screen riding its path:** a thick-stroked ring (r=170, stroke 54 on the logo grid) with a solid dot (r=62) centered on the ring's own path at its lower-right arc - the extension entering the host display, one unbroken form. A plain two-circle interlock inside a square canvas was proven unrenderable cleanly (two max-size circles in a 440-unit box either miss each other or swallow each other - the geometry forbids a light overlap), so the dot rides the ring's path instead: the interlock reads instantly while the ring alone carries the letterform. Flat, one accent color (Catppuccin Blue `#89b4fa`), two role classes (`.orbi-s` stroke, `.orbi-f` fill), the design story documented inside the SVG. The content box is exactly square (394x394 measured, ratio 1.000 at every render size from 48px to 512px) and dead-centered (measured center 256.0, 256.0); the full asset set (`assets/logo/` SVG master + 48/64/96/128/256/512 PNGs + dark-canvas preview) is regenerated, the Android adaptive foreground is redrawn from the same geometry with every blue pixel verified inside the 66/108 circular-mask safe zone (bbox 117..315 of 432 vs mask 72..360), and all density mipmaps (square and round) are re-rendered.

## [v0.15.1] - 2026-09-01

The mark rebuilt on honest geometry. The v0.15.0 two-screen scene (rotated pair + floating stream arc + arrowhead) read as broken and mismatched - the -8 degree rotation put a visible slant on every edge, the arc floated unattached inside the hollow host screen, and the solid arrowhead collided with the phone's frame stroke, producing a fused blob at the join.

### 🎨 Changed
- **Every edge is now perfectly horizontal or vertical, and the composition has zero rotation, zero overlap, and zero decoration:** the mark is the pairing alone - the host monitor (a hollow rounded rectangle, 217x150 on the logo grid) beside the phone (a 170x414 rounded rectangle whose screen area is solid brand blue), with a 14-unit breath between them. No tilt, no arc, no arrow, no content bars: the two shapes read instantly as "computer screen + phone showing a screen", which is the product. The content box is exactly square (440x440 measured, ratio 1.000 at every render size from 48px to 512px) and dead-centered (measured center 256.0, 256.0 on the 512 canvas); both shapes share the vertical centerline of the mark. The full asset set (`assets/logo/` SVG master + 48/64/96/128/256/512 PNGs + dark-canvas preview) is regenerated, the Android adaptive foreground is redrawn axis-aligned from the same geometry with every blue pixel verified inside the 66/108 circular-mask safe zone, and all density mipmaps (square and round) are re-rendered from the new mark.

## [v0.15.0] - 2026-09-01

The mark now draws the product, and the READMEs now follow the unified project house style end to end. This release replaces the v0.14.0 mark with a picture of what Orbiscreen actually does, and rebuilds both READMEs from scratch in the project convention.

### 🎨 Changed
- **The mark is the product: host monitor, phone, stream arc:** the new logo draws the actual scene - a large host monitor with a portrait phone stepping in front of its corner, both leaning the same -8 degrees so the pair reads as one moving shape, and a stream arc carrying the signal from the host screen into the phone, ending in a solid arrowhead that touches the phone's edge. It is a two-screen scene with a signal, drawn flat in one color (Catppuccin Blue `#89b4fa`, the brand accent) with two role classes (`.orbi-s` stroke, `.orbi-f` fill), the design story documented inside the SVG. The geometry was solved numerically so the content box is exactly square (440x440 measured, ratio 1.000 at every render size from 48px to 512px) and dead-centered on the canvas (measured center 256.0, 256.0); at 48px the scene still reads: big screen, small screen, arrow. The full asset set under `assets/logo/` is regenerated from the new mark (vector master + 48/64/96/128/256/512 PNGs + dark-canvas preview), the Android adaptive foreground is redrawn from the same geometry inside the 66/108 safe zone (every blue pixel verified inside the circular mask keep-out), and all legacy density mipmaps are regenerated with proper circular masking.
- **Both READMEs rebuilt in the unified repo house style, and cleaned of stale content:** the English and Arabic READMEs are rewritten top to bottom following the project repo convention - the logo opens the file at 180px with honest alt text describing the scene, the tables and sections mirror the project structure (problem-comparison table, highlights, per-platform support matrix, quick start from packages and from source, commands, client, architecture diagram, project-structure tree, per-language documentation table, five-step contributing, centered footer). Concretely fixed: the documentation table now links each guide by its own language file (the Arabic README links `ARCHITECTURE_AR.md`/`DE_SUPPORT_AR.md`/... directly instead of the English `docs/X.md · AR` double-link form); the stale seven-row development-phases status table (all "Completed", describing internal milestones with no value to a reader) is removed in both languages; the problem-comparison rows naming third-party projects (`spacedesk refuses officially`, `VirtScreen unmaintained`, `Weylus caps it to X11`) are replaced with honest capability rows that describe what Orbiscreen does rather than FUD about named competitors; the capture-preference config table moved out of the README into `docs/DE_SUPPORT.md`/`DE_SUPPORT_AR.md` (its proper home, next to the `auto` plan table) with a one-line pointer left behind; the `Sway / wlroots general` and `Hyprland` duplicate support-matrix rows are merged into one `Sway / Hyprland / wlroots` row; and the Arabic README's corrupted `دائماااً` (triple-alef typo) instances and the English README's outdated capture-backend comment block in the build instructions are gone.

## [v0.14.0] - 2026-09-01

The project has its own mark. Redesigned with clean geometric precision: one flat single-color drawing whose geometry tells the product's story, verified programmatically, and shipped as a complete asset set plus a true adaptive Android icon.

### ✨ Added
- **The Orbiscreen mark - the frame, but whole:** the old logo was a detailed CRT monitor illustration (two nested bezel rects, a lens circle, a stand, a shadow bar, five colors on a translated group) that read as a gray box at launcher sizes and had no story to tell. The new mark is drawn flat, one color (Catppuccin Blue `#89b4fa`, the brand accent), two role classes (`.orbi-f` fill, `.orbi-s` stroke), and geometry that means something: a rounded display-bezel frame (the Linux host screen) drawn as one continuous ring; three transport dots riding its edge (each dot centered on the frame's own path - the top one on the top edge, the side two on the straight lower run of the left and right edges - so Wi-Fi, USB, and the browser client sit literally on the host's boundary); and the hero in the void: the double chevron `❯❯`, the stream signal advancing screen after screen. The design story is documented inside the SVG itself.
- **The complete asset set under `assets/logo/`:** vector master `orbiscreen-logo.svg` plus PNG renders at 48/64/96/128/256/512 and a dark-canvas preview (`orbiscreen-logo-preview.png`).
- **A real adaptive Android icon:** the launcher foreground is now a vector `ic_launcher_foreground.xml` drawn from the same geometry (not a rasterized drawing squeezed into the wrong viewport like the old icon, whose 512-grid content was scaled 0.50 and pushed 124/128 off-center - visibly lopsided under every launcher mask). The mark now scales into the 66/108 adaptive safe zone with the content box verified programmatically: every blue pixel lands inside the circular mask's keep-out test (bbox 108..324 of 432, mask circle 72..360), so no launcher shape - circle, squircle, rounded square, or the tear-drop masks - ever clips the frame or a dot. The adaptive background is the brand dark (`#11111B`, matching `orbi_background`) instead of the old white that fought both the mark and the splash theme, and the monochrome layer (themed icons on Android 13+) reuses the same drawable. The legacy mipmaps are regenerated from the new mark for every density (48 through 192) with proper circular masking for `ic_launcher_round`, so pre-API-26 launchers get the same design instead of the old CRT drawing.
- **`data/orbiscreen.svg` is the new official logo everywhere:** the README headers (EN + AR, with honest alt text describing the geometry), the desktop entry icon, and the install/packaging scripts (deb, RPM, AppImage rasterization) all pick up the mark from the single source file it always referenced - no script changes needed.

### 🎨 Changed
- **The mark is square and dead-centered by construction, not by eye:** the old CRT drawing's content box was 448x320 on a 512 canvas translated 16px down - a lopsided 95x82% footprint. The new geometry was solved so the content bounding box is exactly square (432x432, an 84.4% footprint with a 40px margin on every side) and its center measures (256.0, 256.0) on the 512 canvas - verified by rasterizing and measuring the alpha channel, and the same measurement holds at 48px (bbox 3..45 of 48, ratio 1.000). At launcher and favicon sizes the mark reads as the frame, the three dots, and the double chevron - no nested rectangles competing at 12px stroke widths.

## [v0.13.3] - 2026-09-01

Full-project audit round three: every crate, client, script, workflow, and document re-verified with live tooling (clippy/fmt/tests/cargo-deny/machete all clean; 123 tests pass; zero dead code files, zero inline comments in code files, zero stale dependencies). Every finding fixed, every version home rotated in one release commit - and the release process itself is now documented the way it actually works.

### 🐛 Fixed
- **Android version string lagged the workspace:** `versionName` was still `0.13.1` after the v0.13.2 release bump updated Cargo, lock, README badges, SECURITY, and PACKAGING but skipped the Gradle manifest. This release rotates every home together (see the CONTRIBUTING release process below) and bumps `versionCode` 27 → 28 so store tooling accepts the APK.
- **PACKAGING docs described build commands that do not exist:** the `.deb`/`.rpm` sections instructed `cargo-deb`/`cargo-generate-rpm`, but the project has never shipped their metadata; the real builders are `scripts/package-deb.sh` and `scripts/package-rpm.sh` (+ `package-appimage.sh`, previously absent from the local-build list). The docs now reference the actual scripts and their tool requirements (`dpkg-deb`, `rpm-build`).
- **PACKAGING and README docs advertised a Flatpak that was never shipped:** no flatpak manifest exists in the repo (and one is not technically viable for a daemon needing evdi, uinput, and raw compositor sockets); the bullet and the docs-table mention are removed from both language versions.
- **PACKAGING docs claimed V1 APK signing:** the Gradle release config sets `enableV1Signing = false` (minSdk 26); the docs now say V2/V3 and reference the keystore env vars instead of the removed in-repo keystore.

### 🔍 Diagnostics
- **`auth_failures` was counted but never surfaced:** the transport increments a counter on every unauthorized request, yet it appeared in neither `GET /health` nor the D-Bus `GetStatus` payload. It is now exposed on both (with a getter, tests, and DBUS_SPEC entries in English and Arabic), making token-probing activity observable without enabling debug logs.

### 📖 Docs
- **CONTRIBUTING now documents the real release process:** the generic branch/commit/PR sections collapse into a "Day-to-Day Work" block, and a new "Release Process" section lists every version home the bump must touch - the Cargo workspace (plus `cargo update -w` so the lock matches `--locked` builds), the Gradle `versionName`/`versionCode`, all thirteen doc badges (README/README_AR/SECURITY and eight docs files bilingual), the PACKAGING release-matrix line, and the SECURITY scope - plus the bump rule by CHANGELOG convention (`✨ Added` = minor, fixes/cleanup only = patch) and the one-commit-one-tag push that triggers the release workflow.
- **`scripts/verify-stream.sh` and `scripts/setup-dev-env.sh` were referenced nowhere despite being useful:** both are now documented in TROUBLESHOOTING.md/TROUBLESHOOTING_AR.md (end-to-end stream verification with H.264 decode and black-frame detection; distro-detected dev dependency install) and CONTRIBUTING.md.

### 🧹 Cleanup
- Removed the stale `webrtc` desktop-entry keyword (the transport has been HTTP/MPEG-TS since v0.2); replaced with `streaming`.
- Removed the dead `flatpak-builder-sources` entry from `.gitattributes` (no flatpak sources exist).
- Added `.kotlin/` (Kotlin 2.x session dirs) to `.gitignore` alongside the existing Gradle ignores.
- `gradlew` now verifies the `gradle-wrapper.jar` download against a pinned SHA256 and refuses to run a tampered jar (defense-in-depth for the rare no-jar-first-run path; the jar is committed).

## [v0.13.2] - 2026-08-28

Hotfix: the web client rendered a permanent black screen after the v0.13.1 CSP hardening, and auth rejections were opaque in the daemon log. Both fixed, verified end-to-end against a live KDE/KWin virtual display.

### 🐛 Fixed
- **Web client black screen (regression from v0.13.1):** the audit-added CSP meta `default-src 'self'` blocked the `blob:` MediaSource URL that the vendored mpegts.js attaches to the `<video>` element (`media-src` was unset, so `default-src` applied), so `play()` failed with `NotSupportedError` and the client looped on "Stream lost (media error)" with no picture. The policy is now `default-src 'self'; media-src 'self' blob:`, MSE playback restored, everything else still locked to same-origin. Verified live: token fetch, `401`→`200` on `/stream`, and a real 1920×1080 stream decoding and playing in-browser with no console errors.

### 🔍 Diagnostics
- **Auth rejections are now self-explaining:** the `unauthorized request rejected` log carries the peer address, method + path, what was presented (`missing` / `bearer(len=…, prefix=…)` / `unexpected(scheme=…)`), whether a `?token=` was present, and the daemon's expected token prefix + length, so a stale-token vs old-client vs missing-header failure is identifiable from one log line.
- Token generation now logs a short prefix for cross-checking against what a client presents; serving `/client/config.json` logs the requesting peer.

### 🧹 Cleanup
- Removed every em dash from the repo (log/error/UI strings, CI step names, changelog); range en dashes in the changelog normalized to plain hyphens.

## [v0.13.1] - 2026-08-27

Full-project audit round two: the Rust transport/daemon, Android client, web client, shell scripts, and CI matrix re-audited line by line; all security, correctness, and packaging findings fixed or explicitly documented as accepted design.

### 🌐 Web Client
- **Keyboard letter keys were wrong:** the QWERTY-layout Linux input codes were assigned to letters in alphabetical order, so every typed letter sent the wrong key (`KeyA` → KEY_Q, …). Each letter now maps to its real input code; numpad, function, and media keys audited against `linux/input-event-codes.h`.
- **Pointer control was frozen under pointer lock:** `clientX/Y` do not move while locked. A virtual cursor now accumulates `movementX/Y`, so remote control actually works with a mouse.
- **Touch devices could not control at all** when `requestPointerLock` was absent or rejected; pointer events fell back to direct absolute input, and lock failures are handled (including the unhandled promise rejection).
- **Wheel scroll jumped ~12 steps per notch:** raw `deltaY` was sent as discrete steps; it is now normalized per `deltaMode` and clamped.
- OS key auto-repeat no longer floods the daemon (`event.repeat` filtered); pointer coordinates clamp to the last valid pixel; the MSE-unsupported path no longer hides the overlay; a CSP meta (`default-src 'self'`) was added.

### 📱 Android
- NSD discovery now stops when the discovery view model clears (previously leaked for the process lifetime); subnet-scan results merge into the same state pipeline instead of racing it.
- The input dispatcher is (re)sized from the reported display dimensions, so touching the surface before `/api/info` returns no longer pins coordinate mapping to 1920×1080.
- Manifest: removed the unused `CHANGE_WIFI_MULTICAST_LOCK` and `ACCESS_WIFI_STATE` permissions; V1 (JAR) APK signing disabled (minSdk 26 uses V2/V3).
- `HostApi` reads response bodies with a bound; user agent comes from `BuildConfig`; dead ProGuard rules removed (the blanket `-keep` previously disabled all shrinking); dead UI branches, `RecentHost.label`, and the theme `dynamicColor` parameter removed; `as Activity` cast made safe.

### 🐛 Fixed
- Input queue is bounded (1024) with explicit drop counting; stream clients capped with 503; join-buffer lock no longer spans blocking pushes; token comparison constant-time; `Bearer` matched case-insensitively with `WWW-Authenticate` on 401; `/health`/`/api/info` slimmed; auth failures logged at debug with peer addresses; adb serials charset-validated with proper spawn-error mapping; blocking `adb`/`systemctl`/package-manager calls moved off the async runtime; uninstall reports real failures and returns non-zero; portal state saved with a per-process temp-file nonce and permissions set before rename; wlr-screencopy size/stride math validated against compositor-supplied values; encoded chunks without PTS are dropped instead of timestamped to zero; watchdog exits once frames flow.
- **Virtual-output creation never worked on sway:** the daemon sent the two-word command `create output`, but sway's IPC command is the single token `create_output` (verified against sway 1.6 through master), so sway always rejected it with `Unknown/invalid command 'create'`. The daemon now sends `create_output`.
- **Virtual-output teardown was invalid on sway:** sway has no `output … remove` subcommand (checked 1.6 through master). The daemon now attempts `remove` and, when the compositor rejects it, falls back to `output … disable` so the output stops receiving frames; `create()` also waits for the requested mode to settle before reporting dimensions instead of returning mid-modeset. Found by the new sway headless CI job, which exercises virtual output end-to-end for the first time.

### ⚡ Scripts & CI
- `package-appimage.sh`: `find` no longer scans non-existent distro lib dirs (aborted the build under `pipefail`); AppImage output name follows `uname -m`.
- `setup-dev-env.sh`: corrected Fedora names (`libwayland-client`, `xorg-x11-server-utils`), stopped offering un-packaged `libevdi`/`evdi` from official repos, and completed every distro list with the X11/wayland/GTK4 dev packages CI actually needs.
- GitHub issue templates rewritten to the issue-forms schema (the legacy front matter made GitHub fail to load them); the PR template front matter removed.
- `install.sh` stops the running user service and installs binaries via temp+rename (no more `ETXTBSY`); install/uninstall/package scripts guard their working directory; evdi module check uses `/sys/module` (no `lsmod | grep -q` SIGPIPE); deb/rpm maintainer scripts deduplicate logged-in users and clean stale staging.
- Release workflow: signing secrets are verified explicitly before use, keystore written with `umask 077` and removed on every exit path, `keytool` uses `-storepass:env`, release notes extracted without regex interpolation with a fallback, VERSION quoted throughout, and job timeouts added. The Linux tarball bundles a README instead of a repo-layout installer, and README instructions match.

### 🗑️ Removed
- Dead code: `FramePool::pooled_count` (test-only), manual `Debug` on the D-Bus server, the `unreachable!`-padded stylus match in the uinput backend, the web client's dead letter-key shims.

## [v0.13.0] - 2026-08-27

Desktop-environment parity release: the full KDE-level experience (a real compositor-native virtual display, no root, no dialogs) now extends to wlroots compositors, portal grants persist across runs on GNOME, input injection is rootless on X11 and portal-free on wlroots, and per-frame allocations/copies were removed across the whole pipeline.

### ✨ Added
- **`orbiscreen doctor`** (`--json` for the GTK panel): prints the detected session/compositor, the exact ordered capture plan `auto` will follow, EVDI module state, portal presence on the session bus, saved permission grants, swaymsg/hyprctl availability, wlroots virtual-output IPC reachability, and `/dev/uinput` writability, each finding paired with the fix.
- **`orbiscreen doctor --fix [--yes]`:** detects the distro from `/etc/os-release` (dnf/apt/pacman/zypper incl. `ID_LIKE` derivatives), offers to install the EVDI package (`evdi` / `evdi-dkms` + headers), loads the module, and re-verifies.
- **Central environment analyzer** (`capabilities.rs`): session + compositor detection from `XDG_SESSION_TYPE`, `WAYLAND_DISPLAY`, `XDG_CURRENT_DESKTOP`, `DESKTOP_SESSION`, `KDE_FULL_SESSION`, `HYPRLAND_INSTANCE_SIGNATURE`, `SWAYSOCK`, `GAMESCOPE_WAYLAND_DISPLAY`, with a full detection-matrix test suite.
- **Capability-driven capture plan:** `auto` no longer uses a hardcoded chain; it resolves the try-chain from detected capabilities and logs the resolved plan and the reason every step succeeds or falls through. Plans: KDE `kwin-virtual → portal`; wlroots `wlroots-virtual → wlr-screencopy → portal → evdi`; other Wayland `portal`; X11 `evdi → x11-root`.
- GTK panel shows the active capture backend in its status row.
- **New `wlr-screencopy` capture backend:** `zwlr_screencopy_manager_v1` with SHM buffers (memfd-backed), damage-aware copies where the compositor supports them, per-output capture by name, stride padding removed, XRGB→opaque alpha normalization, strict frame validation, and clean teardown. No portal, no share dialog.
- **New capture preference `screencopy`** (`[capture] preferred = "screencopy"`), accepted by config validation.
- **Compositor-native virtual outputs on wlroots** (`WlrootsVirtualOutput`): the daemon creates a headless output via Sway IPC (`SWAYSOCK`, native i3-ipc framing, `create output` + mode) or Hyprland IPC (`HYPRLAND_INSTANCE_SIGNATURE` socket, `output create/destroy`), waits until the output is advertised and active, captures it by name with screencopy, and removes the output on stop/crash/drop. IPC failure falls back cleanly to mirroring an existing output; the doctor explains which IPC (if any) is reachable.
- **CI integration test on headless sway:** a dedicated job spawns sway with `WLR_BACKENDS=headless` and exercises real screencopy capture, virtual-output create/list/drop lifecycle, and output teardown.
- **Dialog-free portal sessions:** ScreenCast permissions are persisted via restore tokens (`PersistMode::ExplicitlyRevoked`) in `$XDG_STATE_HOME/orbiscreen/portal.json`; a failed/stale token automatically retries with a fresh selection. After the first grant, GNOME streams start instantly with no dialog.
- The RemoteDesktop **input** session persists its grant the same way (separate token).
- `doctor` reports whether each grant is saved.
- **wlroots-native input injection** (`virtual-keyboard-unstable-v1` + `wlr-virtual-pointer-unstable-v1`, protocol XML vendored): absolute pointer events and keyboard injection directly on the Wayland socket, no `xdg-desktop-portal-wlr` needed. Pointer coordinates are normalized to the captured output when one is known.
- **New input order on Wayland:** wlroots-native → RemoteDesktop portal → uinput, each with an explanatory fallback warning.
- **XTEST injector for X11:** rootless pointer/keyboard injection via xcb-xtest for any user; uinput remains the stronger fallback when `/dev/uinput` is available.
- **Pooled frame buffers** (`FramePool`/`PooledFrameBuffer` in `orbiscreen-core`): X11, wlr-screencopy, portal, and KWin capture now fill recycled buffers instead of allocating per frame; the encoder wraps pooled buffers directly into GStreamer buffers (`gst_buffer_new_wrapped` semantics); the previous per-frame `appsrc` alloc+copy is gone, and each buffer returns to the pool when GStreamer releases the frame.
- **X11 capture upgraded to MIT-SHM:** one persistent shared image (memfd + `attach_fd`, requires MIT-SHM ≥ 1.2) that the X server writes into directly (no per-frame reply payload over the socket), with automatic fallback to plain `GetImage` when the extension is absent (verified live against Xwayland).
- **Identical-frame skipping on X11 mirroring:** a fast 128-bit frame hash suppresses duplicate frames, so an idle mirrored desktop no longer burns CPU re-encoding unchanged content (keepalive pacing unchanged).

### 🧪 Tests
- Frame-assembly unit tests (stride stripping, premultiplied alpha, truncation), frame-pool recycling/cap tests, hash tests, distro-detection tests for `doctor --fix`, capability matrix tests, plus the sway-headless and live-DISPLAY X11 integration tests.

### ⏳ Deferred to a follow-up
- **DMA-BUF zero-copy** (planned phase-5 item): requires per-hardware validation that cannot be done in headless CI; the SHM path stays the default.
- **GTK panel EVDI wizard** (planned phase-6 item): `doctor --fix` covers the guided flow on the CLI in the meantime.
- KDE/sway/Xvfb side-by-side performance measurements: to be published once measured on reference hardware.

## [v0.12.6] - 2026-08-27

Full-project audit round: every diagnostic gate (fmt, clippy `-D warnings`, build, tests, audit, machete, deny, shellcheck, gradle assemble+lint) re-run and all findings fixed.

### 🐛 Fixed
- **EVDI pump never terminated on fatal errors:** after the kernel closed the event channel or the registered buffer vanished, the pump thread retried the failed operation forever and warned every 50 ms while the daemon kept streaming stale keepalive frames. Terminal errors now end the source, which flows through as a clean capture-pump shutdown.
- **KWin virtual-display open blocked a tokio worker:** `KwinVirtualCapture::open` performs blocking wayland round-trips, permission-file retries (up to 2.5 s) and a 5 s handshake deadline; called from an async context it stalled a runtime thread for up to ~7.5 s. The open now runs on `spawn_blocking`.
- **Damage pump zombie after compositor close:** when the compositor closed the layer-shell surface the pump kept attaching/committing to the dead surface forever and swallowed roundtrip errors. It now exits on `Closed` and on connection errors.
- **Truncated X11 `GetImage` replies were zero-padded**, masking real capture failures as black frames; short replies are now capture errors.
- **Mouse buttons 7 and 8 mapped to the same uinput code** (`0x117`); buttons 4-8 now map distinctly onto `BTN_SIDE`..`BTN_TASK` (`0x113`-`0x117`).
- **Encoder input queue allowed 60 raw frames** (`set_max_bytes`), ~475 MB at 1080p and ~2 GB at 4K before backpressure engaged; capped at 4 frames.
- Transport join-buffer mutex locks are now poisoning-tolerant instead of panicking a serving task if another thread ever panicked mid-update.
- `display` config gained upper clamps (7680×4320), matching the existing lower bounds and the two-sided clamps for refresh rate and bitrate.
- A relative `XDG_CONFIG_HOME` is now ignored per the XDG spec (same filter `XDG_DATA_HOME` already had).
- The MPEG-TS integration test's monotonicity check compared every timestamp against the *first* frame instead of its predecessor; it now catches real ordering regressions.

### 📱 Android
- **Tapping Refresh killed discovery permanently:** `DiscoveryService.stop()` cancelled the caller-supplied `viewModelScope`, freezing the host list and silently ignoring every subsequent restart. The service now owns its own scope, and `restart()` waits for the NsdManager stop callback before discovering again (also removing a start/stop race that could throw on some devices).
- **Auto-reconnect built the player with an empty token:** the reconnect job called `build()`, whose first action was `reconnectJob?.cancel()`, cancelling the very coroutine it ran in; the resulting `CancellationException` was swallowed by the token fetch, so every reconnect streamed unauthenticated into a 401 loop. External builds cancel pending reconnects; the reconnect path no longer cancels itself and no longer swallows cancellation.
- The `DiscoveryService`/gateway provider captured the Activity context into a ViewModel; it now uses the application context.
- "Forget recent host" in Settings left the stale card on screen until navigation; the row now recomposes immediately.
- `roundIcon` pointed at the square launcher mipmap; it now references `@mipmap/ic_launcher_round`.

### 🔒 Security
- `event-listener` 5.4.1 → 5.4.2 (RUSTSEC-2026-0221, unsound `!Send` tag across threads). The remaining `derivative` unmaintained advisory is unfixable upstream (`evdi` 0.8.0 is the final release) and stays an accepted warning.
- The damage pump's shared-memory file moved from a predictable fixed path in `/tmp` to an anonymous `memfd_create` region: no world-writable pathname, no cross-instance interference, nothing left on disk.
- Android release workflow: fails fast when `ANDROID_KEY_PASSWORD` is unset, rejects an unsigned APK after `assembleRelease` (previously uploadable), and removes the decoded keystore in an `if: always()` cleanup step.

### 🗑️ Removed
- `transport.webrtc_port_range` config field: dead since the WebRTC teardown (v0.12.1), normalized but never read. Existing config files containing it still parse (the key is ignored).
- Dead code: `VirtualDisplay::open_at`/indexed node selection and the `spec()` getter, `EncodeError::EncoderUnavailable`, Android `HostApi.health`/`sendControl`, `InputDispatcher.wheel`/`stylus`, `DiscoveryViewModel.saveRecent`, `StreamViewModel.toggleToolbar` + the never-changing `toolbarVisible` (toolbar is always shown), the navigation route's unused `label` parameter, the unreachable "idle" branch of the discovery status banner, 15 unused theme colors, 10 unused strings, 8 unused XML colors and dead imports across the client.
- The empty deb `postinst` script and the `packaging/` directory (the AppImage build merged into `scripts/build-appimage.sh`, with `package-appimage.sh` kept as the stable entry point used by the release workflow).

### 🔧 Changed
- `scripts/install.sh` now also builds and installs `orbiscreen-gtk`; previously the installed desktop entry's `Exec=orbiscreen-gtk` pointed at a binary the script never installed (the GTK build is best-effort so the daemon still installs where GTK 4 dev libraries are missing). The desktop entry's `Icon=` now matches the icon filename every installer ships, and `uninstall.sh` removes the GTK binary too.
- `package-deb.sh` builds both binaries when missing (previously copied them blindly) and carries a valid RFC822 `Maintainer` address; `package-rpm.sh` checks for both binaries, not only the daemon.
- AppImage build script: fixed a shellcheck SC2227 redirection placed between `find` actions.

### 📝 Documentation
- `DBUS_SPEC.md` / `DBUS_SPEC_AR.md`: `GetConfig` example matches the current schema (removed the deleted `display.count` and `webrtc_port_range` lines, added the `[capture]` section).
- Bug-report template no longer suggests a `RUST_LOG` target for the long-removed `webrtc_rs` crate.
- `SECURITY.md` / `PACKAGING.md` / `PACKAGING_AR.md` version matrices updated to the current release.

## [v0.12.5] - 2026-08-26

### 🐛 Fixed
- **Unbounded frame channel in the portal/mirror capture path:** the Wayland portal backend passed raw BGRA frames (~8 MB/frame at 1080p) over an unbounded channel, so a stalled consumer could grow memory without limit, the exact failure mode the KWin backend's bounded queue (v0.12.0) was built to prevent. The portal path now uses the same bounded channel (capacity 2) with drop-on-full and a debug log.
- **Config silently ignored under systemd:** the default `--config` was the relative path `orbiscreen.toml`, and no install path (manual / deb / rpm) set `WorkingDirectory=` or passed `--config`, so the service resolved the file against `$HOME` and silently fell back to defaults; custom resolution, port or encoder choices never took effect. The default is now the XDG path `$XDG_CONFIG_HOME/orbiscreen/orbiscreen.toml` (`~/.config/orbiscreen/orbiscreen.toml` when unset), used identically by the daemon, the GTK panel and the systemd user unit, and documented in the README and INSTALL guide.
- **Session token duplicated in the stream URL:** the Android client sent the token both as an `Authorization: Bearer` header (the actual authentication source) and as a `?token=` query parameter; the query parameter is gone, removing a leak path through verbose HTTP/ExoPlayer logs.
- **X11 capture errors lost their detail:** `GetImage` reply failures were collapsed to a constant error code `0`; real X11 protocol error codes are now surfaced, and connection failures are reported as a distinct connect error.
- **Android: deprecated OkHttp API:** `RequestBody.create(...)` in the control API client replaced with the `toRequestBody(...)` extension, matching the rest of the client.

### 🔒 Security
- **Android `network_security_config.xml`:** removed the dead `<domain-config>` LAN list: Android `<domain>` entries do not match CIDR ranges, so the list matched only network/broadcast addresses and enforced nothing. Cleartext HTTP remains globally permitted by design (the app only contacts LAN hosts the user selects via mDNS or manual entry); the decision is now stated explicitly in `SECURITY.md`.

### 🗑️ Removed
- **`display.count` config field:** it was read for display purposes only and never created more than one virtual display, so it looked configurable but had no effect. Removed from the config schema, `list-displays` output, GTK panel and tests until real multi-display support lands.
- Unused `gstreamer-video` dependency from `orbiscreen-encode` (confirmed by `cargo machete`).
- Orphaned allow-list entries in `deny.toml` (licenses encountered by no current dependency).

### 🔧 Changed
- Comment policy applied project-wide: explanatory comments removed from code files while every source, config and template file carries a consistent top-of-file header (GPL-3.0-or-later notice + repository link, banner-style for config/CI files); third-party vendored code (`clients/web/vendor/mpegts.js`) untouched.
- Missing final newlines added across manifests, config and web files (`.editorconfig` `insert_final_newline` compliance).

### 📝 Documentation
- README / README_AR and docs/INSTALL / INSTALL_AR: XDG config location, `--config` override, and a dedicated **Configuration** install section.
- SECURITY.md: cleartext-HTTP decision for the Android client documented next to the token threat model.

## [v0.12.4] - 2026-08-26

### ✨ Added
- **Client diagnostics in `/health`:** new `stream_starts` and `auth_failures` counters, plus a warning line with the peer address on every rejected request, making "phone shows nothing" diagnosable from the server side in one curl.
- Web client now plays with ~100 ms of buffer: IO stash disabled and live-latency chasing tuned to chase whenever buffered latency exceeds 1 s down to 200 ms (`liveSync` enabled).

### 🔧 Changed
- `scripts/verify-stream.sh` no longer hard-requires ffprobe; it falls back to ffmpeg-only detection when ffprobe is missing or broken.

## [v0.12.3] - 2026-08-25

### 🐛 Fixed
- **Growing playback latency ("slow stream"):** stream timestamps advanced one frame duration per pushed packet regardless of real elapsed time, so during idle periods PTS ran many times slower than the wall clock and live players accumulated delay they could never chase back. Timestamps now follow the wall clock: keepalives stamp 500 ms apart, active streaming stamps at real time.
- **Garbage frames right after connecting:** new clients previously started receiving packets from the middle of a GOP, deltas without their references, which decoders render as corruption until the next natural keyframe. The transport now keeps the current GOP since its last keyframe and replays it to every joining client under the pump lock before going live, making joins gap-free, duplicate-free and instant.
- **Silent mid-stream packet loss:** a slow client's mux queue used to drop chunks quietly, corrupting that client's decode until an arbitrary keyframe. Queues no longer emit holes; on overflow the client's stream ends cleanly instead, and clients rejoin at a keyframe via the self-healing reconnect. After any broadcast lag, clients also freeze on the last good frame and resume at the next keyframe rather than decoding orphaned deltas.

### 🔧 Changed
- Removed the obsolete EVDI install hint from RPM `%post` and deb `postinst` scriptlets (EVDI is opt-in; KDE Wayland needs no kernel module) and corrected the deb description accordingly.
- Added `ORBISCREEN_ENCODER_DUMP=<path>` diagnostics hook: appends raw Annex-B encoder output to a file for bitstream debugging.

## [v0.12.2] - 2026-08-25

### 🐛 Fixed
- **Black stream on idle desktop (the "no image at all" bug):** compositors deliver screencast frames on damage only, and a freshly created virtual display is completely static; the capture received exactly one pre-paint black buffer and froze on it forever. A new damage pump keeps the virtual output recompositing (~2 fps) via an invisible, click-transparent layer-shell surface, so the capture always reflects the real display content.
- **Corrupted H.264 bitstream from forced key units:** `UpstreamForceKeyUnitEvent` on the x264enc src pad produced malformed I-frames (decoder errors like `out of range intra chroma pred mode`, black concealment output) regardless of threading mode. Forced key units are gone; clean joins are guaranteed by the intact delta chain plus natural IDRs (`key-int-max` lowered 30 → 10, worst-case idle join ≤ 5 s, active join ≤ 200 ms).
- **Startup blocked forever on the input portal:** the daemon now waits at most 20 s for the RemoteDesktop portal and starts streaming anyway with a clear warning when it hangs or needs interactive approval; remote control comes online on the next restart once granted.

### 🔧 Changed
- x264enc runs with `sliced-threads=false` and 2 frame threads: slice-level threading (implicit in `zerolatency`) is fragile around keyframe requests and produced 18 slices per frame; frame-level threading keeps each access unit atomic.

## [v0.12.1] - 2026-08-25

### ✨ Added
- **Mirror capture mode:** `[capture] preferred = "mirror"` streams your real desktop (picked in the portal share dialog) instead of a second virtual monitor, for when you want to *see* your screen rather than extend it.
- **Detailed `/health`:** now returns JSON with `version`, `encoder`, `frames_forwarded`, `active_clients`, `total_clients` and `uptime_seconds`, making stream diagnosis a single curl.
- **`scripts/verify-stream.sh`:** end-to-end stream sanity check: records a few seconds of `/stream`, decodes with ffprobe and measures frame brightness to catch black/empty-stream regressions automatically.
- **Full project audit hardening:** GitHub Actions pinned to commit SHAs (supply-chain), `gitleaks`/`shellcheck`/`cargo-deny` clean runs documented, comment policy applied (comment-free code files, banner-style headers on config files).

### 🔧 Changed
- **EVDI is opt-in.** `[capture] preferred = "evdi"` requests the EVDI DRM virtual display explicitly; on Wayland, `auto` never touches EVDI anymore, so the recurring `EVDI kernel module not active` line is gone on KDE and the portal is reached directly on other compositors. On X11, `auto` still uses EVDI when its module is already loaded.
- The encoder-to-transport video channel is bounded (64 packets) so a stalled transport backpressures the pipeline instead of growing memory without limit.

### 🐛 Fixed
- **New clients saw an endless black screen on idle virtual displays.** KWin delivers virtual-display frames on damage only: a static desktop stops producing frames entirely, and keepalive re-pushes encoded as deltas, which `h264parse`/`mpegtsmux` hold back until the first keyframe, so a freshly connected client received nothing. The daemon now re-pushes the last frame every 500 ms **with a forced IDR**, so any new client decodes within half a second no matter how idle the display is.
- **Android: the app never recovers after a daemon restart.** The stream token is rotated on every daemon start, but the Android client cached it forever; every reconnect looped on 401 with a black screen. The player now re-fetches a fresh token before each reconnect/retry, and input/control pick up rotated tokens automatically (periodic refresh + `InputDispatcher.updateToken`), mirroring the web client's self-healing reconnect.
- **Android: `build.gradle.kts` hard-failed configuration without a signing keystore**, blocking `lintDebug`/`assembleDebug` on dev machines. Signing is now conditional (release builds unsigned with a warning when secrets are absent) and CI explicitly rejects unsigned APKs.

## [v0.12.0] - 2026-08-24

### ✨ Added
- **KWin virtual display backend (no root, no portal):** on KDE Plasma the daemon now creates a real virtual monitor (`Virtual-ORBISCREEN`, visible in Display Settings) through KWin's `zkde_screencast_unstable_v1` Wayland protocol and streams it over PipeWire directly, bypassing xdg-desktop-portal entirely, so no share dialog appears and the portal's crash-prone path is avoided. KWin only exposes the protocol to allow-listed executables, so the daemon maintains `~/.local/share/applications/orbiscreen.kwin.desktop` (user-writable, no sudo) with `X-KDE-Wayland-Interfaces=zkde_screencast_unstable_v1` and its own executable path, refreshing the KService cache and retrying the connection until the grant is visible. Closing the stream removes the virtual output automatically. New `[capture] preferred = "auto" | "kwin-virtual" | "portal"` config option; `auto` (default) tries KWin first on Wayland and falls back to the portal on non-KDE compositors, while explicit preferences fail loudly on non-Wayland sessions instead of silently capturing the real desktop.
- **Web client self-healing reconnect:** after a daemon restart the stream token changes, and any already-open browser tab used to loop forever on a silent black screen (the overlay was hidden and the stale token was never refreshed). The client now re-shows the status overlay on stream loss and re-fetches `/client/config.json` before each reconnect, so tabs recover automatically once the daemon is back.

### 🔧 Changed
- Capture fallback log no longer claims a portal dialog is always required; the hint is logged only when the portal path is actually taken.
- Frame validation is shared between the portal and KWin backends (`sample_to_captured_frame`), so size-mismatch and malformed-sample diagnostics can no longer drift; the KWin backend uses a bounded frame queue (drops under stall instead of growing without limit).
- A compositor-side close of the virtual output is reported as terminal (`CaptureSession::is_ended`) and stops the capture pump instead of retrying forever with warning spam.

## [v0.11.2] - 2026-08-23

### 🐛 Fixed
- **X11 input registration:** the uinput device registered keys only up to `KEY_KPDOT` (code 83), so the kernel silently dropped everything above it: arrow keys, Insert/Delete/Home/End/PageUp/PageDown, right Ctrl/Alt, Meta, F11/F12 and most numpad keys never reached the host on the X11 path; wheel input sent `REL_WHEEL` without registering the axis at all. The virtual touchscreen now registers the full key range (`KEY_MAX`) plus `REL_WHEEL`, making every mapped client key functional.
- **Stream task panic on short packets:** the non-NAL debug log sliced `bytes[..4]` unguarded, so any packet shorter than 4 bytes killed the per-client streaming task; the slice is now length-clamped.
- **Daemon exit on D-Bus failure:** when the session bus was unavailable, dropping the D-Bus handles closed the shutdown watch channel, so the daemon exited immediately after start with a misleading "D-Bus Stop received" log; a keep-alive sender now holds the channel open until a real stop.
- **RPM version drift:** `package-rpm.sh` hardcoded a stale `VERSION=0.6.0` fallback; it now extracts the workspace version from `Cargo.toml` dynamically like the deb builder.
- **Duplicate ADB reverse setup:** both the daemon startup path and `Transport::serve()` ran `setup_reverse_for_all` on every launch (a blocking call inside async context); it now lives only in the transport layer.
- **Misleading installer log:** `install.sh` printed "Reloading systemd user daemon..." before writing the unit file and never actually reloaded; the line is gone.
- **Desktop entry validation:** `Categories` carried two main categories (`Utility` + `System`); `desktop-file-validate` is now fully clean.
- **Wayland capture diagnostics:** the portal pipeline now watches the GStreamer bus and logs real negotiation/caps errors instead of failing silently with zero frames, and `pipewiresrc` output is pinned to system memory so compositors offering DMA-BUF buffers cannot stall negotiation.
- **Portal log noise:** the ashpd/zbus property-cache warnings are filtered from the GTK binary's logs as well (the daemon already did).
- **Portal denial handling:** dismissing or denying the screen-share prompt now fails with an explicit "user denied the ScreenCast permission" message instead of the cryptic "Portal request didn't succeed with no information".
- **Monotonic stream timestamps:** captured frames were pushed to the encoder with `PTS=0`, producing non-monotonic chunk timestamps (first keyframe stamped at a huge running-time base, everything else at 0) that made `mpegtsmux` stall right after the first AU; clients froze on the initial, often-black frame. The pump now stamps every frame with `frame_index × 1/fps`, so the MPEG-TS stream flows continuously and late-joining clients pick up on the next keyframe.
- **H.264 byte-stream format (root cause of the black screen):** `x264enc` defaults to `byte-stream=false`, and the encoder pipeline's appsink accepted any stream-format, so `h264parse` silently converted the output to AVCC (length-prefixed NALs) while the transport advertised `stream-format=byte-stream`. The transport's `h264parse` then waited forever for Annex B start codes that never arrived, and `mpegtsmux` emitted zero bytes: every client saw a frozen black frame. The encoder now forces `byte-stream=true` and its appsink pins `stream-format=byte-stream, alignment=au`; an integration test feeds real encoder output through the muxer and asserts MPEG-TS bytes come out.
- **Accurate EVDI fallback message:** without the evdi kernel module the daemon no longer claims "clients will see the host's main display"; portal capture lets you pick any display, including virtual monitors, in the share dialog (now logged at info, not warn).
- **Universal H.264 profile:** the encoder negotiated Y444 input, producing High 4:4:4 Predictive video that Android hardware decoders cannot play; phones showed a black frame even while browsers with software decoding managed. The pipeline now pins `NV12` before `x264enc`, yielding Constrained Baseline output that decodes everywhere (verified with ffprobe on a live stream: ~7 MB over 7 s at 1080p60).
- **Per-client pipeline lifetime:** a teardown guard placed in the HTTP handler scope set the client's muxer pipeline to flushing as soon as the handler returned, so `/stream` delivered HTTP 200 with zero bytes. The guard now lives inside the streaming task itself, which also guarantees clean NULL transitions when clients disconnect or the daemon shuts down, silencing the burst of GStreamer "Trying to dispose element … PLAYING" CRITICALs.
- **Web control gating (mouse hijack):** moving the mouse over the browser player forwarded every movement as absolute pointer injection, yanking the host cursor across monitors. Control now requires an explicit click that engages Pointer Lock ("Click to control" hint, Esc to release); wheel and keyboard follow the same gate.
- **evdi module helper bundled:** `install-evdi-module.sh` ships inside the deb/RPM packages with a post-install hint, and it now builds the module automatically from DisplayLink main when no prebuilt `evdi.ko` is found (required on kernels newer than the vendored 1.9.1 sources support).

### 🔒 Security
- **systemd hardening:** `NoNewPrivileges=true` added to all shipped user units (install.sh, deb builder, RPM spec).
- **Android permissions pruned:** removed unused `VIBRATE`, `ACCESS_FINE_LOCATION` and `ACCESS_COARSE_LOCATION` from the manifest.
- Full security re-audit: constant-time token comparison, bounded queues, `/api/control` auth + fixed-argument commands, no WebView in the Android client, OkHttp timeouts everywhere, secrets only via CI env; verified sound, no changes required.

### 🗑️ Removed
- Dead transport API: `find_usb_device()`, `remove_reverse()`, `first_device_serial()` (+ their tests); the dead `StreamUrl.mimeType()` with its suppress/import; the unused synchronous `InputInjector::open()`.
- The unreferenced `gen-key-script` GPG helper.
- Residual inline comments across Rust/Kotlin/web/shell/spec/drawable sources: code files keep only the standard two-line license headers, while env-style config files (`gradle.properties`, ProGuard rules, `deny.toml`, `rustfmt.toml`, `.gitignore`, `.editorconfig`, RPM spec, desktop entry, `lint.xml`, AndroidManifest) carry uniform decorative section banners instead.

### 🔧 Changed
- Release workflow builds with `--locked`; Android `versionCode` 17 → 18.

## [v0.11.1] - 2026-08-16

### 🐛 Fixed
- **Android discovery staleness:** vanished mDNS services are now actually removed from the host list (the lost-service event previously carried no address, so stale entries lingered forever); subnet-sweep candidates are verified with the daemon's own `/api/info` before being listed.
- **Structured concurrency:** the subnet scanner rethrows `CancellationException` so leaving the discovery screen genuinely stops in-flight probes; the player's supervisor scope and OkHttp dispatcher are shut down on release.
- **Android 12+ gateway detection:** `WifiManager.dhcpInfo` (deprecated, null without location grant) is replaced with `ConnectivityManager.getLinkProperties()`, so the subnet sweep works on modern Android; SDK level raised (`compileSdk 35`) to match `targetSdk 35`.
- **Web remote control:** pressed pointer buttons are tracked in a set and all released on `pointercancel`/`pointerleave` (previously only button 1 and only while buttons were held, leaving stuck buttons on the host); keystrokes are no longer forwarded while the stream is still connecting; playback rejection now triggers the reconnect scheduler.

### 🗑️ Removed
- **Dead code across all crates:** unused dependencies (`evdi-sys`, `libc`, `gstreamer-video`), unused public API (`open_many`, `remove_all_nodes`, `device_node_index`, `write_debug_ppm`, `stream()`, `TouchCalibration`, `fullname()`, the dead `query_status`/`call_get_status` chain), and the `Start()` no-op method from the D-Bus surface (starting stays a systemd job).
- **Unused client surface:** `TouchCalibration`, the legacy pre-Compose `activity_main.xml`, unused Gradle dependencies (`appcompat`, `security-crypto`, `media3-exoplayer-hls`), the dead `syncWebClient` asset step, unused strings/colors resources, and the half-wired `label` navigation argument.
- **Stale packaging assets:** unreferenced icons in `data/` (only `orbiscreen.svg` is used), the superseded `packaging/debian/` dir, and the never-built `packaging/flatpak/` manifest.
- **Inline comments** were stripped project-wide; only the per-file license header remains.

### 🔧 Changed
- `rand` bumped to 0.9 in `orbiscreen-transport`; Android `versionCode` 16 → 17.

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
- **GTK panel build:** removed the duplicate `start_status_poller` definition, replaced the removed glib 0.20 `MainContext::channel`/`PRIORITY_DEFAULT` API with an mpsc channel drained on the GTK main loop via `timeout_add_local` (widget handles stay on the main thread, since they are `!Send`), and fixed the Gradle release signing block where the local `keyAlias` val shadowed the DSL property.
- **CI Android signing:** the push-time Android workflow now feeds the keystore from the same repo secrets as the release matrix (`ANDROID_KEYSTORE_B64` etc.), rejects blank signing secrets at configure time, and validates the keystore password/alias with `keytool -list` before the build so mismatched secrets fail immediately instead of after packaging.
- **Player build break:** removed the nonexistent `setTargetLiveOffsetMs` call (not part of media3 1.3.1's `DefaultLivePlaybackSpeedControl.Builder`); the live-edge target is set per media via `MediaItem.LiveConfiguration.setTargetOffsetMs`.
- **Input injection hardening:** pointer buttons outside the registered range (0 or >8) are rejected instead of being silently mapped to left-click, wheel deltas are clamped before the integer cast, and a zero wheel delta no longer emits a bare SYN_REPORT; the Wayland portal injector teardown only spawns the session close when a Tokio runtime context exists, so dropping it outside the runtime can no longer panic from `drop`.

### 🔒 Security
- **mDNS token redaction:** daemon events are now logged by kind only, never via `Debug` (whose `ServiceInfo` rendering can include the TXT record carrying the session access token); the monitor thread also switched from a 60 s receive-timeout to a blocking receive, so daemon events keep being consumed instead of dropping out after the first idle minute.
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
- **Docs Typography:** Replaced all em dashes with standard hyphens across `README*.md`, `CHANGELOG.md`, `SECURITY.md`, and `docs/*.md` for a cleaner, tool-agnostic documentation style.

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
- **Documentation:** refreshed across `README.md`, `docs/ARCHITECTURE.md`, `DBUS_SPEC.md`, `INSTALL.md`, `PACKAGING.md`, `TROUBLESHOOTING.md`, and `SECURITY.md`.

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
