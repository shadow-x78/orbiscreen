#!/usr/bin/env bash
# Orbiscreen - build-deb (GPL-3.0-or-later)
# https://github.com/shadow-x78/orbiscreen
set -euo pipefail
cd "$(dirname "$0")/../.."
REPO_ROOT="$(pwd)"

VERSION="${VERSION:-0.1.0}"
PKGROOT="orbiscreen_${VERSION}_amd64"

rm -rf "$PKGROOT"
mkdir -p "$PKGROOT/DEBIAN" "$PKGROOT/usr/bin" "$PKGROOT/usr/share/orbiscreen/client"

echo "Building release binary..."
cargo build --release --bin orbiscreen

install -m755 target/release/orbiscreen "$PKGROOT/usr/bin/orbiscreen"
install -m644 clients/web/index.html "$PKGROOT/usr/share/orbiscreen/client/index.html"
install -m644 clients/web/style.css  "$PKGROOT/usr/share/orbiscreen/client/style.css"
install -m644 clients/web/app.js     "$PKGROOT/usr/share/orbiscreen/client/app.js"

cat > "$PKGROOT/DEBIAN/control" <<EOF
$(sed '/^Version: /d' packaging/debian/control)
Version: $VERSION
EOF

install -m755 /dev/stdin "$PKGROOT/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
if [ ! -e /dev/uinput ]; then
    echo "Orbiscreen: /dev/uinput is not available; install the evdi/uinput kernel modules."
fi
EOF

install -m755 /dev/stdin "$PKGROOT/DEBIAN/prerm" <<'EOF'
#!/bin/sh
set -e
EOF

mkdir -p dist
dpkg-deb --build "$PKGROOT" "dist/${PKGROOT}.deb"
echo "Built dist/${PKGROOT}.deb"