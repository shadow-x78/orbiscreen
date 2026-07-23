#!/usr/bin/env bash
# Orbiscreen - Fedora/RHEL (.rpm) Package Builder
# https://github.com/shadow-x78/orbiscreen

set -euo pipefail

VERSION="${1:-0.6.0}"
ARCH="x86_64"
RPM_NAME="orbiscreen-${VERSION}-1.${ARCH}.rpm"
BUILD_ROOT="target/rpm-staging"

echo "==> Building RPM package for Orbiscreen v${VERSION} (${ARCH})..."

mkdir -p "${BUILD_ROOT}/usr/bin"
mkdir -p "${BUILD_ROOT}/usr/share/applications"
mkdir -p "${BUILD_ROOT}/usr/share/icons/hicolor/scalable/apps"
mkdir -p target/rpmbuild/BUILD
mkdir -p target/rpmbuild/RPMS
mkdir -p target/rpmbuild/SOURCES
mkdir -p target/rpmbuild/SPECS
mkdir -p target/rpmbuild/SRPMS

if [ ! -f target/release/orbiscreen ]; then
    echo "==> Building release binaries for RPM..."
    cargo build --release --workspace --exclude orbiscreen-gtk
fi

cp -f target/release/orbiscreen "${BUILD_ROOT}/usr/bin/"
cp -f target/release/orbiscreen-daemon "${BUILD_ROOT}/usr/bin/" || true
cp -f target/release/orbiscreen-gtk "${BUILD_ROOT}/usr/bin/" || true
cp -f data/com.orbiscreen.OrbiscreenGtk.desktop "${BUILD_ROOT}/usr/share/applications/" || true
cp -f data/orbiscreen.svg "${BUILD_ROOT}/usr/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg" || true

if command -v rpmbuild >/dev/null 2>&1; then
    rpmbuild -bb \
        --buildroot "$(pwd)/${BUILD_ROOT}" \
        --define "_topdir $(pwd)/target/rpmbuild" \
        --define "_projectroot $(pwd)" \
        --define "_version ${VERSION}" \
        data/orbiscreen.spec
    cp -f target/rpmbuild/RPMS/"${ARCH}"/orbiscreen-"${VERSION}"-1.*."${ARCH}".rpm "${RPM_NAME}" 2>/dev/null || cp -f target/rpmbuild/RPMS/"${ARCH}"/orbiscreen*.rpm "${RPM_NAME}"
    echo "==> RPM package built successfully: ${RPM_NAME}"
else
    echo "==> rpmbuild not found; staging files ready in ${BUILD_ROOT}"
fi
