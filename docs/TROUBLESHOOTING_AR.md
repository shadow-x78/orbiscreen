<div align="center">

# استكشاف الأخطاء وإصلاحها - Orbiscreen

[![الإصدار](https://img.shields.io/badge/version-0.23.6-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![المنصّة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 اللغة

<a href="TROUBLESHOOTING.md">🇬🇧 English</a> · <a href="TROUBLESHOOTING_AR.md">🇸🇦 العربية</a>

---

## 📋 المحتويات

### إجراءات سير عمل CI (`‎.github/workflows/ci.yml`)

- [الإجراء: `Check formatting` ‏(`cargo fmt --all -- --check`)](#ci-fmt)
- [الإجراء: `Clippy (deny warnings)` ‏(`cargo clippy --workspace --all-targets --locked -- -D warnings`)](#ci-clippy)
- [الإجراء: `Build` ‏(`cargo build --workspace --locked`)](#ci-build)
- [الإجراء: `Test` ‏(`cargo test --workspace --locked`)](#ci-test)
- [الإجراء: `Run cargo-deny` ‏(`cargo deny check`)](#ci-deny)
- [الإجراء: `Android assembleDebug` + `lintDebug`](#ci-android)

### وقت التشغيل

- [وقت التشغيل: فشل `orbiscreen start` - `kernel module is not installed`](#runtime-evdi)
- [وقت التشغيل: KDE Plasma (شاشة افتراضية بدون evdi وبدون root)](#runtime-kwin)
- [وقت التشغيل: واجهة الالتقاط غير متاحة على Wayland](#runtime-wayland)
- [وقت التشغيل: تحذيرات lint `unsafe_op_in_unsafe_fn` / `missing_debug_implementations`](#runtime-lints)

### Android

- [Android / ChromeOS: فشل اتصال ADB أو بقاء الرسالة "Looking for host" على ASUS Chromebook CM3001](#android-chromebook-adb)
- [Android: القلم لا يرسم، أو حساسية ضغط غير صحيحة، أو توقف التطبيق على Lenovo Tab](#android-stylus)
- [Android: سحب النوافذ وتحديد النصوص في وضع لوحة اللمس (Touchpad Drag-and-Drop)](#android-touchpad-drag)
- [Android: التطبيق يتعطل أو تموت العملية عند النقر على Connect](#android-connect-crash)
- [Android: شاشة سوداء بعد Connect](#android-black-screen)
- [Android: قائمة الاكتشاف فارغة رغم وجود مضيفين على نفس شبكة Wi-Fi](#android-no-hosts)
- [Android: اللمس مُدوَّر / غير محاذٍ](#android-touch-offset)
- [Android: إجراءات شريط التحكم تُرجع 404](#android-control-404)
- [Android: التطبيق يتعطل فوراً عند التشغيل](#android-crash)
- [Android: اتصال USB يعرض "Looking for host…"](#android-usb)

### البث والعملاء

- [البث: بطء شديد أو تقطيع في حركة الفأرة عبر شبكة 5GHz Wi-Fi](#streaming-wifi-latency)
- [البث: وميض وإعادة اتصال لانهائية عند حدوث خطأ في البث بدل التعرف على انقطاع الاتصال](#stream-disconnect-retry)
- [تعدد الشاشات / X11: هروب مؤشر الفأرة من الشاشة الافتراضية إلى الشاشات المادية الأخرى](#cursor-clamping)
- [العميل يعرض الشاشة الخطأ (سطح المكتب الرئيسي بدل الشاشة الافتراضية)](#wrong-screen)
- [عميل الويب يُحمَّل لكن بلا صورة](#web-no-picture)
- [لا يوجد مُرمَّز - البث يبدأ لكنه يفشل (غياب x264)](#no-encoder)
- [رفض 401 من `/stream` أو `/input` أو `/api/control` (التوكن)](#token-401)
- [الـ daemon غير موجود على D-Bus](#dbus-missing)

### الـ Daemon

- [الـ Daemon: استهلاك 100% للمعالج أو تجمّد](#daemon-cpu)

### ما زلت عالقاً؟

- [ما زال البناء يفشل؟ راجع سجلات الإجراء](#still-stuck)
- [إعادة تشغيل مهمة CI واحدة](#re-run-job)
- [التحقق من بث حي من طرف إلى طرف ‏(`scripts/verify-stream.sh`)](#verify-stream)
- [تجهيز بيئة تطوير ‏(`scripts/setup-dev-env.sh`)](#setup-dev-env)

---

<a id="ci-fmt"></a>
## 🧪 إجراء CI: `Check formatting` ‏(`cargo fmt --all -- --check`)

**العرَض:**
```
Diff in /path/to/file.rs:
   println!("x");
-  println!("y");
+  println!("z");
```

**السبب:**
ملفات مصدر Rust لا تطابق تنسيق `cargo fmt`.

**الإصلاح:**
```bash
cargo fmt --all
git add -A
git commit -m "orbiscreen | v0.23.6 | style: cargo fmt --all"
```

**الوقاية:**
شغّل `./gradlew :app:lintDebug` و `cargo fmt --all` محلياً قبل الدفع.

---

<a id="ci-clippy"></a>
## 🧪 إجراء CI: `Clippy (deny warnings)`

**العَرَض:**
```
error: this operation is not supported for derived errors
  --> src/lib.rs:42:5
```

**السبب:**
يعامل أمر `cargo clippy -D warnings` كل تحذيرات clippy كأخطاء توقف البناء.

**الحل:**
```bash
cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 | head -50
cargo clippy --workspace --all-targets --locked --fix
git add -A
git commit -m "orbiscreen | v0.23.6 | fix: resolve clippy warnings"
```

**الوقاية:**
شغّل `cargo clippy` محلياً قبل الدفع.

---

<a id="ci-build"></a>
## 🧪 إجراء CI: `Build` (`cargo build --workspace --locked`)

**العَرَض:**
```
error[E0463]: can't find crate for `gstreamer`
```

**الحل:**
```bash
cargo update -p gstreamer
cargo build --workspace --locked
git add Cargo.lock
git commit -m "orbiscreen | v0.23.6 | chore: refresh Cargo.lock"
```

---

<a id="ci-test"></a>
## 🧪 إجراء CI: `Test` ‏(`cargo test --workspace --locked`)

تفترض الاختبارات وجود إضافات GStreamer على المضيف (`x264enc`، `vaapih264enc`، `nvh264enc`). ثبّتها محلياً:
```bash
sudo dnf install gstreamer1.0-plugins-{good,bad,ugly,libav}
```

---

<a id="ci-deny"></a>
## 🧪 إجراء CI: `Run cargo-deny`

هذا فحص **غير مانع** لأغراض معلوماتية. راجع `deny.toml` لقائمة السماح.

---

<a id="ci-android"></a>
## 🧪 إجراء CI: `Android assembleDebug` + `lintDebug`

يشغّل سير عمل Android الأمر `./gradlew :app:assembleDebug :app:lintDebug`. الإخفاقات الشائعة:

- **خطأ lint الخاص بـ UnstableApi:** يشترك `clients/android/app/lint.xml` في `androidx.media3.common.util.UnstableApi`. إذا ناديت واجهة Media3 جديدة، تأكد من تعليم الصنف المحيط بـ `@OptIn(UnstableApi::class)`.
- **استيرادات Compose:** شغّل `./gradlew :app:compileDebugKotlin` لتحديد مكان الخطأ أولاً؛ فالـ lint أبطأ.

---

<a id="runtime-evdi"></a>
## 🚀 وقت التشغيل: فشل `orbiscreen start` - `kernel module is not installed`

**العرَض:**
```
Error: evdi kernel module is not installed
```

**الإصلاح:**
1. ثبّت `evdi` (بناء DKMS) على المضيف:
   ```bash
   # Fedora / Nobara
   sudo dnf install dkms gcc make kernel-devel-$(uname -r) displaylink
   sudo modprobe evdi
   ```
   ```bash
   # Ubuntu / Pop!_OS
   sudo apt install dkms
   git clone https://github.com/DisplayLink/evdi.git
   cd evdi && sudo make dkms-install
   sudo modprobe evdi
   ```
2. تحقق:
   ```bash
   lsmod | grep evdi
   ls /dev/dri/card*
   ```

---

<a id="runtime-kwin"></a>
## 🚀 وقت التشغيل: KDE Plasma (شاشة افتراضية بدون evdi وبدون root)

**الأعراض:** يسجّل `orbiscreen start` رسالة `EVDI kernel module not active` ولا تريد بناء وحدة نواة.

**الحل:** على KDE Plasma Wayland لا شيء إضافي مطلوب. مع الإعداد الافتراضي `[capture] preferred = "auto"` يطلب الـ daemon من KWin إنشاء مونيتور افتراضي (`Virtual-ORBISCREEN`، يظهر في إعدادات النظام ← إعداد العرض) عبر بروتوكول Wayland‏ `zkde_screencast_unstable_v1` ويبثه عبر PipeWire مباشرة، بلا root وبلا نافذة مشاركة. KWin لا يعرض هذا البروتوكول إلا للتنفيذيات المصرّح لها، لذا يحافظ الـ daemon على الملف `~/.local/share/applications/orbiscreen.kwin.desktop` (قابل للكتابة من المستخدم) ويحدّث ذاكرة KService تلقائياً؛ قد يستغرق التشغيل الأول ثوانٍ إضافية حتى يصبح الترخيص مرئياً.

ملاحظات:
- يمكن فرض المسار عبر `[capture] preferred = "kwin-virtual"` (فشل صريح إن لم يتوفر) أو `"portal"` (إظهار نافذة المشاركة دائماً).
- تختفي الشاشة الافتراضية عند إيقاف الـ daemon؛ هذا متوقع.
- **ترى خلفية سطح المكتب فقط في البث؟** هذا صحيح: الشاشة الافتراضية هي *شاشة ثانية فارغة*. اسحب النوافذ إلى `Virtual-ORBISCREEN`، أو اجعل `[capture] preferred = "mirror"` لبث شاشتك الحقيقية بدلاً منها.
- على GNOME / wlroots البروتوكول غير موجود ويرجع `auto` تلقائياً إلى نافذة مشاركة portal.
- أصبح EVDI اختيارياً (`preferred = "evdi"`)؛ لا يلمسه `auto` على Wayland إطلاقاً، لذا لن يظهر سطر `EVDI kernel module not active` القديم على KDE.

---

<a id="runtime-wayland"></a>
## 🚀 وقت التشغيل: واجهة الالتقاط غير متاحة على Wayland

استخدم `CaptureSession::open_with_preference()` (الـ daemon يفعل ذلك مسبقاً).

---

<a id="runtime-lints"></a>
## 🚀 وقت التشغيل: `unsafe_op_in_unsafe_fn` / `missing_debug_implementations`

استخدم `#[allow(missing_debug_implementations)]` أو `#[allow(unsafe_code)]` على النوع أو الدالة المعنية.

---

## 📱 أجهزة وعميل Android

<a id="android-chromebook-adb"></a>
### Android / ChromeOS: فشل اتصال ADB أو بقاء الرسالة "Looking for host" على ASUS Chromebook CM3001

**العَرَض:**
عند تشغيل تطبيق Orbiscreen على جهاز ASUS Chromebook CM3001 (أو أجهزة ChromeOS الأخرى)، يعلق التطبيق في وضع USB على "Looking for host" ولا يكتشف خادم لينكس.

**السبب:**
يعزل نظام ChromeOS تطبيقات أندرويد داخل حاوية ARC++ مع نطاق شبكة فرعي خاص (`100.115.92.0/28`). توجيهات `adb reverse` القياسية الموجهة إلى `127.0.0.1` داخل حاوية لينكس Crostini لا تصل إلى تطبيقات أندرويد بدون مسار توجيه داخلي.

**الحل:**
- يقوم Orbiscreen v0.20.0 تلقائياً بفحص وتوجيه المنفذ الداخلي لبوابة ARC++ على `100.115.92.2:5555` إلى جانب `localhost:5555`.
- في إعدادات ChromeOS، انتقل إلى **خيارات متقدمة** -> **المطورون** -> **تطوير تطبيقات Android** وفعّل **تصحيح أخطاء ADB**.
- أعد تشغيل الجهاز إن طُلب منك ذلك، ثم شغّل `orbiscreen start` داخل طرفية لينكس، وسيتصل التطبيق فورياً عبر USB.

---

<a id="android-stylus"></a>
### Android: القلم لا يرسم، أو حساسية ضغط غير صحيحة، أو توقف التطبيق على Lenovo Tab

**العَرَض:**
عند استخدام القلم الذكي على أجهزة مثل Lenovo Tab (IdeaTab) أو Chromebook:
1. يتجمد التطبيق أو ينهار مع خطأ `NetworkOnMainThreadException` بمجرد تحريك القلم.
2. غياب مؤشر القلم عند التحليق في الهواء فوق الشاشة.
3. خطأ في زوايا الميلان أو بقاء الضغط معلقاً عند رفع القلم.

**السبب:**
في الإصدارات السابقة، كانت حزم شبكة القلم تُرسل مباشرة على الخيط الرسومي الرئيسي لتطبيق أندرويد. كما كانت واجهة الاستماع لحركة التحليق `ACTION_HOVER_MOVE` غير مفعلة، ولم تكن إشارة رفع القلم تُرسل عند وصول الضغط إلى صفر.

**الحل:**
- حدّث التطبيق إلى الإصدار **v0.20.0** أو أحدث.
- ينقل v0.20.0 معالجة حزم القلم بالكامل إلى خلفية غير متزامنة عبر `Dispatchers.IO` مع دمج الأحداث السريعة عبر `latestStylus` لمنع تجمد الواجهة أو انهيار التطبيق.
- يفعل `setOnGenericMotionListener` لتتبع حركة المؤشر في الهواء أثناء تحليق القلم.
- يصحح معادلة زاوية الميلان (`-altitudeDeg * cos(orientationRad)`) ويرسل إشارة `BTN_TOOL_PEN: RELEASED` عند رفع القلم وانعدام الضغط.

---

<a id="android-touchpad-drag"></a>
### Android: سحب النوافذ وتحديد النصوص في وضع لوحة اللمس (Touchpad Drag-and-Drop)

**العَرَض:**
في وضع لوحة اللمس (Touchpad)، عند النقر والسحب على شاشة التابلت يتحرك المؤشر فقط دون سحب النوافذ أو تحديد النصوص.

**السبب:**
كان وضع لوحة اللمس سابقاً يفسر جميع الحركات كمجرد تحريك للمؤشر دون دعم إيماءة السحب.

**الحل:**
- التحديث إلى **v0.20.0**.
- **النقر المزدوج مع السحب:** انقر مرتين سريعاً على الشاشة مع إبقاء إصبعك مضغوطاً في النقرة الثانية. أثناء تحريك إصبعك، يظل زر الفأرة الأيسر مضغوطاً لسحب النوافذ أو نقل الملفات أو تحديد النصوص بسلاسة.
- رفع إصبعك عن الشاشة يحرر زر الفأرة فوراً.

---

<a id="android-connect-crash"></a>
### Android: التطبيق يتعطل أو تموت العملية عند النقر على Connect

**العرَض:**
النقر على مضيف في شاشة Discovery يقتل فوراً عملية التطبيق أو يعيده إلى المشغّل (launcher).

**السبب:**
يجب إنشاء `PlayerHolder.build()` على الخيط الرئيسي. يتطلب ExoPlayer الإنشاء على الخيط الرئيسي؛ وإنشاء مكوّنات المشغّل على خيوط IO يرمي استثناءات وصول خيطي تنهي العملية.

**الإصلاح:**
- حدّث إلى `orbiscreen-android-release.apk` الإصدار **v0.20.0** أو أحدث.
- ينشئ `StreamViewModel` مشغّل ExoPlayer على `Dispatchers.Main` مع تحصين عبر try-catch لتظهر الأخطاء كبطاقة `StreamEvent.Error` قابلة لإعادة المحاولة بدل الانهيار.

---

<a id="android-black-screen"></a>
### Android: شاشة سوداء بعد Connect

**العرَض:**
النقر على مضيف مكتشف يعرض سطحاً أسود؛ لا فيديو؛ ولا يظهر شريط التحكم.

**السبب:**
محاولة ExoPlayer التعرف التلقائي على نوع الوسائط (MIME sniffing) لمسار `/stream` والتراجع لسطح أسود عند تعذر الكشف التلقائي.

**الإصلاح:**
- حدّث إلى `orbiscreen-android-release.apk` الإصدار **v0.20.0** أو أحدث.
- يقوم `PlayerHolder` بضبط `MediaItem` بنوع صريح `setMimeType(MimeTypes.VIDEO_MP2T)` لفك ترميز البث مباشرة دون الحاجة للتعرف التلقائي.
- تظهر الأخطاء كبطاقة إعادة محاولة واضحة بدل السطح الأسود.

إذا استمرت المشكلة بعد التحديث:
1. تأكد من إمكانية الوصول إلى المضيف عبر `curl http://host:8788/health` من نفس شبكة Wi-Fi.
2. تأكد من استجابة `/api/info`: `curl http://host:8788/api/info`.
3. افحص `adb logcat -s OrbiPlayer:*` بحثاً عن أسطر `player error:`.

---

<a id="android-no-hosts"></a>
### Android: قائمة الاكتشاف فارغة رغم وجود مضيفين على نفس شبكة Wi-Fi

**السبب:**
بروتوكول mDNS محظور على الشبكة (شبكات العمل المقيدة، جدار الحماية، إلخ).

**الإصلاح:**
1. افتح بطاقة **Add manually** وأدخل `host:port` (مثال `192.168.1.50:8788`).
2. اختياري: فعّل خيار **Scan subnet for hosts** في **Settings**. يفحص الماسح نطاق ‎/24 عبر TCP ويضيف أي جهاز يستجيب على المنفذ 8788.

---

<a id="android-touch-offset"></a>
### Android: اللمس مُدوَّر / غير محاذٍ

**السبب:**
يعتمد تعيين المؤشر إلى المضيف على دقة الشاشة المُبلَّغ عنها من `/api/info`. إذا كان المضيف مُدوَّراً (مثلاً شاشة افتراضية عمودية) لكن JSON ما زال يبلّغ عن اتجاه أفقي، فسيكون التعيين غير متطابق.

**الإصلاح:**
دوّر المضيف بدلاً من شاشة Android. يطبّق `PlayerView` ضبط النطاق تلقائياً للحفاظ على نسبة العرض إلى الارتفاع المحددة.

---

<a id="android-control-404"></a>
### Android: إجراءات شريط التحكم تُرجع 404

**السبب:**
المضيف يشغّل إصداراً أقدم من الخدمة لا يدعم نقطة `/api/control`.

**الإصلاح:**
أعد تشغيل الخدمة على المضيف:
```bash
orbiscreen stop
orbiscreen start
```

---

<a id="android-crash"></a>
### Android: التطبيق يتعطل فوراً عند التشغيل

**العرَض:**
تفتح تطبيق Orbiscreen على Android فيتعطل فوراً ويعود إلى الشاشة الرئيسية.

**السبب:**
مشاكل سابقة متعلقة بـ WebView في الإصدارات القديمة.

**الإصلاح:**
يعتمد تطبيق Orbiscreen على Jetpack Compose + `PlayerView` أصيلاً دون WebView. تأكد من تثبيت `orbiscreen-android-release.apk` الإصدار **v0.20.0** أو أحدث. إذا واجهت أي مشكلة، التقط السجل عبر `adb logcat *:E | grep orbiscreen` وافتح بلاغاً في GitHub.

---

<a id="android-usb"></a>
### Android: اتصال USB يعرض "Looking for host…"

**الإصلاح:**
يدير Orbiscreen دورة حياة `adb reverse` كاملة من تلقاء نفسه: ينشئ النفق على كل جهاز متصل عند بدء الدامن، يلتقط جهازاً جديداً موصولاً خلال ثانيتين (hot-plug)، يعيد إنشاء نفق مات مع خروج غير نظيف من الدامن (الإنشاء idempotent)، ويزيل كل الأنفاق عند الإيقاف الرشيق. تأكد من:
1. تفعيل **USB Debugging** في خيارات مطوّر Android.
2. تخويل جهاز المضيف في رسالة التأكيد على هاتفك/جهازك اللوحي.
3. تحقق مما يراه الدامن:
   ```bash
   orbiscreen doctor          # يطبع سطر usb: وجود adb؟ الأجهزة؟ الأنفاق النشطة؟
   adb devices
   adb reverse --list
   ```
4. انقر بطاقة **USB mode** في شاشة Discovery. تفحص البطاقة `http://127.0.0.1:8788/health` وتعرض الحالة الحية: **النفق جاهز** (علامة خضراء) أو **لا نفق** (شغّل الدامن على المضيف أو أعد توصيل الكابل).

عدد الأنفاق لدى الدامن مرئي في أي لحظة عبر `GET /health`‏ (الحقل `usb_devices`) وفي حمولة `GetStatus` عبر D-Bus.

<a id="streaming-wifi-latency"></a>
## ⚡ البث: بطء شديد أو تقطيع في حركة الفأرة عبر شبكة 5GHz Wi-Fi

**العَرَض:**
عند الاتصال عبر شبكة واي فاي 5GHz (مثل أجهزة Lenovo Tab أو الهواتف)، تبدو حركة الفأرة ثقيلة جداً أو متأخرة بفارق زمني ملحوظ، أو يتأخر بث الشاشة عن المضيف.

**السبب:**
1. استخدام ذاكرة تخزين مؤقت كبيرة لتشغيل الفيديو يؤدي إلى تراكم الإطارات وزيادة التأخير في البث التفاعلي المباشر.
2. تباعد الإطارات المفتاحية (GOP) يجبر المشغل على الانتظار طويلاً عند فقدان أي حزمة بيانات عبر الشبكة اللاسلكية.
3. تجميع أحداث الفأرة بفاصل زمني طويل نسبياً.

**الحل:**
- يقوم الإصدار v0.20.0 بضبط مسار البث بالكامل لأدنى كمون واستجابة فورية:
  - **إطار مفتاحي كل 6 إطارات (GOP 6):** ترسل المرمّزات العتادية إطاراً مفتاحياً كل 100ms، مما يتيح استعادة البث اللحظية فور حدوث أي تشويش دون تراكم.
  - **توليف ذاكرة ExoPlayer (40-120ms):** تخفيض التخزين المؤقت إلى 40ms كحد أدنى و 120ms كحد أقصى للحفاظ على البث المباشر اللحظي.
  - **حلقة إرسال الفأرة 8ms:** تقليص نافذة تجميع الفأرة إلى 8ms لمنح استجابة فائقة تماثل شاشات 120Hz.
- تأكد من ضبط راوتر Wi-Fi 5GHz على قناة غير مزدحمة وبعرض نطاق 80MHz.

---

<a id="stream-disconnect-retry"></a>
## 🔁 البث: وميض وإعادة اتصال لانهائية عند حدوث خطأ في البث بدل التعرف على انقطاع الاتصال

**العَرَض:**
عند إيقاف خادم لينكس أو تعطل الشبكة، يظل تطبيق أندرويد يومض ويحاول إعادة الاتصال بلا نهاية دون إظهار شاشة توقف واضحة.

**السبب:**
كانت المشغلات تفتقر إلى حالة انقطاع صريحة، وتستمر في محاولات الاتصال دون حد أقصى.

**الحل:**
- في v0.20.0، أُضيفت حالة صريحة `StreamEvent.Disconnected`.
- فور حدوث خطأ في الشبكة، يُطلق التطبيق فحصاً سريعاً خلال 500ms لنقطة `/health` للتأكد من حالة الخادم.
- حُددت محاولات إعادة الاتصال بـ 3 محاولات فقط؛ وعند تعذر الوصول للخادم يعرض التطبيق بطاقة انقطاع الاتصال مع زر لإعادة المحاولة اليدوية.

---

<a id="cursor-clamping"></a>
## 🖥 تعدد الشاشات / X11: هروب مؤشر الفأرة من الشاشة الافتراضية إلى الشاشات المادية الأخرى

**العَرَض:**
عند تحريك الفأرة أو القلم على التابلت، يقفز المؤشر خارج حدود الشاشة الافتراضية إلى شاشة اللابتوب أو الشاشات المادية الأخرى.

**السبب:**
حقن إحداثيات XTEST بدون تقييد أبعاد المخرج يمتد على كامل مساحة سطح المكتب المجمعة.

**الحل:**
- في v0.20.0، يستعلم Orbiscreen عن أبعاد الشاشة الافتراضية بدقة عبر XRandR ويقيد حركة المؤشر والقلم تماماً داخل مستطيل الشاشة الافتراضية (`InputProp::DIRECT`).
- ينحصر المؤشر داخل شاشة التابلت دون القفز إلى الشاشات الأخرى.

---

<a id="wrong-screen"></a>
## 🖥 العميل يعرض الشاشة الخطأ (سطح المكتب الرئيسي بدل الشاشة الافتراضية)

**العرض:**
يتصل عميل Android/الويب ويعرض فيديو، لكنه يعكس سطح مكتب المضيف الرئيسي بدل شاشة ثانية نظيفة. سحب النوافذ إلى شاشة ثانية لا يفعل شيئاً.

**السبب:**
وحدة النواة `evdi` غير محملة، فيتراجع Orbiscreen إلى التقاط سطح المكتب الرئيسي (Wayland portal أو نافذة جذر X11). هذا الوضع المتدهور مقصود: يبلغ `GetStatus.capture_backend` عن `wayland-portal-fallback` أو `x11-portal-fallback` بدل `evdi`، ويسجل الدامن تحذير `EVDI kernel module missing/inactive ... Falling back` عند البدء.

**الإصلاح:**
1. ثبّت وحمّل `evdi` عبر DKMS - راجع [وقت التشغيل: فشل `orbiscreen start`](#runtime-evdi)، ثم:
   ```bash
   sudo modprobe evdi && lsmod | grep evdi
   ```
2. أعد تشغيل الدامن (`orbiscreen stop && orbiscreen start`) وتحقق:
   ```bash
   busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetStatus
   # "capture_backend":"evdi"
   ```
3. انقل نافذة إلى مخرج Orbiscreen (‏`EVDI-0`) من إعدادات الشاشات في المنشئ.

---

<a id="web-no-picture"></a>
## 🌐 عميل الويب يُحمَّل لكن بلا صورة

**العرض:**
`http://<host>:8788/` يُحمل، وشريط الحالة يظل يعرض "Connecting to stream…" أو يبلغ فوراً "This browser does not support MSE playback".

**السبب:**
عميل الويب يفك MPEG-TS عبر `mpegts.js` المورّدة محلياً ويضخ H.264 إلى MediaSource Extensions (MSE). المتصفحات بدون MSE أو مع حظر التشغيل التلقائي لن تستطيع فك البث. لا يوجد مسار WebRTC.

**الإصلاح:**
1. استخدم متصفحاً يدعم البث الحي عبر MSE: Chrome أو Firefox أو Edge على سطح المكتب. لا يدعم Safari على iOS فيديو MSE، وFirefox على الهاتف بدون MSE أيضاً.
2. إذا حظر المتصفح التشغيل التلقائي، انقر على الفيديو مرة لبدء التشغيل.
3. تأكد أن الصفحة خدمها الدامن نفسه (يُحمل `vendor/mpegts.js` من `/client/vendor/mpegts.js`) - ليس نسخة قديمة مخزونة مؤقتاً.
4. تحقق من وحدة التحكم/الشبكة في أدوات المطوّر: 401 على `/stream` تعني فشل مسار التوكن - راجع [رفض 401](#token-401).

---

<a id="no-encoder"></a>
## 🎞 لا يوجد مُرمَّز - البث يبدأ لكنه يفشل (غياب x264)

**العرض:**
يبدأ الدامن ويتصل العملاء، لكن الفيديو لا يصل أو يظهر في السجل خطأ ربط عناصر GStreamer يذكر `x264enc` / `no element found`.

**السبب:**
الترميز يمر عبر GStreamer. عنصر التراجع البرمجي `x264enc` يأتي في حزمة الإضافات `ugly`؛ والمرمزات العتادية تحتاج `vaapih264enc` أو `nvh264enc` (حزمة `bad`). بدونها لا يمكن إنتاج H.264.

**الإصلاح:**
```bash
# Fedora / Nobara
sudo dnf install gstreamer1-plugins-ugly gstreamer1-plugins-bad-free gstreamer1-plugins-good

# Ubuntu / Debian
sudo apt install gstreamer1.0-plugins-ugly gstreamer1.0-plugins-bad gstreamer1.0-plugins-good

# تحقق من وجود عنصر الترميز
gst-inspect-1.0 x264enc
```
ثم أعد تشغيل الدامن؛ يبلغ `GetStatus.encoder` عن المُرمَّز المستخدم فعلياً.

---

<a id="token-401"></a>
## 🔑 رفض 401 من `/stream` أو `/input` أو `/api/control` (التوكن)

**العرض:**
يحصل العملاء (Android أو الويب أو سكربتات مكتوبة يدوياً) على `401 Unauthorized`. ‏`curl http://host:8788/health` يعمل بشكل طبيعي، لكن `/stream` و`/input` و`/api/control` ترفض الطلب.

**السبب:**
تشترط هذه النقاط توكن الوصول الخاص بالجلسة الذي يتم توليده مع كل تشغيل للدامن.
- **التحقق من التوكن:** تأكد من أن العميل يمرر توكن الجلسة الصحيح المطابق لجلسة الدامن الحالية. عند الاتصال عبر المتصفح، تقوم النقطة `/client/config.json` بتمهيد التوكن تلقائياً، أو يمكن تمريره عبر `#token=` في الرابط.
- يجب على متصفحات الويب الخارجية تقديم التوكن في عنوان الرابط مباشرة.

**الإصلاح:**
1. لمتصفحات الويب الخارجية، أضف التوكن عبر تجزئة الرابط (Hash) أو الاستعلام:
   ```
   http://<host-ip>:8788/#token=<SECRET_TOKEN>
   ```
   أو:
   ```
   http://<host-ip>:8788/?token=<SECRET_TOKEN>
   ```
2. استخرج التوكن من جهاز المضيف:
   ```bash
   orbiscreen doctor
   # أو قراءة ملف التوكن المحمي بصلاحيات 0o600:
   cat ~/.config/orbiscreen/stream_token
   ```
3. يستقبل تطبيق أندرويد التوكن تلقائياً عبر سجلات mDNS TXT. وفي حال الإضافة اليدوية لمضيف، أدخل التوكن في نافذة الإعدادات.
4. مرر التوكن في السكربتات عبر ترويسة المصادقة:
   ```bash
   curl -H "Authorization: Bearer $TOKEN" http://host:8788/stream --output - | head -c 1000
   ```

---

<a id="dbus-missing"></a>
## 🚌 الـ daemon غير موجود على D-Bus

**العرض:**
يطبع `orbiscreen stop` الرسالة `daemon is not running (no com.orbiscreen.Daemon on the session bus)`

**السبب:**
خدمة D-Bus (‏`com.orbiscreen.Daemon`) تُسجل على **ناقل جلسة المستخدم** من طرف عملية الدامن طالما هي قيد التشغيل. أسباب غيابها الشائعة:
- الدامن لم يبدأ (أو انهار) في جلسة المستخدم الحالية.
- بدأ `orbiscreen start` بمستخدم آخر أو بـ `sudo` - ناقل النظام/المستخدم الآخر ليس ناقل جلستك.
- ‏`DBUS_SESSION_BUS_ADDRESS` غير مضبوط أو متجاوز في الصدفة التي يُشغَّل فيها `orbiscreen stop`.

**الإصلاح:**
1. تحقق من الخدمة والحالة:
   ```bash
   busctl --user status com.orbiscreen.Daemon 2>&1 || echo "not on the bus"
   systemctl --user status orbiscreen
   ```
2. شغله بمستخدمك العادي: `orbiscreen start` (دون `sudo`) أو `systemctl --user start orbiscreen`.
3. إذا شُغل تحت systemd فضل `systemctl --user stop orbiscreen` لإيقافه (يعمل `orbiscreen stop` أيضاً ويتراجع إلى تابع D-Bus `Stop`).

---

<a id="daemon-cpu"></a>
## 🚀 الـ Daemon: استهلاك 100% للمعالج أو تجمّد

**السبب:**
كانت حلقة الالتقاط تعمل دون إخلاء للمعالج أو تراكم غير محدود في الطابور.

**الإصلاح:**
حدّث إلى الإصدار الأخير (v0.23.6 أو أحدث).

---

<a id="still-stuck"></a>
## 🛟 ما زلت عالقاً؟

<a id="re-run-job"></a>
### إعادة تشغيل مهمة CI واحدة

في صفحة PR الفاشلة:
1. افتح قسم **Checks**.
2. انقر اسم الفحص الفاشل.
3. انقر **Re-run jobs** ← **Re-run failed jobs**.

### مراجعة سجلات الإجراء

يعرض قسم **Run logs** مخرجات `cargo` / `gradlew` الدقيقة. قارنها مع الأقسام أعلاه.

### فتح issue

استخدم `.github/ISSUE_TEMPLATE/bug.yml`. أرفق:
- مخرجات خطأ `cargo` / `gradlew` الدقيقة.
- رابط تشغيل CI.
- نظام التشغيل / المنشّئ (compositor) للمضيف (إن كانت المشكلة وقت تشغيل).
- مخرجات `adb logcat *:E` (إن كانت المشكلة متعلقة بـ Android).

<a id="verify-stream"></a>
### التحقق من بث حي من طرف إلى طرف

```bash
./scripts/verify-stream.sh [المنفذ] [مدة_بالثواني]
```

يسجّل بضع ثوانٍ من `/stream`، يتحقق من أن الحمولة تُفكّ ترميزها كـ H.264، ويقيس سطوع الإطارات (YAVG) لالتقاط تراجعات البث الأسود/الفارغ تلقائياً. يتطلب `curl` و`python3` و`ffmpeg` على المضيف.

<a id="setup-dev-env"></a>
### تجهيز بيئة تطوير

```bash
./scripts/setup-dev-env.sh
```

يثبّت سلسلة أدوات Rust واعتماديات البناء (GStreamer وWayland/X11 وlibevdev) لتوزيعات Fedora وDebian وArch المكتشفة من `/etc/os-release`.

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[العودة إلى README](../README_AR.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
