<div align="center">

# دليل التغليف متعدد التوزيعات - Orbiscreen

[![الإصدار](https://img.shields.io/badge/version-0.33.8-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.92%2B-16a34a?style=flat-square&logo=rust)
![المنصّة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

---

<a id="packaging-matrix"></a>
## مصفوفة حزم التوزيعات

مصفوفة الإصدار: `0.33.8` (مساحة العمل)، `versionCode = 144` (Android). ملاحظة: keystore إصدار Android لم تعد مضمنة في المستودع: راجع SECURITY.md؛ وفّر `ORBISCREEN_KEYSTORE_PATH`/`ORBISCREEN_STORE_PASSWORD`/`ORBISCREEN_KEY_ALIAS`/`ORBISCREEN_KEY_PASSWORD` عند بناء APK الإصدار.

`versionCode` قيمة تُعدَّل يدوياً ويزيدها واحد مع كل إصدار (وهي عدد صحيح تصاعدي لا يمكن أن يتبع إصداراً دلالياً). `scripts/check-versions.sh` يتحقق من أن كل مواضع الإصدار الـ25 متطابقة مع مساحة عمل Cargo، ويعمل أيضاً ضمن CI.

يوفّر Orbiscreen تكوينات البناء وتعريفات الحزم لجميع توزيعات Linux الرئيسية وAndroid:

- **AppImage:** حزمة محمولة لجميع توزيعات Linux.
- **Debian / Ubuntu (.deb):** حزمة Debian أصلية لـ Ubuntu وDebian وMint وPop!_OS.
- **Fedora / RHEL (.rpm):** حزمة RPM أصلية لـ Fedora وRHEL وCentOS وopenSUSE.
- **Arch Linux / Manjaro (`PKGBUILD`):** حزمة Arch أصلية تُبنى عبر `makepkg`.
- **أرشيف عام (.tar.gz):** أرشيف إصدار مستقل مع مثبّت بأمر واحد.
- **Android APK (.apk):** عميل Material 3 + Jetpack Compose لأجهزة Android اللوحية والهواتف.

---

## بناء الحزم محلياً

### 1. الأرشيف المستقل والمثبّت بأمر واحد
```bash
cargo build --release --workspace
./scripts/install.sh
```

### 2. حزمة Debian / Ubuntu (`.deb`)
```bash
./scripts/package-deb.sh
```
يتطلب `dpkg-deb` (من حزمة `dpkg`)؛ ويبني السكربت ثنائيات الإصدار أولاً عند غيابها.

### 3. حزمة Fedora / RHEL / openSUSE (`.rpm`)
```bash
./scripts/package-rpm.sh
```
يتطلب `rpmbuild` (من حزمة `rpm-build`)؛ وبدونه يجهّز السكربت شجرة الملفات في `target/rpm-staging`.

### 4. آرش لينكس / مانجارو (`PKGBUILD`)
```bash
sudo pacman -Syu --needed git base-devel
git clone https://github.com/shadow-x78/orbiscreen.git
cd orbiscreen
makepkg -si
```
يتضمن المستودع ملف `PKGBUILD` في المجلد الرئيسي. يؤدي تشغيل `makepkg -si` إلى تنزيل أرشيف الإصدار تلقائياً، وتثبيت كافة اعتماديات البناء والتشغيل عبر pacman، وبناء مساحة العمل عبر `cargo build --release --workspace --locked`، وتشغيل الفحوصات، وتثبيت البرنامج وملفات سطح المكتب في المسارات النظامية.

### 5. AppImage
```bash
./scripts/package-appimage.sh
```

### 6. عميل Android (`orbiscreen-android-release.apk`)
```bash
cd clients/android
./gradlew assembleRelease
```
موقع ملف APK الناتج: `clients/android/app/build/outputs/apk/release/app-release.apk`

يوقَّع ملف APK للإصدار بمفتاح keystore المزوَّد عبر `ORBISCREEN_KEYSTORE_PATH` (عند تكوينه) باستخدام مخططات V2/V3. قواعد ProGuard في `clients/android/app/proguard-rules.pro` تحافظ على صفوف `androidx.media3` وOkHttp وCompose وNSD الانعكاسية.

---

## التوزيعات والأهداف المدعومة

| الهدف | التنسيق | التوزيعات | الحالة |
|--------|---------|-------------|--------|
| Linux x86_64 | `.deb` | Ubuntu 22.04+, Debian 12+, Mint 21+, Pop!_OS 22.04+ | ✅ آلي |
| Linux x86_64 | `.rpm` | Fedora 38+, RHEL 9+, Rocky/Alma 9+, openSUSE Tumbleweed/Leap 15.5+ | ✅ آلي |
| Linux x86_64 | `PKGBUILD` | Arch Linux, Manjaro, EndeavourOS, Garuda | ✅ آلي |
| Linux x86_64 | `.AppImage` | أي توزيع يعتمد على glibc | ✅ آلي |
| Linux x86_64 | `.tar.gz` | أي توزيع (تثبيت يدوي) | ✅ آلي |
| Android ARM64 | `.apk` | Android 7.0+ (API 24+) | ✅ آلي |

---

## أرشيفات المصدر

يتم إنشاء أرشيفات مصدر الإصدار عبر مسار CI وإرفاقها بـ GitHub Releases:

```
orbiscreen-<version>.tar.gz          # الكود المصدري فقط
orbiscreen-<version>-linux-x86_64.tar.gz  # ثنائيات جاهزة
```

يحتوي أرشيف المصدر على مساحة العمل الكاملة مع جميع الحزم والعملاء. يحتوي الأرشيف الثنائي على ثنائيات الإصدار المُخفّفة لـ `orbiscreen` و `orbiscreen-gui` (عند التوفر).

---

## Fedora / RHEL / CentOS (COPR)

### مستودع COPR
```bash
sudo dnf copr enable shadow-x78/orbiscreen
sudo dnf install orbiscreen
```

### البناء من ملف Spec
```bash
./scripts/package-rpm.sh
```
ملف RPM الناتج: `orbiscreen-<version>-1.x86_64.rpm`

### ملف Spec
`data/orbiscreen-copr.spec` يحتوي على تعريف بناء COPR مع:
- BuildRequires: `cargo`, `rust >= 1.92`, `gstreamer1-devel`, `gstreamer1-plugins-base-devel`, `wayland-devel`, `libdrm-devel`, `kwayland-devel`, `kde-cli-tools`, `libinput-devel`, `libudev-devel`, `systemd-devel`, `openssl-devel`
- Requires: `gstreamer1`, `gstreamer1-plugins-base`, `gstreamer1-plugins-good`, `gstreamer1-plugins-bad-free`, `gstreamer1-libav`, `kwayland`, `kwin`, `libinput`, `systemd`, `openssl-libs`, `webkit2gtk4.1` (GUI)

---

## Debian / Ubuntu / Linux Mint (.deb)

### البناء من المصدر
```bash
./scripts/package-deb.sh
```

### ملف Control
`debian/control` يحدّد:
- Build-Depends: `cargo (>= 1.92)`, `rustc (>= 1.92)`, `libgstreamer1.0-dev`, `libgstreamer-plugins-base1.0-dev`, `libwayland-dev`, `libdrm-dev`, `libkf5waylandclient-dev`, `libkf5kdelibs4support-dev`, `libinput-dev`, `libudev-dev`, `libsystemd-dev`, `libssl-dev`, `pkg-config`, `debhelper-compat (= 13)`, `cmake`
- Depends: `${shlibs:Depends}`, `${misc:Depends}`, `gstreamer1.0-plugins-base`, `gstreamer1.0-plugins-good`, `gstreamer1.0-plugins-bad`, `gstreamer1.0-libav`, `libwayland-client0`, `libkf5waylandclient5`, `libinput10`, `libudev1`, `libsystemd0`, `libssl3`, `webkit2gtk-4.1-0` (GUI)

### التثبيت
```bash
sudo dpkg -i orbiscreen_<version>_amd64.deb
sudo apt-get install -f
```

---

## Arch Linux / Manjaro (PKGBUILD)

يحتوي جذر المستودع على ملف `PKGBUILD` مستقل:

```bash
makepkg -si
```

سيقوم هذا بـ:
1. تنزيل أرشيف مصدر الإصدار
2. التحقق من مجموعات التحقق SHA256
3. البناء عبر `cargo build --release --workspace --locked`
4. تشغيل اختبارات الوحدة
5. تثبيت الثنائيات، إدخالات سطح المكتب، قواعد udev، ووحدات systemd

---

## تغليف إصدار Android

### المتطلبات
- Android SDK / Android Studio
- Keystore لتوقيع الإصدار (ليس في المستودع؛ راجع SECURITY.md)

### البناء
```bash
cd clients/android
ORBISCREEN_KEYSTORE_PATH=/path/to/keystore.jks \
ORBISCREEN_STORE_PASSWORD=**** \
ORBISCREEN_KEY_ALIAS=orbiscreen \
ORBISCREEN_KEY_PASSWORD=**** \
./gradlew assembleRelease
```

### المخرجات
- `app/build/outputs/apk/release/app-release.apk` (موقّع، V2/V3)
- إصدار الكود: `144` (2)، يتم زيادته تلقائياً لكل إصدار

### التحقق
```bash
apksigner verify --print-certs app-release.apk
aapt dump badging app-release.apk | grep version
```

---

## التوقيع والتوزيع

### حزم Linux
- **RPM:** موقّع بـ `rpmsign` باستخدام مفتاح GPG الخاص بالمُحافظ (COPR يتولّى ذلك)
- **DEB:** موقّع بـ `dpkg-sig` أو `debsign` لرفع PPA
- **AppImage:** توقيع GPG منفصل (`.sig`) مرفق بـ GitHub Release
- **أرشيف:** مجموعات SHA256 (`.sha256`) مرفقة بـ GitHub Release

### Android
- يتم إنشاء keystore للإصدار مرة واحدة، ويُخزّن خارجياً
- يبني CI باستخدام `ORBISCREEN_KEYSTORE_PATH` + كلمات المرور من GitHub Secrets
- لا توجد مادة keystore في المستودع

### مجموعات التحقق
```bash
# التحقق من الأرشيف
sha256sum -c orbiscreen-<version>-linux-x86_64.tar.gz.sha256

# التحقق من AppImage
gpg --verify orbiscreen-x86_64.AppImage.sig orbiscreen-x86_64.AppImage
```

---

## الرخصة

Orbiscreen مرخص تحت **GPL-3.0-or-later**.

راجع [LICENSE](../LICENSE) للنص الكامل. جميع سكربتات التغليف وملفات spec ترث هذه الرخصة.

Copyright (C) 2024-present shadow-x78 and contributors.