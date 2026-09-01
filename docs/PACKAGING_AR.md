<div align="center">

# دليل التغليف متعدد التوزيعات - Orbiscreen

[![الإصدار](https://img.shields.io/badge/version-0.14.0-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![المنصّة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 اللغة

<a href="PACKAGING.md">🇬🇧 English</a> · <a href="PACKAGING_AR.md">🇸🇦 العربية</a>

---

مصفوفة الإصدار: `0.14.0` (مساحة العمل)، `versionCode = 29` (Android). ملاحظة: keystore إصدار Android لم تعد مضمنة في المستودع - راجع SECURITY.md؛ وفّر `ORBISCREEN_KEYSTORE_PATH`/`ORBISCREEN_STORE_PASSWORD`/`ORBISCREEN_KEY_ALIAS`/`ORBISCREEN_KEY_PASSWORD` عند بناء APK الإصدار.

يوفّر Orbiscreen تكوينات البناء وتعريفات الحزم لجميع توزيعات Linux الرئيسية وAndroid:

- **AppImage:** حزمة محمولة لجميع توزيعات Linux.
- **Debian / Ubuntu (.deb):** حزمة Debian أصلية لـ Ubuntu وDebian وMint وPop!_OS.
- **Fedora / RHEL (.rpm):** حزمة RPM أصلية لـ Fedora وRHEL وCentOS وopenSUSE.
- **أرشيف عام (.tar.gz):** أرشيف إصدار مستقل مع مثبّت بأمر واحد.
- **Android APK (.apk):** عميل Material 3 + Jetpack Compose لأجهزة Android اللوحية والهواتف.

---

## 🔨 بناء الحزم محلياً

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

### 4. AppImage
```bash
./scripts/package-appimage.sh
```

### 5. عميل Android (`orbiscreen-android-release.apk`)
```bash
cd clients/android
./gradlew assembleRelease
```
موقع ملف APK الناتج: `clients/android/app/build/outputs/apk/release/app-release.apk`

يوقَّع ملف APK للإصدار بمفتاح keystore المزوَّد عبر `ORBISCREEN_KEYSTORE_PATH` (عند تكوينه) باستخدام مخططات V2/V3. قواعد ProGuard في `clients/android/app/proguard-rules.pro` تحافظ على صفوف `androidx.media3` وOkHttp وCompose وNSD الانعكاسية.

---

## 🗑️ إزالة الحزم

يتعامل كل مدير حزم مع الإزالة بنظافة:

- **Debian / Ubuntu (`.deb`):** `sudo apt-get remove orbiscreen`
- **Fedora / RHEL (`.rpm`):** `sudo dnf remove orbiscreen`
- **الأرشيف المستقل:** شغّل `./scripts/uninstall.sh` المرفق في المصدر أو مجلد الأرشيف.
- **Android:** اضغط مطوّلاً على أيقونة التطبيق ← **App info** ← **Uninstall**.

---

## 🔐 التوقيع التشفيري

جميع البنى موقّعة تشفيرياً (v0.9.0+):

- **حزم Linux:** حزم RPM موقّعة بـ GPG (`orbiscreen.asc`)، حزم DEB موقّعة بـ `debsigs`، ويحتوي AppImage على توقيعة مُجزّأة.
- **Android APK:** موقّع بملف keystore المزوَّد (V2/V3).

للتحقق من حزمة RPM يدوياً:
```bash
sudo rpm --import https://raw.githubusercontent.com/shadow-x78/orbiscreen/main/orbiscreen.asc
rpm -K orbiscreen_x86_64.rpm
```

---

## 🚀 مصفوفة إصدارات GitHub Actions

عند دفع وسم إصدار (مثلاً `git tag v0.10.3 && git push origin v0.10.3`)، يبني سير العمل `.github/workflows/release.yml` تلقائياً جميع حزم الإصدار ويرفقها بصفحة GitHub Releases.

يُولَّد محتوى `body` للإصدار من كتلة `## orbiscreen | v0.10.3 | …` في `CHANGELOG.md`.

---

## 📱 خيارات بناء Android

| نوع البناء | الأمر | ملاحظات |
|------------|---------|-------|
| Debug غير موقّع | `./gradlew assembleDebug` | بلا توقيع؛ غير مخصص للتوزيع |
| Release موقّع | `./gradlew assembleRelease` | يستخدم keystore المزوَّد عبر `ORBISCREEN_KEYSTORE_PATH` عند تكوينه |
| فحص Lint ثابت | `./gradlew lintDebug` | اشتراك المشروع في `androidx.media3 UnstableApi` |

يبلغ حجم APK الـ Debug حوالي 22 ميغابايت؛ يقلّص R8 حجم APK الـ Release إلى نحو 4 ميغابايت.

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[العودة إلى README](../README_AR.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
