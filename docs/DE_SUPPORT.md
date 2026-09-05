<div align="center">

# Desktop Environment Support - Orbiscreen

[![Version](https://img.shields.io/badge/version-0.22.4-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 Language

<a href="DE_SUPPORT.md">🇬🇧 English</a> · <a href="DE_SUPPORT_AR.md">🇸🇦 العربية</a>

---

Orbiscreen adapts to the desktop environment it runs on. This document lists,
for every supported family of compositors, which virtual-display path is used,
what the requirements are, and how to fix common problems.

Start with the doctor; it prints everything on this page for *your* machine:

```bash
orbiscreen doctor          # human-readable
orbiscreen doctor --json   # machine-readable output
orbiscreen doctor --fix    # install/load the EVDI kernel module where possible
```

## How `auto` decides

The daemon reads the environment (`XDG_SESSION_TYPE`, `XDG_CURRENT_DESKTOP`,
`WAYLAND_DISPLAY`, `SWAYSOCK`, `HYPRLAND_INSTANCE_SIGNATURE`, …) and builds an
ordered capture plan. The plan is logged on every `orbiscreen start`:

| Environment | `auto` capture plan (in order) |
|---|---|
| KDE Plasma (Wayland) | `portal-virtual` → `kwin-virtual` → `portal` |
| GNOME (Wayland) | `portal-virtual` → `evdi` → `portal` |
| COSMIC (Wayland) | `evdi` → `portal` |
| Sway / Hyprland / other wlroots | `wlroots-virtual` → `wlr-screencopy` → `portal` → `evdi` |
| X11 (any DE) | `evdi` → `x11-root` (XShm mirror) |
| Unknown session | `portal-virtual` → `portal` → `x11-root` (or `evdi` → `x11-root` when no desktop is declared) |

- `wlroots-virtual` fails fast and falls through when the compositor IPC is
  not reachable, so the chain above also covers wlroots compositors without
  virtual-output support.

- `portal-virtual`, `wlroots-virtual`, `kwin-virtual`, and `evdi` create a **real second
  display** that starts empty; drag windows onto it.
- `wlr-screencopy`, `x11-root`, and `portal` **mirror an existing screen**.

## KDE Plasma (Wayland)

- **Virtual display:** native, via `zkde_screencast_unstable_v1` or XDG Portal
  ScreenCast `SourceType::Virtual`. No root, no kernel module, no share dialog.
  The monitor appears as `Virtual-ORBISCREEN`.
- **Capture:** PipeWire stream from the virtual monitor.
- **Input:** RemoteDesktop portal (the grant is remembered after the first
  run, no dialog afterwards).
- **Nothing to install.**

## Sway and general wlroots

- **Virtual display:** created through the compositor IPC. Sway exposes its
  built-in headless backend: the daemon sends `create_output` over the IPC
  socket (`$SWAYSOCK`), waits for the output to appear (e.g. `HEADLESS-2`),
  applies the requested mode, and disables the output when the daemon stops.
  Sway has no IPC command to destroy a dynamically created headless output, so
  the daemon disables it as the closest cleanup (it no longer receives
  frames); it is reclaimed when the compositor exits.
- **Capture:** `zwlr_screencopy_manager_v1` (the same protocol `grim` uses):
  no portal, no dialog, damage-driven, by output name.
- **Input:** `virtual-keyboard-unstable-v1` + `wlr-virtual-pointer-unstable-v1`
  directly on the Wayland socket, no `xdg-desktop-portal-wlr` required.
- **Requirements:** Sway ≥ 1.6 (or any wlroots compositor with `create_output`).
  If the IPC is not reachable, `auto` falls back to mirroring an
  existing output via screencopy.
- **Troubleshooting:**
  - `doctor` shows `virtual out: no compositor IPC reachable`; check that
    `SWAYSOCK` is set in the daemon's environment (systemd user units inherit
    it from the session).
  - Capture fails with `zwlr_screencopy_manager_v1 is not available`; the
    compositor disabled screencopy; mirroring via the portal still works.

## Hyprland

- **Virtual display:** `hyprctl`-style IPC on
  `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`:
  the daemon creates a headless output and destroys it on shutdown.
- **Capture / input:** same as Sway above (screencopy + native virtual
  devices).
- **Requirements:** Hyprland ≥ 0.44.

## GNOME (Wayland / Mutter)

Starting with GNOME 46+, mutter supports creating rootless virtual monitors through the XDG Desktop Portal ScreenCast API:

- **Virtual display:**
  - **Portal Virtual Output (GNOME 46+):** Orbiscreen requests `SourceType::Virtual` via `ashpd::desktop::screencast`. When supported by mutter, an independent virtual display is created on-the-fly without root permissions, kernel modules, or display-manager reconfiguration.
  - **EVDI Fallback (Pre-GNOME 46 or unsupported backends):** Kernel-level virtual DRM driver (`orbiscreen doctor --fix` guides the setup).
- **Capture:** portal ScreenCast (PipeWire). The permission grant is **persisted** (restore token in `$XDG_STATE_HOME/orbiscreen/portal.json`):
  the share dialog appears only on the first run, never again afterwards
  (unless revoked).
- **Input:** RemoteDesktop portal, likewise persisted.
- **Troubleshooting:**
  - Dialog appears on every run -> the backend does not honour restore tokens,
    or the state file was deleted. `doctor` shows `screencast grant saved:
    yes/no`.
  - `portal: org.freedesktop.portal.Desktop NOT on the session bus` -> install
    `xdg-desktop-portal` and `xdg-desktop-portal-gnome`.

## COSMIC Desktop (Wayland / cosmic-comp)

System76's Rust-based COSMIC desktop (`cosmic-comp`, built on Smithay) integrates cleanly with Orbiscreen:

- **Virtual display:** via **EVDI** (kernel virtual DRM driver). When the EVDI
  kernel module is loaded, Orbiscreen creates a hardware-level virtual DRM
  connector (`/dev/dri/card*`) that `cosmic-comp` automatically discovers via
  DRM uevents. You can configure resolution, scaling, and monitor arrangement
  directly in COSMIC Settings. Run `orbiscreen doctor --fix` to install the EVDI
  kernel module automatically.
- **Capture:** PipeWire stream via `xdg-desktop-portal-cosmic`. When EVDI is
  not installed, `auto` falls back to mirroring an existing display via the
  portal. Orbiscreen persists the screencast grant restore token
  (`$XDG_STATE_HOME/orbiscreen/portal.json`), so you only grant permission once.
- **Input:** rootless `/dev/uinput` injector providing native multi-touch
  gestures, mouse, keyboard, and stylus tablet with 4095 pressure levels and
  tilt. `cosmic-comp`'s `libinput` stack discovers the virtual input devices
  immediately.
- **Troubleshooting:**
  - `virtual display: kernel module missing` → Run `orbiscreen doctor --fix`
    (Pop!_OS / Ubuntu: `sudo apt install evdi-dkms`, Fedora: `sudo dnf install evdi`,
    Arch: `sudo pacman -S evdi`).
  - Screen share dialog appears on every run → ensure `xdg-desktop-portal-cosmic`
    is installed and `$XDG_STATE_HOME/orbiscreen/portal.json` is writable.

## X11 (XFCE, MATE, LXQt, Cinnamon, Budgie, KDE-X11)

- **Virtual display:** **EVDI** is the only true extension path on X11
  (`orbiscreen doctor --fix` installs it on detected distros).
- **Capture:** mirrored root window via **MIT-SHM**: one persistent shared
  image the X server writes into directly (no per-frame reply payload), with
  pooled frame buffers and automatic skipping of frames identical to the
  previous one. Falls back to plain `GetImage` when MIT-SHM ≥ 1.2 is absent.
- **Input:** **XTEST** injection, rootless, works for any user on any X11.
  uinput is kept as the stronger fallback (needs `/dev/uinput`).
- **Troubleshooting:**
  - `display: kernel module missing` → `orbiscreen doctor --fix`, or build
    from source: `bash scripts/install-evdi-module.sh`.
  - No input without root → XTEST should have engaged; check `doctor` output
    for the input backend line.

## Chromebooks / ChromeOS (ASUS CM3001 & ARC++)

When running Orbiscreen Android client inside ChromeOS (e.g. ASUS Chromebook CM3001 or any ChromeOS tablet supporting Android apps via ARC++):

- **Android Subnet Isolation:** ChromeOS runs Android applications inside an isolated container (ARC++) behind an internal virtual NAT bridge, typically assigning the Android container an IP on `100.115.92.0/28` (default gateway `100.115.92.2`).
- **Automatic Internal ADB Probing:** Orbiscreen automatically probes `100.115.92.2:5555` alongside `localhost:5555` to detect ADB reverse tunnels inside ChromeOS.
- **Stylus Digitizer Integration:** ChromeOS USI styluses report in-air hover events (`ACTION_HOVER_MOVE`) and tilt. Orbiscreen's background coroutine dispatch on `Dispatchers.IO` handles pen events without triggering UI freezes or main thread exceptions.
- **Setup for ChromeOS Linux (Crostini):**
  1. Enable **Linux development environment** in ChromeOS Settings.
  2. Enable **Develop Android apps** -> **Enable ADB debugging**.
  3. Run `orbiscreen start` inside the Linux terminal container.

## Anything else

`auto` falls back through whatever is available: portal mirror on Wayland,
XShm mirror on X11, EVDI anywhere. `orbiscreen doctor` prints exactly which
step is missing and how to fix it.

## Capture backend preference (`orbiscreen.toml`)

By default the daemon reads `$XDG_CONFIG_HOME/orbiscreen/orbiscreen.toml`
(`~/.config/orbiscreen/orbiscreen.toml` when `XDG_CONFIG_HOME` is unset) -
the same path used by the systemd user service. Create the file there, or
point at another location with `--config /path/to/orbiscreen.toml`.

```toml
[capture]
preferred = "auto"
```

| Value | Behaviour |
|-------|-----------|
| `auto` | KDE Plasma Wayland: KWin virtual display. Sway/Hyprland/wlroots: compositor virtual output via IPC, else wlr-screencopy mirror, else portal. X11: EVDI when its module is loaded, else root capture. |
| `kwin-virtual` | Always the KWin virtual monitor (fails on non-KDE compositors). |
| `screencopy` | Always wlroots screencopy capture (needs a wlroots-based compositor). |
| `evdi` | Always the EVDI DRM virtual display (opt-in, needs the root-installed kernel module). |
| `portal` | Always the portal share dialog; pick any screen. |
| `mirror` | Show your **real** desktop instead of a second monitor: pick the screen to mirror in the share dialog. |

> A virtual display starts **empty** (desktop wallpaper only); that is what a second monitor is. Drag windows onto `Virtual-ORBISCREEN`, or use `mirror` to stream your actual screen.

## Environment variables read by Orbiscreen

| Variable | Purpose |
|---|---|
| `XDG_SESSION_TYPE`, `WAYLAND_DISPLAY`, `DISPLAY` | Wayland vs X11 detection |
| `XDG_CURRENT_DESKTOP`, `DESKTOP_SESSION`, `KDE_FULL_SESSION` | compositor family |
| `SWAYSOCK` | Sway IPC for virtual outputs |
| `HYPRLAND_INSTANCE_SIGNATURE`, `XDG_RUNTIME_DIR` | Hyprland IPC socket path |
| `XDG_STATE_HOME` | location of the saved portal grants (`orbiscreen/portal.json`) |
