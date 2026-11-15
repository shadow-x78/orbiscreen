#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Fedora/RHEL (.rpm) Package Builder
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Environment & Directory ──
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION="${1:-$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')}"
ARCH="x86_64"
RPM_NAME="orbiscreen-${VERSION}-1.${ARCH}.rpm"
BUILD_ROOT="target/rpm-staging"
RPM_DEFINES=()

echo "[Orbiscreen] Building RPM package for Orbiscreen v${VERSION} (${ARCH})..."

# ── Build Binaries ──
rm -rf "${BUILD_ROOT}"
mkdir -p "${BUILD_ROOT}/usr/bin"
mkdir -p target/rpmbuild/BUILD
mkdir -p target/rpmbuild/RPMS
mkdir -p target/rpmbuild/SOURCES
mkdir -p target/rpmbuild/SPECS
mkdir -p target/rpmbuild/SRPMS
mkdir -p target/tmp

if [ "${ORBISCREEN_FORCE_REBUILD:-0}" = "1" ] || [ ! -f target/release/orbiscreen ]; then
    echo "[Orbiscreen] Building release binaries for RPM..."
    cargo build --release --workspace --locked
fi
if command -v cmake >/dev/null 2>&1 && pkg-config --exists Qt6Widgets 2>/dev/null; then
    cmake -S gui -B build-gui -DCMAKE_BUILD_TYPE=Release
    cmake --build build-gui -j"$(nproc)"
fi

cp -f target/release/orbiscreen "${BUILD_ROOT}/usr/bin/"
if [ -f build-gui/orbiscreen-gui ]; then
    cp -f build-gui/orbiscreen-gui "${BUILD_ROOT}/usr/bin/"
    RPM_DEFINES+=(--define "_orbiscreen_gui 1")
fi

# ── Build RPM Package ──
if command -v rpmbuild >/dev/null 2>&1; then
    rpmbuild -bb "${RPM_DEFINES[@]}" \
        --buildroot "$(pwd)/${BUILD_ROOT}" \
        --define "_topdir $(pwd)/target/rpmbuild" \
        --define "_tmppath $(pwd)/target/tmp" \
        --define "_projectroot $(pwd)" \
        --define "_version ${VERSION}" \
        scripts/orbiscreen-local.spec
    cp -f target/rpmbuild/RPMS/"${ARCH}"/orbiscreen-"${VERSION}"-1.*."${ARCH}".rpm "${RPM_NAME}" 2>/dev/null || cp -f target/rpmbuild/RPMS/"${ARCH}"/orbiscreen*.rpm "${RPM_NAME}"
    echo "[Orbiscreen] RPM package built successfully: ${RPM_NAME}"
else
    echo "[Orbiscreen] rpmbuild not found; staging files ready in ${BUILD_ROOT}"
fi
