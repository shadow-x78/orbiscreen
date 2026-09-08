# ─────────────────────────────────────────────
# Orbiscreen - RPM Spec
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Macros ──
%define _builddir %{_topdir}/../..
%global debug_package %{nil}
%global __brp_mangle_shebangs %{nil}

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
mkdir -p %{buildroot}/usr/share/icons/hicolor/scalable/apps
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/lib/udev/rules.d

install -m 0755 %{_projectroot}/target/release/orbiscreen %{buildroot}/usr/bin/orbiscreen
if [ -f %{_projectroot}/target/release/orbiscreen-gui ]; then
    install -m 0755 %{_projectroot}/target/release/orbiscreen-gui %{buildroot}/usr/bin/orbiscreen-gui
fi
install -m 0644 %{_projectroot}/data/orbiscreen.svg %{buildroot}/usr/share/icons/hicolor/scalable/apps/orbiscreen.svg
ln -sf orbiscreen.svg %{buildroot}/usr/share/icons/hicolor/scalable/apps/orbiscreen-gui.svg
for size in 16 24 32 48 64 128 256 512; do
    if [ -f "%{_projectroot}/crates/orbiscreen-gui/icons/${size}x${size}.png" ]; then
        install -Dm 0644 "%{_projectroot}/crates/orbiscreen-gui/icons/${size}x${size}.png" "%{buildroot}/usr/share/icons/hicolor/${size}x${size}/apps/orbiscreen.png"
        ln -sf orbiscreen.png "%{buildroot}/usr/share/icons/hicolor/${size}x${size}/apps/orbiscreen-gui.png"
    fi
done
install -m 0644 %{_projectroot}/data/orbiscreen.desktop %{buildroot}/usr/share/applications/orbiscreen.desktop
install -m 0644 %{_projectroot}/data/99-orbiscreen-usb.rules %{buildroot}/usr/lib/udev/rules.d/99-orbiscreen-usb.rules
install -m 0755 %{_projectroot}/scripts/install-evdi-module.sh %{buildroot}/usr/share/orbiscreen/install-evdi-module.sh

for f in index.html style.css app.js favicon.svg favicon.png apple-touch-icon.png; do
    install -m 0644 "%{_projectroot}/clients/web/$f" "%{buildroot}/usr/share/orbiscreen/client/$f"
done
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

# ── Post-Install Script ──
%post
/bin/touch --no-create /usr/share/icons/hicolor &>/dev/null || :
if [ -x /usr/bin/gtk-update-icon-cache ]; then
    /usr/bin/gtk-update-icon-cache /usr/share/icons/hicolor &>/dev/null || :
fi
if [ -x /usr/bin/update-desktop-database ]; then
    /usr/bin/update-desktop-database /usr/share/applications &>/dev/null || :
fi

# ── Uninstall Script ──
%preun
if [ $1 -eq 0 ]; then
    for u in $(users); do
        su -s /bin/sh -c "systemctl --user stop orbiscreen || true" "$u" || true
    done
fi

%postun
if [ $1 -eq 0 ]; then
    /bin/touch --no-create /usr/share/icons/hicolor &>/dev/null || :
    if [ -x /usr/bin/gtk-update-icon-cache ]; then
        /usr/bin/gtk-update-icon-cache /usr/share/icons/hicolor &>/dev/null || :
    fi
    if [ -x /usr/bin/update-desktop-database ]; then
        /usr/bin/update-desktop-database /usr/share/applications &>/dev/null || :
    fi
    echo "Orbiscreen has been removed."
fi



# ── Packaged Files ──
%files
/usr/bin/orbiscreen
/usr/bin/orbiscreen-gui
/usr/share/icons/hicolor/*/apps/orbiscreen*.*
/usr/share/applications/orbiscreen.desktop
/usr/lib/udev/rules.d/99-orbiscreen-usb.rules
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
