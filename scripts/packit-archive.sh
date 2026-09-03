#!/usr/bin/env bash
# Orbiscreen - Packit SRPM Archive Generator (Source0 + Source1 vendor crates)
# https://github.com/shadow-x78/orbiscreen
set -euo pipefail

VERSION=$(grep -m1 -oP '(?<=^version = ")[^"]+' Cargo.toml)
ARCHIVE="orbiscreen-${VERSION}.tar.gz"
VENDOR_ARCHIVE="orbiscreen-vendor-${VERSION}.tar.zst"

# 1. Vendor Rust crates for offline mock build (Source1)
cargo vendor vendor >/dev/null 2>&1
tar --use-compress-program="zstd -19 -T0" -cf "${VENDOR_ARCHIVE}" vendor
rm -rf vendor

# Copy vendor archive to data/ so Packit/rpmbuild finds it next to the specfile
mkdir -p data
cp -f "${VENDOR_ARCHIVE}" data/

# 2. Create the main source archive (Source0)
git archive --format=tar.gz --prefix="orbiscreen-${VERSION}/" -o "${ARCHIVE}" HEAD

# Output the main archive name for Packit Source0
echo "${ARCHIVE}"
