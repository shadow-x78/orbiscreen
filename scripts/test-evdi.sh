#!/usr/bin/env bash
# Orbiscreen - test-evdi (GPL-3.0-or-later)
# https://github.com/shadow-x78/orbiscreen
set -euo pipefail
cd "$(dirname "$0")/.."
REPO_ROOT="$(pwd)"

red()   { printf '\033[31m%s\033[0m\n' "$*" >&2; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
blue()  { printf '\033[34m%s\033[0m\n' "$*"; }

need_root() {
    if [[ $EUID -ne 0 ]]; then
        red "This script must be run as root (it loads kernel modules and writes to /dev/dri)."
        echo "Re-run with: sudo $0"
        exit 1
    fi
}

detect_distro() {
    if [[ -f /etc/os-release ]]; then
        # shellcheck disable=SC1091
        . /etc/os-release
        echo "${ID:-unknown}"
    else
        echo "unknown"
    fi
}

ensure_kernel_module() {
    blue "[1/6] Checking evdi kernel module…"
    if lsmod | grep -q '^evdi '; then
        green "  evdi module already loaded."
        return 0
    fi

    if [[ -d "/lib/modules/$(uname -r)/extra" ]] \
        && find "/lib/modules/$(uname -r)" -name 'evdi.ko*' -quit 2>/dev/null | grep -q .; then
        blue "  found evdi.ko on disk - attempting modprobe…"
        modprobe evdi || {
            red "  modprobe evdi failed. Secure Boot may require a signed DKMS module."
            exit 1
        }
        green "  evdi module loaded."
        return 0
    fi

    local distro
    distro="$(detect_distro)"
    red "  evdi kernel module not found for running kernel $(uname -r)."
    echo
    echo "  Install it from the DisplayLink/evdi repository before re-running:"
    case "$distro" in
        fedora|nobara) echo "    sudo dnf install dkms gcc make kernel-devel-$(uname -r) displaylink" ;;
        ubuntu|pop)    echo "    sudo apt install dkms" ;;
        arch|cachyos)  echo "    sudo pacman -S evdi-dkms" ;;
        *)             echo "    See https://github.com/DisplayLink/evdi" ;;
    esac
    echo
    echo "  Then clone + build:"
    echo "    git clone https://github.com/DisplayLink/evdi"
    echo "    cd evdi && sudo make dkms-install"
    exit 1
}

ensure_libevdi() {
    blue "[2/6] Checking libevdi userspace library…"
    if ldconfig -p 2>/dev/null | grep -q 'libevdi\.so'; then
        green "  libevdi found via ldconfig."
        return 0
    fi
    if find /usr -name 'libevdi.so*' -print -quit 2>/dev/null | grep -q .; then
        green "  libevdi found under /usr."
        return 0
    fi
    red "  libevdi not found. Phase 1 will need it at runtime."
    echo "  Install libevdi (usually shipped with the evdi-dkms package or the DisplayLink driver)."
    return 0
}

verify_evdi_crates() {
    blue "[3/6] Verifying evdi crate metadata on crates.io…"
    if cargo search evdi --limit 1 2>/dev/null | grep -q '^evdi ='; then
        green "  crates.io lists evdi."
    else
        red "  could not query crates.io for evdi (offline?)"
        return 0
    fi
}

cargo_check_workspace() {
    blue "[4/6] Running cargo check --workspace…"
    cargo check --workspace --offline 2>/dev/null || cargo check --workspace
    green "  cargo check completed successfully."
}

probe_drm_connectors() {
    blue "[5/6] Probing /sys/class/drm for evdi connectors…"
    local found=0
    local entry name
    for entry in /sys/class/drm/*-evdi-*/status; do
        [[ -e "$entry" ]] || continue
        if grep -q '^connected' "$entry"; then
            name="$(basename "$(dirname "$entry")")"
            green "  evdi connector present and connected: $name"
            found=$((found + 1))
        fi
    done
    if [[ $found -eq 0 ]]; then
        red "  No active evdi DRM connector was detected."
        echo "  That usually means no client has opened /dev/dri/cardX yet."
        echo "  We will create one from Rust in Phase 1; for now this is expected."
        return 2
    fi
}

check_xrandr_if_x11() {
    blue "[6/6] xrandr check (X11 only)…"
    if [[ -n "${DISPLAY:-}" ]] && command -v xrandr >/dev/null 2>&1; then
        if xrandr --listmonitors | grep -qi evdi; then
            green "  xrandr reports an evdi monitor."
        else
            echo "  xrandr did not list an evdi monitor yet (expected before Phase 1)."
        fi
    else
        echo "  Skipped - no DISPLAY or xrandr not installed (Wayland session or headless)."
    fi
}

main() {
    blue "Orbiscreen Phase 0 - evdi feasibility probe"
    echo "  Working from: $REPO_ROOT"
    echo

    need_root
    ensure_kernel_module
    ensure_libevdi
    verify_evdi_crates
    cargo_check_workspace
    probe_drm_connectors
    check_xrandr_if_x11

    echo
    green "Phase 0 feasibility check finished."
}

main "$@"