#!/usr/bin/env bash
# Orbiscreen - Fedora/RHEL (.rpm) Package Builder
# https://github.com/shadow-x78/orbiscreen
set -euo pipefail

cd "$(dirname "$0")/.."

VERSION="${1:-$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')}"
ARCH="x86_64"
RPM_NAME="orbiscreen-${VERSION}-1.${ARCH}.rpm"
BUILD_ROOT="target/rpm-staging"

echo "[Orbiscreen] Building RPM package for Orbiscreen v${VERSION} (${ARCH})..."

rm -rf "${BUILD_ROOT}"
mkdir -p "${BUILD_ROOT}/usr/bin"
mkdir -p target/rpmbuild/BUILD
mkdir -p target/rpmbuild/RPMS
mkdir -p target/rpmbuild/SOURCES
mkdir -p target/rpmbuild/SPECS
mkdir -p target/rpmbuild/SRPMS

if [ ! -f target/release/orbiscreen ]; then
    echo "[Orbiscreen] Building release binaries for RPM..."
    cargo build --release --workspace
fi

cp -f target/release/orbiscreen "${BUILD_ROOT}/usr/bin/"

if command -v rpmbuild >/dev/null 2>&1; then
    rpmbuild -bb \
        --buildroot "$(pwd)/${BUILD_ROOT}" \
        --define "_topdir $(pwd)/target/rpmbuild" \
        --define "_projectroot $(pwd)" \
        --define "_version ${VERSION}" \
        data/orbiscreen.spec
    cp -f target/rpmbuild/RPMS/"${ARCH}"/orbiscreen-"${VERSION}"-1.*."${ARCH}".rpm "${RPM_NAME}" 2>/dev/null || cp -f target/rpmbuild/RPMS/"${ARCH}"/orbiscreen*.rpm "${RPM_NAME}"
    echo "[Orbiscreen] RPM package built successfully: ${RPM_NAME}"
else
    echo "[Orbiscreen] rpmbuild not found; staging files ready in ${BUILD_ROOT}"
fi
