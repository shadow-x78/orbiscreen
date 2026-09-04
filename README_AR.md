<div align="center">

<a href="https://github.com/shadow-x78/orbiscreen">
  <img src="https://raw.githubusercontent.com/shadow-x78/orbiscreen/main/assets/logo/orbiscreen-banner-ar.png" alt="Orbiscreen - تحويل أي جهاز أو هاتف أندرويد إلى شاشة ثانية لنظام لينكس" width="100%" />
</a>

<br><br>

# Orbiscreen: تحويل جهاز Android إلى شاشة ثانية لنظام Linux

**شاشة افتراضية ثانية حقيقية ومستقلة وبزمن استجابة فائق السرعة لنظام Linux (Wayland و X11) تُبث إلى أجهزة وهواتف Android.**

[![الإصدار](https://img.shields.io/badge/version-0.18.3-2563eb?style=for-the-badge&logo=semver)](CHANGELOG.md)
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
## 🤔 ما هو Orbiscreen؟

يحوّل **Orbiscreen** أي جهاز لوحي (تابلت) أو هاتف Android إضافي إلى شاشة عرض ثانية مستقلة وحقيقية لسطح مكتب Linux. يُنشئ التطبيق **شاشة افتراضية على مستوى النواة** عبر مشغّل `evdi`، أو **شاشة افتراضية أصيلة لمدير النوافذ** على KDE Plasma و wlrootsدون الحاجة لصلاحيات root وبدون أي نوافذ تأكيد، ويقوم ببثها بصيغة **MPEG-TS/H.264** مع تحكم لمس عكسي متعدد، وفأرة، ولوحة مفاتيح، وقلم رسم بحساسية ضغط كاملة.

<a id="comparison"></a>
### 🆚 مقارنة Orbiscreen بالبدائل الأخرى

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

- **شاشة افتراضية حقيقية عبر `evdi`** (X11 *و* Wayland)، **أو بدون أي root على KDE Plasma**: مونيتور افتراضي يُنشئه KWin عبر `zkde-screencast` (بلا وحدة نواة وبلا نافذة مشاركة)، مع تراجع التقاط portal في غير ذلك
- **لوح رسم رقمي بحساسية ضغط وميلان القلم**: دعم كامل لـ 4095 مستوى ضغط على نواة لينكس عبر `uinput` لبرامج Krita و GIMP و Blender
- **تدوير تلقائي للدقة (Auto-Orientation)**: يتعرف تلقائياً على تدوير الجهاز اللوحي بين الوضع الأفقي والرأسي ويبدل الأبعاد فورياً
- **لوحة مفاتيح علوية واسعة من 3 أسطر**: تثبيت في أعلى الشاشة لضمان عدم حجب شريط المهام السفلي أو سجلات الأوامر
- **عميل Android بواجهة Material 3**: مبني بأحدث معايير Jetpack Compose ولوحة ألوان Catppuccin المتطورة
- **عميل ويب مبنّى داخلياً**: شاهد من أي متصفح على `http://<host>:8788/` (MSE عبر `mpegts.js` المضمنة محلياً دون الحاجة لإنترنت)
- **اكتشاف مباشر**: مسح mDNS / NSD للمضيفين القريبين مع اكتشاف لحظي
- **بث أصلي منخفض الكمون**: ExoPlayer مع `DefaultLoadControl` لبث فائق السرعة
- **حماية بالتوكن**: تشفير والتحقق من الجلسات بتوكن دوار يتم توليده مع كل تشغيل
- **نقل عبر USB**: بواسطة `adb reverse` مع دعم الاتصال الساخن التلقائي فور توصيل الكابل
- **ترميز عتادي متكامل**: NVIDIA NVENC، و Intel/AMD VA-API، مع تراجع برمجي لـ x264
- **توقيع تشفيري**: لكل حزم وتطبيقات Linux و Android

---

<a id="desktop-support"></a>
## 🖥️ دعم بيئات سطح المكتب

| البيئة | شاشة ثانية افتراضية | الالتقاط | الإدخال |
|--------|---------------------|----------|---------|
| KDE Plasma (Wayland) | ✅ أصلي عبر zkde-screencast (بدون root) | ✅ التقاط عبر PipeWire | ✅ تحكم عبر uinput و portal RemoteDesktop |
| Sway / Hyprland / wlroots | ✅ مخرج headless أصيل عبر IPC (بدون root) | ✅ التقاط عبر wlr-screencopy (بدون حوار) | ✅ تحكم عبر virtual-pointer و virtual-keyboard (بدون portal) |
| GNOME (Wayland) | ⚠️ شاشة افتراضية عبر EVDI | ✅ التقاط عبر portal (حوار لمرة واحدة فقط) | ✅ تحكم عبر portal RemoteDesktop (بتوكن دائم) |
| XFCE / MATE / LXQt / Cinnamon (X11) | ✅ شاشة افتراضية عبر EVDI | ✅ التقاط عبر XShm للشاشة الجذرية | ✅ تحكم عبر XTEST و uinput (بدون root) |
| أي بيئة أخرى | ✅ عبر EVDI (تثبيت تلقائي عبر doctor --fix) | أفضل واجهة متاحة | أفضل واجهة متاحة |

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

- **من قائمة التطبيقات مباشرة (بدون سطر أوامر):**
  ابحث عن **Orbiscreen** في قائمة التطبيقات على حاسوبك واضغط عليها لبدء البث فوراً!
  يمكنك النقر بالزر الأيمن على الأيقونة في أي وقت للإيقاف أو إجراء الفحص.

- **من سطر الأوامر:**
  ```bash
  orbiscreen start
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
- **لوح رسم رقمي**: استجابة تامة لضغط وميلان القلم الذكي (S-Pen) متوافقة مع برامج الرسم مثل Krita.
- **تدوير تلقائي**: تبديل أبعاد الشاشة تلقائياً عند تدوير الهاتف أو التابلت.
- **لوحة مفاتيح علوية واسعة**: مكونة من 3 أسطر وتتضمن مفاتيح الوظائف والأسهم دون حجب محتوى الشاشة.
- **لوحة تحكم وتحكم بالسرعة**: شريط مخصص للتحكم بدقة سرعة المؤشر واختيار الدقة فورياً.

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
## 🏗️ هيكل المشروع

```
orbiscreen/
├── crates/
│   ├── orbiscreen-core/        # الأنواع المشتركة والإعدادات والأخطاء
│   ├── orbiscreen-display/     # الشاشات الافتراضية المدعومة بـ evdi
│   ├── orbiscreen-capture/     # التقاط X11 و Wayland (zkde-screencast / ashpd PipeWire)
│   ├── orbiscreen-encode/      # خط معالجة GStreamer (VAAPI / NVENC / x264)
│   ├── orbiscreen-input/       # اللمس والألواح الرقمية uinput وبوابة RemoteDesktop
│   ├── orbiscreen-transport/   # خادم axum واكتشاف mDNS ونفق USB التلقائي
│   └── orbiscreen-daemon/      # التطبيق الرئيسي الذي يربط كافة الطبقات معاً
├── clients/
│   ├── web/                    # عميل الويب للمتصفحات (HTML / CSS / JS)
│   └── android/                # تطبيق الأندرويد الأصلي (Material 3 Compose)
├── assets/
│   └── logo/                  # أيقونات وشعارات المشروع المتجهة (SVG)
├── data/                      # اختصار سطح المكتب وملفات الحزم وخدمة systemd
├── scripts/                   # سكربتات التثبيت وبناء الحزم (.deb / .rpm / AppImage)
└── docs/                      # التوثيق الشامل ثنائي اللغة (عربي وإنجليزي)
```

---

<a id="faq"></a>
## ❓ الأسئلة الشائعة (FAQ)

<details>
<summary><b>هل يمكنني استخدام التابلت كشاشة ثانية ممتدة وليس مجرد تكرار (Mirror)؟</b></summary>
<br>
<b>نعم بالتأكيد!</b> على عكس برامج تكرار الشاشة، ينشئ Orbiscreen شاشة افتراضية حقيقية ومستقلة على نظام Linux (عبر تقنية KWin للشاشات الافتراضية في KDE، أو مخارج wlroots، أو مشغّل EVDI). يمكنك وضعها في أي اتجاه بجوار شاشتك الرئيسية، وسحب النوافذ إليها، وتعديل دقتها حتى 4K.
</details>

<details>
<summary><b>هل يعمل Orbiscreen على Wayland بدون صلاحيات root؟</b></summary>
<br>
<b>نعم!</b> على واجهات لينكس الحديثة مثل KDE Plasma (Wayland) ومدراء wlroots (مثل Sway و Hyprland)، ينشئ التطبيق شاشات افتراضية أصيلة بدون أي حاجة لصلاحيات root وبدون نوافذ مشاركة مزعجة. وعلى واجهات GNOME و X11 يستخدم المشغّل EVDI المعتمد.
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
## 📚 التوثيق المكتبي

| الوثيقة | الوصف |
|---------|-------|
| [ARCHITECTURE_AR.md](docs/ARCHITECTURE_AR.md) | طوبولوجيا النظام وخط أنابيب الإطارات وبنية D-Bus |
| [DE_SUPPORT_AR.md](docs/DE_SUPPORT_AR.md) | مصفوفة دعم بيئات سطح المكتب وخطط الالتقاط واستكشاف الأخطاء |
| [PACKAGING_AR.md](docs/PACKAGING_AR.md) | مواصفات التغليف متعدد التوزيعات (.deb و .rpm و AppImage) |
| [DBUS_SPEC_AR.md](docs/DBUS_SPEC_AR.md) | مواصفات واجهة D-Bus Session Bus |
| [TROUBLESHOOTING_AR.md](docs/TROUBLESHOOTING_AR.md) | المشاكل الشائعة والتشخيص وإصلاحات التسريع العتادي |

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

راجع [إرشادات المساهمة](CONTRIBUTING.md) لبيئة التطوير وأسلوب الكود وعملية الإصدار.

---

<a id="license"></a>
## 📜 الرخصة

مرخّص تحت [رخصة GPL-3.0](LICENSE).

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[سجل التغييرات](CHANGELOG.md) ·
[الأمان](SECURITY.md)

<sub>&copy; 2026 Orbiscreen</sub>

</div>
