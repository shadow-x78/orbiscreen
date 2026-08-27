# Multi-Distro Packaging Guide - Orbiscreen

## 🌐 Language

<a href="PACKAGING.md">🇬🇧 English</a> · <a href="PACKAGING_AR.md">🇸🇦 العربية</a>

---

> Applies to **v0.13.0** and later. The release matrix is: `0.13.0` (workspace), `versionCode = 26` (Android). Note: the Android release keystore is no longer shipped in the repo - see SECURITY.md; supply `ORBISCREEN_KEYSTORE_PATH`/`ORBISCREEN_STORE_PASSWORD`/`ORBISCREEN_KEY_ALIAS`/`ORBISCREEN_KEY_PASSWORD` when building a release APK.

Orbiscreen provides build configurations and package definitions for all major Linux distributions and Android:

- **AppImage:** Portable bundle for all Linux distributions.
- **Flatpak:** Sandboxed container format compatible with Flathub.
- **Debian / Ubuntu (.deb):** Native Debian package for Ubuntu, Debian, Mint, and Pop!_OS.
- **Fedora / RHEL (.rpm):** Native RPM package for Fedora, RHEL, CentOS, and openSUSE.
- **Generic Tarball (.tar.gz):** Standalone release archive with one-command installer.
- **Android APK (.apk):** Material 3 + Jetpack Compose client for Android tablets and smartphones.

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

### 4. Android Client (`orbiscreen-android-release.apk`)
```bash
cd clients/android
./gradlew assembleRelease
```
Output APK location: `clients/android/app/build/outputs/apk/release/app-release.apk`

The release APK is signed with `orbiscreen-release.keystore` (when present) using V1/V2/V3 schemes. ProGuard rules in `clients/android/app/proguard-rules.pro` keep `androidx.media3`, OkHttp, Compose, and NSD reflective classes.

---

## 🗑️ Uninstalling Packages

Each package manager handles uninstallation cleanly:

- **Debian / Ubuntu (`.deb`):** `sudo apt-get remove orbiscreen`
- **Fedora / RHEL (`.rpm`):** `sudo dnf remove orbiscreen`
- **Standalone Tarball:** Run `./scripts/uninstall.sh` provided in the source or tarball directory.
- **Android:** Long-press the app icon → **App info** → **Uninstall**.

---

## 🔐 Cryptographic Signing

All artifacts are cryptographically signed (v0.9.0+):

- **Linux packages:** RPM signed with GPG (`orbiscreen.asc`), DEB signed with `debsigs`, AppImage contains a hashed signature.
- **Android APK:** Signed with the production `orbiscreen-release.keystore` (V1/V2/V3).

To verify the Linux RPM manually:
```bash
sudo rpm --import https://raw.githubusercontent.com/shadow-x78/orbiscreen/main/orbiscreen.asc
rpm -K orbiscreen_x86_64.rpm
```

---

## 🚀 GitHub Actions Release Matrix

When a version tag is pushed (e.g. `git tag v0.10.3 && git push origin v0.10.3`), the `.github/workflows/release.yml` workflow automatically builds and attaches all release packages to the GitHub Releases page.

The release `body` is generated from the `## orbiscreen | v0.10.3 | …` block in `CHANGELOG.md`.

---

## 📱 Android Build Options

| Build type | Command | Notes |
|------------|---------|-------|
| Unsigned debug | `./gradlew assembleDebug` | No signing; not for distribution |
| Signed release | `./gradlew assembleRelease` | Uses `orbiscreen-release.keystore` if present |
| Static lint | `./gradlew lintDebug` | Project opt-in to `androidx.media3 UnstableApi` |

The debug APK is ~22 MB; R8 shrinks the release APK to ~4 MB.

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Back to README](../README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
