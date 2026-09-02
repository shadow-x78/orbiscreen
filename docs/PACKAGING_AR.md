<div align="center">

# دليل التغليف متعدد التوزيعات - Orbiscreen

[![الإصدار](https://img.shields.io/badge/version-0.17.0-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الإصدار](https://img.shields.io/badge/version-0.17.1-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![المنصّة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 اللغة

<a href="PACKAGING.md">🇬🇧 English</a> · <a href="PACKAGING_AR.md">🇸🇦 العربية</a>

---

مصفوفة الإصدار: `0.17.0` (مساحة العمل)، `versionCode = 38` (Android). ملاحظة: keystore إصدار Android لم تعد مضمنة في المستودع - راجع SECURITY.md؛ وفّر `ORBISCREEN_KEYSTORE_PATH`/`ORBISCREEN_STORE_PASSWORD`/`ORBISCREEN_KEY_ALIAS`/`ORBISCREEN_KEY_PASSWORD` عند بناء APK الإصدار.
مصفوفة الإصدار: `0.17.1` (مساحة العمل)، `versionCode = 39` (Android). ملاحظة: keystore إصدار Android لم تعد مضمنة في المستودع - راجع SECURITY.md؛ وفّر `ORBISCREEN_KEYSTORE_PATH`/`ORBISCREEN_STORE_PASSWORD`/`ORBISCREEN_KEY_ALIAS`/`ORBISCREEN_KEY_PASSWORD` عند بناء APK الإصدار.

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

## 🏪 مستودعات التوزيعات

### Fedora COPR (آلي عبر Packit)

يحمل المستودع `.packit.yaml` مع spec بناء-من-المصدر ‏(`data/orbiscreen-copr.spec`، المستقل عن `data/orbiscreen.spec` المحلي الذي يغلّف الثنائيات الجاهزة): طلبات الدمج تبني الـRPM على Fedora المستقرة كفحص CI، وكل وسم ريليز على GitHub ينشره إلى COPR تلقائياً.

يعمل البناء دون اتصال بالإنترنت بالكامل داخل sandbox الـmock في COPR: يحمل SRPM شجرة اعتماديات كرايتس Rust في حزمة `Source1` مجهّلة ‏(`cargo vendor`، نحو 19MB مضغوطة)، لأن بناة COPR بلا وصول شبكي أثناء `%build` وسيفشل `cargo build --locked` العادي في الجلب من crates.io. لإعادة توليدها بعد تغيير اعتمادية: فُك أرشيف الريليز، شغّل `cargo vendor vendor`، اضغطها بـ `zstd -19`، وأعد بناء SRPM.

إعداد المُصين (مرة واحدة):
1. سجّل الدخول إلى <https://copr.fedorainfracloud.org> بـ GitHub (يُنشئ الحساب).
2. فعّل Packit من <https://packit.dev> ‏(Sign in with GitHub → وافق على مستودع `orbiscreen`).
3. ادفع وسم الإصدار التالي؛ ينشئ Packit مشروع COPR ‏`shadow-x78/orbiscreen` عند أول بناء.

تثبيت المستخدم بعد النشر:
```bash
dnf copr enable shadow-x78/orbiscreen
dnf install orbiscreen
```

### Ubuntu / Pop!_OS / Linux Mint (Launchpad PPA)

مستودع Launchpad PPA الرسمي:
```bash
sudo add-apt-repository ppa:shadow-x78/ppa -y
sudo apt update
sudo apt install orbiscreen -y
```

أو عبر مستودع APT المباشر:
```bash
curl -fsSL https://shadow-x78.github.io/orbiscreen/KEY.gpg | sudo gpg --dearmor -o /etc/apt/keyrings/orbiscreen.gpg
echo "deb [signed-by=/etc/apt/keyrings/orbiscreen.gpg] https://shadow-x78.github.io/orbiscreen /" | sudo tee /etc/apt/sources.list.d/orbiscreen.list
sudo apt update
sudo apt install orbiscreen -y
```

### Arch Linux (AUR)

`PKGBUILD` في جذر المستودع يبني من tarball الريليز عبر cargo. تحمل نسخة المستودع `sha256sums=('SKIP')` عمداً: بصمة أرشيف الوسم لا توجد إلا بعد اكتمال سير الريليز، فتثبيتها بالمستودع سيجعلها تتخلف عن الوسم دائماً. التثبيت يحدث وقت النشر - أمر `updpkgsums` على جهاز المصين يكتب البصمة الحقيقية في نسخة PKGBUILD الخاصة بـAUR (وليس في هذا المستودع أبداً). تدفق النشر/التحديث:
```bash
git clone ssh://aur@aur.archlinux.org/orbiscreen.git aur-orbiscreen
cp PKGBUILD aur-orbiscreen/
cd aur-orbiscreen
updpkgsums                     # يجلب أرشيف الوسم ويثبّت sha256 الحقيقية
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO && git commit -m "orbiscreen v0.17.0" && git push
```
تثبيت المستخدم: `yay -S orbiscreen` (أو أي مساعد AUR) / `makepkg -si`.

### AppStream metainfo

ملف `data/com.orbiscreen.OrbiscreenGtk.metainfo.xml` (مُتحقق منه بـ `appstreamcli`) يعرض التطبيق في GNOME Software/Discover مع ملاحظات الإصدارات وتصنيف OARS؛ وهو مُغلَّف في spec الخاص بـCOPR وPKGBUILD الخاص بـAUR وأشجار تثبيت سكربتات deb/RPM.

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
