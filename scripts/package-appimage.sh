#!/usr/bin/env bash
# Orbiscreen - AppImage Package Builder
# https://github.com/shadow-x78/orbiscreen
set -euo pipefail

cd "$(dirname "$0")/.."

exec packaging/appimage/build-appimage.sh "$@"
