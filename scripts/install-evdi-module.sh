#!/usr/bin/env bash
# Orbiscreen - install-evdi-module (GPL-3.0-or-later)
# https://github.com/shadow-x78/orbiscreen

set -euo pipefail

SRC_DIR="${1:-$HOME/src/evdi-main}"
KO="$SRC_DIR/module/evdi.ko"

if [[ ! -f $KO ]]; then
    echo "[Orbiscreen] evdi.ko not found at $KO - building from source"
    if [[ ! -d $SRC_DIR ]]; then
        git clone --depth 1 https://github.com/DisplayLink/evdi.git "$SRC_DIR"
    fi
    make -C "$SRC_DIR/module" KVERSION="$(uname -r)"
fi

sudo mkdir -p "/lib/modules/$(uname -r)/kernel/drivers/gpu/drm/evdi"
sudo cp "$KO" "/lib/modules/$(uname -r)/kernel/drivers/gpu/drm/evdi/evdi.ko"
sudo depmod -a
sudo modprobe evdi
echo evdi | sudo tee /etc/modules-load.d/evdi.conf > /dev/null

if lsmod | grep -q '^evdi '; then
    echo "[Orbiscreen] evdi module loaded and persisted across reboots."
else
    echo "[Orbiscreen] WARNING: modprobe returned success but module is not listed."
    exit 2
fi
