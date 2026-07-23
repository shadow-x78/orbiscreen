# D-Bus API Specification - Orbiscreen

---

## 🌐 Language

<a href="DBUS_SPEC.md">🇬🇧 English</a> · <a href="DBUS_SPEC_AR.md">🇸🇦 العربية</a>

---

## 🛰 Overview

Orbiscreen exposes a D-Bus Session Service interface allowing desktop control panels (GTK4 GUI), CLI scripts, and system tray indicators to inspect status, configure display settings, and control the daemon process.

- **Bus Type:** Session Bus
- **Service Name:** `com.orbiscreen.Daemon`
- **Object Path:** `/com/orbiscreen/Daemon`
- **Interface Name:** `com.orbiscreen.Daemon`

---

## 🛠 Methods

### 1. `GetStatus() -> String`
Returns the current daemon execution state.
- **Return Value:** `"Running"` or `"Stopped"`

### 2. `Start() -> String`
Starts the Orbiscreen display capture, encoding, and transport engine.
- **Return Value:** `"Orbiscreen daemon started via D-Bus"`

### 3. `Stop() -> String`
Stops display capture and disconnects active streams.
- **Return Value:** `"Orbiscreen daemon stopped via D-Bus"`

### 4. `ListClients() -> Vec<String>`
Returns a list of currently connected web and Android clients.
- **Return Value:** `["HTTP Direct /stream", "WebRTC Signaling Active"]`

### 5. `GetConfig() -> String`
Returns the active configuration formatted as a JSON string.
- **Return Value:** `{"width":1920,"height":1080,"refresh_rate":60,"encoder":"auto"}`

---

## 💻 CLI Usage Example (`busctl`)

```bash
# Introspect the Orbiscreen D-Bus interface
busctl --user introspect com.orbiscreen.Daemon /com/orbiscreen/Daemon

# Get daemon status
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetStatus

# List connected clients
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon ListClients
```
