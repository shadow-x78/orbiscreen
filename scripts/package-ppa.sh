#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Launchpad PPA Source Package Builder & Uploader
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────
set -euo pipefail

cd "$(dirname "$0")/.."

VERSION="${1:-$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')}"
SERIES="${2:-noble}"
KEY_ID="${3:-6FE2CA0A10CB1084FA8265A0C1B40F190D010EEF}"
REVISION="3"
PKG_VERSION="${VERSION}-1~ubuntu24.04.${REVISION}"
RFC_DATE="$(date -R)"

echo "[Orbiscreen] Preparing Launchpad PPA source package for v${PKG_VERSION}..."

if [ ! -f bin/orbiscreen ] || [ ! -f bin/orbiscreen-gtk ]; then
    echo "::error:: bin/orbiscreen and bin/orbiscreen-gtk must exist before building source package." >&2
    exit 1
fi

cat << CHLOG > debian/changelog
orbiscreen (${PKG_VERSION}) ${SERIES}; urgency=medium

  * Release ${VERSION} for Ubuntu ${SERIES}.
  * Fast offline Launchpad PPA packaging with explicit dependencies.
  * Include machine-readable debian/copyright file.

 -- shadow-x78 <shadow.xox78@gmail.com>  ${RFC_DATE}
CHLOG

CONTAINER_CMD="podman"
command -v podman >/dev/null 2>&1 || CONTAINER_CMD="docker"

mkdir -p target/ppa
gpg --armor --export-secret-keys "${KEY_ID}" > target/ppa/key.asc

$CONTAINER_CMD run --rm \
    -v "$(pwd):/src:Z" \
    -v "$(pwd)/target/ppa/key.asc:/key.asc:ro,Z" \
    -w /src \
    docker.io/library/ubuntu:24.04 bash -c "
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -qq && apt-get install -y -qq debhelper devscripts dput dpkg-dev gnupg >/dev/null
        gpg --batch --import /key.asc
        debuild -d -S -sa -k\"${KEY_ID}\" -p'gpg --batch --pinentry-mode loopback'
        echo '[Orbiscreen] Uploading source package to Launchpad PPA...'
        dput ppa:shadow-x78/ppa ../orbiscreen_${PKG_VERSION}_source.changes
"

rm -rf target/ppa/key.asc
echo "[Orbiscreen] Source package uploaded successfully to Launchpad PPA!"
