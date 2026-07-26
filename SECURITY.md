# Security Policy - Orbiscreen



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
| 0.4.x | ✅ Active development |
| < 0.4 | ❌ Not released |

Only the latest development release receives security updates. Ensure you build from `main` before reporting.

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

### Scope

Orbiscreen is a Linux host daemon that:
- Creates kernel-level virtual displays via the `evdi` DRM module
- Captures screen contents via X11 (`x11rb`) or Wayland (`ashpd` + PipeWire)
- Injects input events via `evdevil` (uinput) or `ashpd` RemoteDesktop
- Streams encoded H.264 over a local WebRTC peer connection

### Known Risk Areas

| Area | Risk | Mitigation |
|------|------|------------|
| uinput injection | Any process holding the virtual touchscreen can inject arbitrary input | The daemon opens the uinput device exclusively; restrict `/dev/uinput` permissions on the host |
| Screen capture | Frames contain everything rendered to the virtual display | v1 binds to the evdi-backed virtual display only, not the primary desktop |
| WebRTC signaling | Signaling server binds `0.0.0.0` by default | Local network only in v1; no cloud relay; no TURN server |
| evdi kernel module | DKMS + Secure Boot signing is distro-specific | Module loading is the host administrator's responsibility |
| mDNS advertising | Hostname + port broadcast on the local network | No credentials are advertised; client must still complete the WebRTC handshake |

### Recommendations

1. **Run the daemon as a non-root user** with explicit `/dev/uinput` + `/dev/dri/card*` permissions via `udev` rules.

2. **Do not expose the signaling port** (`8788` by default) to untrusted networks. Bind to `127.0.0.1` and use `adb reverse` for USB transport on untrusted networks.

3. **Build from source** from the official repository:
   ```bash
   git clone https://github.com/shadow-x78/orbiscreen.git
   ```

4. **Review the `evdi` kernel module** provenance before loading it; Secure Boot hosts must sign it.

5. **Never log raw input events** in production - `tracing` is set to `INFO` by default and does not dump pointer coordinates.

---

<a id="audit"></a>
## 🔬 Security Audit

Orbiscreen is written entirely in Rust (edition 2021) and a thin Kotlin Android WebView host. A running daemon performs:

- `open()` on `/dev/dri/card*` evdi nodes
- `UinputDevice` construction via `evdevil`
- `GetImage` (X11) or `Screencast` portal (Wayland) capture
- GStreamer pipeline construction for H.264 encoding
- `axum` HTTP/WS listener on the configured signaling port
- `adb reverse` subprocess invocation when a USB device is attached

All logic is readable in plain Rust. If you perform an audit, please share findings via the private reporting channels above.

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
