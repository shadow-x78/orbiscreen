# ─────────────────────────────────────────────
# Orbiscreen - RPM Spec (source build, COPR/Fedora)
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

%global debug_package %{nil}
%global __brp_mangle_shebangs %{nil}

# ── Metadata ──
Name:           orbiscreen
Version:        0.27.8
Release:        1%{?dist}
Summary:        Turn Android devices into high-performance secondary monitors for Linux

License:        GPL-3.0-or-later
URL:            https://github.com/shadow-x78/orbiscreen
Source0:        %{url}/archive/v%{version}/orbiscreen-%{version}.tar.gz
Source1:        orbiscreen-vendor-%{version}.tar.zst

BuildRequires:  cargo
BuildRequires:  rust >= 1.75
BuildRequires:  pkgconfig(gstreamer-1.0)
BuildRequires:  pkgconfig(gstreamer-app-1.0)
BuildRequires:  pkgconfig(gstreamer-video-1.0)
BuildRequires:  pkgconfig(libevdev)
BuildRequires:  pkgconfig(xkbcommon)
BuildRequires:  pkgconfig(dbus-1)
BuildRequires:  pkgconfig(gtk+-3.0)
BuildRequires:  pkgconfig(webkit2gtk-4.1)
BuildRequires:  pkgconfig(ayatana-appindicator3-0.1)
BuildRequires:  git-core

Requires:       gstreamer1 gstreamer1-plugins-base gstreamer1-plugins-good gstreamer1-plugins-bad-free
Requires:       libevdev libxkbcommon
Recommends:     gstreamer1-plugins-ugly-free

%description
Orbiscreen turns Android tablets and phones into high-performance
extended monitors for Linux desktops (Wayland and X11). Features include
virtual display backends (KWin, wlroots, EVDI), low-latency hardware
encoding (NVENC, VAAPI), stylus digitizer support with pressure and tilt,
touch and mouse control, and Wi-Fi or USB tunneling.

%prep
%autosetup -n orbiscreen-%{version} -p1 -a1
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

%build
cargo build --release --workspace --locked

%install
install -Dm0755 target/release/orbiscreen %{buildroot}%{_bindir}/orbiscreen
if [ -f target/release/orbiscreen-gui ]; then
    install -Dm0755 target/release/orbiscreen-gui %{buildroot}%{_bindir}/orbiscreen-gui
fi
install -Dm0644 data/orbiscreen.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/orbiscreen.svg
ln -sf orbiscreen.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/orbiscreen-gui.svg
for size in 16 24 32 48 64 128 256 512; do
    if [ -f "crates/orbiscreen-gui/icons/${size}x${size}.png" ]; then
        install -Dm0644 "crates/orbiscreen-gui/icons/${size}x${size}.png" "%{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps/orbiscreen.png"
        ln -sf orbiscreen.png "%{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps/orbiscreen-gui.png"
    fi
done
install -Dm0644 data/orbiscreen.desktop %{buildroot}%{_datadir}/applications/orbiscreen.desktop
install -Dm0755 scripts/install-evdi-module.sh %{buildroot}%{_datadir}/orbiscreen/install-evdi-module.sh
install -Dm0644 data/99-orbiscreen-usb.rules %{buildroot}%{_udevrulesdir}/99-orbiscreen-usb.rules

for f in index.html style.css app.js favicon.svg favicon.png apple-touch-icon.png; do
    install -Dm0644 "clients/web/$f" "%{buildroot}%{_datadir}/orbiscreen/client/$f"
done
install -Dm0644 clients/web/vendor/mpegts.js %{buildroot}%{_datadir}/orbiscreen/client/vendor/mpegts.js

install -Dm0644 /dev/null %{buildroot}%{_userunitdir}/orbiscreen.service
cat > %{buildroot}%{_userunitdir}/orbiscreen.service << 'EOF'
[Unit]
Description=Orbiscreen Virtual Secondary Display Service
Documentation=https://github.com/shadow-x78/orbiscreen
After=graphical-session.target

[Service]
Type=exec
ExecStart=%{_bindir}/orbiscreen start
NoNewPrivileges=true
Restart=on-failure
RestartSec=3s

[Install]
WantedBy=graphical-session.target
EOF

%check
cargo test --workspace --locked --offline || true

%post
/bin/touch --no-create %{_datadir}/icons/hicolor &>/dev/null || :
if [ -x %{_bindir}/gtk-update-icon-cache ]; then
    %{_bindir}/gtk-update-icon-cache %{_datadir}/icons/hicolor &>/dev/null || :
fi
if [ -x %{_bindir}/update-desktop-database ]; then
    %{_bindir}/update-desktop-database %{_datadir}/applications &>/dev/null || :
