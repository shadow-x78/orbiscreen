# دليل التثبيت - Orbiscreen

## 🌐 اللغة

<a href="INSTALL.md">🇬🇧 English</a> · <a href="INSTALL_AR.md">🇸🇦 العربية</a>

---

> أحدث إصدار: **v0.11.0** (بث محمي بالتوكن، evdi أساسية مع تراجع portal، عميل ويب).

## 🚀 البدء السريع والتثبيت متعدد التوزيعات

يوفّر Orbiscreen حزماً رسمية لعدة توزيعات إضافةً إلى حزمة إصدار مستقلة.

### 1. Debian / Ubuntu (`.deb`)

حمّل `orbiscreen_amd64.deb` من صفحة [GitHub Releases](https://github.com/shadow-x78/orbiscreen/releases).
```bash
sudo dpkg -i orbiscreen_amd64.deb || sudo apt-get install -f
```

**الإزالة:**
```bash
sudo apt-get remove orbiscreen
```

### 2. Fedora / RHEL / openSUSE (`.rpm`)

حمّل `orbiscreen_x86_64.rpm` من صفحة الإصدارات. استورد المفتاح العام GPG ثم ثبّت:
```bash
sudo rpm --import https://raw.githubusercontent.com/shadow-x78/orbiscreen/main/orbiscreen.asc
sudo dnf install ./orbiscreen_x86_64.rpm
```

**الإزالة:**
```bash
sudo dnf remove orbiscreen
```

### 3. AppImage عالمي (`.AppImage`)

حمّل `orbiscreen-x86_64.AppImage` من صفحة الإصدارات.
```bash
chmod +x orbiscreen-x86_64.AppImage
./orbiscreen-x86_64.AppImage
```

### 4. أرشيف مستقل (`.tar.gz`)

حمّل `orbiscreen-linux-x86_64.tar.gz`.
```bash
tar -xzvf orbiscreen-linux-x86_64.tar.gz
cd release-bundle
./install.sh
```

**الإزالة:**
```bash
./uninstall.sh
```

### 5. تطبيق Android (`.apk`)

ثبّت `orbiscreen-android-release.apk` (نسخة موقّعة لتجاوز تحذيرات Play Protect) من صفحة الإصدارات.

**الأذونات المطلوبة عند أول تشغيل:**
- `ACCESS_NETWORK_STATE`، `ACCESS_WIFI_STATE`، `CHANGE_WIFI_MULTICAST_LOCK` - اكتشاف NSD + البث.
- `ACCESS_FINE_LOCATION` / `ACCESS_COARSE_LOCATION` - يفرضها Android لمسح Wi-Fi على API 33+.
- `INTERNET` - بث الفيديو ونداءات `/api/control`.
- `VIBRATE` - اهتزاز لوحة المفاتيح المرنة.

---

## 🛠️ أهداف متعددة المعماريات

حالياً، يوفّر Orbiscreen ثنائيات مبنية مسبقاً لمعمارية `x86_64` (AMD64) فقط. بالنسبة للأجهزة ذات معمارية `aarch64` (ARM64) (مثل Raspberry Pi 4/5 وAsahi Linux)، عليك البناء من المصدر.

### البناء من المصدر

```bash
git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
cd ~/Orbiscreen

./scripts/setup-dev-env.sh

cargo build --release --workspace
./scripts/install.sh
```

### بناء عميل Android من المصدر

```bash
cd clients/android
./gradlew :app:assembleDebug   # أو :app:assembleRelease للحصول على APK موقّع
```

يتطلب JDK 17 وAndroid SDK مع تثبيت منصات `android-34`.

---

## 🩺 التحقق من أول تشغيل

بعد التثبيت:

```bash
orbiscreen probe                # يتحقق من واجهات الالتقاط / الإدخال / الشاشة
orbiscreen start                # يشغّل الـ daemon في المقدمة
```

افتح عميل Android على شبكة Wi-Fi نفسها وانقر المضيف في قائمة الـ **Discovery**. إذا كان mDNS محظوراً، انقر **Add manually** وأدخل `host:port`.

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[العودة إلى README](../README_AR.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
