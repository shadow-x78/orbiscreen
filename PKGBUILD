# ─────────────────────────────────────────────
# Orbiscreen - AUR PKGBUILD
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# Maintainer: shadow-x78 <107577376+shadow-x78@users.noreply.github.com>

pkgname=orbiscreen
pkgver=0.17.1
pkgrel=1
pkgdesc="Real virtual secondary displays for Linux, streamed to Android over Wi-Fi or USB"
arch=('x86_64')
url="https://github.com/shadow-x78/orbiscreen"
license=('GPL-3.0-or-later')
depends=(
    'gstreamer'
    'gst-plugins-base'
    'gst-plugins-good'
    'gst-plugins-bad'
    'gtk4'
    'libadwaita'
    'libevdev'
    'libxkbcommon'
    'hicolor-icon-theme'
)
makedepends=('cargo' 'git' 'pkgconf' 'python')
optdepends=(
    'android-tools: USB transport via adb reverse'
    'evdi-dkms: kernel-level virtual display on X11/GNOME (not needed on KDE Plasma Wayland or wlroots)'
)
provides=('orbiscreen')
conflicts=('orbiscreen-bin')
source=("${pkgname}-${pkgver}.tar.gz::${url}/archive/v${pkgver}/${pkgname}-${pkgver}.tar.gz")
# Checksum strategy: a release tag's archive checksum only exists after the
# release workflow finishes, so a hard-pinned sum here would always lag the
# tag. The integrity anchor is instead the tag itself: makepkg verifies the
# archive against this SKIP entry, and the AUR publish flow regenerates
# .SRCINFO right after `updpkgsums` on the maintainer's machine, where the
# pinned sum lands in the published PKGBUILD copy - not in this repo.
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
    install -Dm0755 target/release/orbiscreen-gtk "${pkgdir}/usr/bin/orbiscreen-gtk"
    install -Dm0644 data/com.orbiscreen.OrbiscreenGtk.desktop "${pkgdir}/usr/share/applications/com.orbiscreen.OrbiscreenGtk.desktop"
    install -Dm0644 data/orbiscreen.svg "${pkgdir}/usr/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg"
    install -Dm0644 data/com.orbiscreen.OrbiscreenGtk.metainfo.xml "${pkgdir}/usr/share/metainfo/com.orbiscreen.OrbiscreenGtk.metainfo.xml"
    install -Dm0755 scripts/install-evdi-module.sh "${pkgdir}/usr/share/orbiscreen/install-evdi-module.sh"

    for f in index.html style.css app.js; do
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
