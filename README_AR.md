<div align="center">

<pre align="center">
 ██████╗ ██████╗ ██████╗ ██╗███████╗ ██████╗██████╗ ███████╗███████╗███╗   ██╗
██╔═══██╗██╔══██╗██╔══██╗██║██╔════╝██╔════╝██╔══██╗██╔════╝██╔════╝████╗  ██║
██║   ██║██████╔╝██████╔╝██║███████╗██║     ██████╔╝█████╗  █████╗  ██╔██╗ ██║
██║   ██║██╔══██╗██╔══██╗██║╚════██║██║     ██╔══██╗██╔══╝  ██╔══╝  ██║╚██╗██║
╚██████╔╝██║  ██║██████╔╝██║███████║╚██████╗██║  ██║███████╗███████╗██║ ╚████║
 ╚═════╝ ╚═╝  ╚═╝╚═════╝ ╚═╝╚══════╝ ╚═════╝╚═╝  ╚═╝╚══════╝╚══════╝╚═╝  ╚═══╝
</pre>

# Orbiscreen

شاشة افتراضية ثانية حقيقية لنظام Linux، تُبَثّ إلى Android — أمر واحد، بلا تعقيد

[![الإصدار](https://img.shields.io/badge/version-0.10.3-2563eb?style=flat-square&logo=semver)](CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![المنصّة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)
[![النجوم](https://img.shields.io/github/stars/shadow-x78/orbiscreen?style=flat-square&color=eab308&logo=github)](https://github.com/shadow-x78/orbiscreen/stargazers)

</div>

---

## 🌐 اللغة

<a href="README.md">🇬🇧 English</a> · <a href="README_AR.md">🇸🇦 العربية</a>

---

## 📋 المحتويات

- [ما هو Orbiscreen؟](#what-is-orbiscreen)
- [لماذا وُجد Orbiscreen](#why-orbiscreen-exists)
- [المميّزات](#highlights)
- [الحالة](#status)
- [البدء السريع والتثبيت متعدد التوزيعات](#quick-start)
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
|---------|---------------|------------|
| لا يوجد دعم Host على Linux | ❌ spacedesk يرفض رسمياً | ✅ شاشة افتراضية حقيقية على مستوى النواة |
| حل مؤقت محصور بـ X11 | ❌ VirtScreen غير محدّث منذ 2018 | ✅ X11 **و** Wayland عبر evdi/DRM |
| غياب شاشة ثانية على Wayland | ❌ Weylus محدود بـ X11 | ✅ مسار Wayland كامل عبر ashpd + PipeWire |
| إعداد يدوي لعناوين IP | ❌ معظم المشاريع | ✅ اكتشاف mDNS + مسح شبكي مباشر + إضافة يدوية |
| عميل وحيد الغرض | ❌ spacedesk فقط | ✅ شاشة Android أصلية + لوحة تحكم بالمضيف |

---

<a id="highlights"></a>
## ✨ المميّزات

- شاشة افتراضية حقيقية عبر `evdi` (X11 *و* Wayland).
- **عميل Android بواجهة Material 3** — Jetpack Compose، لوحة ألوان Catppuccin Mocha / Latte مطابقة لـ `data/orbiscreen-app.svg`، مع سمة فاتحة/داكنة.
- **شاشة بداية وأيقونة لونية** — SplashScreen بخلفية العلامة وأيقونة مُكيّفة (adaptive) بخلفية بيضاء تعكس شعار SVG.
- **اكتشاف مباشر** — مسح NSD للمضيفين القريبين، إدخال يدوي `host:port`، وماسح Subnet اختياري.
- **بث أصلي** — بناء ExoPlayer على الخيط الرئيسي مع `OkHttpDataSource` + `DefaultLoadControl` لبث MPEG-TS / H.264 منخفض الكمون.
- **لمس عكسي** — مؤشر مطلق / لوحة مفاتيح / قلم / عجلة يتدفق من Android إلى المضيف.
- **لوحة تحكم بالمضيف** — لوحة مفاتيح، قفل، تعتيم، Ctrl+Alt+Del، مدير الملفات، وإعادة المحاولة.
- **نقل عبر USB** بواسطة `adb reverse`، دون مشغلات خاصة.
- **ترميز عتادي** — VAAPI، NVENC، وتراجع برمجي x264.
- **توقيع تشفيري** لكل حزم Linux و Android.

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

> راجع `CHANGELOG.md` لسِجل الإصدارات الكامل.

---

<a id="quick-start"></a>
## 🚀 البدء السريع والتثبيت متعدد التوزيعات

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

# فحص واجهات الالتقاط والإدخال والشاشة المحلية
orbiscreen probe

# تشغيل خدمة Orbiscreen (مع تراجع تلقائي EVDI DRM أو Wayland Portal)
orbiscreen start
```

---

<a id="commands"></a>
## ⌨️ الأوامر

| الأمر | الوصف |
|-------|-------|
| `orbiscreen start` | ينشئ الشاشة الافتراضية ويبدأ البث |
| `orbiscreen start --no-mdns` | تشغيل دون إعلان mDNS |
| `orbiscreen list-displays` | سرد الشاشات الافتراضية المُهيّأة |
| `orbiscreen probe` | فحص واجهات الالتقاط / الإدخال / الشاشة |
| `orbiscreen print-config` | طباعة الإعدادات الفعلية |
| `orbiscreen uninstall` | إزالة الخدمة وخدمة systemd ومدخلات سطح المكتب |

```bash
orbiscreen --config orbiscreen.toml --verbose probe
```

---

<a id="android-app"></a>
## 📱 تطبيق Android

عميل Android هو تطبيق **Material 3 + Jetpack Compose** بنشاط واحد (single-Activity). ثلاث شاشات مربوطة عبر Compose Navigation:

### شاشة الاكتشاف

- شريط حالة يعرض المسح النشط وعدد المضيفين المكتشفين.
- **قائمة مباشرة** بخدمات `_orbiscreen._tcp.` المكتشفة عبر `NsdManager`، مع الاسم وIP والمنفذ.
- **شرائح سريعة** لكل مضيف: انقر للاتصال، أو اضغط مطوّلاً للتفاصيل.
- بطاقة **إضافة يدوية** — تتوسع لإظهار `OutlinedTextField` يتحقق من صيغة `host:port`.
- بطاقة **وضع USB** — تملأ مسبقاً `127.0.0.1:8788` لأجل `adb reverse tcp:8788 tcp:8788`.
- زر تحديث يلغي جلسة NSD الحالية ويعيد المسح.
- **المضيف الأخير** محفوظ في `SharedPreferences` ويظهر أعلى القائمة بشريحة "Recent".

### شاشة البث

- `Scaffold` بخلفية سوداء مع شريط علوي بألوان Catppuccin Mocha/Latte وشريط `ControlToolbar`.
- **ExoPlayer** يُبنى بأمان على الخيط الرئيسي (`withContext(Dispatchers.Main)`)، مغلَّفاً بـ `PlayerView` (عبر `AndroidView`) مع `useController = false` حتى يكون الشريط الداخلي هو الواجهة الوحيدة.
- `PlayerHolder.build()` المحصَّن يغلّف كل خطوات التهيئة بـ try-catch لتظهر أخطاء البناء كبطاقات `StreamEvent.Error` قابلة لإعادة المحاولة بدل الانهيار.
- `StreamUrl` يستهدف `/stream` مع `setMimeType(MimeTypes.VIDEO_MP2T)` ليفك ترميز MPEG-TS عبر HTTP دون sniffing.
- `OkHttpDataSource` يستخدم مهلة قراءة صفرية للبث المباشر مع `DefaultLoadControl` مضبوط للتخزين المستقر.

### شريط التحكم

شريط إجراءات عائم فوق سطح المشغّل:

| الإجراء | التأثير |
|---------|---------|
| Keyboard | إظهار لوحة المفاتيح المرنة + تمرير IME النظام |
| Lock | `POST /api/control {action:"lock"}` |
| Blank | تبديل `POST /api/control {action:"blank", state:"on"|"off"}` |
| Ctrl+Alt+Del | `POST /api/control {action:"ctrl_alt_del"}` |
| Files | `POST /api/control {action:"open", target:"files"}` |
| Retry | إعادة تهيئة المشغّل |

### شاشة الإعدادات

- السمة: النظام / فاتح / داكن (النظام افتراضياً).
- البث: إجبار المُرمّز البرمجي؛ تفعيل ماسح Subnet المتقدم.
- المضيف الأخير: عرض ونسيان آخر اتصال ناجح.
- حول: إصدار التطبيق، مستوى SDK، ونسخ معلومات الإصدار.

### نموذج الإدخال

`InputDispatcher` يحوّل أحداث اللمس على Android إلى إحداثيات مطلقة للمضيف باستخدام أبعاد الشاشة المُبلَّغ عنها ويُرسل مغلّفات بروتوكول يفككها الـ daemon مباشرة:

```kotlin
fun move(localX: Float, localY: Float, containerW: Int, containerH: Int) {
    val (x, y) = map(localX, localY, containerW, containerH)
    moves.tryEmit(JSONObject().apply {
        put("Pointer", JSONObject().apply {
            put("Move", JSONObject().apply { put("x", x); put("y", y) })
        })
    })
}
```

أحداث المؤشر تُزال تكرارها عبر `MutableSharedFlow` مع `BufferOverflow.DROP_OLDEST` حتى لا تتراكم الشبكة أثناء السحب السريع.

### نقاط تحكم المضيف (جانب Rust)

يستخدم عميل Android ثلاث نقاط JSON خفيفة إضافةً إلى `/stream` و `/input`:

| النقطة | الطريقة | الغرض |
|--------|---------|-------|
| `/api/info` | GET | دقة الشاشة والمُرمّز والإصدار |
| `/api/control` | POST | أوامر المضيف (blank، lock، ctrl-alt-del، open) |
| `/health` | GET | فحص الحيوية |

---

<a id="architecture"></a>
## 🏗️ المعمارية

```
orbiscreen/
├── crates/
│   ├── orbiscreen-core/        # الأنواع والإعدادات والأخطاء
│   ├── orbiscreen-display/     # شاشات افتراضية مدعومة بـ evdi
│   ├── orbiscreen-capture/     # X11 (x11rb) + Wayland (ashpd + PipeWire)
│   ├── orbiscreen-encode/      # خط أنابيب GStreamer ‏(VAAPI / NVENC / x264)
│   ├── orbiscreen-input/       # evdevil + ashpd RemoteDesktop
│   ├── orbiscreen-transport/   # axum + mDNS + /api/info + /api/control
│   └── orbiscreen-daemon/      # ثنائي CLI يربط كل الطبقات
├── clients/
│   ├── web/                    # عميل متصفح MPEG-TS ‏(HTML / CSS / JS)
│   └── android/                # تطبيق Material 3 Compose
│       └── app/src/main/java/com/orbiscreen/android/
│           ├── MainActivity.kt
│           ├── data/           # PrefsStore (المضيف الأخير + الإعدادات)
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
├── scripts/{setup-dev-env.sh,test-evdi.sh,install.sh,uninstall.sh}
├── .github/{workflows/,ISSUE_TEMPLATE/,dependabot.yml}
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
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) · [عربي](docs/ARCHITECTURE_AR.md) | طوبولوجيا النظام وخط الأنابيب zero-copy ومعمارية D-Bus |
| [docs/INSTALL.md](docs/INSTALL.md) · [عربي](docs/INSTALL_AR.md) | خطوات التثبيت عبر التوزيعات |
| [docs/PACKAGING.md](docs/PACKAGING.md) · [عربي](docs/PACKAGING_AR.md) | مواصفات التغليف متعدد التوزيعات ‏(.deb، .rpm، AppImage، Flatpak)‏ |
| [docs/DBUS_SPEC.md](docs/DBUS_SPEC.md) · [عربي](docs/DBUS_SPEC_AR.md) | مواصفات واجهة D-Bus Session Bus |
| [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) · [عربي](docs/TROUBLESHOOTING_AR.md) | المشاكل الشائعة والتشخيص وإصلاحات التسريع العتادي |
| [SECURITY.md](SECURITY.md) | نموذج الأمان وسلامة النقل وسياسات الشبكة |
| [CHANGELOG.md](CHANGELOG.md) | سِجل الإصدارات الكامل |
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
