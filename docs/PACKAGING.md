# Multi-Distro Packaging Guide - Orbiscreen

---

## 🌐 Language

<a href="PACKAGING.md">🇬🇧 English</a> · <a href="PACKAGING_AR.md">🇸🇦 العربية</a>

---

## 📦 Overview

Orbiscreen provides build configurations and package definitions for all major Linux distributions and Android:

- **AppImage:** Portable bundle for all Linux distributions.
- **Flatpak:** Sandboxed container format compatible with Flathub.
- **Debian / Ubuntu (.deb):** Native Debian package for Ubuntu, Debian, Mint, and Pop!_OS.
- **Fedora / RHEL (.rpm):** Native RPM package for Fedora, RHEL, CentOS, and openSUSE.
- **Generic Tarball (.tar.gz):** Standalone release archive with one-command installer.
- **Android APK (.apk):** Client application for Android tablets and smartphones.

---

## 🔨 Building Packages Locally

### 1. Standalone Tarball & One-Command Installer
```bash
cargo build --release --workspace
./scripts/install.sh
```

### 2. Debian / Ubuntu Package (`.deb`)
```bash
cargo install cargo-deb
cargo deb -p orbiscreen-daemon
```

### 3. Fedora / RHEL / openSUSE Package (`.rpm`)
```bash
cargo install cargo-generate-rpm
cargo generate-rpm -p orbiscreen-daemon
```

### 4. Android Client (`app-debug.apk`)
```bash
cd clients/android
./gradlew assembleDebug
```
Output APK location: `clients/android/app/build/outputs/apk/debug/app-debug.apk`

---

## 🚀 GitHub Actions Release Matrix

When a version tag is pushed (e.g., `git tag v0.4.2 && git push origin v0.4.2`), the `.github/workflows/release.yml` workflow automatically builds and attaches all release packages to the GitHub Releases page.
