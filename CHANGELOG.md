# Changelog

## [v0.9.0] - 2026-07-26

### 🚀 Added
- **Security:** Universal artifact signing implemented.
- **Android:** The release APK is now cryptographically signed with a production Keystore to prevent "Untrusted Developer" warnings.
- **Linux:** RPM, DEB, and AppImage packages are now cryptographically signed with GPG keys (`orbiscreen.asc`).

---

## [v0.8.7] - 2026-07-26

### 🚀 Added
- **Architecture Migration:** Completely migrated the internal streaming architecture from WebRTC/WebView to a native GStreamer pipeline.
- **Android Native Player:** Replaced the heavy, stuttering WebView with a fully native hardware-accelerated ExoPlayer implementation.
- **Zero-Config Discovery:** Implemented mDNS service discovery; the Android app now automatically discovers and connects to the Orbiscreen daemon without manual IP entry.
- **Touch Input Injection:** Touch events on the Android screen are now streamed back and injected directly into the Linux Wayland compositor using `uinput` and `evdev`.

### 🐛 Fixed
- **Latency & Performance:** Solved severe latency and dropped frames issues on older Android devices.
- **Wayland Compatibility:** Removed the failing EVDI dependency. The capture backend now robustly utilizes the XDG Desktop Portal for Wayland/X11 screen capture.
- **CI/CD Reliability:** Fixed various `cargo fmt` errors, `clippy` warnings, and `Cargo.lock` drift issues in the GitHub Actions workflow.

---

## [v0.7.4] - 2026-07-26

### 🚀 Added
- **Uninstall Command:** Added `orbiscreen uninstall` built-in CLI command to cleanly remove the daemon, systemd services, desktop entries, and icons.

### 🐛 Fixed
- **Web Client Packaging:** Fixed a major bug where `index.html`, `style.css`, and `app.js` were missing from RPM, DEB, and AppImage packages, causing "webpage not available" (404) errors on Android clients.
- **Daemon Path Resolution:** Enhanced `orbiscreen-daemon` to intelligently locate web client files across multiple system paths (`~/.local/share/orbiscreen`, `/usr/share/orbiscreen`, `/app/share/orbiscreen`) depending on the installation method.

---

## [v0.7.3] - 2026-07-26

### 🚀 Added
- **Release Automation:** Updated `release.yml` with dynamic packaging versions and sha256 checksums generation.
- **Packaging Fixes:** Fixed flatpak ID and Debian version injection.

### 📝 Documentation
- **Complete Reformatting:** Cleaned up and updated all markdown documentation including README, ARCHITECTURE, DBUS_SPEC, PACKAGING, and TROUBLESHOOTING.

---

## [v0.7.2] - 2026-07-26

### 🐛 Fixed
- **Daemon Infinite Loop:** Added `tokio::time::sleep` rate-limiting to the capture frame pump loop to eliminate 100% CPU usage.
- **Uninstall System:** Added `scripts/uninstall.sh` and updated `scripts/install.sh` to support complete uninstallation of daemon and desktop entries.

## [v0.7.1] - 2026-07-26

### 🐛 Fixed
- **Android Immediate Crash:** Wrapped `MainActivity` initialization in `try/catch` and removed the stray `-->` from `index.html` that crashed the Android WebView parser.
- **Android Target SDK:** Bumped to API 35 and explicitly enabled V1/V2/V3 signing.
- **Android Network Security Config:** Allowed cleartext traffic for local networks only via `network_security_config.xml`.

