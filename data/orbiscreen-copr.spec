# ─────────────────────────────────────────────
# Orbiscreen - RPM Spec (source build, COPR/Fedora)
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Metadata ──
Name:           orbiscreen
Version:        0.23.2
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
install -Dm0644 data/orbiscreen.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/orbiscreen.svg
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

%preun
if [ $1 -eq 0 ]; then
    for u in $(users); do
        su -s /bin/sh -c "systemctl --user stop orbiscreen || true" "$u" || true
    done
fi

%files
%{_bindir}/orbiscreen
%{_datadir}/icons/hicolor/scalable/apps/orbiscreen.svg
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
* Sun Sep 06 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.2-1
- Release 0.23.2: Fix video freezing, black screen, and distortion over USB; restore uncorrupted MPEG-TS stream delivery and stabilize ExoPlayer buffer thresholds.

* Sun Sep 06 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.1-1
- Release 0.23.1: Restore direct touch as a virtual multitouch device with evdev type-B multi-touch slots.

* Sun Sep 06 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.23.0-1
- Release 0.23.0: Ultra-low latency USB pipeline tuning, resilient interface claim, and graceful USB detach navigation.

* Sat Sep 06 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.22.9-1
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
