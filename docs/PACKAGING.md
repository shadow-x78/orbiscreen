<div align="center">

# Multi-Distro Packaging Guide - Orbiscreen

[![Version](https://img.shields.io/badge/version-0.20.0-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![License](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## 📋 Table of Contents

- [Supported Distros & Targets](#supported-distros--targets)
- [Source Tarballs](#source-tarballs)
- [Packaging Matrix](#packaging-matrix)
- [Fedora / RHEL / CentOS (COPR)](#fedora--rhel--centos-copr)
- [Debian / Ubuntu / Linux Mint (.deb)](#debian--ubuntu--linux-mint-deb)
- [Android Release Packaging](#android-release-packaging)
- [Signing & Distribution](#signing--distribution)
- [License](#license)

---

<a id="packaging-matrix"></a>
## 📦 Packaging Matrix

The release matrix is: `0.20.0` (workspace), `versionCode = 48` (Android). The Android release keystore is no longer shipped in the repo (see SECURITY.md); supply `ORBISCREEN_KEYSTORE_PATH`/`ORBISCREEN_STORE_PASSWORD`/`ORBISCREEN_KEY_ALIAS`/`ORBISCREEN_KEY_PASSWORD` when building a release APK.

Orbiscreen provides build configurations and package definitions for all major Linux distributions and Android:

- **AppImage:** Portable bundle for all Linux distributions.
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
./scripts/package-deb.sh
```
Requires `dpkg-deb` (from the `dpkg` package); the script builds release binaries first when missing.

### 3. Fedora / RHEL / openSUSE Package (`.rpm`)
```bash
./scripts/package-rpm.sh
```
Requires `rpmbuild` (from `rpm-build`); without it the script still stages the file tree under `target/rpm-staging`.

### 4. AppImage
```bash
./scripts/package-appimage.sh
```

### 5. Android Client (`orbiscreen-android-release.apk`)
```bash
cd clients/android
./gradlew assembleRelease
```
Output APK location: `clients/android/app/build/outputs/apk/release/app-release.apk`

The release APK is signed with the keystore supplied via `ORBISCREEN_KEYSTORE_PATH` (when configured) using V2/V3 schemes. ProGuard rules in `clients/android/app/proguard-rules.pro` keep `androidx.media3`, OkHttp, Compose, and NSD reflective classes.

---

## 🏪 Distribution Repositories

### Fedora COPR (automated via Packit)

The repo carries `.packit.yaml` and a source-build spec (`data/orbiscreen-copr.spec`, distinct from the local prebuilt-binary `data/orbiscreen.spec`): pull requests build the RPM on stable Fedora as a CI check, and every GitHub release tag publishes it to COPR automatically.

The build runs fully offline inside COPR's mock sandbox: the SRPM carries the crate dependency tree as a vendored `Source1` tarball (`cargo vendor`, ~19 MB compressed), because COPR builders have no network access during `%build` and a plain `cargo build --locked` would fail to fetch from crates.io. To regenerate after a dependency change: extract the release tarball, run `cargo vendor vendor`, pack it with `zstd -19`, and rebuild the SRPM.

One-time maintainer setup:
1. Sign in at <https://copr.fedorainfracloud.org> with GitHub (creates the account).
2. Enable Packit at <https://packit.dev> (Sign in with GitHub → approve the `orbiscreen` repository).
3. Push the next release tag; Packit creates the `shadow-x78/orbiscreen` COPR project on the first build.

User install once published:
```bash
dnf copr enable shadow-x78/orbiscreen
dnf install orbiscreen
```

### Fedora Native CI & RPM Packaging (GitHub Actions)

A dedicated GitHub Actions workflow (`.github/workflows/fedora.yml`) runs on native Fedora containers (`fedora:latest`, `41`, `40`). It compiles the workspace, executes unit tests against official Fedora dependencies, and packages binary RPMs.

Maintainers can trigger a rebuild anytime directly from the **Actions** tab on GitHub:
1. Navigate to **Actions** → **Fedora RPM & CI**.
2. Click **Run workflow**, choose the target branch and Fedora version, and trigger the job.
3. Download the compiled `.rpm` directly from the workflow's **Artifacts** section.


### Ubuntu / Pop!_OS / Linux Mint (Launchpad PPA)

Official Launchpad PPA:
```bash
sudo add-apt-repository ppa:shadow-x78/ppa -y
sudo apt update
sudo apt install orbiscreen -y
```

Or configure the direct APT repository:
```bash
curl -fsSL https://shadow-x78.github.io/orbiscreen/KEY.gpg | sudo gpg --dearmor -o /etc/apt/keyrings/orbiscreen.gpg
echo "deb [signed-by=/etc/apt/keyrings/orbiscreen.gpg] https://shadow-x78.github.io/orbiscreen /" | sudo tee /etc/apt/sources.list.d/orbiscreen.list
sudo apt update
sudo apt install orbiscreen -y
```

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
- **Android APK:** Signed with the supplied release keystore (V2/V3).

To verify the Linux RPM manually:
```bash
sudo rpm --import https://raw.githubusercontent.com/shadow-x78/orbiscreen/main/orbiscreen.asc
rpm -K orbiscreen_x86_64.rpm
```

---

## 🚀 GitHub Actions Release Matrix

When a version tag is pushed (e.g. `git tag v0.20.0 && git push origin v0.20.0`), the `.github/workflows/release.yml` workflow automatically builds and attaches all release packages to the GitHub Releases page.

The release `body` is generated from the `## [v0.20.0]` block in `CHANGELOG.md`.

---

## 📱 Android Build Options

| Build type | Command | Notes |
|------------|---------|-------|
| Unsigned debug | `./gradlew assembleDebug` | No signing; not for distribution |
| Signed release | `./gradlew assembleRelease` | Uses the keystore supplied via `ORBISCREEN_KEYSTORE_PATH` if configured |
| Static lint | `./gradlew lintDebug` | Project opt-in to `androidx.media3 UnstableApi` |

The debug APK is ~22 MB; R8 shrinks the release APK to ~4 MB.

---

<div align="center">

Built by <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[Back to README](../README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
