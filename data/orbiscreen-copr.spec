# ─────────────────────────────────────────────
# Orbiscreen - RPM Spec (source build, COPR/Fedora)
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Metadata ──
Name:           orbiscreen
Version:        0.18.1
Release:        1%{?dist}
Summary:        Turn any Android tablet or phone into a low-latency second monitor for Linux

License:        GPL-3.0-or-later
URL:            https://github.com/shadow-x78/orbiscreen
Source0:        %{url}/archive/v%{version}/orbiscreen-%{version}.tar.gz
# Vendored crate sources so %build runs fully offline inside mock (no network).
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
Recommends:     android-tools

%description
Orbiscreen turns Android tablets and phones into high-performance (~25ms)
extended monitors for Linux desktops on Wayland and X11. Features native
graphic tablet digitizer with 4095 levels of stylus pressure and tilt for
Krita/GIMP, auto-rotating display orientation, rootless KWin/wlroots virtual
displays, hardware encoding (NVENC/VAAPI), reverse multi-touch and mouse
control, and USB ADB tunneling.

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

for f in index.html style.css app.js; do
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
%{_datadir}/orbiscreen/client/vendor/mpegts.js
%{_datadir}/orbiscreen/install-evdi-module.sh

%changelog
* Wed Sep 03 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.18.1-1
- Release 0.18.1: Pure CLI daemon refactoring, removal of desktop launcher, and package description synchronization.

* Wed Sep 03 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.18.0-1
- Release 0.18.0: Stylus pressure/tilt digitizer, auto-orientation, SEO-optimized docs and banner.
* Wed Sep 02 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.17.4-1
- Release 0.17.4: Zero-latency NVENC, 60 FPS damage pump, Android UI overhaul, and clean packaging.

* Tue Sep 01 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.16.0-1
- Source-build spec for COPR/Fedora: builds from the release tarball with cargo,
  packages the daemon, web client, and systemd user unit. USB transport completed in this release.
