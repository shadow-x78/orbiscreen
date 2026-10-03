<div align="center">

# Multi-Distro Packaging Guide - Orbiscreen

[![Version](https://img.shields.io/badge/version-0.32.10-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.92%2B-16a34a?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## Table of Contents

- [Supported Distros & Targets](#supported-distros--targets)
- [Source Tarballs](#source-tarballs)
- [Packaging Matrix](#packaging-matrix)
- [Local Build Instructions](#local-build-instructions)
- [Version Synchronization](#version-synchronization)
- [Fedora / RHEL / CentOS (COPR)](#fedora--rhel--centos-copr)
- [Debian / Ubuntu / Linux Mint (.deb)](#debian--ubuntu--linux-mint-deb)
- [Arch Linux / Manjaro (PKGBUILD)](#arch-linux--manjaro-pkgbuild)
- [Android Release Packaging](#android-release-packaging)
- [Signing & Distribution](#signing--distribution)
- [License](#license)

---

## Version synchronization

When cutting a release, the version must be updated across all packages:

- `Cargo.toml`: `[workspace.package].version = "0.32.10"`
- `clients/android/app/build.gradle.kts`: `versionName = "0.32.10"`, `versionCode = 135`
- `crates/orbiscreen-gui/tauri.conf.json`: `"version": "0.32.10"`
- `PKGBUILD`: `pkgver=0.32.10`
- `debian/changelog`: new entry for `0.32.10-1`
- `data/orbiscreen-copr.spec`: `Version: 0.32.10`

Verify with the consistency check, which compares every location above against the Cargo
workspace and exits non-zero on any mismatch. It runs in CI as well:

```bash
scripts/check-versions.sh
```

`cargo run -p orbiscreen-daemon -- --version` only prints the daemon's own version; it
does not detect drift between the packaging files.

---

<a id="packaging-matrix"></a>
## Packaging Matrix

The release matrix is: `0.32.10` (workspace), `versionCode = 135` (Android). The Android release keystore is no longer shipped in the repo (see SECURITY.md); supply `ORBISCREEN_KEYSTORE_PATH`/`ORBISCREEN_STORE_PASSWORD`/`ORBISCREEN_KEY_ALIAS`/`ORBISCREEN_KEY_PASSWORD` when building a release APK.

Orbiscreen provides build configurations and package definitions for all major Linux distributions and Android:

- **AppImage:** Portable bundle for all Linux distributions.
- **Debian / Ubuntu (.deb):** Native Debian package for Ubuntu, Debian, Mint, and Pop!_OS.
- **Fedora / RHEL (.rpm):** Native RPM package for Fedora, RHEL, CentOS, and openSUSE.
- **Arch Linux / Manjaro (`PKGBUILD`):** Native Arch package built via `makepkg`.
- **Generic Tarball (.tar.gz):** Standalone release archive with one-command installer.
- **Android APK (.apk):** Material 3 + Jetpack Compose client for Android tablets and smartphones.

---

## Building Packages Locally

### 1. Standalone Tarball & One-Command Installer
```bash
cargo build --release --workspace
./scripts/install.sh
```

### 2. Debian / Ubuntu Package (`.deb`)
```bash
./scripts/package-deb.sh
```
Requires `dpkg-deb` (from the `dpkg` package); the script builds release binaries first when missing.

### 3. Fedora / RHEL / openSUSE Package (`.rpm`)
```bash
./scripts/package-rpm.sh
```
Requires `rpmbuild` (from `rpm-build`); without it the script still stages the file tree under `target/rpm-staging`.

<a id="arch-linux--manjaro-pkgbuild"></a>
### 4. Arch Linux / Manjaro (`PKGBUILD`)
```bash
sudo pacman -Syu --needed git base-devel
git clone https://github.com/shadow-x78/orbiscreen.git
cd orbiscreen
makepkg -si
```
The repository includes a standalone `PKGBUILD` in the project root. Running `makepkg -si` automatically downloads the release tarball, resolves all build and runtime dependencies via pacman, compiles the workspace crates with `cargo build --release --workspace --locked`, runs unit tests, and installs the binary and desktop integration files directly to system paths.

### 5. AppImage
```bash
./scripts/package-appimage.sh
```

### 6. Android Client (`orbiscreen-android-release.apk`)
```bash
cd clients/android
./gradlew assembleRelease
```
Output APK location: `clients/android/app/build/outputs/apk/release/app-release.apk`

The release APK is signed with the keystore provided via `ORBISCREEN_KEYSTORE_PATH` (when configured) using V2/V3 signing schemes. ProGuard rules in `clients/android/app/proguard-rules.pro` preserve reflective `androidx.media3`, OkHttp, Compose, and NSD classes.

---

<a id="supported-distros--targets"></a>
## Supported Distros & Targets

| Target | Format | Distributions | Status |
|--------|--------|---------------|--------|
| Linux x86_64 | `.deb` | Ubuntu 22.04+, Debian 12+, Mint 21+, Pop!_OS 22.04+ | ✅ Automated |
| Linux x86_64 | `.rpm` | Fedora 38+, RHEL 9+, Rocky/Alma 9+, openSUSE Tumbleweed/Leap 15.5+ | ✅ Automated |
| Linux x86_64 | `PKGBUILD` | Arch Linux, Manjaro, EndeavourOS, Garuda | ✅ Automated |
| Linux x86_64 | `.AppImage` | Any glibc-based distro | ✅ Automated |
| Linux x86_64 | `.tar.gz` | Any distro (manual install) | ✅ Automated |
| Android ARM64 | `.apk` | Android 7.0+ (API 24+) | ✅ Automated |

---

<a id="source-tarballs"></a>
## Source Tarballs

Release source tarballs are generated by the CI pipeline and attached to GitHub Releases:

```
orbiscreen-<version>.tar.gz          # Source code only
orbiscreen-<version>-linux-x86_64.tar.gz  # Pre-built binaries
```

The source tarball contains the complete workspace with all crates and clients. The binary tarball contains stripped release binaries for `orbiscreen` and `orbiscreen-gui` (when available).

---

## Fedora / RHEL / CentOS (COPR)

### COPR Repository
```bash
sudo dnf copr enable shadow-x78/orbiscreen
sudo dnf install orbiscreen
```

### Build from Spec
```bash
./scripts/package-rpm.sh
```
The generated RPM: `orbiscreen-<version>-1.x86_64.rpm`

### Spec File
`data/orbiscreen-copr.spec` contains the COPR build definition with:
- BuildRequires: `cargo`, `rust >= 1.92`, `gstreamer1-devel`, `gstreamer1-plugins-base-devel`, `wayland-devel`, `libdrm-devel`, `kwayland-devel`, `kde-cli-tools`, `libinput-devel`, `libudev-devel`, `systemd-devel`, `openssl-devel`
- Runtime Requires: `gstreamer1`, `gstreamer1-plugins-base`, `gstreamer1-plugins-good`, `gstreamer1-plugins-bad-free`, `gstreamer1-libav`, `kwayland`, `kwin`, `libinput`, `systemd`, `openssl-libs`, `webkit2gtk4.1` (GUI)

---

## Debian / Ubuntu / Linux Mint (.deb)

### Build from Source
```bash
./scripts/package-deb.sh
```

### Control File
`debian/control` defines:
- Build-Depends: `cargo (>= 1.92)`, `rustc (>= 1.92)`, `libgstreamer1.0-dev`, `libgstreamer-plugins-base1.0-dev`, `libwayland-dev`, `libdrm-dev`, `libkf5waylandclient-dev`, `libkf5kdelibs4support-dev`, `libinput-dev`, `libudev-dev`, `libsystemd-dev`, `libssl-dev`, `pkg-config`, `debhelper-compat (= 13)`, `cmake`
- Depends: `${shlibs:Depends}`, `${misc:Depends}`, `gstreamer1.0-plugins-base`, `gstreamer1.0-plugins-good`, `gstreamer1.0-plugins-bad`, `gstreamer1.0-libav`, `libwayland-client0`, `libkf5waylandclient5`, `libinput10`, `libudev1`, `libsystemd0`, `libssl3`, `webkit2gtk-4.1-0` (GUI)

### Installation
```bash
sudo dpkg -i orbiscreen_<version>_amd64.deb
sudo apt-get install -f
```

---

## Arch Linux / Manjaro (PKGBUILD)

The repository root contains a standalone `PKGBUILD`:

```bash
makepkg -si
```

This will:
1. Download the release source tarball
2. Verify SHA256 checksums
3. Build with `cargo build --release --workspace --locked`
4. Run unit tests
5. Install binaries, desktop entries, udev rules, and systemd units

---

## Android Release Packaging

### Prerequisites
- Android SDK / Android Studio
- Keystore for release signing (not in repo; see SECURITY.md)

### Build
```bash
cd clients/android
ORBISCREEN_KEYSTORE_PATH=/path/to/keystore.jks \
ORBISCREEN_STORE_PASSWORD=**** \
ORBISCREEN_KEY_ALIAS=orbiscreen \
ORBISCREEN_KEY_PASSWORD=**** \
./gradlew assembleRelease
```

### Output
- `app/build/outputs/apk/release/app-release.apk` (signed, V2/V3)
- Version code: `135` (0.32.10), edited by hand. Android requires a strictly increasing
  integer, so it cannot track a semantic version; bump it by one per release.
  `scripts/check-versions.sh` verifies that it still matches the value quoted in this
  document, but it cannot tell you whether the release needs to advance it.

### Verification
```bash
apksigner verify --print-certs app-release.apk
aapt dump badging app-release.apk | grep version
```

---

## Signing & Distribution

### Linux Packages
- **RPM:** Signed with `rpmsign` using maintainer's GPG key (COPR handles this)
- **DEB:** Signed with `dpkg-sig` or `debsign` for PPA upload
- **AppImage:** GPG detached signature (`.sig`) attached to GitHub Release
- **Tarball:** SHA256 sums (`.sha256`) attached to GitHub Release

### Android
- Release keystore generated once, stored externally
- CI builds use `ORBISCREEN_KEYSTORE_PATH` + passwords from GitHub Secrets
- No keystore material in repository

### Checksums
```bash
# Verify tarball
sha256sum -c orbiscreen-<version>-linux-x86_64.tar.gz.sha256

# Verify AppImage
gpg --verify orbiscreen-x86_64.AppImage.sig orbiscreen-x86_64.AppImage
```

---

## License

Orbiscreen is licensed under **GPL-3.0-or-later**.

See [LICENSE](../LICENSE) for full text. All packaging scripts and spec files inherit this license.

Copyright (C) 2024-present shadow-x78 and contributors.