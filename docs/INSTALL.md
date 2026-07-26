# Installation Guide - Orbiscreen

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

Download `orbiscreen_x86_64.rpm` from the releases page.
```bash
sudo dnf install --nogpgcheck ./orbiscreen_x86_64.rpm
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

---

## 🛠️ Multi-Architecture Targets

Currently, Orbiscreen provides pre-built binaries for `x86_64` (AMD64) architecture only. For `aarch64` (ARM64) devices (e.g. Raspberry Pi 4/5, Asahi Linux), you must build from source.

### Building from Source

```bash
# Clone the repository
git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
cd ~/Orbiscreen

# Install dependencies (see scripts/setup-dev-env.sh for details)
./scripts/setup-dev-env.sh

# Build the release binary
cargo build --release --workspace

# Install the daemon and systemd service
./scripts/install.sh
```

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Back to README](../README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
