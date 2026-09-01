<div align="center">

# Architecture Specification - Orbiscreen

[![Version](https://img.shields.io/badge/version-0.13.3-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 Language

<a href="ARCHITECTURE.md">🇬🇧 English</a> · <a href="ARCHITECTURE_AR.md">🇸🇦 العربية</a>

---

Orbiscreen is built as a modular multi-crate Rust workspace separating system display drivers, frame capture engines, hardware-accelerated video encoders, inter-process communication (D-Bus), and multi-protocol network transports.

---

## 🏛 System Architecture Overview

```mermaid
graph TD
    subgraph Host Linux Machine
        A[evdi Kernel Module] -->|Virtual DRM Device| B(Display Server X11/Wayland)
        B -->|evdi framebuffer| C1(orbiscreen-display EvdiFramePump)
        B -.->|portal fallback only| C0(orbiscreen-capture portal/X11)
        C1 -->|Tight BGRA frames| D(orbiscreen-encode)
        C0 -.->|BGRA frames (primary desktop)| D
        D -->|GStreamer HW/SW Encode| E(H.264 AU stream)
        E --> F(orbiscreen-transport)
        F -->|MPEG-TS HTTP /stream| G((Network/USB))
        F -->|mDNS _orbiscreen._tcp.| G
        F -->|GET /api/info| G
        F -->|POST /api/control| G
        F -->|GET /health| G
    end

    subgraph Clients
        G -->|MPEG-TS| W(Web client - mpegts.js MSE)
        W -->|POST /input| F
        G -->|NSD discovery + token| H(Android DiscoveryService)
        H -->|onConnect| J(StreamViewModel)
        J -->|PlayerHolder.build| K(OkHttpDataSource)
        K -->|MPEG-TS + Bearer token| L(ExoPlayer + MediaCodec)
        L -->|Touch| N(InputDispatcher)
        N -->|POST /input + Bearer token| F
        J -->|POST /api/control| F
    end
```

---

## 📦 Workspace Crate Topology

| Crate | Responsibility | Key Dependencies |
|-------|----------------|------------------|
| `orbiscreen-core` | Shared configuration, error types, serialization | `serde`, `toml` |
| `orbiscreen-display` | EVDI DRM virtual display creation, EDID synthesis, framebuffer → tight BGRA conversion, `EvdiFramePump` | `evdi`, `drm-fourcc`, `libc` |
| `orbiscreen-capture` | Wayland Portal (ashpd) & X11 (x11rb) capture engines: **fallback source only** | `ashpd`, `x11rb` |
| `orbiscreen-encode` | Hardware & software H.264 encoding pipelines | `gstreamer`, `gstreamer-app` |
| `orbiscreen-input` | Reverse touch, stylus, and keyboard injection (uinput) | `evdev`, `ashpd` |
| `orbiscreen-transport` | Axum HTTP `/stream` + token auth, mDNS, ADB reverse, `/api/info`, `/api/control`, `/health` | `axum`, `gstreamer`, `tokio`, `rand`, `base64` |
| `orbiscreen-daemon` | Main daemon binary, systemd integration & live D-Bus service | `zbus`, `clap`, `tokio` |
| `orbiscreen-gtk` | Native GTK4 / Libadwaita desktop GUI control panel (reads state via D-Bus) | `gtk4`, `libadwaita`, `zbus` |

---

## 📱 Android Client Package Layout

```
com.orbiscreen.android/
├── MainActivity.kt                # Compose host, observes PrefsStore.themePrefFlow
├── data/
│   └── PrefsStore.kt              # SharedPreferences (recent host, theme, scanner toggle)
├── net/
│   ├── DiscoveryService.kt        # NsdManager wrapper -> StateFlow<Map<DiscoveredHost>>
│   ├── SubnetScanner.kt           # /24 sweep with Semaphore-bounded parallelism
│   ├── HostApi.kt                 # OkHttp client for /client/config.json (token), /api/info, /api/control, /health
│   ├── WifiGatewayProvider.kt     # Reads WifiManager.dhcpInfo.gateway
│   └── DiscoveryModel.kt          # HostSpec regex validator
├── player/
│   ├── PlayerHolder.kt            # ExoPlayer + OkHttpDataSource + DefaultLoadControl + auto-reconnect
│   └── StreamUrl.kt               # Builds http://host:port/stream (mpegts.js-free MPEG-TS URL + token query)
├── input/
│   └── InputDispatcher.kt         # Absolute-coord pointer / wheel / keyboard / stylus; resiz() re-scales mapping
└── ui/
    ├── theme/                     # Material 3 Color.kt, Theme.kt, Type.kt
    ├── nav/OrbiNav.kt             # NavHost (Discovery / Stream / Settings)
    ├── discovery/                 # DiscoveryScreen + DiscoveryViewModel
    ├── stream/                    # StreamScreen, PlayerSurface, ControlToolbar
    └── settings/                  # SettingsScreen (theme, software-decoder toggle, scanner, recent host)
```

---

## 🎞 Stream Pipeline

Each stage owns its data; frames are copied between stages (no zero-copy, this keeps the lifetimes simple at the cost of one extra copy per stage):

1. **Virtual Monitor Provisioning:** `orbiscreen-display` provisions a virtual DRM connector via EVDI and the compositor draws onto it. When the EVDI kernel module is unavailable, the daemon falls back to capturing the **primary desktop** via `xdg-desktop-portal` ScreenCast (Wayland) or X11 `GetImage`, and logs a prominent warning that clients will see the host's main screen instead of the virtual display.
2. **Frame Read & Conversion:** `orbiscreen-display::EvdiFramePump` drives EVDI on a dedicated thread (the underlying handle is `!Send`), waiting on content updates (`request_update` with `UPDATE_BUFFER_TIMEOUT`) and converting the stride-padded XRGB8888/Rgb565 framebuffer to tightly-packed BGRA in `to_tight_bgra()`.
3. **Encoding:**
   - `orbiscreen-encode` takes BGRA frames sized to the **actual** negotiated display mode (not the requested spec) through a live `appsrc → videoconvert → x264enc/vaapih264enc/nvh264enc → h264parse` pipeline. PTS is assigned by GStreamer's live appsrc (`do-timestamp`). `push_frame` rejects mis-sized buffers explicitly.
   - Encoded chunks flow through a bounded channel (drop-oldest under consumer stalls) into `orbiscreen-transport`, which wraps each H.264 AU into a per-client `mpegtsmux` instance served as the HTTP response body.
4. **Playback:**
   - **Web:** Chrome/Firefox/Edge cannot play raw MPEG-TS in a `<video>` element. The bundled client uses the locally-vendored `mpegts.js` (no CDN) to demux via MSE; on any error it tears down and reconnects with exponential backoff.
   - **Android:** `PlayerHolder.build()` builds ExoPlayer with `MimeTypes.VIDEO_MP2T`, live-tuned load control (1.5s min / 5s max buffer), sends the session token as `Authorization: Bearer`, and auto-reconnects on `STATE_ENDED`/errors with backoff capped at 10s. A `forceSoftwareDecoder` setting swaps to software codecs when hardware decode misbehaves.
5. **Reverse Input:**
   - Clients send pointer / wheel / stylus / keyboard events to `POST /input` (token required). Coordinates map from client rect to the **actual** stream resolution. Events are debounced through `MutableSharedFlow` with `BufferOverflow.DROP_OLDEST` to prevent backlog during fast drags.
6. **Host Control:**
   - `HostApi.sendControl` posts JSON to `POST /api/control` (token required): `{"action":"lock"}` (loginctl/xdg-screensaver), `{"action":"blank"}`/`{"action":"unblank"}` (DPMS via swaymsg/hyprctl/xset), `{"action":"ctrl_alt_del"}` (injected through the input pipeline). The legacy `open` action is rejected: opening arbitrary URLs from remote clients is not permitted.
7. **CLI Control:**
   - `orbiscreen stop` calls the D-Bus `Stop` method to shut the running daemon down gracefully; the GTK control panel polls `GetStatus` once per second and shows live encoder/frame/client counters from the same D-Bus service.

---

## 🔐 Authentication

Every session generates a random 32-byte base64url token at startup:

- Announced in the mDNS TXT record (`token=…`) for desktop discovery, and served unauthenticated from `GET /client/config.json` next to the web client bundle.
- Required on `POST /input`, `GET /stream` and `POST /api/control` via `Authorization: Bearer <token>` header or a `?token=` query parameter (compared constant-time).
- `/health` and `/api/info` remain open so discovery and health checks work without credentials.

**Scope note:** anyone who can reach the HTTP port can read the token from `/client/config.json` or mDNS. The token is a convenience guard against accidental/ambient use, not strong authentication; a hostile device on the same LAN can obtain it. Strong auth (TLS/mTLS) is future work; see SECURITY.md.

---

## 🌐 HTTP API Contract

| Endpoint | Method | Auth | Body | Response |
|----------|--------|------|------|----------|
| `/` | GET | - | - | Redirects to `/client/index.html` (web client) |
| `/stream` | GET | token | - | `video/mp2t` MPEG-TS live stream |
| `/input` | POST | token | pointer/key/stylus JSON | `202 Accepted` |
| `/api/control` | POST | token | `{"action":"lock"\|"blank"\|"unblank"\|"ctrl_alt_del"}` | `200 OK` / `501` when the host lacks the required tool / `400` for unknown actions |
| `/api/info` | GET | - | - | `{"display_width":1920,"display_height":1080,"refresh_hz":60,"encoder":"x264","version":"0.11.0"}` |
| `/health` | GET | - | - | `200 OK "ok"` |

Input events (`/input`) accept the same payload schema as the web client: `{"Pointer":{"Move":{"x","y"}\|"Button":{"button","pressed"}\|"Wheel":{"delta_y"}}}`, `{"Key":{"code","pressed"}}` (Linux evdev keycodes), `{"Stylus":{"Tilt":{"x","y","pressure","tilt_x_deg","tilt_y_deg"}}}`.

---

## 🔌 Transport Optimisations

- **Per-client muxing:** every `/stream` request spawns an `appsrc → mpegtsmux → appsink` pipeline with `h264parse config-interval=1`, so SPS/PPS re-emit every keyframe and late-joining clients decode within one GOP.
- **OkHttpDataSource:** zero read-timeout, long-lived socket, custom `User-Agent: Orbiscreen-Android/1.0` for friendlier server logs.
- **DefaultLoadControl + live tuning:** buffers 1.5 s minimally, 5 s maximally to absorb Wi-Fi jitter; `targetLiveOffsetMs = 1000` keeps the decoder chasing the live edge instead of buffering up.
- **Bounded fan-out:** the encode pipeline uses a bounded mpsc channel; `broadcast::RecvError::Lagged` is tolerated (slow clients fast-forward to the next keyframe) instead of tearing down the HTTP stream; unbounded memory growth from a stalled client is impossible.
- **Protobuf-free:** payloads use `org.json.JSONObject` for both directions to keep the on-wire contract symmetric with the web client.

---

## 🔁 Lifecycle

- `PlayerHolder` is owned by `StreamViewModel`; release happens in `onCleared()`.
- `InputDispatcher` is constructed lazily on first touch and released together with the player.
- `DiscoveryService` is started in `DiscoveryViewModel.init` and detached with the view model scope.
- `EvdiFramePump` stops when its receiver is dropped; the daemon's capture pump aborts cleanly on SIGINT or D-Bus Stop.

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Back to README](../README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
