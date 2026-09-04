#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Run Android Debug App
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Environment & Directory ──
set -euo pipefail
cd "$(dirname "$0")/../clients/android"

# ── Check Prerequisites ──
if ! command -v adb >/dev/null 2>&1; then
    echo "[Error] adb command not found. Please install android-tools: sudo apt install android-tools-adb" >&2
    exit 1
fi

DEVICES=$(adb devices | grep -v "List of devices" | grep "device$" | awk '{print $1}')
if [ -z "$DEVICES" ]; then
    echo "[Error] No Android device connected with USB debugging authorized." >&2
    echo "Please connect your phone, enable Developer Options -> USB Debugging, and authorize the connection." >&2
    exit 1
fi

# ── Build & Install APK ──
echo "[Orbiscreen] Clearing historical device logs..."
adb logcat -c || true

echo "[Orbiscreen] Building debug APK..."
./gradlew assembleDebug

echo "[Orbiscreen] Installing APK onto device..."
APK_PATH="app/build/outputs/apk/debug/app-debug.apk"

if ! adb install -r "$APK_PATH" 2>/dev/null; then
    echo "[Notice] Reinstalling cleanly..."
    adb uninstall com.orbiscreen.android || true
    adb install "$APK_PATH"
fi

# ── Launch & Follow Logs ──
echo "[Orbiscreen] Launching Orbiscreen on device..."
adb shell am start -n com.orbiscreen.android/.MainActivity

echo "[Orbiscreen] Running! Streaming Orbiscreen logs (Press Ctrl+C to exit):"
adb logcat -v time "Orbi*:V" "AndroidRuntime:E" "*:S"
