#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Packit SRPM Archive Generator
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Environment & Versions ──
set -euo pipefail

VERSION=$(grep -m1 -oP '(?<=^version = ")[^"]+' Cargo.toml)
ARCHIVE="orbiscreen-${VERSION}.tar.gz"
VENDOR_ARCHIVE="orbiscreen-vendor-${VERSION}.tar.zst"

# ── Vendor Rust Crates (Source1) ──
cargo vendor vendor >/dev/null 2>&1
tar --use-compress-program="zstd -19 -T0" -cf "${VENDOR_ARCHIVE}" vendor
rm -rf vendor

# ── Stage Vendor Archive ──
mkdir -p data
cp -f "${VENDOR_ARCHIVE}" data/

# ── Main Source Archive (Source0) ──
git archive --format=tar.gz --prefix="orbiscreen-${VERSION}/" -o "${ARCHIVE}" HEAD

# ── Emit Result ──
echo "${ARCHIVE}"
