<div align="center">

# D-Bus API Specification - Orbiscreen

[![Version](https://img.shields.io/badge/version-0.22.7-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 Language

<a href="DBUS_SPEC.md">🇬🇧 English</a> · <a href="DBUS_SPEC_AR.md">🇸🇦 العربية</a>

---

Orbiscreen exposes a D-Bus Session Service interface allowing desktop control panels, CLI scripts, and system tray indicators to inspect live status and control the daemon process. The implementation is located in `crates/orbiscreen-daemon/src/dbus.rs`.

- **Bus Type:** Session Bus (user bus)
- **Service Name:** `org.shadow-x78.Orbiscreen` (alias: `com.orbiscreen.Daemon`)
- **Object Path:** `/com/orbiscreen/Daemon`
- **Interface Name:** `com.orbiscreen.Daemon`

The service is registered by the running daemon process. If the service name is absent (`ServiceUnknown`), the daemon is not running.

---

## 🛰 Companion HTTP Control Surface

Android and web clients talk to the daemon over HTTP, not D-Bus. Current routing table (see `orbiscreen-transport`):

| Endpoint | Auth | Purpose |
|----------|------|---------|
| `GET /health` | public | Liveness probe |
| `GET /api/info` | public | Display dimensions, encoder, version |
| `GET /stream?token=…` | token | MPEG-TS (H.264) video stream |
| `POST /input` | token | Pointer / key / stylus input events |
| `POST /api/control` | token | `lock`, `blank`, `unblank`, `ctrl_alt_del` host actions |
| `GET /client/config.json` | public | Bootstrap for the web client: `{token, display_width, display_height}` |
| `GET /` | public | Redirect to the bundled web client |
| `GET /client/*` | public | Bundled web client static files (MSE player via vendored mpegts.js) |

**Token model:** every daemon start generates a fresh random token (32 bytes, base64url). Protected routes require it via `Authorization: Bearer <token>` or `?token=<token>`. Clients obtain it from the mDNS TXT record or from `/client/config.json`. Because the token is readable by anyone who can reach the port, this is abuse protection, not strong authentication - see `SECURITY.md`.

D-Bus remains the canonical interface for native Linux clients (CLI scripts, `orbiscreen stop`). Both surfaces share the same source of truth in `orbiscreen-transport` (`Stats`).

---

## 🛠 D-Bus Methods

All methods are exposed on the session bus. zbus maps the Rust names to PascalCase on the wire.

### 1. `GetStatus() -> String` (signature `s`)

Returns the live daemon status as a **JSON object string**:

```json
{
  "running": true,
  "frames_forwarded": 184320,
  "active_clients": 2,
  "total_clients": 5,
  "auth_failures": 0,
  "usb_devices": 1,
  "encoder": "x264",
  "capture_backend": "evdi"
}
```

| Field | Type | Meaning |
|-------|------|---------|
| `running` | bool | Pipeline running flag (`false` briefly after `Stop` is requested) |
| `frames_forwarded` | u64 | Frames handed to the transport since start |
| `active_clients` | u64 | Currently connected `/stream` clients |
| `total_clients` | u64 | Total `/stream` connections since start |
| `auth_failures` | u64 | Unauthorized requests rejected since start (also exposed on `GET /health`) |
| `usb_devices` | u64 | Android devices with an active `adb reverse` tunnel right now (updated live while the daemon runs; also exposed on `GET /health`) |
| `encoder` | string | Encoder actually in use (`x264`, `vaapi`, `nvenc`) |
| `capture_backend` | string | `evdi` for the virtual display; `x11-portal-fallback` / `wayland-portal-fallback` when the evdi module is missing |

### 2. `Stop() -> String` (signature `s`)

Requests a graceful daemon shutdown. The handler flips the running flag and signals the main loop through an internal watch channel; the daemon then tears down capture/encode/transport and exits.

- **Return:** `"Orbiscreen daemon shutting down"`
- **If already stopped:** `"Orbiscreen is not running"`

`orbiscreen stop` is a thin client of exactly this method: it calls `Stop()` over the session bus, prints the reply, and exits 1 with a `systemctl --user stop orbiscreen` hint when the service name is not found.

The daemon cannot be started over D-Bus: there is no interface activation, and no `Start()` method - manage the service unit instead (`systemctl --user start orbiscreen`).

### 3. `ListClients() -> Array of String` (signature `as`)

Live client connection counts for the stream transport:

```json
["HTTP MPEG-TS /stream: 2 active client(s), 5 total connection(s)"]
```

### 4. `GetConfig() -> String` (signature `s`)

Returns the sanitized configuration the daemon was started with, serialized as **TOML** (not JSON) by `orbiscreen-core::dump_config`:

```toml
[display]
width = 1920
height = 1080
refresh_rate_hz = 60

[capture]
preferred = "auto"

[encode]
bitrate_kbps = 8000
preferred_encoder = "x264"

[transport]
signaling_port = 8788
mdns_advertise = true
```

On serialization failure it returns `config serialize error: <detail>`.

### Not implemented

`SetScreenState` was previously proposed but is **not implemented**. Host-side display state (`blank` / `unblank`) is only reachable via the authenticated HTTP `POST /api/control`.

---

## 💻 CLI Usage Example (`busctl`)

```bash
# Introspect the Orbiscreen D-Bus interface
busctl --user introspect com.orbiscreen.Daemon /com/orbiscreen/Daemon

# Get daemon status (JSON string)
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetStatus

# List connected clients (string array)
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon ListClients

# Print the running configuration (TOML string)
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetConfig

# Gracefully stop the daemon
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon Stop
```

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Back to README](../README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
