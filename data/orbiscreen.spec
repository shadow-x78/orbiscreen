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
Summary:        Turn any Android tablet or phone into a low-latency second monitor for Linux

License:        GPL-3.0-or-later
URL:            https://github.com/shadow-x78/orbiscreen

Requires:       gstreamer1 gstreamer1-plugins-base gstreamer1-plugins-good gstreamer1-plugins-bad-free libevdev libxkbcommon
Recommends:     gstreamer1-plugins-ugly-free

%description
Orbiscreen turns Android tablets and phones into high-performance (~25ms)
extended monitors for Linux desktops on Wayland and X11. Features native
graphic tablet digitizer with 4095 levels of stylus pressure and tilt for
Krita/GIMP, auto-rotating display orientation, rootless KWin/wlroots virtual
displays, hardware encoding (NVENC/VAAPI), reverse multi-touch and mouse
control, USB ADB tunneling, and zero-CLI desktop application integration.

%prep

%build

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/icons/hicolor/scalable/apps
mkdir -p %{buildroot}/usr/lib/systemd/user
mkdir -p %{buildroot}/usr/share/orbiscreen/client

install -m 0755 %{_projectroot}/target/release/orbiscreen %{buildroot}/usr/bin/orbiscreen
install -m 0755 %{_projectroot}/scripts/install-evdi-module.sh %{buildroot}/usr/share/orbiscreen/install-evdi-module.sh
install -m 0644 %{_projectroot}/data/orbiscreen.desktop %{buildroot}/usr/share/applications/orbiscreen.desktop || true
install -m 0644 %{_projectroot}/data/orbiscreen.svg %{buildroot}/usr/share/icons/hicolor/scalable/apps/orbiscreen.svg || true

install -m 0644 %{_projectroot}/clients/web/index.html %{buildroot}/usr/share/orbiscreen/client/index.html
install -m 0644 %{_projectroot}/clients/web/style.css %{buildroot}/usr/share/orbiscreen/client/style.css
install -m 0644 %{_projectroot}/clients/web/app.js %{buildroot}/usr/share/orbiscreen/client/app.js
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
/usr/share/applications/orbiscreen.desktop
/usr/share/icons/hicolor/scalable/apps/orbiscreen.svg
/usr/lib/systemd/user/orbiscreen.service
/usr/share/orbiscreen/client/index.html
/usr/share/orbiscreen/client/style.css
/usr/share/orbiscreen/client/app.js
/usr/share/orbiscreen/client/vendor/mpegts.js
/usr/share/orbiscreen/install-evdi-module.sh

%changelog
* Fri Jul 24 2026 shadow-x78 <https://github.com/shadow-x78/orbiscreen> - %{_version}-1
- Release Orbiscreen %{_version} RPM package
