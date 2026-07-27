# Installation Guide - Orbiscreen

> Latest release: **v0.10.2** (Material 3 Android client, fixed black screen, live discovery).

## 🚀 Quick Start & Multi-Distro Installation

Orbiscreen provides official packages for multiple distributions and a standalone release bundle.

### 1. Debian / Ubuntu (`.deb`)

Download `orbiscreen_amd64.deb` from the [GitHub Releases](https://github.com/shadow-x78/orbiscreen/releases) page.
```bash
sudo dpkg -i orbiscreen_amd64.deb || sudo apt-get install -f
```

**Uninstall:**
```bash
sudo apt-get remove orbiscreen
```

### 2. Fedora / RHEL / openSUSE (`.rpm`)

Download `orbiscreen_x86_64.rpm` from the releases page. Import the public GPG key and install:
```bash
sudo rpm --import https://raw.githubusercontent.com/shadow-x78/orbiscreen/main/orbiscreen.asc
sudo dnf install ./orbiscreen_x86_64.rpm
```

**Uninstall:**
```bash
sudo dnf remove orbiscreen
```

### 3. Universal AppImage (`.AppImage`)

Download `orbiscreen-x86_64.AppImage` from the releases page.
```bash
chmod +x orbiscreen-x86_64.AppImage
./orbiscreen-x86_64.AppImage
```

### 4. Standalone Tarball (`.tar.gz`)

Download `orbiscreen-linux-x86_64.tar.gz`.
```bash
tar -xzvf orbiscreen-linux-x86_64.tar.gz
cd release-bundle
./install.sh
```

**Uninstall:**
```bash
./uninstall.sh
```

### 5. Android App (`.apk`)

Install `orbiscreen-android-release.apk` (signed release build to bypass Play Protect warnings) from the releases page.

**Permissions requested on first launch:**
- `ACCESS_NETWORK_STATE`, `ACCESS_WIFI_STATE`, `CHANGE_WIFI_MULTICAST_LOCK` — NSD discovery + streaming.
- `ACCESS_FINE_LOCATION` / `ACCESS_COARSE_LOCATION` — required by Android for Wi-Fi scanning on API 33+.
- `INTERNET` — video stream and `/api/control` calls.
- `VIBRATE` — soft keyboard feedback.

---

## 🛠️ Multi-Architecture Targets

Currently, Orbiscreen provides pre-built binaries for `x86_64` (AMD64) architecture only. For `aarch64` (ARM64) devices (e.g. Raspberry Pi 4/5, Asahi Linux), you must build from source.

### Building from Source

```bash
git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
cd ~/Orbiscreen

./scripts/setup-dev-env.sh

cargo build --release --workspace
./scripts/install.sh
```

### Building the Android Client from Source

```bash
cd clients/android
./gradlew :app:assembleDebug   # or :app:assembleRelease for a signed APK
```

Requires JDK 17 and the Android SDK with platforms `android-34` installed.

---

## 🩺 First-Run Verification

After installation:

```bash
orbiscreen probe                # verifies capture / input / display backends
orbiscreen start                # boots the daemon in the foreground
```

Open the Android client on the same Wi-Fi and tap the host in the **Discovery** list. If mDNS is blocked, tap **Add manually** and enter `host:port`.

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Back to README](../README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
