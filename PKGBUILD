# ─────────────────────────────────────────────
# Orbiscreen - AUR PKGBUILD
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# Maintainer: shadow-x78 <107577376+shadow-x78@users.noreply.github.com>

pkgname=orbiscreen
pkgver=0.28.0
pkgrel=1
pkgdesc="Turn Android devices into high-performance secondary monitors for Linux (Wayland & X11)"
arch=('x86_64')
url="https://github.com/shadow-x78/orbiscreen"
license=('GPL-3.0-or-later')
depends=(
    'gstreamer'
    'gst-plugins-base'
    'gst-plugins-good'
    'gst-plugins-bad'
    'libevdev'
    'libxkbcommon'
    'hicolor-icon-theme'
)
makedepends=('cargo' 'git' 'pkgconf' 'python' 'gtk3' 'webkit2gtk-4.1')
optdepends=(
    'android-tools: USB transport via adb reverse'
    'evdi-dkms: kernel-level virtual display on X11/GNOME (not needed on KDE Plasma Wayland or wlroots)'
)
provides=('orbiscreen')
conflicts=('orbiscreen-bin')
source=("${pkgname}-${pkgver}.tar.gz::${url}/archive/v${pkgver}/${pkgname}-${pkgver}.tar.gz")
sha256sums=('SKIP')

options=('!lto')

build() {
    cd "${pkgname}-${pkgver}"
    cargo build --release --workspace --locked
}

check() {
    cd "${pkgname}-${pkgver}"
    cargo test --workspace --locked
}

package() {
    cd "${pkgname}-${pkgver}"

    install -Dm0755 target/release/orbiscreen "${pkgdir}/usr/bin/orbiscreen"
    if [ -f target/release/orbiscreen-gui ]; then
        install -Dm0755 target/release/orbiscreen-gui "${pkgdir}/usr/bin/orbiscreen-gui"
    fi
    install -Dm0644 data/orbiscreen.svg "${pkgdir}/usr/share/icons/hicolor/scalable/apps/orbiscreen.svg"
    ln -sf orbiscreen.svg "${pkgdir}/usr/share/icons/hicolor/scalable/apps/orbiscreen-gui.svg"
    for size in 16 24 32 48 64 128 256 512; do
        if [ -f "crates/orbiscreen-gui/icons/${size}x${size}.png" ]; then
            install -Dm0644 "crates/orbiscreen-gui/icons/${size}x${size}.png" "${pkgdir}/usr/share/icons/hicolor/${size}x${size}/apps/orbiscreen.png"
            ln -sf orbiscreen.png "${pkgdir}/usr/share/icons/hicolor/${size}x${size}/apps/orbiscreen-gui.png"
        fi
    done
    install -Dm0644 data/orbiscreen.desktop "${pkgdir}/usr/share/applications/orbiscreen.desktop"
    install -Dm0755 scripts/install-evdi-module.sh "${pkgdir}/usr/share/orbiscreen/install-evdi-module.sh"
    install -Dm0644 data/99-orbiscreen-usb.rules "${pkgdir}/usr/lib/udev/rules.d/99-orbiscreen-usb.rules"

    for f in index.html style.css app.js favicon.svg favicon.png apple-touch-icon.png; do
        install -Dm0644 "clients/web/${f}" "${pkgdir}/usr/share/orbiscreen/client/${f}"
    done
    install -Dm0644 clients/web/vendor/mpegts.js "${pkgdir}/usr/share/orbiscreen/client/vendor/mpegts.js"

    install -Dm0644 /dev/null "${pkgdir}/usr/lib/systemd/user/orbiscreen.service"
    cat > "${pkgdir}/usr/lib/systemd/user/orbiscreen.service" << 'EOF'
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
}
