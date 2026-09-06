<div align="center">

# Security Policy - Orbiscreen

[![Version](https://img.shields.io/badge/version-0.23.4-2563eb?style=flat-square&logo=semver)](CHANGELOG.md)
[![Version](https://img.shields.io/badge/version-0.23.5-2563eb?style=flat-square&logo=semver)](CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## 📋 Table of Contents

- [Supported Versions](#supported-versions)
- [Reporting a Vulnerability](#reporting)
- [Disclosure Policy](#disclosure)
- [Security Considerations](#considerations)
- [Security Audit](#audit)
- [Hall of Fame](#hall-of-fame)

---

<a id="supported-versions"></a>
## 🛡️ Supported Versions

| Version | Supported |
|---------|-----------|
| 0.20.x | ✅ Active development |
| 0.19.x | ⚠️ Maintenance |
| 0.18.x | ⚠️ Maintenance |
| < 0.18  | ❌ Not supported |

Only the latest minor release receives security updates. Ensure you build from `main` before reporting.

---

<a id="reporting"></a>
## 🚨 Reporting a Vulnerability

If you discover a security vulnerability in Orbiscreen, please report it **responsibly** and **privately**.

**Preferred method:**
- Open a private security advisory on GitHub:
  [Security Advisories →](https://github.com/shadow-x78/orbiscreen/security/advisories/new)

**Alternative method:**
- Email the maintainers via the GitHub security contact above.

**What to include:**

| Field | Details |
|-------|---------|
| Description | Clear explanation of the vulnerability |
| Reproduction | Steps to reproduce - minimal PoC if possible |
| Component | Affected crate / module and version |
| Impact | Privilege escalation, input injection, data exposure, etc. |
| Fix | Suggested mitigation (optional) |

**Response timeline:**

| Phase | Timeframe |
|-------|-----------|
| Initial acknowledgment | Within 72 hours |
| Impact assessment | Within 7 days |
| Patch development | Within 30 days (critical) |
| Public disclosure | Coordinated after fix is released |

---

<a id="disclosure"></a>
## 📢 Disclosure Policy

We follow a **coordinated disclosure** model:

1. Report received and acknowledged
2. Vulnerability validated and severity assessed
3. Fix developed and tested
4. Patch released to all supported versions
5. Public disclosure with credit to reporter (if desired)

> **No premature disclosure.** Do not open public issues or pull requests for security bugs until the fix is released.

---

<a id="considerations"></a>
## 🔍 Security Considerations

### Scope (v0.23.4)
### Scope (v0.23.5)

Orbiscreen is a Linux host daemon plus a Material 3 Android client and a browser web client that:
- Creates compositor-native virtual displays without root: KWin's `zkde_screencast_unstable_v1` on Plasma, headless outputs via sway/Hyprland IPC on wlroots, falling back to the `evdi` kernel module or primary-desktop capture (Wayland portal or X11) when unavailable
- Captures screen contents via the evdi framebuffer, `zwlr_screencopy_manager_v1` (wlroots), Wayland portal (`ashpd` + PipeWire), KWin virtual streams, or X11 (MIT-SHM `x11rb`)
- Injects input events via wlroots-native protocols (`virtual-keyboard` / `wlr-virtual-pointer`), X11 XTEST, `ashpd` RemoteDesktop, or `evdevil` (uinput)
- Streams MPEG-TS/H.264 over HTTP (`/stream`) to Android and browser clients - WebRTC is not used
- Exposes a token-authenticated control-plane HTTP API at `/api/control` (lock, blank, unblank, ctrl-alt-del)
- Exposes public `/api/info` (display resolution, encoder, version) and `/health`
- Persists portal restore tokens (GNOME dialog-free re-grant) in `$XDG_STATE_HOME/orbiscreen/portal.json`
- Provides `orbiscreen doctor` for diagnostics and `doctor --fix` to install/load the EVDI module via the distro package manager

### Token Model and Its Limits

Since v0.11.0, `/stream`, `/input` and `/api/control` require a per-session access token (32 random bytes, base64url) presented as `Authorization: Bearer <token>` or `?token=<token>`.

Clients obtain the token in two ways:

1. **mDNS TXT record** of the advertised `_orbiscreen._tcp.` service (`token=...`)
2. **`GET /client/config.json`:** intentionally unauthenticated, so the bundled web client can bootstrap itself

**Threat model:** anyone who can reach the HTTP port can read `/client/config.json` and therefore learn the token. The token is therefore **abuse protection against casual/unintended use** (scanners, wrong-device connections, neighbors probing the port), **not** protection against a determined attacker on your LAN. It stops nothing from an attacker who already has network access to the port, and it is transmitted in cleartext.

- **TLS is planned** for a future release; until then the session token rides over plain HTTP. The USB transport (`adb reverse`) keeps the stream entirely inside the USB cable - no LAN exposure at all on that path - and the token is still required.
- The Android client's `network_security_config.xml` therefore permits cleartext HTTP globally. This is deliberate: the app only ever connects to LAN hosts the user selects (mDNS discovery or manual entry), and Android's per-domain cleartext exceptions cannot express arbitrary LAN IP addresses. All requests still require the per-session token.
- The token is regenerated on every daemon start, so restarting the daemon invalidates all previously issued tokens.
- `/health`, `/api/info`, `/client/config.json`, `/client/*` stay public by design (liveness, metadata, web-client bootstrap).

Run `orbiscreen start --no-mdns` to stop advertising the host (and the token TXT record) if discovery is not needed.

### Known Risk Areas

| Area | Risk | Mitigation |
|------|------|------------|
| `uinput` injection | Any process holding the virtual touchscreen can inject arbitrary input | The daemon opens the uinput device exclusively; restrict `/dev/uinput` permissions on the host |
| Screen capture | Frames contain everything rendered to the captured display | With evdi/KWin/wlroots virtual outputs capture targets a dedicated output; portal/X11 fallbacks capture the *primary* desktop (see `GetStatus.capture_backend`) |
| Cleartext HTTP `/stream` | A LAN attacker who knows the token can view the desktop stream | Binds on `0.0.0.0` by default so Android/web clients can connect on the LAN; access requires the per-session token; use `adb reverse` and firewall the port on untrusted networks |
| `/api/control` | A client holding the token can call lock/blank/ctrl-alt-del | Token-authenticated since v0.11.0; host tools (`loginctl`, `xset`, …) are invoked as the daemon user |
| evdi kernel module | DKMS + Secure Boot signing is distro-specific | Module loading is the host administrator's responsibility |
| mDNS advertising (`_orbiscreen._tcp.`) | Host name, port and session token are broadcast on the local network | Start with `--no-mdns` to disable advertising |
| Android / web input model | `InputDispatcher` / web client post absolute pointer / wheel / stylus / keyboard events to `/input` | `/input` requires the session token (v0.11.0); wheel steps are clamped per event on the host |
| Token readable at `/client/config.json` | Any peer reaching the port can learn the token | LAN convenience, documented above - abuse protection only; restrict the port or use TLS when available |
| Compositor IPC (sway/Hyprland) | The daemon trusts `$SWAYSOCK` / the Hyprland instance socket from the session environment to create/destroy headless outputs | Sockets are local and owned by the session user; output names returned by the compositor are charset-validated before use in IPC commands; a compromised compositor already owns the session |
| Portal restore tokens | `$XDG_STATE_HOME/orbiscreen/portal.json` holds ScreenCast/RemoteDesktop grants; theft allows silent screen sharing as long as the grant lives | Written atomically with `0600` file and `0700` state directory (v0.13.0); delete the file to revoke; GNOME revokes grants marked `ExplicitlyRevoked` when the token is removed |
| `orbiscreen doctor --fix` | Runs the detected package manager (`dnf`/`apt`/`pacman`/`zypper`) and `sudo modprobe evdi` | Install commands are hardcoded per-distro (never built from file contents), the plan is printed and requires explicit confirmation unless `--yes` is given, and it only runs when the user invokes it |

### Keystore Rotation (Android)

The Android release signing key (`orbiscreen-release.keystore`) was removed from the repository in v0.11.0; it remains in git history for anyone who cloned before the removal. Consequences:

- Anyone who pulled the old keystore still holds the private key and could sign APKs that Android treats as updates to existing installations.
- Builds going forward use a new key, so APKs signed with the old key have a **mismatched signature** and cannot be upgraded in place - uninstall the old app before installing APKs signed with the new key.
- Treat any pre-removal clone of this repository as leaked key material; do not grant it trust.

### Recommendations

1. **Run the daemon as a non-root user** with explicit `/dev/uinput` + `/dev/dri/card*` permissions via `udev` rules.

2. **Do not expose the signaling port** (`8788` by default) to untrusted networks. The daemon binds to `0.0.0.0` so LAN clients can connect; use `adb reverse` plus a firewall on the port (or run the daemon inside a network namespace that only exposes `127.0.0.1`) when you do not want LAN exposure.

3. **Build from source** from the official repository:
   ```bash
   git clone https://github.com/shadow-x78/orbiscreen.git
   ```

4. **Review the `evdi` kernel module** provenance before loading it; Secure Boot hosts must sign it.

5. **Never log raw input events** in production - `tracing` is set to `INFO` by default and does not dump pointer coordinates. The Android `InputDispatcher` similarly does not log coordinates in release builds.

6. **Restart the daemon to rotate the session token.** Every restart invalidates the previous token, forcing all clients to re-authenticate with the new one.

7. **Verify package signatures** before installing. See `docs/PACKAGING.md` for the GPG / keystore fingerprints.

---

<a id="audit"></a>
## 🔬 Security Audit

Orbiscreen (v0.23.4) is written in Rust (edition 2021) plus a Kotlin Android client (Material 3 + Jetpack Compose) and a small browser web client (MSE via vendored mpegts.js). A running daemon performs:
Orbiscreen (v0.23.5) is written in Rust (edition 2021) plus a Kotlin Android client (Material 3 + Jetpack Compose) and a small browser web client (MSE via vendored mpegts.js). A running daemon performs:
 
- `open()` on `/dev/dri/card*` evdi nodes for capture
- Compositor IPC over session-local Unix sockets: sway i3-ipc (`$SWAYSOCK`) and Hyprland (`HYPRLAND_INSTANCE_SIGNATURE`) to create/destroy headless outputs
- `zwlr_screencopy_manager_v1` SHM capture on wlroots, `zkde_screencast_unstable_v1` + PipeWire on KWin, Screencast portal (Wayland) or MIT-SHM/`GetImage` (X11) as fallbacks
- Input injection via `zwp_virtual_keyboard_v1` + `zwlr_virtual_pointer_v1` (wlroots), XTEST (X11), RemoteDesktop portal, or `UinputDevice` via `evdevil`
- GStreamer pipeline construction for H.264 encoding
- `axum` HTTP listener serving `/stream`, `/input`, `/api/control` behind a per-session token, plus public `/health`, `/api/info`, `/client/config.json`, and `/client/*`
- A D-Bus session service (`org.shadow-x78.Orbiscreen` / `com.orbiscreen.Daemon`) exposing `GetStatus` / `Stop` / `Start` / `ListClients` / `GetConfig`
- Direct USB cable streaming via Android Open Accessory (AOA) protocol over usbfs ioctl
- On explicit user request (`doctor --fix`): the distro package manager for EVDI installation and `sudo modprobe evdi`
- `NsdManager.discoverServices` on the Android client (no outbound traffic outside the LAN)

All logic is readable in plain Rust and Kotlin. If you perform an audit, please share findings via the private reporting channels above.

---

<a id="hall-of-fame"></a>
## 🏆 Hall of Fame

We thank the following security researchers for responsible disclosure:

*(None yet - be the first!)*

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
<a href="https://github.com/shadow-x78/orbiscreen">orbiscreen</a> ·
[Back to README](README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>

