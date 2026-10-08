<div align="center">

# Troubleshooting - Orbiscreen

[![Version](https://img.shields.io/badge/version-0.35.5-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.92%2B-16a34a?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## Language

<a href="TROUBLESHOOTING.md">🇬🇧 English</a> · <a href="TROUBLESHOOTING_AR.md">🇸🇦 العربية</a>

---

## Table of Contents

- [Debian package build fails on the PPA](#debian-package-build-fails-on-the-ppa)
- [RPM build fails on COPR](#rpm-build-fails-on-copr)
- [Release workflow fails before publishing](#release-workflow-fails-before-publishing)
- [Android SDK not found](#android-sdk-not-found)
- [uinput permission denied](#uinput-permission-denied)
- [Missing evdi kernel module when building for older X11 setups](#missing-evdi-kernel-module-when-building-for-older-x11-setups)
- [GStreamer plugins missing](#gstreamer-plugins-missing)
- [mDNS discovery finds no host](#mdns-discovery-finds-no-host)
- [Connection refused or timed out](#connection-refused-or-timed-out)
- [Bandwidth saturation on a wireless link](#bandwidth-saturation-on-a-wireless-link)
- [UDP video stutters while HTTP video is fine](#udp-video-stutters-while-http-video-is-fine)
- [Cannot pair: the request arrives with no way to accept it](#cannot-pair-the-request-arrives-with-no-way-to-accept-it)
- [GNOME / Wayland (Mutter) shows no picture](#gnome--wayland-mutter-shows-no-picture)
- [sway / Hyprland reports a capture backend that never starts](#sway--hyprland-reports-a-capture-backend-that-never-starts)
- [COSMIC (cosmic-comp) does not offer a capture backend](#cosmic-cosmic-comp-does-not-offer-a-capture-backend)
- [X11: the cursor escapes the virtual display](#x11-the-cursor-escapes-the-virtual-display)
- [No screen displayed on the client](#no-screen-displayed-on-the-client)
- [Mouse or touch does nothing](#mouse-or-touch-does-nothing)
- [Host daemon exits immediately](#host-daemon-exits-immediately)
- [Decoder falls back to software, or the picture never appears](#decoder-falls-back-to-software-or-the-picture-never-appears)
- [App crashes immediately on launch](#app-crashes-immediately-on-launch)
- [USB connection shows "Looking for host…" forever](#usb-connection-shows-looking-for-host-forever)
- [ADB connection fails on ASUS Chromebook CM3001](#adb-connection-fails-on-asus-chromebook-cm3001)
- [Dragging windows or selecting text in Touchpad mode](#dragging-windows-or-selecting-text-in-touchpad-mode)
- [Touch is rotated or misaligned](#touch-is-rotated-or-misaligned)
- [Control toolbar actions return 404](#control-toolbar-actions-return-404)
- [Discovery list is empty though hosts are on the same Wi-Fi](#discovery-list-is-empty-though-hosts-are-on-the-same-wi-fi)
  - [Still Stuck?](#still-stuck)
- [CI Action: `Format & Lint` (`cargo fmt --check`)](#ci-action-format--lint-cargo-fmt---check)
- [CI Action: `Clippy (deny warnings)`](#ci-action-clippy-deny-warnings)
- [CI Action: `Build` (`cargo build --workspace --locked`)](#ci-action-build-cargo-build---workspace---locked)
- [CI Action: `Test` (`cargo test --workspace --locked`)](#ci-action-test-cargo-test---workspace---locked)
- [CI Action: `Run cargo-deny`](#ci-action-run-cargo-deny)
- [CI Action: `Android assembleDebug` + `lintDebug`](#ci-action-android-assembledebug--lintdebug)
- [Runtime: `orbiscreen start` fails - `kernel module is not installed`](#runtime-orbiscreen-start-fails---kernel-module-is-not-installed)
- [Runtime: KDE Plasma (virtual display without evdi or root)](#runtime-kde-plasma-virtual-display-without-evdi-or-root)
- [Runtime: capture backend unavailable on Wayland](#runtime-capture-backend-unavailable-on-wayland)
- [Runtime: `unsafe_op_in_unsafe_fn` / `missing_debug_implementations`](#runtime-unsafe_op_in_unsafe_fn--missing_debug_implementations)
- [Android Client & Devices](#android-client--devices)
  - [Android / ChromeOS: ADB connection fails on ASUS Chromebook CM3001](#android--chromeos-adb-connection-fails-on-asus-chromebook-cm3001)
  - [Android: Stylus / Pen not drawing, incorrect pressure, or app crash on Lenovo Tab](#android-stylus--pen-not-drawing-incorrect-pressure-or-app-crash-on-lenovo-tab)
  - [Android: Dragging windows or selecting files in Touchpad mode](#android-dragging-windows-or-selecting-files-in-touchpad-mode)
  - [Android: app crashes or process dies when tapping Connect](#android-app-crashes-or-process-dies-when-tapping-connect)
  - [Android: black screen after Connect](#android-black-screen-after-connect)
  - [Android: discovery list is empty even though hosts are on the same Wi-Fi](#android-discovery-list-is-empty-even-though-hosts-are-on-the-same-wi-fi)
  - [Android: touch is rotated / misaligned](#android-touch-is-rotated--misaligned)
  - [Android: control toolbar actions return 404](#android-control-toolbar-actions-return-404)
  - [Android: app crashes immediately on launch](#android-app-crashes-immediately-on-launch)
  - [Android: USB connection shows "Looking for host…"](#android-usb-connection-shows-looking-for-host)
- [Streaming: High latency, stutter, or slow mouse movement on 5GHz Wi-Fi](#streaming-high-latency-stutter-or-slow-mouse-movement-on-5ghz-wi-fi)
- [Streaming: Stream error causes infinite reconnect flicker instead of detecting disconnect](#streaming-stream-error-causes-infinite-reconnect-flicker-instead-of-detecting-disconnect)
- [Multi-Monitor / X11: Mouse cursor escapes virtual display to other physical screens](#multi-monitor--x11-mouse-cursor-escapes-virtual-display-to-other-physical-screens)
- [Client shows the wrong screen (primary desktop instead of virtual display)](#client-shows-the-wrong-screen-primary-desktop-instead-of-virtual-display)
- [Web client loads but shows no picture](#web-client-loads-but-shows-no-picture)
- [No encoder available - stream starts but errors out (x264 missing)](#no-encoder-available---stream-starts-but-errors-out-x264-missing)
- [401 Unauthorized from `/stream`, `/input` or `/api/control` (token)](#401-unauthorized-from-stream-input-or-apicontrol-token)
- [Daemon not found on D-Bus](#daemon-not-found-on-d-bus)
- [Daemon: 100% CPU usage or freeze](#daemon-100-cpu-usage-or-freeze)

# CI Packaging Failures

Packaging runs in dedicated workflows rather than in `ci.yml`, so a green
`CI` check does not mean the packages build. The release gate re-runs
`cargo deny check` and `cargo audit` before anything is published.

---

<a id="ci-deb"></a>
## Debian package build fails on the PPA

**Symptom:**
```
dpkg-buildpackage: error: unmet build-dependency: libwebkit2gtk-4.1-dev
```
or, from the Ubuntu job:
```
error: Package 'libjavascriptcoregtk-4.1-dev' has no installation candidate
```

**Cause:**
`debian/rules` builds the whole workspace, which includes the Tauri control
centre. That needs the WebKitGTK development packages, and `debian/control`
has to list every one of them or the build cannot start.

**Fix:**
`Build-Depends` must carry `libwebkit2gtk-4.1-dev`,
`libjavascriptcoregtk-4.1-dev`, `libsoup-3.0-dev`, `libgtk-3-dev`,
`librsvg2-dev` and `libayatana-appindicator3-dev` next to the GStreamer
packages. The workflow also installs them before `debuild`; if the PPA image
lacks them, the build fails in the workflow rather than in `debian/rules`.

---

<a id="ci-rpm"></a>
## RPM build fails on COPR

**Symptom:**
```
error: Failed build dependencies:
/bin/sh: line 31: gstreamer1-devel: command not found
```

**Cause:**
`data/orbiscreen-copr.spec` compiles the same workspace, so it needs the
Fedora equivalents of the packages the Debian build needs.

**Fix:**
Confirm `BuildRequires` in the spec lists `gstreamer1-devel`,
`gstreamer1-plugins-base-devel`, `libdrm-devel`, `libxkbcommon-devel`,
`webkit2gtk4.1-devel`, `gtk3-devel`, `libappindicator-gtk3-devel` and
`librsvg2-devel`, and that the requested Rust is at least as new as
`rust-version` in `Cargo.toml`. `scripts/check-versions.sh` catches a stale
`Version:` tag but not a missing build requirement.

---

<a id="ci-release"></a>
## Release workflow fails before publishing

**Symptom:**
`Release Matrix` fails in `Verify code integrity (release gate)`, and no
assets are uploaded.

**Cause:**
The gate blocks the release when the integrity checks fail: formatting,
clippy with warnings denied, the test suite, `cargo machete`, `cargo deny
check`, or `cargo audit`. It runs on untrusted refs too, so a tag on a commit
that does not pass cannot publish.

**Fix:**
Read which step failed in the run summary and fix that locally:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo machete && cargo deny check && cargo audit
```
The gate runs on the workspace including the GUI crate, so
`cargo clippy --workspace` needs the WebKitGTK development packages that
`ci.yml` installs.

---

# Local Build Issues

---

<a id="local-sdk"></a>
## Android SDK not found

**Symptom:**
```
SDK location not found. Define a valid SDK location with an ANDROID_HOME environment variable
```

**Cause:**
`clients/android` needs an SDK with the platform the app compiles against.
The directory has to be the one holding `platforms/`, not the
`cmdline-tools` parent.

**Fix:**
Point `ANDROID_HOME` at the SDK root and accept the licences once:
```bash
export ANDROID_HOME="$HOME/Android/Sdk"
yes | "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --licenses
"$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --install \
  "platform-tools" "platforms;android-35" "build-tools;35.0.0"
```
Then build with `./gradlew assembleDebug` from `clients/android`.

---

<a id="local-uinput"></a>
## uinput permission denied

**Symptom:**
```
no usable input injector found (uinput and portal both failed)
```
or, when the daemon starts:
```
orbiscreen_input: permission denied on /dev/uinput
```

**Cause:**
Input injection goes through kernel `uinput` devices, and `/dev/uinput`
normally belongs to `root:input` with mode `0660`. A user outside the
`input` group cannot open it.

**Fix:**
Add the user to the group and start a new login session:
```bash
sudo usermod -aG input "$USER"
# log out and back in, then verify
id -nG | tr ' ' '\n' | grep -x input
ls -l /dev/uinput
```
`scripts/install.sh` and the systemd unit rely on the same group, and the
udev rules grant console-user access to the USB nodes. Do not run the daemon
as root to work around this: it would put the injected input outside the
session's ACLs.

---

<a id="local-evdi"></a>
## Missing evdi kernel module when building for older X11 setups

**Symptom:**
```
Error: evdi kernel module is not installed
```

**Cause:**
DisplayLink userspace needs a matching kernel module. See
[Runtime: `orbiscreen start` fails](#runtime-evdi) for the install and the
capture-backend preference that avoids needing it at all.

**Fix:**
Install the DKMS build for the running kernel, then confirm:
```bash
lsmod | grep evdi
```

---

<a id="local-gst"></a>
## GStreamer plugins missing

**Symptom:**
```
ERROR ... no element found: nvh264enc
```
or the build stops at `vaapih264enc`.

**Cause:**
H.264 lives in the `good`, `bad` and `ugly` plugin sets, and the hardware
encoders need `bad`. Without them the pipeline cannot be assembled.

**Fix:**
```bash
# Fedora / Nobara
sudo dnf install gstreamer1-plugins-good gstreamer1-plugins-bad-free gstreamer1-plugins-ugly
# Ubuntu / Debian
sudo apt install gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly
gst-inspect-1.0 nvh264enc || gst-inspect-1.0 x264enc
```
See [No encoder available](#no-encoder) for the full symptom list.

---
---

# Network & Connection Issues

---

<a id="net-mdns"></a>
## mDNS discovery finds no host

**Symptom:**
The Android client shows an empty host list even though host and device are
on the same subnet. The daemon logs:
```
orbiscreen_transport::mdns: Advertised Orbiscreen service via mDNS instance=... port=8788
```
yet nothing appears on the device.

**Cause:**
Discovery relies on multicast, which does not cross a routed network, and
which many Wi-Fi access points, VPNs and container bridges drop or filter.
A client and host on different subnets will never see each other this way.

**Fix:**
Confirm the daemon advertised itself, then check the path:
```bash
avahi-browse -rt _orbiscreen._tcp
ping -c 3 <host-lan-ip>
```
If multicast is filtered, connect by address instead: the client accepts a
host typed in, or a LAN IP passed on the command line. Nothing else needs
changing. `docs/DE_SUPPORT.md` covers the capture backend; discovery is a
separate concern and does not depend on it.

---

<a id="net-timeout"></a>
## Connection refused or timed out

**Symptom:**
```
curl: (7) Failed to connect to 192.168.1.10 port 8788: Connection refused
```
or the client sits on `Looking for host…` without an error.

**Cause:**
Either nothing is listening, or a firewall is in between. The daemon binds
all interfaces, so a refusal means the process is not running.

**Fix:**
Check what the host reports, from the host itself:
```bash
curl -s http://127.0.0.1:8788/health
ss -tlnp | grep -E ':(8788|8790)'
```
If the loopback probe answers but a client does not, the path is blocked:
```bash
sudo firewall-cmd --list-ports    # Fedora
sudo ufw status                   # Debian, Ubuntu
```
Ports `8788` (signalling and control), `8789` (UDP video) and `8790`
(HTTPS client and WebTransport) must be reachable, and UDP `5353` for
discovery.

---

<a id="net-bandwidth"></a>
## Bandwidth saturation on a wireless link

**Symptom:**
The stream works on a wired link and stutters over Wi-Fi. Latency climbs
while the picture is still moving, and dropping the resolution helps.

**Cause:**
A 2560x1600 stream at 90 fps needs well over a gigabit per second, which a
5 GHz link in a busy radio cannot deliver. The encoder then fills its own
buffers, and what the viewer sees is the queue draining rather than the live
screen.

**Fix:**
Lower the resolution or the framerate on the client. The delay figures in
`docs/UDP_TRANSPORT.md` assume a link that keeps up; on a saturated link
measure with the picture paused to separate transport delay from queueing.

---

<a id="net-udp"></a>
## UDP video stutters while HTTP video is fine

**Symptom:**
Switching a client from HTTP Annex-B to UDP video removes the stutter, or
the reverse.

**Cause:**
The UDP path runs its own path-MTU search and congestion handling, and the
tuning hooks in `docs/UDP_TRANSPORT.md` apply to it only. A path that
cannot hold the datagram size is forced back down, which shows up as jitter
rather than as a failure.

**Fix:**
Set the hooks from that document and re-measure:
```bash
export ORBISCREEN_UDP_MAX_DATAGRAM=1200
export ORBISCREEN_UDP_MAX_CLIENTS=8
```

---

## Cannot pair: the request arrives with no way to accept it

**Symptom:**
The Android or web client waits on "awaiting host approval", and the PC receives
the pairing request, but nothing on the desktop offers an accept action.

**Cause:**
The approval endpoint existed, but nothing on the host called it: the only reachable
surface was the web client's host-approvals modal, which requires pasting the host
admin token by hand. The daemon now exposes the queue over D-Bus and `orbiscreen
pair` manages it.

**Fix:**
1. Review what is waiting:
   ```bash
   orbiscreen pair list
   ```
   The daemon also logs every incoming request with the device label, its IP, and
   the same command hint.
2. Accept it:
   ```bash
   orbiscreen pair approve          # newest pending request
   orbiscreen pair approve <id>     # a specific id from the list
   ```
   The device picks up its credential automatically on its next status poll.
3. Requests expire after 10 minutes. `orbiscreen pair deny [id]` rejects one, and
   `orbiscreen pair revoke <client id>` removes an already paired device.

---

# Compositor-Specific Issues

Capture support per compositor, and how the `auto` backend chooses, are
documented in `docs/DE_SUPPORT.md`. These entries cover only what a user sees
when something does not work.

---

<a id="comp-mutter"></a>
## GNOME / Wayland (Mutter) shows no picture

**Symptom:**
The stream connects and the virtual display is created, but the client
receives no frames, and the daemon logs a capture backend line for GNOME
without a matching first frame.

**Cause:**
GNOME needs the screen-cast portal, and the portal prompt is per session.
A prompt answered once for the login session still applies, but a daemon
started before the prompt was answered keeps the un-granted handle.

**Fix:**
Start the daemon after granting the prompt, and confirm the session has a
wayland display of its own:
```bash
echo "$WAYLAND_DISPLAY" "$XDG_SESSION_TYPE"
```
See the GNOME section of `docs/DE_SUPPORT.md` for the portal flow.

---

<a id="comp-wlroots"></a>
## sway / Hyprland reports a capture backend that never starts

**Symptom:**
`sway` starts, the output is created, and then nothing is encoded.

**Cause:**
wlroots compositors need the screencopy protocol advertised by the session.
A nested or headless session (`sway --headless` under another compositor)
usually does not provide it.

**Fix:**
Confirm the protocol is present before blaming the encoder:
```bash
swaymsg -t get_outputs | head -40
```
The sway and Hyprland sections of `docs/DE_SUPPORT.md` describe what a
working session reports.

---

<a id="comp-cosmic"></a>
## COSMIC (cosmic-comp) does not offer a capture backend

**Symptom:**
The daemon starts, reports `capture_backend` as `auto`, and never opens a
capture.

**Cause:**
COSMIC support depends on the compositor exposing the screencast interface
`orbiscreen` looks for. When it does not, `auto` has nothing to pick.

**Fix:**
Check what the compositor advertises, then read the COSMIC section of
`docs/DE_SUPPORT.md` for the current support status before assuming a bug.

---

<a id="comp-x11"></a>
## X11: the cursor escapes the virtual display

**Symptom:**
Moving the mouse over the virtual screen moves it onto a physical monitor,
and clicks land on the wrong display.

**Cause:**
The X11 input path maps the injected pointer through the server, and a
cursor that leaves the virtual screen's bounds is grabbed by whichever
physical output is under the physical pointer position.

**Fix:**
Reposition the virtual output so it does not overlap a physical one, and
verify the scaling the daemon applied:
```bash
xrandr --listmonitors
```
See [Multi-Monitor / X11](#cursor-clamping) for the cursor behaviour itself.

---

# Runtime Behaviour

---

<a id="runtime-no-screen"></a>
## No screen displayed on the client

**Symptom:**
The client connects, reports a session, and shows nothing at all.

**Cause:**
Usually one of three things: the capture backend produced no frames, the
encoder has no element, or the client is showing the wrong output.

**Fix:**
Work through them in order:
1. Confirm frames exist: `busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetStatus`
   and look at `frames_forwarded`
2. Confirm the encoder resolved: the same output reports `encoder`
3. Confirm the client targets the virtual screen: see
   [Client shows the wrong screen](#wrong-screen)

---

<a id="runtime-input"></a>
## Mouse or touch does nothing

**Symptom:**
Video streams, but input has no effect on the host.

**Cause:**
Input needs `/dev/uinput`, and the injector is opened once when a session
starts. A daemon started without the permission opens no injector at all.

**Fix:**
Look for the injector line at startup:
```bash
journalctl --user -u orbiscreen | grep -i 'input injector'
```
If it is missing, see [uinput permission denied](#local-uinput). If it is
present and input still does nothing, the session header may not be
addressing the display you are clicking on; see
[401 Unauthorized](#token-401).

---

<a id="runtime-exits"></a>
## Host daemon exits immediately

**Symptom:**
`orbiscreen start` prints the banner and the process is gone before any
client connects.

**Cause:**
A failed `systemd` unit is reported by the journal rather than the
terminal, and a missing capture backend is fatal at start.

**Fix:**
Ask the daemon what it recorded, and read the unit log for the reason:
```bash
journalctl --user -u orbiscreen -n 40
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetStatus
```
The D-Bus surface is described in `docs/DBUS_SPEC.md`.

---
---

# Android Client Gaps

---

<a id="android-decoder"></a>
## Decoder falls back to software, or the picture never appears

**Symptom:**
The client logs
```
AOA Annex-B video up 2560x1600
MediaCodec configured 2560x1600
codec input full; hold until IDR
```
and then stays on the waiting message instead of showing a picture.

**Cause:**
The decoder is configured and fed, but no keyframe has arrived yet. The
codec cannot start without one, so this is a symptom of the transport rather
than of the decoder.

**Fix:**
Confirm the host actually sent one:
```bash
journalctl --user -u orbiscreen | grep -i 'IDR requested from encoder'
```
If IDRs are requested but the device still waits, the frames are not
reaching it. Over USB that means the link cannot keep up:
```bash
for d in /sys/bus/usb/devices/*; do cat "$d/speed" 2>/dev/null && echo " <- $d"; done
```
A High-Speed link reports `12`. Lower the resolution or the framerate until
it fits; the ceiling is the link, not the encoder.

---

<a id="android-app-crash-launch"></a>
## App crashes immediately on launch

**Symptom:**
Tapping the icon shows a brief window and nothing else.

**Cause:**
Usually a stale install whose native libraries do not match the APK, or a
build signed with a different key than the installed copy, which Android
refuses to replace.

**Fix:**
Read the crash, then reinstall cleanly:
```bash
adb logcat -c && adb shell am start -n com.orbiscreen.android/.MainActivity
adb logcat -d | grep -i orbiscreen
adb uninstall com.orbiscreen.android
adb install orbiscreen-android-release.apk
```

---

<a id="android-usb-host"></a>
## USB connection shows "Looking for host…" forever

**Symptom:**
The host claims the device and logs
```
AOA accessory claimed on "...", in_ep=0x81, out_ep=0x01
```
but the client never leaves "Looking for host…".

**Cause:**
The host has the accessory role, but the client has not been granted
permission to open it. Android shows its own prompt for this; until it is
answered the client can only wait.

**Fix:**
Accept the USB permission dialog on the device, and confirm the accessory is
actually attached rather than merely claimed:
```bash
adb shell dumpsys usb | grep -i current_functions
```
`ACCESSORY` means the role is active. If the prompt never appears, the
installed app is not the one whose accessory filter matches the host; compare
the `usb-manufacturer`, `usb-model` and `usb-version` it advertises against
what the daemon logs.

---

<a id="android-evdi-host"></a>
## ADB connection fails on ASUS Chromebook CM3001

**Symptom:**
`adb devices` lists nothing on a CM3001 or another ChromeOS device, while
the host and the device are on the same network.

**Cause:**
ChromeOS runs the host-side adb over a network transport, and that transport
has to be allowed separately from USB.

**Fix:**
Use the network form, once the device is reachable:
```bash
adb connect <device-lan-ip>:5555
adb devices
```
The Chromebook-specific path is described in `docs/DE_SUPPORT.md`.

---

<a id="android-touchpad-drag"></a>
## Dragging windows or selecting text in Touchpad mode

**Symptom:**
A click-and-drag selects text or moves a window instead of drawing.

**Cause:**
A pen or finger reports as a touch, and the host injects it as one. Touchpad
mode on the device layer reports button events the compositor already acts
on, so the gesture is consumed before it reaches the virtual screen.

**Fix:**
Use the dedicated tablet or stylus tool in the toolbar rather than a direct
finger contact, and check which device the host bound:
```bash
journalctl --user -u orbiscreen | grep 'bound KWin input device'
```
The daemon binds the mouse, touch and pen devices separately, so the log
tells you which one a gesture arrived through.

---

<a id="android-touch-offset"></a>
## Touch is rotated or misaligned

**Symptom:**
Drawing lands offset from the finger, or on the wrong side, while video is
correct.

**Cause:**
The capture rotation and the injected touch rotation are decided separately.
When the output is scaled, the touch coordinates need the same transform as
the picture.

**Fix:**
Confirm the scale the daemon applied, then change one thing at a time:
```bash
journalctl --user -u orbiscreen | grep 'Enabled and scaled KWin output'
```
Restarting the client after a resolution change re-attaches the stream and
rebinds the input devices, which usually clears a stale transform.

---

<a id="android-control-404"></a>
## Control toolbar actions return 404

**Symptom:**
Tapping a control in the client produces a 404 in the daemon log.

**Cause:**
The toolbar posts to `/api/control`, which requires the session token. A
client that never fetched it sends no credential, and the route rejects it.

**Fix:**
Confirm the client obtained the token:
```bash
adb logcat -d | grep -i 'token fetch'
```
A successful line reports `available=true`. If it fails, the session cannot
authenticate at all; see [401 Unauthorized](#token-401).

---

<a id="android-discovery-empty"></a>
## Discovery list is empty though hosts are on the same Wi-Fi

**Symptom:**
No hosts appear in the client's list, while a host typed in by hand works.

**Cause:**
Discovery is multicast based, and multicast is commonly filtered on
Wi-Fi. A manually entered address takes a different path, which is why it
can still work.

**Fix:**
Verify from the host that the service is advertised:
```bash
avahi-browse -rt _orbiscreen._tcp
```
Then see [mDNS discovery finds no host](#net-mdns).

---

### Still Stuck?
- [CI Failures](#ci-failures)
  - [Format & Lint](#ci-action-format--lint-cargo-fmt---check)
  - [Clippy (deny warnings)](#ci-action-clippy-deny-warnings)
  - [Build](#ci-action-build-cargo-build---workspace---locked)
  - [Test](#ci-action-test-cargo-test---workspace---locked)
  - [cargo-deny](#ci-action-run-cargo-deny)
  - [Android assembleDebug + lintDebug](#ci-action-android-assembledebug--lintdebug)
- [Runtime](#runtime-orbiscreen-start-fails---kernel-module-is-not-installed)
  - [orbiscreen start fails - kernel module not installed](#runtime-orbiscreen-start-fails---kernel-module-is-not-installed)
  - [KDE Plasma without evdi or root](#runtime-kde-plasma-virtual-display-without-evdi-or-root)
  - [Capture backend unavailable on Wayland](#runtime-capture-backend-unavailable-on-wayland)
  - [unsafe_op_in_unsafe_fn lint warnings](#runtime-unsafe_op_in_unsafe_fn--missing_debug_implementations)
- [Android Client & Devices](#android-client--devices)
  - [ADB connection fails on ASUS Chromebook CM3001](#android--chromeos-adb-connection-fails-on-asus-chromebook-cm3001)
  - [Stylus / Pen not drawing on Lenovo Tab](#android-stylus--pen-not-drawing-incorrect-pressure-or-app-crash-on-lenovo-tab)
  - [Touchpad drag and select](#android-dragging-windows-or-selecting-files-in-touchpad-mode)
  - [App crashes when tapping Connect](#android-app-crashes-or-process-dies-when-tapping-connect)
  - [App crashes immediately on launch](#android-app-crashes-immediately-on-launch)
  - [Black screen after Connect](#android-black-screen-after-connect)
  - [Discovery list empty on the same Wi-Fi](#android-discovery-list-is-empty-even-though-hosts-are-on-the-same-wi-fi)
  - [Touch rotated or misaligned](#android-touch-is-rotated--misaligned)
  - [Control toolbar actions return 404](#android-control-toolbar-actions-return-404)
  - [USB connection shows "Looking for host…"](#android-usb-connection-shows-looking-for-host)
- [Streaming & Clients](#streaming-high-latency-stutter-or-slow-mouse-movement-on-5ghz-wi-fi)
  - [High latency or stuttering on 5GHz Wi-Fi](#streaming-high-latency-stutter-or-slow-mouse-movement-on-5ghz-wi-fi)
  - [Infinite reconnect flicker](#streaming-stream-error-causes-infinite-reconnect-flicker-instead-of-detecting-disconnect)
  - [Mouse cursor escapes to other screens](#multi-monitor--x11-mouse-cursor-escapes-virtual-display-to-other-physical-screens)
  - [Wrong screen shown](#client-shows-the-wrong-screen-primary-desktop-instead-of-virtual-display)
  - [Web client loads but shows no picture](#web-client-loads-but-shows-no-picture)
  - [No encoder available](#no-encoder-available---stream-starts-but-errors-out-x264-missing)
- [Daemon](#daemon-not-found-on-d-bus)
  - [401 Unauthorized from /stream, /input or /api/control](#401-unauthorized-from-stream-input-or-apicontrol-token)
  - [Daemon not found on D-Bus](#daemon-not-found-on-d-bus)
  - [100% CPU usage or freeze](#daemon-100-cpu-usage-or-freeze)
- [Still Stuck?](#still-stuck)

---

<a id="ci-failures"></a>
# CI Failures

<a id="ci-fmt"></a>
## CI Action: `Format & Lint` (`cargo fmt --check`)

**Symptom:**
```
Diff in src/main.rs:
-    let x = 1;
+    let x = 1;
```

**Cause:**
Source code does not adhere to standard Rust formatting (`rustfmt`).

**Fix:**
```bash
cargo fmt --all
git add -A
git commit -m "orbiscreen | v0.31.3 | style: cargo fmt --all"
```

**Prevention:**
Run `./gradlew :app:lintDebug` and `cargo fmt --all` locally before pushing.

---

<a id="ci-clippy"></a>
## CI Action: `Clippy (deny warnings)`

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
git commit -m "orbiscreen | v0.31.3 | fix: resolve clippy warnings"
```

**Prevention:**
Run `cargo clippy` locally before pushing.

---

<a id="ci-build"></a>
## CI Action: `Build` (`cargo build --workspace --locked`)

**Symptom:**
```
error[E0463]: can't find crate for `gstreamer`
```

**Fix:**
```bash
cargo update -p gstreamer
cargo build --workspace --locked
git add Cargo.lock
git commit -m "orbiscreen | v0.31.3 | chore: refresh Cargo.lock"
```

---

<a id="ci-test"></a>
## CI Action: `Test` (`cargo test --workspace --locked`)

Tests assume the host has GStreamer plugins (`x264enc`, `vaapih264enc`, `nvh264enc`). Install them locally:
```bash
sudo dnf install gstreamer1.0-plugins-{good,bad,ugly,libav}
```

---

<a id="ci-deny"></a>
## CI Action: `Run cargo-deny`

This is a **non-blocking** informational check. See `deny.toml` for the allowlist.

---

<a id="ci-android"></a>
## CI Action: `Android assembleDebug` + `lintDebug`

The Android workflow runs `./gradlew :app:assembleDebug :app:lintDebug`. Common failures:

- **UnstableApi lint error.** `clients/android/app/lint.xml` opts in to `androidx.media3.common.util.UnstableApi`. If you call a new Media3 API, make sure the surrounding class is annotated with `@OptIn(UnstableApi::class)`.
- **Compose imports.** Run `./gradlew :app:compileDebugKotlin` to localise the error first; lint is slower.

---

<a id="runtime-evdi"></a>
## Runtime: `orbiscreen start` fails - `kernel module is not installed`

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
## Runtime: KDE Plasma (virtual display without evdi or root)

**Symptom:** `orbiscreen start` logs `EVDI kernel module not active` and you do not want to build a kernel module.

**Fix:** on KDE Plasma Wayland nothing else is needed. With the default `[capture] preferred = "auto"`, a connecting client asks the daemon to create a KWin virtual monitor through the `zkde_screencast_unstable_v1` Wayland protocol (description is the device name, connector `Virtual-Orbi-<key>`) and streams it over PipeWire, no root, no share dialog. KWin only exposes that protocol to allow-listed executables, so the daemon maintains `~/.local/share/applications/orbiscreen.kwin.desktop` (user-writable) and refreshes the KService cache automatically; the first run may take a few extra seconds while the grant becomes visible.

Notes:
- Force the path with `[capture] preferred = "kwin-virtual"` (fail loudly if unavailable) or `"portal"` (always show the share dialog).
- The output for a client disappears when that client disconnects; all of them disappear when the daemon stops.
- **You see only the desktop wallpaper in the stream?** That is correct: the virtual monitor is a *second, empty* screen. Drag windows onto that client's output (`Virtual-Orbi-<device>`), or set `[capture] preferred = "mirror"` to stream your real screen instead.
- On GNOME / wlroots compositors the protocol does not exist and `auto` falls back to the portal share dialog.
- EVDI is now opt-in (`preferred = "evdi"`); `auto` on Wayland never touches it, so the old `EVDI kernel module not active` line no longer appears on KDE.

---

<a id="runtime-wayland"></a>
## Runtime: capture backend unavailable on Wayland

Use `CaptureSession::open_with_preference()` (the daemon already does so).

---

<a id="runtime-lints"></a>
## Runtime: `unsafe_op_in_unsafe_fn` / `missing_debug_implementations`

Use `#[allow(missing_debug_implementations)]` or `#[allow(unsafe_code)]` on the offending type/function.

---

## Android Client & Devices

<a id="android-chromebook-adb"></a>
### Android / ChromeOS: ADB connection fails on ASUS Chromebook CM3001

**Symptom:**
Running the Orbiscreen Android app on ASUS Chromebook CM3001 (or other ChromeOS devices) shows "Looking for host" in USB mode and fails to find the Linux daemon.

**Cause:**
ChromeOS isolates Android apps inside an ARC++ container with its own virtual network namespace (`100.115.92.0/28`). USB streaming uses Android Open Accessory, not `adb reverse` into Crostini.

**Fix:**
- Run `orbiscreen start` in the Linux container and grant the accessory permission dialog on the Android side.
- When AOA is not up, the client still probes ARC/USB-tether gateways (`100.115.92.2`, …) on the signaling port.

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
- Update to `orbiscreen-android-release.apk` **v0.31.3** or later.
- v0.31.3 moves stylus dispatch to background coroutines (`Dispatchers.IO`) with `latestStylus` coalescing, preventing UI freezes and thread exceptions.
- Implements `setOnGenericMotionListener` for in-air hover cursor tracking.
- Calibrates tilt math (`-altitudeDeg * cos(orientationRad)`) and ensures `BTN_TOOL_PEN: RELEASED` is cleanly emitted when lifting the pen.

---

<a id="android-touchpad-drag"></a>
### Android: Dragging windows or selecting files in Touchpad mode

**Symptom:**
In Touchpad mode, tapping and dragging on the tablet moves the mouse pointer, but does not drag windows or select text.

**Cause:**
Prior to v0.31.3, Touchpad mode only sent hover pointer movements, lacking a drag gesture.

**Fix:**
- Update to **v0.31.3**.
- **Double-tap and drag:** Double-tap on the touch surface and keep your finger held down on the second tap. As you move your finger, mouse button 1 remains pressed, smoothly dragging the window, file, or selection.
- Lifting your finger releases mouse button 1.

---

<a id="android-connect-crash"></a>
### Android: app crashes or process dies when tapping Connect

**Symptom:**
Tapping a host on the Discovery screen immediately kills the app process or crashes back to the launcher.

**Cause:**
`PlayerHolder.build()` must be executed on the main thread. ExoPlayer requires main-thread construction; creating player components on IO threads throws thread access exceptions that terminate the process.

**Fix:**
- Upgrade to `orbiscreen-android-release.apk` **v0.31.3** or later.
- `StreamViewModel` constructs ExoPlayer on `Dispatchers.Main` with try-catch hardening so errors surface as `StreamEvent.Error` retry cards instead of crashing.

---

<a id="android-black-screen"></a>
### Android: black screen after Connect

**Symptom:**
Tapping a discovered host shows a black surface; no video; the control toolbar does not appear.

**Cause:**
ExoPlayer MIME sniffing for `/stream` falling back to a black surface when failing to detect MPEG-TS automatically.

**Fix:**
- Upgrade to `orbiscreen-android-release.apk` **v0.31.3** or later.
- `PlayerHolder` configures `MediaItem` with explicit `setMimeType(MimeTypes.VIDEO_MP2T)`, ensuring the stream decodes without sniffing.
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
Restart the daemon on the host:
```bash
orbiscreen stop
orbiscreen start
```

---

<a id="android-crash"></a>
### Android: app crashes immediately on launch

**Symptom:**
You open the Orbiscreen app on Android and it immediately crashes back to the home screen.

**Cause:**
Legacy WebView issues in obsolete versions.

**Fix:**
Orbiscreen uses Jetpack Compose + `PlayerView` exclusively without WebView. Ensure you are running `orbiscreen-android-release.apk` **v0.31.3** or later. If any crash occurs, capture a logcat with `adb logcat *:E | grep orbiscreen` and open an issue.

---

<a id="android-usb"></a>
### Android: USB connection shows "Looking for host…"

**Fix:**
USB uses Android Open Accessory, not `adb reverse`. Ensure:
1. The host daemon is running (`orbiscreen start`).
2. The cable is plugged in. The tablet should show the USB accessory permission dialog (Orbiscreen Display Server), tap **OK**.
3. Verify what the daemon sees:
   ```bash
   orbiscreen doctor          # USB Direct / Cable card
   ```
4. Tap the **USB** card on the Discovery screen if the dialog was dismissed. The card is ready when the AOA proxy is up, not when `127.0.0.1:8788` happens to answer.

The daemon's accessory count is also visible in `GET /health` (`usb_devices`) and the D-Bus `GetStatus` payload.

<a id="streaming-wifi-latency"></a>
## Streaming: High latency, stutter, or slow mouse movement on 5GHz Wi-Fi

**Symptom:**
When connected over 5GHz Wi-Fi (e.g. from a Lenovo Tab or phone), mouse movements feel sluggish or delayed by hundreds of milliseconds, or the stream lags behind the host.

**Cause:**
1. Traditional video pipelines buffer multiple seconds of video to smooth playback, introducing noticeable display latency.
2. Infrequent keyframes (GOP) force clients to wait for the next keyframe if any network packet drops over Wi-Fi.
3. Mouse input batching was queued at long intervals.

**Fix:**
- Orbiscreen v0.31.3 tunes the pipeline for low latency:
  - **6-frame GOP:** Hardware encoders emit a keyframe every 100ms, enabling recovery from packet loss without added buffering.
  - **40-120ms load control:** ExoPlayer buffers are tuned down to 40ms minimum and 120ms maximum.
  - **8ms input loop:** Mouse movement batching interval is reduced to 8ms for 120Hz-class responsiveness.
- Ensure your Wi-Fi router uses 5GHz with an 80MHz channel width and low channel congestion.

---

<a id="stream-disconnect-retry"></a>
## Streaming: Stream error causes infinite reconnect flicker instead of detecting disconnect

**Symptom:**
When the Linux daemon stops or the network drops, the Android app repeatedly flickers and attempts to reconnect indefinitely instead of showing a clean disconnected state.

**Cause:**
Older versions lacked explicit lifecycle states for server disconnection and retried without bounds.

**Fix:**
- In v0.31.3, `PlayerHolder` introduces `StreamEvent.Disconnected`.
- On network errors, an immediate 500ms `/health` probe checks if the daemon is alive.
- Reconnection attempts are capped at 3 retries. If the host is unreachable, the app cleanly shows the disconnected card with manual retry options.

---

<a id="cursor-clamping"></a>
## Multi-Monitor / X11: Mouse cursor escapes virtual display to other physical screens

**Symptom:**
When moving the mouse or stylus on the tablet, the cursor jumps outside the virtual display area onto your physical laptop/desktop monitors.

**Cause:**
XTEST coordinate injection without output geometry boundaries spans the entire X11 root desktop dimensions.

**Fix:**
- In v0.31.3, Orbiscreen queries XRandR output geometry and clamps cursor and stylus coordinates strictly within the virtual output rectangle (`InputProp::DIRECT`).
- The cursor is strictly confined to the tablet's virtual display screen.

---

<a id="wrong-screen"></a>
## Client shows the wrong screen (primary desktop instead of virtual display)

**Symptom:**
The Android/web client connects and displays video, but it mirrors the host's main desktop instead of a clean second monitor. Dragging windows "ac" onto a second screen does nothing.

**Cause:**
The `evdi` kernel module is not loaded, so Orbiscreen falls back to primary-desktop capture (Wayland portal or X11 root window). This degraded mode is intentional: `GetStatus.capture_backend` reports `wayland-portal-fallback` or `x11-portal-fallback` instead of `evdi`, and the daemon logs a `EVDI kernel module missing/inactive ... Falling back` warning at start.

**Fix:**
1. Install and load `evdi` (DKMS) - see [Runtime: `orbiscreen start` fails](#runtime-kde-plasma-virtual-display-without-evdi-or-root), then:
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
## Web client loads but shows no picture

**Symptom:**
`https://<host>:8790/client/` loads, the overlay stays on "Connecting", or it reports "Unsupported browser" and asks for Chrome, Brave, or Edge.

**Cause:**
The web client opens WebTransport and decodes Annex-B with WebCodecs `VideoDecoder` onto a canvas. HTTP `/` and `/client/` on the signaling port redirect there. Accept the self-signed certificate once; the daemon reuses `$XDG_CONFIG_HOME/orbiscreen/wt-cert.pem` (and `wt-key.pem`) across restarts. Browsers without `VideoDecoder` (Firefox Mobile is one) never decode the stream. There is no MSE or WebRTC path.

**Fix:**
1. Open the page in Chrome, Brave, Edge, or another Chromium browser. Firefox Mobile does not implement WebCodecs `VideoDecoder`.
2. Confirm the tab was served over HTTPS on the WebTransport port (`signaling_port + 2`, usually 8790) and that you accepted the certificate.
3. Check DevTools console/network: a 401 on `/au` or a failed Hello means the token flow failed - see [401 Unauthorized](#401-unauthorized-from-stream-input-or-apicontrol-token).

---

<a id="no-encoder"></a>
## No encoder available - stream starts but errors out (x264 missing)

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
## 401 Unauthorized from `/stream`, `/input` or `/api/control` (token)

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
   cat ~/.config/orbiscreen/token
   ```
3. Android clients receive the token automatically via mDNS discovery (`token=...` TXT record). If adding a host manually, enter the token in the host connection settings.
4. Pass the token via Authorization header or query parameter in custom scripts:
   ```bash
   curl -H "Authorization: Bearer $TOKEN" http://host:8788/stream --output - | head -c 1000
   # or: curl "http://host:8788/stream?token=***"
   ```

---

<a id="dbus-missing"></a>
## Daemon not found on D-Bus

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
## Daemon: 100% CPU usage or freeze

**Cause:**
Capture loop running without yielding or unbounded queue backlog.

**Fix:**
Update to the latest release (v0.31.3 or newer).

---

<a id="still-stuck"></a>
## Still Stuck?

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
