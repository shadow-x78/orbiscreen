# استكشاف الأخطاء وإصلاحها - Orbiscreen

## 🌐 اللغة

<a href="TROUBLESHOOTING.md">🇬🇧 English</a> · <a href="TROUBLESHOOTING_AR.md">🇸🇦 العربية</a>

---

> ينطبق على **v0.10.3** والإصدارات الأحدث.

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

### الـ Daemon

- [الـ Daemon: استهلاك 100% للمعالج أو تجمّد](#daemon-cpu)

### ما زلت عالقاً؟

- [ما زال البناء يفشل؟ راجع سجلات الإجراء](#still-stuck)
- [إعادة تشغيل مهمة CI واحدة](#re-run-job)

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

<a id="runtime-wayland"></a>
## 🚀 وقت التشغيل: واجهة الالتقاط غير متاحة على Wayland

استخدم `CaptureSession::open_async()` (الـ daemon يفعل ذلك مسبقاً).

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
يضبط Orbiscreen تلقائياً `adb reverse tcp:8788 tcp:8788` عند التشغيل. تأكد من:
1. تفعيل **USB Debugging** في خيارات مطوّر Android.
2. تخويل جهاز المضيف في رسالة التأكيد على هاتفك/جهازك اللوحي.
3. التحقق يدوياً:
   ```bash
   adb devices
   adb reverse tcp:8788 tcp:8788
   ```
4. انقر بطاقة **USB mode** في شاشة Discovery.

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

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[العودة إلى README](../README_AR.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
