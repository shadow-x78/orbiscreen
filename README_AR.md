<div align="center">

<a href="https://github.com/shadow-x78/orbiscreen">
  <img src="https://raw.githubusercontent.com/shadow-x78/orbiscreen/main/assets/logo/orbiscreen-banner-ar.png" alt="Orbiscreen - تحويل أي جهاز أو هاتف أندرويد إلى شاشة ثانية لنظام لينكس" width="100%" />
</a>

<br><br>

# Orbiscreen: تحويل جهاز Android إلى شاشة ثانية لنظام Linux

**شاشة افتراضية ثانية حقيقية ومستقلة وبزمن استجابة فائق السرعة لنظام Linux (Wayland و X11) تُبث إلى أجهزة وهواتف Android.**

[![الإصدار](https://img.shields.io/badge/version-0.22.7-2563eb?style=for-the-badge&logo=semver)](CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=for-the-badge)](LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=for-the-badge&logo=rust)
![المنصّة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=for-the-badge&logo=linux)
[![النجوم](https://img.shields.io/github/stars/shadow-x78/orbiscreen?style=for-the-badge&color=eab308&logo=github&label=النجوم)](https://github.com/shadow-x78/orbiscreen/stargazers)

<br>

<!-- أزرار النشر السريع للمشروع بنقرة واحدة -->
[![مشاركة على Reddit](https://img.shields.io/badge/مشاركة-Reddit-FF4500?style=flat-square&logo=reddit&logoColor=white)](https://www.reddit.com/submit?url=https%3A%2F%2Fgithub.com%2Fshadow-x78%2Forbiscreen&title=Orbiscreen%20-%20Turn%20any%20Android%20device%20into%20a%20low-latency%20second%20monitor%20for%20Linux)
[![مشاركة على X](https://img.shields.io/badge/مشاركة-X%2FTwitter-000000?style=flat-square&logo=x&logoColor=white)](https://twitter.com/intent/tweet?url=https%3A%2F%2Fgithub.com%2Fshadow-x78%2Forbiscreen&text=Turn%20any%20Android%20tablet%20or%20phone%20into%20a%20low-latency%20second%20monitor%20for%20Linux%20with%20Orbiscreen!%20%23Linux%20%23Rust%20%23OpenSource)
[![مشاركة على Hacker News](https://img.shields.io/badge/مشاركة-Hacker%20News-FF6600?style=flat-square&logo=ycombinator&logoColor=white)](https://news.ycombinator.com/submitlink?u=https%3A%2F%2Fgithub.com%2Fshadow-x78%2Forbiscreen&t=Orbiscreen%20-%20Turn%20Android%20into%20a%20low-latency%20second%20monitor%20for%20Linux)

</div>


---


## 🌐 اللغة

<a href="README.md">🇬🇧 English</a> · <a href="README_AR.md">🇸🇦 العربية</a>

---

## 📋 فهرس المحتويات

- [ما هو Orbiscreen؟](#what-is-orbiscreen)
- [مقارنة مع البدائل الأخرى](#comparison)
- [حالات الاستخدام الشائعة](#use-cases)
- [المميزات البارزة](#highlights)
- [دعم بيئات سطح المكتب](#desktop-support)
- [البدء السريع](#quick-start)
- [الأوامر المتاحة](#commands)
- [تطبيق الأندرويد](#android-app)
- [المعمارية التقنية](#architecture)
- [هيكل المشروع](#project-structure)
- [الأسئلة الشائعة (FAQ)](#faq)
- [التوثيق المكتبي](#documentation)
- [ادعم المشروع وانشره](#support)
- [المساهمة](#contributing)
- [الرخصة](#license)

---

<a id="what-is-orbiscreen"></a>
<h2 dir="rtl" align="right">&rlm;🤔 ما هو Orbiscreen؟</h2>

يحوّل **Orbiscreen** أي جهاز لوحي (تابلت) أو هاتف Android إضافي إلى شاشة عرض ثانية مستقلة وحقيقية لسطح مكتب Linux. يُنشئ التطبيق **شاشة افتراضية على مستوى النواة** عبر مشغّل `evdi` (في COSMIC و GNOME و X11)، أو **شاشة افتراضية أصيلة لمدير النوافذ** على KDE Plasma و wlroots دون الحاجة لصلاحيات root وبدون أي نوافذ تأكيد، ويقوم ببثها بصيغة **MPEG-TS/H.264** مع تحكم لمس عكسي متعدد، وفأرة، ولوحة مفاتيح، وقلم رسم بحساسية ضغط كاملة.

<a id="comparison"></a>
<h3 dir="rtl" align="right">&rlm;🆚 مقارنة Orbiscreen بالبدائل الأخرى</h3>

<div dir="rtl" align="right">

| الميزة / الإمكانية | Spacedesk | Deskreen | Weylus | Apple Sidecar | **Orbiscreen** |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **دعم نظام Linux كمستضيف** | ❌ (ويندوز فقط) | ✅ (عبر المتصفح) | ✅ (عبر المتصفح) | ❌ (macOS فقط) | **✅ مبني لنظام Linux أولاً** |
| **دعم Wayland و X11** | ❌ | ⚠️ (يتطلب Dummy Plug) | ⚠️ (شاشة مكررة فقط) | ❌ | **✅ أصيل على Wayland و X11** |
| **شاشة ثانية ممتدة حقيقية** | ✅ (ويندوز فقط) | ❌ (يحتاج وصلة وهمية) | ❌ (تكرار الشاشة فقط) | ✅ (أجهزة Apple فقط) | **✅ شاشة افتراضية حقيقية ومستقلة** |
| **تطبيق أندرويد أصلي** | ✅ | ❌ (متصفح ويب فقط) | ❌ (متصفح ويب فقط) | ❌ (أجهزة iPad فقط) | **✅ تطبيق أصلي (Jetpack Compose)** |
| **ترميز عتادي سريع** | ✅ | ❌ | ⚠️ | ✅ | **✅ تسريع عتادي NVENC و VA-API** |
| **زمن استجابة فائق السرعة** | ~50-80ms | ~150-300ms | ~80-120ms | ~30ms | **⚡ ~25-40ms** |
| **حساسية ضغط وميلان القلم** | ❌ | ❌ | ⚠️ (للقلم فقط) | ✅ | **✅ 4095 مستوى (Krita و GIMP)** |
| **لمس وفأرة وكيبورد عكسي** | ✅ | ❌ | ⚠️ (قلم فقط) | ✅ | **✅ تحكم متعدد، فأرة ومفاتيح** |
| **بدون روت (KDE و wlroots)** | غير متاح | ✅ | ❌ | غير متاح | **✅ بلا روت إطلاقاً (KWin / wlroots)** |
| **مفتوح المصدر** | ❌ (مغلق ومحتكر) | ✅ (GPL-3.0) | ✅ (AGPL-3.0) | ❌ (مغلق كلياً) | **✅ حر ومفتوح المصدر (GPL-3.0)** |

</div>

---

<a id="use-cases"></a>
## 🎯 حالات الاستخدام الشائعة

- 📱 **إعادة إحياء الهواتف والأجهزة اللوحية القديمة**: لا تدع جهازك اللوحي القديم (Samsung Galaxy Tab أو Xiaomi Pad أو Lenovo Tab) مهملاً؛ حوّله إلى شاشة ثانية مستقلة لزيادة إنتاجيتك.
- 🎨 **لوح رسم وتصميم رقمي (Graphic Tablet)**: حوّل جهازك وقلمك الذكي (S-Pen أو الأقلام السعوية) إلى لوح رسم رقمي يدعم **حساسية الضغط الحقيقية والميلان** في برامج لينكس الاحترافية مثل **Krita و GIMP و Blender و Inkscape**.
- 💻 **شاشة ثانية أثناء التنقل والسفر**: تنقّل بحرية دون الحاجة لحمل شاشات إضافية ثقيلة وهشة. وسّع شاشة لابتوبك في المقاهي ومساحات العمل المشتركة والفنادق بضغطة زر.
- 🖥️ **شاشة رأسية للقراءة والبرمجة**: أدِر الجهاز للوضع الرأسي (Portrait) لقراءة التوثيق والكتب البرمجية، وتصفح الأكواد، ومراقبة سجلات الطرفية (Logs)، ومتابعة محادثات Discord و Slack.
- ⚡ **أداء فائق واستقرار مطلق عبر كابل USB**: وصّل كابل USB فقط؛ يقوم نفق ADB التلقائي بنقل الإشارات بدون أي تداخل وبزمن استجابة فوري فائق السرعة.

---

<a id="highlights"></a>
## ✨ المميزات البارزة

- **واجهة إنشاء الشاشات الافتراضية عبر XDG Desktop Portal**: إنشاء شاشات افتراضية بدون صلاحيات root على GNOME 46+ و KDE Plasma 6+ عبر ScreenCast `SourceType::Virtual`، مع تراجع تلقائي إلى EVDI
- **لوح رسم رقمي متكامل مع تتبع التحليق والضغط وميلان القلم**: دعم 4095 مستوى ضغط، وتتبع زاوية الميلان، وتتبع مؤشر القلم في الهواء (Hover Cursor Tracking) محقونة في نواة لينكس عبر `uinput` لبرامج Krita و GIMP و Blender
- **إيماءة السحب والإفلات عبر لوحة اللمس (Touchpad Drag-and-Drop)**: النقر المزدوج مع السحب في وضع Touchpad لنقل النوافذ وسحب الملفات وتحديد النصوص بسهولة تامة، مع تقييد صارم للمؤشر داخل حدود الشاشة الافتراضية
- **تحسينات زمن الاستجابة الفائق لشبكات واي فاي 5GHz**: توليد الإطارات المفتاحية بمعدل 6 إطارات (كل 100ms) مع مسار تخزين مؤقت أقل من 50ms لاستعادة البث الفورية وانعدام التأخير
- **دعم متكامل لأجهزة Chromebook CM3001 ونظام ChromeOS**: استكشاف تلقائي لمسار ADB الداخلي عبر `100.115.92.2:5555` لتشغيل البث عبر USB فورياً على حاوية ARC++
- **حماية وتمهيد توكن الجلسة**: توليد توكن جلسة آمن مع تحديد صلاحيات ملف التوكن بـ `0o600`، ودعم التوثيق لمتصفحات الويب عبر تجزئة الرابط `#token=` والاستكشاف التلقائي عبر الشبكة المحلية
- **إدارة دورة حياة الاتصال واكتشاف الانقطاع اللحظي**: رصد صريح لانقطاع الاتصال وفحص لحظي عبر `/health` خلال 500ms مع حد أقصى 3 محاولات لمنع حلقات الوميض اللانهائية
- **شاشة افتراضية حقيقية عبر `evdi`** (X11 و Wayland)، **أو بدون أي root على KDE Plasma / wlroots**: مونيتور افتراضي يُنشئه KWin أو مخرج headless بدون وحدات نواة
- **عميل Android بواجهة Material 3**: مبني بأحدث معايير Jetpack Compose ولوحة ألوان Catppuccin المتطورة
- **تدوير تلقائي للدقة (Auto-Orientation)**: يتعرف تلقائياً على تدوير الجهاز اللوحي بين الوضع الأفقي والرأسي ويبدل الأبعاد فورياً
- **لوحة مفاتيح علوية واسعة من 3 أسطر**: تثبيت في أعلى الشاشة لضمان عدم حجب شريط المهام السفلي أو سجلات الأوامر
- **عميل ويب مبنّى داخلياً**: شاهد من أي متصفح على `http://<host>:8788/` (MSE عبر `mpegts.js` المضمنة محلياً دون الحاجة لإنترنت)
- **اكتشاف مباشر**: مسح mDNS / NSD للمضيفين القريبين مع اكتشاف لحظي
- **بث أصلي فائق السرعة**: ExoPlayer مع `OkHttpDataSource` وتوليف استجابة دقيق لمنع التراكم
- **لوحة تحكم وتحكم بالاستجابة**: شريط تحكم كامل يتضمن القفل والإطفاء و Ctrl+Alt+Del وإعادة المحاولة
- **نقل عبر USB**: بواسطة `adb reverse` مع دعم الاتصال الساخن التلقائي فور توصيل الكابل
- **ترميز عتادي متكامل**: NVIDIA NVENC، و Intel/AMD VA-API، مع تراجع برمجي لـ x264
- **توقيع تشفيري**: لكل حزم وتطبيقات Linux و Android

---

<a id="desktop-support"></a>
<h2 dir="rtl" align="right">&rlm;🖥️ دعم بيئات سطح المكتب</h2>

<div dir="rtl" align="right">

| البيئة | شاشة ثانية افتراضية | الالتقاط | الإدخال |
| :--- | :---: | :---: | :---: |
| KDE Plasma (Wayland) | ✅ أصلي عبر Portal Virtual / zkde-screencast (بدون root) | ✅ التقاط عبر PipeWire | ✅ تحكم عبر uinput و portal RemoteDesktop |
| COSMIC (Wayland) | ⚠️ شاشة افتراضية عبر EVDI | ✅ التقاط عبر portal (بث PipeWire) | ✅ تحكم كامل عبر uinput (لمس متعدد وقلم) |
| Sway / Hyprland / wlroots | ✅ مخرج headless أصيل عبر IPC (بدون root) | ✅ التقاط عبر wlr-screencopy (بدون حوار) | ✅ تحكم عبر virtual-pointer و virtual-keyboard (بدون portal) |
| GNOME (Wayland) | ✅ شاشة افتراضية عبر Portal (GNOME 46+) / EVDI | ✅ التقاط عبر portal ScreenCast (PipeWire) | ✅ تحكم عبر portal RemoteDesktop (بتوكن دائم) |
| XFCE / MATE / LXQt / Cinnamon (X11) | ✅ شاشة افتراضية عبر EVDI | ✅ التقاط عبر XShm للشاشة الجذرية | ✅ تحكم عبر XTEST و uinput (بدون root) |
| أي بيئة أخرى | ✅ عبر EVDI (تثبيت تلقائي عبر doctor --fix) | أفضل واجهة متاحة | أفضل واجهة متاحة |

</div>

يطبع `orbiscreen doctor` الـ compositor المكتشف وخطة الالتقاط التي سيتبعها `auto` وما الناقص في النظام؛ وينفّذ `orbiscreen doctor --fix` تثبيت وحدة نواة EVDI على التوزيعات المكتشفة. التفاصيل الكاملة في [دليل دعم بيئات سطح المكتب](docs/DE_SUPPORT_AR.md).

---

<a id="quick-start"></a>
## 🚀 البدء السريع

### 1. التثبيت

- **أوبونتو / Pop!_OS / لينكس مينت (Launchpad PPA):**
  ```bash
  sudo add-apt-repository ppa:shadow-x78/ppa -y
  sudo apt update
  sudo apt install orbiscreen -y
  ```

- **فيدورا (COPR):**
  ```bash
  sudo dnf copr enable shadow-x78/orbiscreen -y
  sudo dnf install orbiscreen -y
  ```

- **حزمة AppImage الشاملة لكافة التوزيعات (`.AppImage`):**
  حمّل الملف من [صفحة الإصدارات على GitHub](https://github.com/shadow-x78/orbiscreen/releases):
  ```bash
  chmod +x orbiscreen-x86_64.AppImage
  ./orbiscreen-x86_64.AppImage
  ```

- **مثبّت تلقائي بضغطة زر واحدة:**
  ```bash
  git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
  cd ~/Orbiscreen
  ./scripts/install.sh
  ```

- **تطبيق الأندرويد (`.apk`):**
  حمّل `orbiscreen-android-release.apk` من [صفحة الإصدارات](https://github.com/shadow-x78/orbiscreen/releases).

### 2. تشغيل Orbiscreen

- **من الطرفية (سطر الأوامر):**
  ```bash
  orbiscreen start
  ```

- **كخدمة خلفية (systemd user unit):**
  ```bash
  systemctl --user start orbiscreen
  # للتشغيل التلقائي عند تسجيل الدخول:
  systemctl --user enable orbiscreen
  ```

---

<a id="commands"></a>
## ⚙️ الأوامر المتاحة

```bash
# تشغيل خادم العرض الافتراضي باكتشاف البيئة التلقائي
orbiscreen start

# تشغيل بدقة وتردد محدد
orbiscreen start --width 1920 --height 1080 --fps 60

# فرض استخدام مرمّز عتادي محدد (nvenc أو vaapi أو x264)
orbiscreen start --encoder nvenc

# فحص توافق النظام والتحقق من المشغلات
orbiscreen doctor

# التثبيت التلقائي لوحدات النواة والاعتماديات الناقصة
orbiscreen doctor --fix

# إيقاف خادم العرض بأمان
orbiscreen stop
```

---

<a id="android-app"></a>
## 📱 تطبيق الأندرويد

- **اكتشاف mDNS فوري**: اكتشاف تلقائي لخوادم لينكس على نفس شبكة الواي فاي.
- **اتصال سلكي فائق عبر USB**: توصيل كابل USB مع نفق ADB التلقائي لصفر تداخل وأعلى استقرار.
- **دعم مخصص لأجهزة Chromebook و ChromeOS**: توجيه تلقائي لنفق ADB عبر الشبكة الداخلية لحاوية ARC++ (`100.115.92.2:5555`).
- **لوح رسم رقمي وتتبع القلم بالهواء**: تتبع حركة القلم أثناء التحليق في الهواء، وحساسية تامة لضغط القلم (4095 مستوى) وزوايا الميلان.
- **إيماءة السحب والإفلات**: إيماءة نقر مزدوج مع السحب في وضع لوحة اللمس لتحريك النوافذ وتحديد النصوص بسلاسة.
- **تحسين زمن الاستجابة لشبكات 5G**: مسار تخزين مؤقت بين 40 و 120ms مع قفزات مفتاحية كل 6 إطارات لاستجابة فورية بدون بطء.
- **اكتشاف صريح لانقطاع الاتصال**: فحص فوري خلال 500ms وحد أقصى لمحاولات إعادة الاتصال لمنع وميض الشاشة المتكرر.
- **تدوير تلقائي**: تبديل أبعاد الشاشة تلقائياً عند تدوير الهاتف أو التابلت.
- **لوحة مفاتيح علوية واسعة**: مكونة من 3 أسطر وتتضمن مفاتيح الوظائف والأسهم دون حجب محتوى الشاشة.
- **إعدادات دقيقة للمؤشر**: شريط مخصص للتحكم بدقة سرعة المؤشر واختيار الدقة فورياً.

---

<a id="architecture"></a>
## 🏛️ المعمارية التقنية

```
┌──────────────────────────────────────────────────────────────┐
│                      orbiscreen-daemon                       │
│  ┌────────────────────┐  ┌────────────────────────────────┐  │
│  │ orbiscreen-display │  │ orbiscreen-capture             │  │
│  │ (evdi kernel/DRM)  │  │ (zkde-screencast/wlr/ashpd)    │  │
│  └────────────────────┘  └────────────────────────────────┘  │
│             │                            │                   │
│             ▼                            ▼                   │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ orbiscreen-encode (GStreamer NVENC/VAAPI/x264)         │  │
│  └────────────────────────────────────────────────────────┘  │
│                              │                               │
│                              ▼                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ orbiscreen-transport (HTTP MPEG-TS + mDNS + ADB USB)   │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

---

<a id="project-structure"></a>
<h2 dir="rtl" align="right">&rlm;🏗️ هيكل المشروع</h2>

<div dir="rtl" align="right">
```
orbiscreen/
├── crates/
│   ├── orbiscreen-core/        # الأنواع المشتركة والإعدادات والأخطاء
│   ├── orbiscreen-display/     # الشاشات الافتراضية المدعومة بـ evdi
│   ├── orbiscreen-capture/     # التقاط X11 و Wayland (zkde-screencast / ashpd PipeWire)
│   ├── orbiscreen-encode/      # خط معالجة GStreamer مسرّع عتادياً (VAAPI / NVENC / x264)
│   ├── orbiscreen-input/       # اللمس والألواح الرقمية uinput وبوابة RemoteDesktop
│   ├── orbiscreen-transport/   # خادم axum واكتشاف mDNS ونفق USB التلقائي
│   └── orbiscreen-daemon/      # التطبيق الرئيسي الذي يربط كافة الطبقات معاً
├── clients/
│   ├── web/                    # عميل الويب للمتصفحات (HTML / CSS / JS)
│   └── android/                # تطبيق الأندرويد الأصلي (Material 3 Compose)
├── assets/
│   └── logo/                   # أيقونات وشعارات المشروع المتجهة (SVG)
├── data/                       # اختصار سطح المكتب وملفات الحزم وخدمة systemd
├── scripts/                    # سكربتات التثبيت وبناء الحزم (.deb / .rpm / AppImage)
└── docs/                       # التوثيق الشامل ثنائي اللغة (عربي وإنجليزي)
```

| المسار / المجلد | الوصف والوظيفة |
| :--- | :--- |
| `orbiscreen/` | المستودع الرئيسي للمشروع |
| ├── `crates/` | حزم ومكتبات Rust الأساسية للمشروع |
| │   ├── `orbiscreen-core/` | الأنواع المشتركة والإعدادات والتهيئة وإدارة الأخطاء |
| │   ├── `orbiscreen-display/` | الشاشات الافتراضية المدعومة بـ evdi و DRM |
| │   ├── `orbiscreen-capture/` | التقاط X11 و Wayland (&rlm;zkde-screencast / ashpd PipeWire) |
| │   ├── `orbiscreen-encode/` | خط معالجة GStreamer مسرّع عتادياً (&rlm;VAAPI / NVENC / x264) |
| │   ├── `orbiscreen-input/` | اللمس والألواح الرقمية uinput وبوابة RemoteDesktop |
| │   ├── `orbiscreen-transport/` | خادم axum واكتشاف mDNS ونفق USB التلقائي |
| │   └── `orbiscreen-daemon/` | التطبيق والخدمة الرئيسية التي تربط كافة الطبقات معاً |
| ├── `clients/` | تطبيقات العميل للأجهزة والمنصات المختلفة |
| │   ├── `web/` | عميل الويب للمتصفحات الحديثة (&rlm;HTML / CSS / JS) |
| │   └── `android/` | تطبيق الأندرويد الأصلي المبني بـ &rlm;Material 3 Compose |
| ├── `assets/` | الملفات والأصول الرسومية الثابتة |
| │   └── `logo/` | أيقونات وشعارات المشروع المتجهة (&rlm;SVG) |
| ├── `data/` | اختصار سطح المكتب وملفات الحزم وخدمة systemd |
| ├── `scripts/` | سكربتات التثبيت وبناء الحزم وتوليد الحزم (&rlm;.deb / .rpm / AppImage) |
| └── `docs/` | التوثيق التقني الشامل ثنائي اللغة (عربي وإنجليزي) |

</div>

---

<a id="faq"></a>
<h2 dir="rtl" align="right">&rlm;❓ الأسئلة الشائعة (FAQ)</h2>

<details>
<summary><b>هل يمكنني استخدام التابلت كشاشة ثانية ممتدة وليس مجرد تكرار (Mirror)؟</b></summary>
<br>
<b>نعم بالتأكيد!</b> على عكس برامج تكرار الشاشة، ينشئ Orbiscreen شاشة افتراضية حقيقية ومستقلة على نظام Linux (عبر تقنية KWin للشاشات الافتراضية في KDE، أو مخارج wlroots، أو مشغّل EVDI). يمكنك وضعها في أي اتجاه بجوار شاشتك الرئيسية، وسحب النوافذ إليها، وتعديل دقتها حتى 4K.
</details>

<details>
<summary><b>هل يعمل Orbiscreen على Wayland بدون صلاحيات root؟</b></summary>
<br>
<b>نعم!</b> على واجهات لينكس الحديثة مثل KDE Plasma (Wayland) ومدراء wlroots (مثل Sway و Hyprland)، ينشئ التطبيق شاشات افتراضية أصيلة بدون أي حاجة لصلاحيات root وبدون نوافذ مشاركة مزعجة. وعلى واجهات COSMIC و GNOME و X11 يستخدم المشغّل EVDI المعتمد.
</details>

<details>
<summary><b>هل يمكنني الرسم بالقلم الذكي بحساسية ضغط في برامج مثل Krita أو GIMP؟</b></summary>
<br>
<b>نعم!</b> يتعرف Orbiscreen على حساسية ضغط القلم الذكي (حتى 4095 مستوى) وزوايا الميلان من أجهزة مثل Samsung S-Pen والأقلام الرقمية ويحقنها كلوح رقمي رسمي في نواة لينكس، لتستمتع بالرسم الرقمي الاحترافي في Krita و GIMP و Blender.
</details>

<details>
<summary><b>كيف يقارن Orbiscreen بالحلول الأخرى؟</b></summary>
<br>
بخلاف الحلول الاحتكارية المقيدة بأنظمة معينة، يوفّر Orbiscreen بديلاً متفوقاً ومفتوح المصدر مخصصاً لنظام لينكس بالكامل ويعمل مع كافة أجهزة وهواتف أندرويد والمتصفحات.
</details>

<details>
<summary><b>هل يمكنني التوصيل عبر كابل USB بدلاً من شبكة Wi-Fi؟</b></summary>
<br>
<b>نعم!</b> يحتوي التطبيق على ميزة التوجيه التلقائي عبر ADB reverse. يكفي تفعيل تصحيح USB (USB Debugging) وتوصيل الكابل؛ سيتعرف النظام على الهاتف فورياً خلال ثانيتين وينشئ اتصالاً سلكياً فائق الاستقرار.
</details>

<details>
<summary><b>كم يبلغ زمن الاستجابة (Latency) أثناء البث؟</b></summary>
<br>
مع تفعيل الترميز العتادي (NVIDIA NVENC أو Intel/AMD VA-API على لينكس، وفك الترميز العتادي على أندرويد)، يأتي زمن الاستجابة منخفضاً واستثنائياً بدون تأخير ملحوظ، مما يعطي تجربة سلسة وفورية للبرمجة، والقراءة، والتصفح، ومتابعة الفيديوهات.
</details>

---

<a id="documentation"></a>
<h2 dir="rtl" align="right">&rlm;📚 التوثيق المكتبي</h2>

<div dir="rtl" align="right">

| الوثيقة | الوصف |
| :--- | :--- |
| [ARCHITECTURE_AR.md](docs/ARCHITECTURE_AR.md) | طوبولوجيا النظام وخط أنابيب الإطارات وبنية D-Bus |
| [DE_SUPPORT_AR.md](docs/DE_SUPPORT_AR.md) | مصفوفة دعم بيئات سطح المكتب وخطط الالتقاط واستكشاف الأخطاء |
| [PACKAGING_AR.md](docs/PACKAGING_AR.md) | مواصفات التغليف متعدد التوزيعات (.deb و .rpm و AppImage) |
| [DBUS_SPEC_AR.md](docs/DBUS_SPEC_AR.md) | مواصفات واجهة D-Bus Session Bus |
| [TROUBLESHOOTING_AR.md](docs/TROUBLESHOOTING_AR.md) | المشاكل الشائعة والتشخيص وإصلاحات التسريع العتادي |

</div>

---

<a id="support"></a>
## ⭐ ادعم المشروع وانشره في مجتمع لينكس

إذا ساعدك Orbiscreen في تحسين إنتاجيتك أو وفّر عليك شراء شاشة خارجية مكلفة:

- ⭐ **ضع نجمة (Star) للمستودع** على GitHub - كل نجمة ترفع من ترتيب المشروع في محركات بحث Google وتساعد مستخدمي لينكس الآخرين في الوصول إليه!
- 📢 **شارك المشروع مع مجتمع المصادر المفتوحة** على Reddit ([r/linux](https://reddit.com/r/linux)، [r/android](https://reddit.com/r/android)، [r/kde](https://reddit.com/r/kde)) أو منصات التواصل.
- 🐛 **أبلغ عن المشاكل واقترح الميزات** عبر [GitHub Issues](https://github.com/shadow-x78/orbiscreen/issues).
- 💡 **ساهم في التطوير والترجمة** عبر [Pull Requests](https://github.com/shadow-x78/orbiscreen/pulls).

---

<a id="contributing"></a>
## 🤝 المساهمة

1. اعمل Fork للمستودع
2. أنشئ فرعاً جديداً: `git checkout -b feature/my-feature`
3. التزم بالتغييرات: `orbiscreen | <type>: <description>`
4. ادفع إلى الفرع
5. افتح Pull Request

راجع [إرشادات المساهمة](CONTRIBUTING.md) و [ميثاق قواعد السلوك](CODE_OF_CONDUCT.md) لمعايير المجتمع وبيئة التطوير وعملية الإصدار.

---

<a id="license"></a>
## 📜 الرخصة

مرخّص تحت [رخصة GPL-3.0](LICENSE).

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[سجل التغييرات](CHANGELOG.md) ·
[ميثاق قواعد السلوك](CODE_OF_CONDUCT.md) ·
[الأمان](SECURITY.md)

<sub>&copy; 2026 Orbiscreen</sub>

</div>
