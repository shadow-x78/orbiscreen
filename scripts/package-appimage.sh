#!/usr/bin/env bash
# Orbiscreen - AppImage Package Builder
# Delegates to the canonical builder in packaging/appimage/, which bundles
# the daemon, web client (incl. vendored mpegts.js) and the GStreamer
# runtime so the AppImage does not depend on the host's GStreamer install.
# https://github.com/shadow-x78/orbiscreen

set -euo pipefail

cd "$(dirname "$0")/.."

exec packaging/appimage/build-appimage.sh "$@"
