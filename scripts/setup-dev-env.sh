#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Development Environment Setup
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Privilege Detection ──
set -euo pipefail

SUDO=""
if [[ $EUID -ne 0 ]]; then
    SUDO="sudo"
fi

# ── Distribution Detection ──
detect_distro() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        echo "${ID:-unknown}"
    else
        echo "unknown"
    fi
}

# ── Package Installation Routines ──
install_fedora() {
    $SUDO dnf install -y \
        rust cargo \
        pkg-config \
        gcc make \
        gstreamer1-devel gstreamer1-plugins-base-devel \
        gstreamer1-plugins-good gstreamer1-plugins-bad-free \
        libxkbcommon-devel libevdev-devel \
        wayland-devel libwayland-client \
        libX11-devel libxcb-devel libXrandr-devel \
        gtk4-devel libadwaita-devel \
        xorg-x11-server-utils
    echo "[Orbiscreen] note: evdi is not packaged for Fedora; build it with scripts/install-evdi-module.sh"
}

install_debian_like() {
    $SUDO apt update
    $SUDO apt install -y \
        rustc cargo \
        pkg-config build-essential \
        libevdi-dev \
        libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
        gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
        libevdev-dev libwayland-dev libxkbcommon-dev \
        libx11-dev libxcb1-dev libxrandr-dev \
        libgtk-4-dev libadwaita-1-dev \
        x11-xserver-utils
}

install_arch() {
    $SUDO pacman -S --needed \
        rust base-devel pkg-config \
        gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad \
        libevdev wayland libxkbcommon \
        libx11 libxcb libxrandr \
        gtk4 libadwaita \
        xorg-xrandr
    echo "[Orbiscreen] note: evdi is only available from the AUR (package: evdi); install it with an AUR helper"
}

# ── Entry Point ──
main() {
    local distro
    distro="$(detect_distro)"
    echo "[Orbiscreen] Detected distro family: $distro"
    case "$distro" in
        fedora|nobara|rhel|centos|rocky|almalinux) install_fedora ;;
        ubuntu|pop|pop_os|debian|linuxmint|elementary|zorin) install_debian_like ;;
        arch|cachyos|manjaro|endeavouros) install_arch ;;
        *)
            echo "Unknown distro '$distro'. Please install the equivalent packages manually:"
            echo "  - rust + cargo"
            echo "  - pkg-config, gcc, make"
            echo "  - libevdi (userspace) and the evdi kernel module"
            echo "  - gstreamer-1.0 + plugins-base/good/bad development packages"
            echo "  - libevdev development headers"
            echo "  - libwayland + libxkbcommon development headers"
            echo "  - xrandr"
            exit 1
            ;;
    esac
    echo "[Orbiscreen] Done."
}

main "$@"
