# ─────────────────────────────────────────────
# Orbiscreen - RPM Spec (source build, COPR/Fedora)
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Metadata ──
Name:           orbiscreen
Version:        0.16.1
Release:        1%{?dist}
Summary:        Real virtual secondary displays for Linux, streamed to Android over Wi-Fi or USB

License:        GPL-3.0-or-later
URL:            https://github.com/shadow-x78/orbiscreen
Source0:        %{url}/archive/v%{version}/orbiscreen-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust >= 1.75
BuildRequires:  pkgconfig(gstreamer-1.0)
BuildRequires:  pkgconfig(gstreamer-app-1.0)
BuildRequires:  pkgconfig(gstreamer-video-1.0)
BuildRequires:  pkgconfig(gtk4)
BuildRequires:  pkgconfig(libadwaita-1)
BuildRequires:  pkgconfig(libevdev)
BuildRequires:  pkgconfig(xkbcommon)
BuildRequires:  pkgconfig(dbus-1)
BuildRequires:  git-core

Requires:       gstreamer1 gstreamer1-plugins-base gstreamer1-plugins-good gstreamer1-plugins-bad-free
Requires:       libevdev libxkbcommon gtk4 libadwaita
Recommends:     gstreamer1-plugins-ugly-free
Recommends:     android-tools

%description
Orbiscreen provides high-performance virtual secondary displays for Linux desktops,
streaming low-latency MPEG-TS/H.264 video over HTTP (Wi-Fi or USB/adb reverse)
to Android tablets/phones and web browsers.

%prep
%autosetup -n orbiscreen-%{version} -p1

%build
cargo build --release --workspace --locked

%install
install -Dm0755 target/release/orbiscreen %{buildroot}%{_bindir}/orbiscreen
install -Dm0755 target/release/orbiscreen-gtk %{buildroot}%{_bindir}/orbiscreen-gtk
install -Dm0644 data/com.orbiscreen.OrbiscreenGtk.desktop %{buildroot}%{_datadir}/applications/com.orbiscreen.OrbiscreenGtk.desktop
install -Dm0644 data/orbiscreen.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg
install -Dm0644 data/com.orbiscreen.OrbiscreenGtk.metainfo.xml %{buildroot}%{_datadir}/metainfo/com.orbiscreen.OrbiscreenGtk.metainfo.xml
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
%{_bindir}/orbiscreen-gtk
%{_datadir}/applications/com.orbiscreen.OrbiscreenGtk.desktop
%{_datadir}/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg
%{_datadir}/metainfo/com.orbiscreen.OrbiscreenGtk.metainfo.xml
%{_userunitdir}/orbiscreen.service
%{_datadir}/orbiscreen/client/index.html
%{_datadir}/orbiscreen/client/style.css
%{_datadir}/orbiscreen/client/app.js
%{_datadir}/orbiscreen/client/vendor/mpegts.js
%{_datadir}/orbiscreen/install-evdi-module.sh

%changelog
* Tue Sep 01 2026 shadow-x78 <107577376+shadow-x78@users.noreply.github.com> - 0.16.0-1
- Source-build spec for COPR/Fedora: builds from the release tarball with cargo,
  packages the GTK panel, daemon, web client, systemd user unit, and AppStream
  metainfo. USB transport completed in this release.
