<!--
  Orbiscreen - README (Arabic) (GPL-3.0-or-later)
  https://github.com/shadow-x78/orbiscreen
-->
<div align="center">

<img src="data/orbiscreen.svg" alt="Orbiscreen" width="160" />

# Orbiscreen

شاشة افتراضية ثانية حقيقية لنظام Linux، تُبَثّ إلى Android - أمر واحد، بلا تعقيد

[![الإصدار](https://img.shields.io/badge/version-0.11.0-2563eb?style=flat-square&logo=semver)](CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![المنصّة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)
[![النجوم](https://img.shields.io/github/stars/shadow-x78/orbiscreen?style=flat-square&color=eab308&logo=github)](https://github.com/shadow-x78/orbiscreen/stargazers)

</div>

---

## 🌐 اللغة

<a href="README.md">🇬🇧 English</a> · <a href="README_AR.md">🇸🇦 العربية</a>

---

## 📋 فهرس المحتويات

- [ما هو Orbiscreen؟](#what-is-orbiscreen)
- [لماذا وُجد Orbiscreen](#why-orbiscreen-exists)
- [المميزات](#highlights)
- [الحالة](#status)
- [البدء السريع](#quick-start)
- [الأوامر](#commands)
- [تطبيق Android](#android-app)
- [المعمارية](#architecture)
- [التوثيق](#documentation)
- [المساهمة](#contributing)
- [الرخصة](#license)

---

<a id="what-is-orbiscreen"></a>
## 🤔 ما هو Orbiscreen؟

**Orbiscreen** يحوّل جهاز Android لوحياً أو هاتفاً إلى شاشة ثانية حقيقية لسطح مكتب Linux. على عكس الحلول البديلة المحدودة بـ X11 أو المتصفح فقط، ينشئ Orbiscreen **شاشة افتراضية على مستوى النواة** عبر `evdi` من DisplayLink، تظهر كشاشة حقيقية لكل من X11 وWayland، ويبثّها عبر **MPEG-TS/H.264** مع دعم إدخال اللمس العكسي natively على Android.

---

<a id="why-orbiscreen-exists"></a>
## 🧭 لماذا وُجد Orbiscreen

| المشكلة | المشاريع الأخرى | Orbiscreen |
|---------|----------------|------------|
| لا يوجد دعم Host على Linux | ❌ spacedesk يرفض رسمياً | ✅ شاشة افتراضية حقيقية على مستوى النواة |
| حل مؤقت محصور بـ X11 | ❌ VirtScreen غير محدّث منذ 2018 | ✅ X11 **و** Wayland عبر evdi/DRM |
| غياب شاشة ثانية على Wayland | ❌ Weylus محدود بـ X11 | ✅ مسار Wayland كامل عبر ashpd + PipeWire |
| إعداد يدوي لعناوين IP | ❌ معظم المشاريع | ✅ اكتشاف mDNS + مسح شبكي مباشر + إضافة يدوية |
| عميل وحيد الغرض | ❌ spacedesk فقط | ✅ شاشة Android أصلية + لوحة تحكم بالمضيف |

---

<a id="highlights"></a>
## ✨ المميزات

- شاشة افتراضية حقيقية عبر `evdi` (X11 *و* Wayland)، **أو بدون أي صلاحيات root على KDE Plasma** — مونيتور افتراضي يُنشئه KWin عبر `zkde-screencast` (بلا وحدة نواة وبلا نافذة مشاركة)، مع تراجع التقاط portal في غير ذلك
- **عميل Android بواجهة Material 3** - Jetpack Compose، لوحة ألوان Catppuccin Mocha / Latte، بسمة فاتحة وداكنة
- **عميل ويب مبنّى داخلياً** - شاهد من أي متصفح على `http://<host>:8788/` (MSE عبر `mpegts.js` المضمنة محلياً، دون CDN)
- **اكتشاف مباشر** - مسح NSD للمضيفين القريبين، إدخال يدوي `host:port`، وماسح Subnet اختياري
- **بث أصلي** - ExoPlayer مع `OkHttpDataSource` + `DefaultLoadControl` لبث MPEG-TS / H.264 منخفض الكمون
- **حماية بالتوكن** - `/stream` و`/input` و`/api/control` تتطلب توكن بخاص للجلسة (mDNS TXT / `/client/config.json`)، يدور مع كل تشغيل للدامن
- **لمس عكسي** - مؤشر مطلق / لوحة مفاتيح / قلم / عجلة يتدفق من Android إلى المضيف
- **لوحة تحكم بالمضيف** - لوحة مفاتيح، قفل، تعتيم، Ctrl+Alt+Del، وإعادة المحاولة
- **نقل عبر USB** بواسطة `adb reverse`، دون مشغلات خاصة
- **ترميز عتادي** - VAAPI، NVENC، وتراجع برمجي x264
- **توقيع تشفيري** لكل حزم Linux و Android

---

<a id="status"></a>
## 📊 الحالة

| المرحلة | الهدف | الحالة |
|---------|-------|--------|
| 0 | تهيئة مساحة العمل + جدوى evdi | ✅ مكتملة |
| 1 | الشاشة + الالتقاط + الترميز + الإدخال (X11) | ✅ مكتملة |
| 2 | عميل Android + نقل USB + mDNS | ✅ مكتملة |
| 3 | التقاط Wayland + portal + الإدخال | ✅ مكتملة |
| 4 | التغليف + واجهة GTK4 + خدمة D-Bus + التثبيت المستقل | ✅ مكتملة |
| 5 | واجهة Material 3 + الاكتشاف المباشر + لوحة التحكم | ✅ مكتملة |

> راجع `CHANGELOG.md` لسجل الإصدارات الكامل.

---

<a id="quick-start"></a>
## 🚀 البدء السريع

### 1. الحزم الرسمية والملفات الجاهزة (GitHub Releases)

حمّل الحزم المبنية مسبقاً من [GitHub Releases](https://github.com/shadow-x78/orbiscreen/releases):

- **Debian / Ubuntu (`.deb`):**
  ```bash
  sudo dpkg -i orbiscreen_amd64.deb || sudo apt-get install -f
  ```

- **Fedora / RHEL (`.rpm`):**
  > **ملاحظة:** حزم RPM موقّعة تشفيرياً. عليك استيراد المفتاح العام أولاً.
  ```bash
  sudo rpm --import https://raw.githubusercontent.com/shadow-x78/orbiscreen/main/orbiscreen.asc
  sudo dnf install ./orbiscreen_x86_64.rpm
  ```

- **AppImage عالمي (`.AppImage`):**
  ```bash
  chmod +x orbiscreen-x86_64.AppImage
  ./orbiscreen-x86_64.AppImage
  ```

- **أرشيف مستقل (`.tar.gz`):**
  ```bash
  tar -xzvf orbiscreen-linux-x86_64.tar.gz
  cd release-bundle && ./install.sh
  ```

- **Android (`.apk`):**
  ثبّت `orbiscreen-android-release.apk` (نسخة موقّعة لتجاوز تحذيرات Play Protect).

### 2. البناء من المصدر

```bash
# استنساخ المستودع
git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
cd ~/Orbiscreen

# أمر التثبيت الواحد لنظام Linux
./scripts/install.sh

# وحدة النواة evdi عبر DKMS - مطلوبة لشاشة ثانية حقيقية على أغلب بيئات
# سطح المكتب. على KDE Plasma Wayland لا حاجة لأي وحدة نواة: ينشئ الـ daemon
# مونيتوراً افتراضياً عبر KWin من تلقاء نفسه (بلا root وبلا نافذة مشاركة).
# وبدون أيٍّ من المسارين يبث Orbiscreen شاشة تختارها من نافذة مشاركة portal.
# راجع docs/TROUBLESHOOTING_AR.md لخطوات التوزيعات. ثم:
sudo modprobe evdi

# فحص واجهات الالتقاط والإدخال والشاشة المحلية
orbiscreen probe

# تشغيل خدمة Orbiscreen (مع تراجع تلقائي: EVDI DRM أو شاشة KWin الافتراضية
# أو Wayland Portal)
orbiscreen start
```

#### تفضيل واجهة الالتقاط (`orbiscreen.toml`)

يقرأ الخادم افتراضيًا `$XDG_CONFIG_HOME/orbiscreen/orbiscreen.toml`
(أو `~/.config/orbiscreen/orbiscreen.toml` عندما لا يكون `XDG_CONFIG_HOME` معرّفًا) —
وهو المسار نفسه الذي تستخدمه وحدة systemd للمستخدم. أنشئ الملف هناك، أو
حدّد موقعًا آخر عبر `--config /path/to/orbiscreen.toml`.

```toml
[capture]
preferred = "auto"   # auto (الافتراضي) | kwin-virtual | evdi | portal | mirror
```

| القيمة | السلوك |
|--------|--------|
| `auto` | KDE Plasma Wayland: شاشة KWin الافتراضية (بلا root وبلا نافذة حوار) ثم portal. X11: ‏EVDI عند تحميل وحدتها، وإلا التقاط الشاشة الجذر. |
| `kwin-virtual` | شاشة KWin الافتراضية دائماً (فشل صريح على غير KDE). |
| `evdi` | شاشة EVDI DRM الافتراضية دائماً (اختيارية — تتطلب وحدة نواة مثبتة بـ root). |
| `portal` | نافذة مشاركة portal دائماً؛ اختر أي شاشة. |
| `mirror` | اعرض **سطح مكتبك الحقيقي** بدل شاشة ثانية: اختر الشاشة المراد عكسها من نافذة المشاركة. |

> الشاشة الافتراضية تبدأ **فارغة** (خلفية سطح المكتب فقط) — هذا معنى الشاشة الثانية. اسحب النوافذ إلى `Virtual-ORBISCREEN`، أو استخدم `mirror` لبث شاشتك الفعلية.

### 3. الاتصال

- **Android:** انقر المضيف المكتشف (عبر mDNS) أو أضفه يدوياً.
- **متصفح الويب:** افتح `http://<host-ip>:8788/` - يخدم الدامن عميل MPEG-TS مباشرةً (MSE + حزمة `mpegts.js` المضمنة محلياً).
- **التوكن:** تُولد كل بدءة للدامن توكن جلسة. يحصل Android عليه تلقائياً من الاكتشاف؛ عميل الويب يجلبه من `/client/config.json`. إذا رفض العميل برفض `401 Unauthorized`، أعد الاكتشاف أو أعد تشغيل العميل - قد يكون التوكن قد دار.

---

<a id="commands"></a>
## ⌨️ الأوامر

| الأمر | الوصف |
|-------|-------|
| `orbiscreen start` | ينشئ الشاشة الافتراضية ويبدأ البث |
| `orbiscreen start --no-mdns` | تشغيل دون إعلان mDNS |
| `orbiscreen stop` | إيقاف دامن قيد التشغيل رشيقاً عبر D-Bus |
| `orbiscreen list-displays` | سرد الشاشات الافتراضية المُهيّأة |
| `orbiscreen probe` | فحص واجهات الالتقاط / الإدخال / الشاشة |
| `orbiscreen print-config` | طباعة الإعدادات الفعلية |
| `orbiscreen uninstall` | إزالة الخدمة وخدمة systemd ومدخلات سطح المكتب |

```bash
orbiscreen --config orbiscreen.toml --verbose probe
```

لإزالة كل شيء، بما فيها الإعدادات المحفوظة وحالة وحدة evdi:

```bash
orbiscreen uninstall && ./scripts/uninstall.sh
```

---

<a id="android-app"></a>
## 📱 تطبيق Android

عميل Android هو تطبيق **Material 3 + Jetpack Compose** بنشاط واحد (single-Activity)، مع ثلاث شاشات مربوطة عبر Compose Navigation:

| الشاشة | ماذا تفعل |
|--------|-----------|
| **Discovery** | مسح NSD مباشر لخدمات `_orbiscreen._tcp.`، شرائح اتصال سريعة، إدخال يدوي `host:port`، وضع USB عبر `adb reverse`، والمضيف الأخير في الأعلى |
| **Stream** | ExoPlayer بملء الشاشة (MPEG-TS عبر HTTP) مع شريط تحكم عائم: لوحة مفاتيح، قفل، تعتيم، Ctrl+Alt+Del، إعادة محاولة |
| **Settings** | السمة (النظام / فاتح / داكن)، إجبار المُرمّز البرمجي، ماسح Subnet المتقدم، المضيف الأخير، حول |

اللمس العكسي يعمل مباشرة: يحوّل `InputDispatcher` لمس Android إلى إحداثيات مطلقة للمضيف عبر نقطة `/input`، مع إزالة تكرار حتى لا تتراكم الشبكة أثناء السحب السريع.

يتصل العميل بالخدمة عبر ثلاث نقاط JSON خفيفة إضافةً إلى `/stream` و `/input`:

| النقطة | الطريقة | الغرض |
|--------|---------|-------|
| `/api/info` | GET | دقة الشاشة والمُرمّز والإصدار |
| `/api/control` | POST | أوامر المضيف (blank، unblank، lock، ctrl-alt-del)؛ يتطلب توكن |
| `/health` | GET | فحص الحيوية |

---

<a id="architecture"></a>
## 🏗️ المعمارية

```
orbiscreen/
├── crates/
│   ├── orbiscreen-core/        # الأنواع والإعدادات والأخطاء
│   ├── orbiscreen-display/     # شاشات افتراضية مدعومة بـ evdi
│   ├── orbiscreen-capture/     # X11 ‏(x11rb) + Wayland ‏(KWin zkde-screencast / ashpd portal + PipeWire)
│   ├── orbiscreen-encode/      # خط أنابيب GStreamer ‏(VAAPI / NVENC / x264)
│   ├── orbiscreen-input/       # evdevil + ashpd RemoteDesktop
│   ├── orbiscreen-transport/   # axum + mDNS + /api/info + /api/control
│   └── orbiscreen-daemon/      # ثنائي CLI يربط كل الطبقات
├── clients/
│   ├── web/                    # عميل متصفح MPEG-TS ‏(HTML / CSS / JS)
│   └── android/                # تطبيق Material 3 Compose
│       └── app/src/main/java/com/orbiscreen/android/
│           ├── MainActivity.kt
│           ├── data/           # PrefsStore ‏(المضيف الأخير + الإعدادات)
│           ├── net/            # DiscoveryService, SubnetScanner, HostApi
│           ├── player/         # PlayerHolder, StreamUrl
│           ├── input/          # InputDispatcher
│           └── ui/
│               ├── theme/      # ألوان Material 3 وطباعة وسمات
│               ├── nav/        # رسم بياني للتنقل Compose
│               ├── discovery/  # DiscoveryScreen + ViewModel
│               ├── stream/     # StreamScreen, PlayerSurface, ControlToolbar
│               └── settings/   # SettingsScreen
├── packaging/{flatpak,appimage,debian}/
├── scripts/{setup-dev-env.sh,install.sh,uninstall.sh,verify-stream.sh}
├── .github/{workflows/,ISSUE_TEMPLATE/,PULL_REQUEST_TEMPLATE.md}
└── .editorconfig, .gitignore, .gitattributes, deny.toml, rustfmt.toml
```

```
┌──────────────────────────────────────────────────────────────┐
│  orbiscreen-daemon ‏(CLI، clap)‏                               │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────┐   │
│  │ display      │  │ capture      │  │ encode            │   │
│  │  evdi crate  │  │ x11rb/ashpd  │  │ gstreamer-rs      │   │
│  └──────────────┘  └──────────────┘  └───────────────────┘   │
│  ┌──────────────┐  ┌──────────────────────────────────────┐  │
│  │ input        │  │ transport                            │  │
│  │ evdevil/ashpd│  │ axum + mdns-sd + adb                 │  │
│  │              │  │ + /api/info + /api/control + /health  │  │
│  └──────────────┘  └──────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐    │
│  │ core: الأنواع والإعدادات والأخطاء المشتركة           │    │
│  └──────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
       │                  │                    │
       ▼                  ▼                    ▼
   /dev/dri/...     X11 / Wayland         الشبكة ‏(mDNS + HTTP)‏
```

---

<a id="documentation"></a>
## 📚 التوثيق

| المستند | الوصف |
|---------|-------|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) · [AR](docs/ARCHITECTURE_AR.md) | طوبولوجيا النظام وخط أنابيب الإطارات ومعمارية D-Bus |
| [docs/INSTALL.md](docs/INSTALL.md) · [AR](docs/INSTALL_AR.md) | خطوات التثبيت عبر التوزيعات |
| [docs/PACKAGING.md](docs/PACKAGING.md) · [AR](docs/PACKAGING_AR.md) | مواصفات التغليف متعدد التوزيعات ‏(.deb، .rpm، AppImage، Flatpak)‏ |
| [docs/DBUS_SPEC.md](docs/DBUS_SPEC.md) · [AR](docs/DBUS_SPEC_AR.md) | مواصفات واجهة D-Bus Session Bus |
| [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) · [AR](docs/TROUBLESHOOTING_AR.md) | المشاكل الشائعة والتشخيص وإصلاحات التسريع العتادي |
| [SECURITY.md](SECURITY.md) | نموذج الأمان وسلامة النقل وسياسات الشبكة |
| [CHANGELOG.md](CHANGELOG.md) | سجل الإصدارات الكامل |
| [CONTRIBUTING.md](CONTRIBUTING.md) | إرشادات المساهمة والبناء من المصدر |

---

<a id="contributing"></a>
## 🤝 المساهمة

راجع [إرشادات المساهمة](CONTRIBUTING.md) لمعرفة كيفية تهيئة بيئة التطوير وتنسيق الكود وإرسال Pull Requests.

عند الالتزام (commit)، اتبع الأسلوب التالي:

```text
orbiscreen | <النطاق>: <الرسالة>
```

على سبيل المثال:

```text
orbiscreen | android | player: retry on transient network errors
orbiscreen | docs | readme: clarify mDNS discovery flow
orbiscreen | v0.10.3 | release: host-input protocol alignment
```

---

<a id="license"></a>
## 📜 الرخصة

مرخّص تحت [رخصة GPL-3.0](LICENSE).

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[السجل](CHANGELOG.md) ·
[الأمان](SECURITY.md)

<sub>&copy; 2026 Orbiscreen</sub>

</div>