fi

%preun
if [ $1 -eq 0 ]; then
    for u in $(users); do
        su -s /bin/sh -c "systemctl --user stop orbiscreen || true" "$u" || true
    done
fi

%postun
if [ $1 -eq 0 ]; then
    /bin/touch --no-create %{_datadir}/icons/hicolor &>/dev/null || :
    if [ -x %{_bindir}/gtk-update-icon-cache ]; then
        %{_bindir}/gtk-update-icon-cache %{_datadir}/icons/hicolor &>/dev/null || :
    fi
    if [ -x %{_bindir}/update-desktop-database ]; then
        %{_bindir}/update-desktop-database %{_datadir}/applications &>/dev/null || :
    fi
fi

%files
%{_bindir}/orbiscreen
%{_bindir}/orbiscreen-gui
%{_datadir}/icons/hicolor/*/apps/orbiscreen*.*
%{_datadir}/applications/orbiscreen.desktop
%{_userunitdir}/orbiscreen.service
%{_datadir}/orbiscreen/client/index.html
%{_datadir}/orbiscreen/client/style.css
%{_datadir}/orbiscreen/client/app.js
%{_datadir}/orbiscreen/client/favicon.svg
%{_datadir}/orbiscreen/client/favicon.png
%{_datadir}/orbiscreen/client/apple-touch-icon.png
%{_datadir}/orbiscreen/client/vendor/mpegts.js
%{_datadir}/orbiscreen/install-evdi-module.sh
%{_udevrulesdir}/99-orbiscreen-usb.rules

%changelog
* Sat Sep 12 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.27.8-1
- Release 0.27.8: Fix secondary display damage pump matching to prevent Virtual-ORBISCREEN-2 from attaching to primary display (#77); eliminate USB AOA frame drops, video stutter, and keyframe stalls by increasing sync channel capacity and bounding pts calculation (#77); fix mouse delta speed and cursor bounds in Android PlayerSurface (#77); completely remove USB audio pipeline and preferences across daemon, transport, and Android app (#77).

* Fri Sep 11 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.27.7-1
- Release 0.27.7: Fix secondary display black screen by pacing independent 60fps damage ticks to Virtual-ORBISCREEN-2 and pushing initial keepalive IDR frame (#77); auto-position secondary screen to the right via kscreen-doctor (#77); eliminate mouse rubberbanding and cursor jitter by routing pointer events strictly as relative motion to mouse_keyboard and isolating stylus pen tools (#77).

* Fri Sep 11 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.27.6-1
- Release 0.27.6: Support concurrent secondary tablet virtual display on KDE Plasma with dynamic port allocation and uinput isolation (#77); expand AOA candidate detection to Allwinner and all MTP/ADB devices (#77); eliminate mouse cursor stutter by isolating relative mouse events from tablet stylus coordinate injection (#77); optimize Android ExoPlayer buffer pacing to 45-120ms with dynamic live playback speed to eliminate frame drops and audio underrun.

* Fri Sep 11 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.27.5-1
- Release 0.27.5: Support concurrent multi-device AOA bridges in supervisor to prevent device collisions (#77); fix Android USB accessory permission flow and fallback (#77); convert battery optimization to interactive switch preference with live status and system intent launcher (#77); enable app-wide keepScreenAwake; center USB audio warning and round touch ripple highlights.

* Fri Sep 11 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.27.4-1
- Release 0.27.4: Fix Android USB AOA connection and auto-connect by using RECEIVER_EXPORTED, adding onResume initialization, and adding replay buffer to autoConnectEvent (#77); fix keepScreenAwake by unwrapping Context to Activity and setting keepScreenOn on View; add battery optimization exemption settings with direct intent launcher (#77).

* Fri Sep 11 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.27.3-1
- Release 0.27.3: Fix Android client audio playback by configuring AudioAttributes with USAGE_MEDIA, enabling audio focus handling, increasing DefaultLoadControl buffer durations to prevent AudioTrack buffer starvation, bypassing low-latency filters for audio decoders, and enforcing 48 kHz stereo ADTS audio in host GStreamer pipeline (#77).

* Fri Sep 11 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.27.2-1
- Release 0.27.2: Fix video stream periodic freezing by removing false-positive seekToDefaultPosition loop in Android client (#77), add independent upstream-leaky queues for both video and audio before mpegtsmux to prevent frame stalls and audio dropouts (#77), fix virtual sink device description formatting to cleanly display "Orbiscreen Audio" with space in system sound settings.

* Fri Sep 11 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.27.1-1
- Release 0.27.1: Fix Trackpad cursor jump on finger lift by routing RelativeMove to relative mouse device instead of tablet on host (#77), track cursor position client-side for correct tap-to-click landing, add hard-drop seek when buffered lag exceeds 80 ms and EWMA clock smoothing for Wi-Fi stability, protect keyframes from stale drop, fix toolbar auto-close removed, add virtual audio sink "Orbiscreen Audio" in PipeWire/PulseAudio via pactl module-null-sink.

* Fri Sep 11 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.26.0-1
- Release 0.26.0: Fix video stream latency accumulation on Wi-Fi and USB/AOA with stale frame drop at 75 ms (#77), aggressively reduce all pipeline buffers to eliminate rubberbanding (#75), reduce GOP from 600 to 120 frames for fast IDR recovery, wire USB audio to server via audio=1 query parameter with BETA badge, replace Input Mode switch with Segmented Button in settings only (#76 referenced).

* Thu Sep 10 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.25.9-1
- Release 0.25.9: Fix USB AOA black screen and ADB endpoint collision (#76), eliminate rubberbanding on USB reconnect and clock drift (#75), remove auto set_resolution display flash, single-tap floating pill handle, in-session settings overhaul, Android M3 settings rows, full light/dark color contrast audit, and desktop GUI D-Bus timeout protection.

* Wed Sep 09 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.25.8-1
- Release 0.25.8: USB audio streaming to device speaker/headphones, auto-detection of native screen resolution and high refresh rate, USB latency pipeline optimization, auto-connect and disconnect behavior, full Arabic localization with NotoKufiArabic typography, draggable edge pill toolbar controls, and practical settings redesign.

* Wed Sep 09 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.25.7-1
- Release 0.25.7: Restore dual USB pipeline with resilient AOA USB permissions and automatic ADB reverse port forwarding, protect D-Bus CLI queries against zombie or suspended process hangs, and update GUI device status display.

* Wed Sep 09 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.25.6-1
- Release 0.25.6: Revert ExoPlayer threshold regression that caused worse rubberbanding in v0.25.5 (#75). Restore shouldDropOutputBuffer to 30ms, revert setMaxPlaybackSpeed to 1.0f, revert shouldDropBuffersToKeyframe to 100ms. Keep proactive IDR request on lag.

* Tue Sep 08 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.25.5-1
- Release 0.25.5: Fix 30-minute latency drift and rubberbanding on USB AOA, quartz clock drift micro-catchup, on-demand IDR keyframe recovery, and periodic recovery keyframes (#75).

* Tue Sep 08 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.25.4-1
- Release 0.25.4: Native Linux desktop GUI redesign (560x620), zero-inline-comments code audit, configuration comment standardization, and vector QR code fix.

* Tue Sep 08 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.25.3-1
- Release 0.25.3: Eliminate USB AOA buffer bloat, lock ExoPlayer playback rate, configure low-latency backpressure, and fix rubberbanding on Android devices (#75).

* Tue Sep 08 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.25.2-1
- Release 0.25.2: Linux Desktop GUI Control Center redesign, system tray icon integration, Wayland StartupWMClass alignment, automated NVIDIA explicit sync stability, and complete raster PNG icon packaging.

* Tue Sep 08 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.25.1-1
- Release 0.25.1: UDP PMTU measurement without truncated ACKs or fragments, immediate unsendable reject, and full datagram Android receive buffer (PR #74 by @sentinelt).

* Tue Sep 08 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.25.0-1
- Release 0.25.0: Linux Desktop GUI Control Center (orbiscreen-gui) with System Tray, streamlined English interface, and orbiscreen gui CLI integration.

* Mon Sep 07 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.24.1-1
- Release 0.24.1: UDP Annex-B video transport with per-client DPLPMTUD, KWin dynamic virtual display with Direct Touch binding, and send-to-assemble delay metrics.

* Mon Sep 07 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.24.0-1
- Release 0.24.0: Harmonize UI/UX and Catppuccin Mocha colors between Web and Android, remove Blank Display action, strengthen Web CSP, sanitize daemon input coordinates, and enforce clean comment standards.

* Mon Sep 07 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.9-1
- Release 0.23.9: Purge GitHub Deployments and gh-pages branch, streamline distribution via Launchpad PPA and Fedora COPR.

* Mon Sep 07 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.8-1
- Release 0.23.8: GitHub multi-target deployment environments, banner-aligned social preview, standard issue forms, and gh-pages APT repository sync.

* Mon Sep 07 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.7-1
- Release 0.23.7: Infinite GOP length, intra-refresh for x264enc, and on-demand IDR recovery via /api/control (PR #72 by @sentinelt).

* Mon Sep 07 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.6-1
- Release 0.23.6: Probe vah264enc hardware encoder, warn on silent software x264 fallback, and tune VA-API low latency (PR #71 by @sentinelt).

* Sun Sep 06 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.5-1
- Release 0.23.5: Shrink HTTP MPEG-TS transport queues, backpressure on full queue, 24ms Android live offset, and MediaCodec low-latency decoding (PR #70 by @sentinelt).

* Sun Sep 06 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.4-1
- Release 0.23.4: Confine trackpad to virtual screen, eliminate duplicated stylus mouse injection, and scale Android motion deltas.

* Sun Sep 06 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.3-1
- Release 0.23.3: Enable Gradle dependency caching in CI workflows to eliminate Cloudflare 403 Forbidden errors; enhance in-page Code of Conduct navigation anchors.

* Sun Sep 06 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.2-1
- Release 0.23.2: Fix video freezing, black screen, and distortion over USB; restore uncorrupted MPEG-TS stream delivery and stabilize ExoPlayer buffer thresholds.

* Sun Sep 06 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.1-1
- Release 0.23.1: Restore direct touch as a virtual multitouch device with evdev type-B multi-touch slots.

* Sun Sep 06 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.0-1
- Release 0.23.0: Ultra-low latency USB pipeline tuning, resilient interface claim, and graceful USB detach navigation.

* Sun Sep 06 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.22.9-1
- Release 0.22.9: Fix AOA USB packet truncation causing session token loss and auth=missing rejections; proper TCP shutdown on FRAME_FLAG_CLOSE; session token caching in Android client.

* Sat Sep 05 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.22.8-1
- Release 0.22.8: Comprehensive multi-vendor udev rules in doctor fix, resilient Android token acquisition, and zero-root USB Tethering guidance.

* Sat Sep 05 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.22.7-1
- Release 0.22.7: Purge ADB dependencies, add unprivileged AOA udev rules, real-time live doctor diagnostics, and Markdown update dialog.

* Sat Sep 05 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.22.6-1
- Release 0.22.6: Native direct USB streaming via Android Open Accessory (AOA), polished and unified tool translations across Web, Android, and CLI.

* Sat Sep 05 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.22.5-1
- Release 0.22.5: Implement native in-app Android updater with live progress bar, SHA-256 checksum verification, and PackageInstaller integration.

* Sat Sep 05 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.22.4-1
- Release 0.22.4: Fix pure pointer classification for libinput/KWin, USB tethering auto-discovery, and styled stop card.

* Sat Sep 05 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.22.3-1
- Release 0.22.3: Eliminate video stuttering with 1s GOP interval, pure absolute pointer mapping, zero cursor snapback, and stylus stability.

* Sat Sep 05 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.22.2-1
- Release 0.22.2: Fix video stuttering and buffer overrun, confine mouse to virtual screen, fix stylus touch and hover.

* Sat Sep 05 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.22.1-1
- Release 0.22.1: Ultra-low latency streaming optimizations, 3 separate uinput devices, direct touch output confinement, and stylus tablet tool resolution support.

* Fri Sep 04 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.22.0-1
- Release 0.22.0: Direct touch pointer confinement, zero-snapback mouse release, green screen MPEG-TS fix, and clean startup branding.

* Fri Sep 04 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.21.0-1
- Release 0.21.0: Rich developer version card, full bilingual i18n support, Arabic architecture diagram, and token security hardening.

* Fri Sep 04 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.20.0-1
- Release 0.20.0: XDG Desktop Portal virtual display API, stylus pressure/tilt/hover overhaul, drag-and-drop gestures, ChromeOS ADB support, token security isolation.

* Fri Sep 04 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.19.0-1
- Release 0.19.0: COSMIC desktop support, native Fedora CI RPM packaging, RTL documentation, zero-comment code standards.

* Thu Sep 03 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.18.3-1
- Release 0.18.3: Android 1:1 web client parity, direct keyboard typing, refined ASCII logo, and web favicons.

* Thu Sep 03 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.18.2-1
- Release 0.18.2: Redesigned CLI interface with brand theme, ASCII logo, status command, and clean code headers.

* Thu Sep 03 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.18.1-1
- Release 0.18.1: Pure CLI daemon refactoring, removal of desktop launcher, and package description synchronization.

* Thu Sep 03 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.18.0-1
- Release 0.18.0: Stylus pressure/tilt digitizer, auto-orientation, SEO-optimized docs and banner.
* Wed Sep 02 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.17.4-1
- Release 0.17.4: Zero-latency NVENC, 60 FPS damage pump, Android UI overhaul, and clean packaging.

* Tue Sep 01 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.16.0-1
- Source-build spec for COPR/Fedora: builds from the release tarball with cargo,
  packages the daemon, web client, and systemd user unit. USB transport completed in this release.
