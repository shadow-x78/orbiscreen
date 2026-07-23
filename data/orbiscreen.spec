%define _builddir %{_topdir}/../..

Name:           orbiscreen
Version:        %{_version}
Release:        1%{?dist}
Summary:        Real virtual secondary displays for Linux, streamed to Android over Wi-Fi or USB

License:        GPL-3.0-or-later
URL:            https://github.com/shadow-x78/orbiscreen

Requires:       gtk4 libadwaita gstreamer1 gstreamer1-plugins-base gstreamer1-plugins-good libevdev libxkbcommon

%description
Orbiscreen provides high-performance virtual secondary displays for Linux desktops,
streaming low-latency video to Android tablets/phones over WebRTC and Wi-Fi/USB.

%prep

%build

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/icons/hicolor/scalable/apps
mkdir -p %{buildroot}/usr/lib/systemd/user

install -m 0755 %{_projectroot}/target/release/orbiscreen %{buildroot}/usr/bin/orbiscreen
if [ -f %{_projectroot}/target/release/orbiscreen-daemon ]; then
    install -m 0755 %{_projectroot}/target/release/orbiscreen-daemon %{buildroot}/usr/bin/orbiscreen-daemon
fi
if [ -f %{_projectroot}/target/release/orbiscreen-gtk ]; then
    install -m 0755 %{_projectroot}/target/release/orbiscreen-gtk %{buildroot}/usr/bin/orbiscreen-gtk
fi

install -m 0644 %{_projectroot}/data/com.orbiscreen.OrbiscreenGtk.desktop %{buildroot}/usr/share/applications/com.orbiscreen.OrbiscreenGtk.desktop || true
install -m 0644 %{_projectroot}/data/orbiscreen.svg %{buildroot}/usr/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg || true

cat << 'EOF' > %{buildroot}/usr/lib/systemd/user/orbiscreen.service
[Unit]
Description=Orbiscreen Virtual Secondary Display Service
Documentation=https://github.com/shadow-x78/orbiscreen
After=graphical-session.target

[Service]
Type=exec
ExecStart=/usr/bin/orbiscreen
Restart=on-failure
RestartSec=3s

[Install]
WantedBy=graphical-session.target
EOF

%files
/usr/bin/orbiscreen
/usr/share/applications/com.orbiscreen.OrbiscreenGtk.desktop
/usr/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg
/usr/lib/systemd/user/orbiscreen.service

%changelog
* Fri Jul 24 2026 shadow-x78 <https://github.com/shadow-x78/orbiscreen> - %{_version}-1
- Release Orbiscreen %{_version} RPM package
