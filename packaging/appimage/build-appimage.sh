#!/usr/bin/env bash
# Orbiscreen — AppImage build helper (GPL-3.0-or-later)
# https://github.com/shadow-x78/orbiscreen
set -euo pipefail
cd "$(dirname "$0")/../.."
REPO_ROOT="$(pwd)"

if ! command -v cargo >/dev/null 2>&1; then
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
fi

DIST="$REPO_ROOT/dist"
mkdir -p "$DIST"

echo "Building release binary..."
cargo build --release --bin orbiscreen

APP="$DIST/orbiscreen.AppDir"
rm -rf "$APP"
mkdir -p "$APP/usr/bin" "$APP/usr/share/orbiscreen/client" "$APP/usr/share/icons/hicolor/256x256/apps"

install -m755 target/release/orbiscreen "$APP/usr/bin/orbiscreen"
install -m644 clients/web/index.html "$APP/usr/share/orbiscreen/client/index.html"
install -m644 clients/web/style.css  "$APP/usr/share/orbiscreen/client/style.css"
install -m644 clients/web/app.js     "$APP/usr/share/orbiscreen/client/app.js"

cat > "$APP/orbiscreen.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=Orbiscreen
GenericName=Virtual Secondary Display
Comment=Stream a virtual display to an Android device
Exec=orbiscreen %u
Icon=orbiscreen
Terminal=true
Categories=Network;System;
StartupNotify=true
EOF

python3 - <<'PY' > "$APP/usr/share/icons/hicolor/256x256/apps/orbiscreen.png"
import base64
import struct
import sys
import zlib

WIDTH = HEIGHT = 256

def chunk(tag, data):
    crc = zlib.crc32(tag + data) & 0xFFFFFFFF
    return (struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc))

ihdr = struct.pack(">IIBBBBB", WIDTH, HEIGHT, 8, 2, 0, 0, 0)
idat_raw = bytearray()
for y in range(HEIGHT):
    idat_raw.append(0)
    for x in range(WIDTH):
        is_border = (x < 4 or x >= WIDTH - 4 or y < 4 or y >= HEIGHT - 4)
        r, g, b = (0xCC, 0x00, 0x00) if not is_border else (0x55, 0x00, 0x00)
        cx, cy = x - WIDTH // 2, y - HEIGHT // 2
        in_oval = (cx * cx) * 9 + (cy * cy) * 25 < 64 * 64 * 25
        if in_oval:
            r, g, b = 0xFF, 0xFF, 0xFF
        idat_raw.extend((r, g, b))
idat = zlib.compress(bytes(idat_raw), 9)
png = (
    b"\x89PNG\r\n\x1a\n"
    + chunk(b"IHDR", ihdr)
    + chunk(b"IDAT", idat)
    + chunk(b"IEND", b"")
)
sys.stdout.buffer.write(png)
PY
cp "$APP/usr/share/icons/hicolor/256x256/apps/orbiscreen.png" "$APP/orbiscreen.png"

cat > "$APP/AppRun" <<'EOF'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/orbiscreen" "$@"
EOF
chmod +x "$APP/AppRun"

if ! command -v appimagetool >/dev/null 2>&1; then
    echo "Installing appimagetool into /usr/local/bin..."
    sudo curl -sL \
        https://github.com/AppImageCommunity/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage \
        -o /usr/local/bin/appimagetool
    sudo chmod +x /usr/local/bin/appimagetool
fi

appimagetool "$APP" "$DIST/orbiscreen-x86_64.AppImage"
echo "Built $DIST/orbiscreen-x86_64.AppImage"