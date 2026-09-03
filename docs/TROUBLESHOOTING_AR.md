<div align="center">

# استكشاف الأخطاء وإصلاحها - Orbiscreen

[![الإصدار](https://img.shields.io/badge/version-0.18.0-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
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

- [Android: التطبيق يتعطل أو تموت العملية عند النقر على Connect](#android-connect-crash)
- [Android: شاشة سوداء بعد Connect](#android-black-screen)
- [Android: قائمة الاكتشاف فارغة رغم وجود مضيفين على نفس شبكة Wi-Fi](#android-no-hosts)
- [Android: اللمس مُدوَّر / غير محاذٍ](#android-touch-offset)
- [Android: إجراءات شريط التحكم تُرجع 404](#android-control-404)
- [Android: التطبيق يتعطل فوراً عند التشغيل](#android-crash)
- [Android: اتصال USB يعرض "Looking for host…"](#android-usb)

### البث والعملاء

- [العميل يعرض الشاشة الخطأ (سطح المكتب الرئيسي بدل الشاشة الافتراضية)](#wrong-screen)
- [عميل الويب يُحمَّل لكن بلا صورة](#web-no-picture)
- [لا يوجد مُرمَّز - البث يبدأ لكنه يفشل (غياب x264)](#no-encoder)
- [رفض 401 من `/stream` أو `/input` أو `/api/control` التوكن](#token-401)
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
git commit -m "orbiscreen | v0.10.3 | style: cargo fmt --all"
```

**الوقاية:**
شغّل `./gradlew :app:lintDebug` و`cargo fmt --all` محلياً قبل الدفع.

---

<a id="ci-clippy"></a>
## 🧪 إجراء CI: `Clippy (deny warnings)`

**العرَض:**
```
error: this operation is not supported for derived errors
  --> src/lib.rs:42:5
```

**السبب:**
`cargo clippy -D warnings` يعامل كل تحذير clippy كخطأ.

**الإصلاح:**
```bash
cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 | head -50
cargo clippy --workspace --all-targets --locked --fix
git add -A
git commit -m "orbiscreen | v0.10.3 | fix: resolve clippy warnings"
```

**الوقاية:**
شغّل `cargo clippy` محلياً قبل الدفع.

---

<a id="ci-build"></a>
## 🧪 إجراء CI: `Build` ‏(`cargo build --workspace --locked`)

**العرَض:**
```
error[E0463]: can't find crate for `gstreamer`
```

**الإصلاح:**
```bash
cargo update -p gstreamer
cargo build --workspace --locked
git add Cargo.lock
git commit -m "orbiscreen | v0.10.3 | chore: refresh Cargo.lock"
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

## 📱 Android ‏(v0.10.3)

<a id="android-connect-crash"></a>
### Android: التطبيق يتعطل أو تموت العملية عند النقر على Connect

**العرَض:**
النقر على مضيف في شاشة Discovery يقتل فوراً عملية التطبيق أو يعيده إلى المشغّل (launcher).

**السبب (أُصلح في v0.10.3):**
كانت `PlayerHolder.build()` تُنفَّذ داخل `withContext(Dispatchers.IO)`. يتطلب ExoPlayer الإنشاء على الخيط الرئيسي؛ وإنشاء مكوّنات المشغّل على خيوط IO يرمي استثناءات وصول خيطي تُنهي العملية.

**الإصلاح:**
- حدّث إلى `orbiscreen-android-release.apk` الإصدار **v0.10.3** أو أحدث.
- ينقل `StreamViewModel` إنشاء ExoPlayer إلى `withContext(Dispatchers.Main)` ويحصّن `build()` بكتلة try-catch بحيث تظهر أخطاء البناء كبطاقات `StreamEvent.Error` قابلة لإعادة المحاولة بدل الانهيار.

---

<a id="android-black-screen"></a>
### Android: شاشة سوداء بعد Connect

**العرَض:**
النقر على مضيف مكتشف يعرض سطحاً أسود؛ لا فيديو؛ ولا يظهر شريط التحكم.

**السبب (أُصلح في v0.10.3):**
اعتمد العميل قبل v0.10.3 على MIME sniffing في ExoPlayer لاستجابة `/stream` وكان يتراجع إلى سطح أسود عند فشل اكتشاف MPEG-TS.

**الإصلاح:**
- حدّث إلى `orbiscreen-android-release.apk` الإصدار **v0.10.3** أو أحدث.
- يبني `PlayerHolder` الجديد كائن `MediaItem` مع `setMimeType(MimeTypes.VIDEO_MP2T)` ويفرض رابطاً بلاحقة `.ts`، حتى يُفك ترميز البث دون sniffing.
- تظهر الأخطاء كبطاقة إعادة محاولة بدل السطح الأسود.

إذا استمرت المشكلة بعد التحديث:
1. تأكد من إمكانية الوصول إلى المضيف عبر `curl http://host:8788/health` من نفس شبكة Wi-Fi.
2. تأكد من استجابة `/api/info`: `curl http://host:8788/api/info`.
3. افحص `adb logcat -s OrbiPlayer:*` بحثاً عن أسطر `player error:`.

---

<a id="android-no-hosts"></a>
### Android: قائمة الاكتشاف فارغة رغم وجود مضيفين على نفس شبكة Wi-Fi

**السبب:**
بروتوكول mDNS محظور على الشبكة (Wi-Fi شركات، مرشّح Apple Bonjour، إلخ).

**الإصلاح:**
1. افتح بطاقة **Add manually** وأدخل `host:port` (مثال `192.168.1.50:8788`).
2. اختياري: فعّل مفتاح **Scan subnet for hosts** في **Settings**. يفحص الماسح شبكة ‎/24 حول البوابة الحالية عبر TCP ويضيف أي مضيف يستجيب على المنفذ 8788.

---

<a id="android-touch-offset"></a>
### Android: اللمس مُدوَّر / غير محاذٍ

**السبب:**
يعتمد تعيين المؤشر إلى المضيف على دقة الشاشة المُبلَّغ عنها من `/api/info`. إذا كان المضيف مُدوَّراً (مثلاً شاشة افتراضية عمودية) لكن JSON ما زال يبلّغ عن اتجاه أفقي، سيكون التعيين خاطئاً.

**الإصلاح:**
دوّر المضيف بدلاً من شاشة Android. يطبّق `PlayerView` letterboxing تلقائياً للحفاظ على نسبة العرض إلى الارتفاع المُبلَّغ عنها من المضيف.

---

<a id="android-control-404"></a>
### Android: إجراءات شريط التحكم تُرجع 404

**السبب:**
المضيف يشغّل daemon أقدم لا يطبّق `/api/control`.

**الإصلاح:**
أعد تشغيل الـ daemon على المضيف لالتقاط ثنائي النقل بإصدار v0.10.3. من المضيف:
```bash
orbiscreen stop
sudo orbiscreen start
```

---

<a id="android-crash"></a>
### Android: التطبيق يتعطل فوراً عند التشغيل

**العرَض:**
تفتح تطبيق Orbiscreen على Android فيتعطل فوراً ويعود إلى الشاشة الرئيسية.

**السبب (أُصلح في v0.7.1):**
ملف `index.html` معطوب (وجود `-->` زائد) أسقط WebView.

**الإصلاح:**
يستخدم v0.10.3 Compose + `PlayerView` حصراً - لا يوجد WebView. إذا استمر تعطل APK الجديد، التقط logcat عبر `adb logcat *:E | grep orbiscreen` وافتح issue.

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
منذ v0.11.0 تشترط هذه النقاط الثلاث توكن الوصول الخاص بالجلسة. يُعاد توليده مع كل تشغيل للدامن ويوصل بطريقتين:
- **mDNS:** يحمل سجل TXT للخدمة المعلنة `token=...`
- **HTTP:** ‏`GET /client/config.json` يعيد `{"token": ..., "display_width": ..., "display_height": ...}`

**الإصلاح:**
1. انتزع التوكن الحالي ومرره بأي من الطريقتين:
   ```bash
   curl -s http://host:8788/client/config.json
   TOKEN=*** -c "import json,sys;print(json.load(sys.stdin)['token'])")
   curl -H "Authorization: Bearer $TOKEN" http://host:8788/stream --output - | head -c 1000
   # أو: curl "http://host:8788/stream?token=***"
   ```
2. يحصل عملاء Android على التوكن تلقائياً من اكتشاف mDNS أو من نقطة الإعداد؛ إذا رفض مضيف مضاف يدوياً الطلب برفض 401، أزله وأضفه من جديد بعد إعادة تشغيل الدامن (التوكن القديم انتهى).
3. ‏`/health` و`/api/info` و`/client/config.json` و`/` و`/client/*` عامة عمداً - رفض 401 عليها يشير إلى وكيل سيئ التوصيف وليس الدامن.

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

**السبب (أُصلح في v0.7.2):**
كانت حلقة الالتقاط تعمل دون إخلاء للمعالج (yielding).

**الإصلاح:**
حدّث إلى v0.10.3 (مساحة العمل).

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
