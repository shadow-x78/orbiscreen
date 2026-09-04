# ─────────────────────────────────────────────
# Orbiscreen - RPM Spec
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Macros ──
%define _builddir %{_topdir}/../..

# ── Package Metadata ──
Name:           orbiscreen
Version:        %{_version}
Release:        1%{?dist}
Summary:        Turn Android devices into high-performance secondary monitors for Linux

License:        GPL-3.0-or-later
URL:            https://github.com/shadow-x78/orbiscreen

Requires:       gstreamer1 gstreamer1-plugins-base gstreamer1-plugins-good gstreamer1-plugins-bad-free libevdev libxkbcommon
Recommends:     gstreamer1-plugins-ugly-free

%description
Orbiscreen turns Android tablets and phones into high-performance
extended monitors for Linux desktops (Wayland and X11). Features include
virtual display backends (KWin, wlroots, EVDI), low-latency hardware
encoding (NVENC, VAAPI), stylus digitizer support with pressure and tilt,
touch and mouse control, and Wi-Fi or USB tunneling.

%prep

%build

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/lib/systemd/user
mkdir -p %{buildroot}/usr/share/orbiscreen/client

install -m 0755 %{_projectroot}/target/release/orbiscreen %{buildroot}/usr/bin/orbiscreen
install -m 0755 %{_projectroot}/scripts/install-evdi-module.sh %{buildroot}/usr/share/orbiscreen/install-evdi-module.sh

install -m 0644 %{_projectroot}/clients/web/index.html %{buildroot}/usr/share/orbiscreen/client/index.html
install -m 0644 %{_projectroot}/clients/web/style.css %{buildroot}/usr/share/orbiscreen/client/style.css
install -m 0644 %{_projectroot}/clients/web/app.js %{buildroot}/usr/share/orbiscreen/client/app.js
install -m 0644 %{_projectroot}/clients/web/favicon.svg %{buildroot}/usr/share/orbiscreen/client/favicon.svg
install -m 0644 %{_projectroot}/clients/web/favicon.png %{buildroot}/usr/share/orbiscreen/client/favicon.png
install -m 0644 %{_projectroot}/clients/web/apple-touch-icon.png %{buildroot}/usr/share/orbiscreen/client/apple-touch-icon.png
mkdir -p %{buildroot}/usr/share/orbiscreen/client/vendor
install -m 0644 %{_projectroot}/clients/web/vendor/mpegts.js %{buildroot}/usr/share/orbiscreen/client/vendor/mpegts.js

cat << 'EOF' > %{buildroot}/usr/lib/systemd/user/orbiscreen.service
[Unit]
Description=Orbiscreen Virtual Secondary Display Service
Documentation=https://github.com/shadow-x78/orbiscreen
After=graphical-session.target

[Service]
Type=exec
ExecStart=/usr/bin/orbiscreen start
NoNewPrivileges=true
Restart=on-failure
RestartSec=3s

[Install]
WantedBy=graphical-session.target
EOF

# ── Uninstall Script ──
%preun
if [ $1 -eq 0 ]; then
    for u in $(users); do
        su -s /bin/sh -c "systemctl --user stop orbiscreen || true" "$u" || true
    done
fi

%postun
if [ $1 -eq 0 ]; then
    echo "Orbiscreen has been removed."
fi



# ── Packaged Files ──
%files
/usr/bin/orbiscreen
/usr/lib/systemd/user/orbiscreen.service
/usr/share/orbiscreen/client/index.html
/usr/share/orbiscreen/client/style.css
/usr/share/orbiscreen/client/app.js
/usr/share/orbiscreen/client/favicon.svg
/usr/share/orbiscreen/client/favicon.png
/usr/share/orbiscreen/client/apple-touch-icon.png
/usr/share/orbiscreen/client/vendor/mpegts.js
/usr/share/orbiscreen/install-evdi-module.sh

%changelog
* Fri Jul 24 2026 shadow-x78 <https://github.com/shadow-x78/orbiscreen> - %{_version}-1
- Release Orbiscreen %{_version} RPM package
