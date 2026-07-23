# استكشاف الأخطاء وإصلاحها - Orbiscreen

---

## 🌐 اللغة

<a href="TROUBLESHOOTING.md">🇬🇧 English</a> · <a href="TROUBLESHOOTING_AR.md">🇸🇦 العربية</a>

---

## 📋 فهرس المحتويات

### إجراءات سير عمل CI (`.github/workflows/ci.yml`)

- [إجراء: `Check formatting` (`cargo fmt --all -- --check`)](#ci-fmt)
- [إجراء: `Clippy (deny warnings)` (`cargo clippy --workspace --all-targets --locked -- -D warnings`)](#ci-clippy)
- [إجراء: `Build` (`cargo build --workspace --locked`)](#ci-build)
- [إجراء: `Test` (`cargo test --workspace --locked`)](#ci-test)
- [إجراء: `Run cargo-deny` (`cargo deny check`)](#ci-deny)

### التشغيل

- [تشغيل: فشل `orbiscreen start` - النواة غير مثبتة](#runtime-evdi)
- [تشغيل: خلفية الالتقاط غير متاحة على Wayland](#runtime-wayland)
- [تشغيل: تحذيرات `unsafe_op_in_unsafe_fn` / `missing_debug_implementations`](#runtime-lints)

### لا تزال عالقاً؟

- [البناء لا يزال يفشل؟ تحقق من سجلات الإجراء](#still-stuck)
- [إعادة تشغيل مهمة CI واحدة](#re-run-job)

---

<a id="ci-fmt"></a>
## 🧪 إجراء CI: `Check formatting` (`cargo fmt --all -- --check`)

**العرض:**
```
Diff in /path/to/file.rs:
   println!("x");
-  println!("y");
+  println!("z");
```

**السبب:**
ملفات مصدر Rust لا تطابق تنسيق `cargo fmt`.

**الحل:**
```bash
cargo fmt --all
git add -A
git commit -m "orbiscreen | v0.1.1 | style: cargo fmt --all"
```

**لماذا يحدث هذا:**
- أضفت ملفاً دون تشغيل `cargo fmt` قبل الالتزام.
- قمت بتعديل `rustfmt.toml` لتغيير قواعد التنسيق (مثل `max_width`).

**الوقاية:**
يشغّل `.github/workflows/ci.yml` الأمر `cargo fmt --all -- --check` على كل
PR. شغّل `cargo fmt --all` محلياً قبل الدفع.

---

<a id="ci-clippy"></a>
## 🧪 إجراء CI: `Clippy (deny warnings)` (`cargo clippy --workspace --all-targets --locked -- -D warnings`)

**العرض:**
```
error: this operation is not supported for derived errors
  --> src/lib.rs:42:5
   |
42 |     let result = self.op().expect("...");   // unwrap in production code
   |                              ^^^^^^^^
   |
   = note: `-D warnings` implied by `-D warnings`
```

**السبب:**
يعامل `cargo clippy -D warnings` كل تحذير clippy كخطأ. أكثر المذنبين شيوعاً:

1. **`let _ = unit_value`** - تعيين `()` لربط مُهمَل. الحل: `let _ = expr;`
   تصبح `expr;` فقط، أو استخدم `drop()`.
2. **`unwrap()`/`expect()` في كود الإنتاج** (مسموح فقط في وحدات
   `#[cfg(test)]` وفقاً لـ `CONTRIBUTING`).
3. **استيرادات / متغيرات / حقول غير مستخدمة** - أصلحها أو ضع `_` في البداية.
4. **`#[derive]` يدوي** حيث يمكن اشتقاق `Default` - استخدم
   `#[derive(Default)]`.
5. **جودة تعليقات الكود** - أحياناً lint قصاصة في clippy.
6. **`unsafe_op_in_unsafe_fn`** - عند العمل مع كتل `unsafe`، أضف
   `#[allow(unsafe_code)]` للدالة الداخلية.

**الحل:**
```bash
cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 | head -50
# طبّق الاقتراحات، ثم:
cargo clippy --workspace --all-targets --locked --fix
# راجع التغييرات والتزم بها
```

**الوقاية:**
شغّل `cargo clippy` محلياً قبل الدفع.

---

<a id="ci-build"></a>
## 🧪 إجراء CI: `Build` (`cargo build --workspace --locked`)

**العرض:**
```
error[E0463]: can't find crate for `webrtc`
  |
  = note: the crate `webrtc` couldn't be found in the registry index
```

**السبب:**
`Cargo.lock` مثبّت على إصدارات لم تعد متاحة على `crates.io`. أو أن الـ
toolchain المثبّت في `rust-toolchain.toml` يختلف عن المنفّذ. مع تفعيل
`--locked`، يرفض cargo تحديث الـ lockfile.

**الحل:**
```bash
# محلياً: أعد توليد الـ lockfile
cargo update -p webrtc
# ثم أعد البناء
cargo build --workspace --locked
git add Cargo.lock
git commit -m "orbiscreen | v0.1.1 | chore: refresh Cargo.lock for webrtc 0.20.0-rc.3"
```

إذا كانت التبعية `webrtc = "0.20.0-rc.3"` قد تمت إزالتها أو استبدالها
على `crates.io`، يجب عليك:
- تثبيت إصدار مختلف (`cargo search webrtc`)
- انتظار إصدار مستقر (مساحة العمل تثبّت release-candidate)

**الوقاية:**
حافظ على `Cargo.lock` مُلتزماً (تم بالفعل) وشغّل `cargo update` فقط
بقصد.

---

<a id="ci-test"></a>
## 🧪 إجراء CI: `Test` (`cargo test --workspace --locked`)

**العرض:**
```
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

---- tests::test_encode_default stdout ----
thread 'tests::test_encode_default' panicked at 'called `Result::unwrap()` on an `Err` value: EncodeError("gstreamer pipeline error: appsrc: ...")', src/encode.rs:125:5
```

**السبب:**
تفترض الاختبارات أن المضيف يحتوي على إضافات GStreamer معينة مثبتة
(`x264enc`، `vaapih264enc`، `nvh264enc`). في CI هذه ليست موجودة دائماً،
لذلك يفشل تهيئة المُرمّز (encoder).

**الحل:**
إما تثبيت الإضافات على CI (سير العمل يثبّت
`gstreamer1.0-plugins-good gstreamer1.0-plugins-bad` بالفعل - أضف
`gstreamer1.0-libav gstreamer1.0-plugins-ugly` إذا كنت بحاجة إلى x264)،
أو اجعل الاختبار الفاشل شرطياً:

```rust
#[test]
fn frame_duration_ns_matches_framerate() {
    let params = EncodeParams {
        framerate: 60,
        ..EncodeParams::default()
    };
    let mut encoder = Encoder::new(params).ok();
    // تخطَّ إذا لم تكن إضافة GStreamer H.264 مثبتة على مضيف الاختبار.
    if let Some(encoder) = encoder.as_mut() {
        assert_eq!(encoder.frame_duration_ns(), 16_666_666);
    }
}
```

**الوقاية:**
حافظ على تبعيات CI محدّثة. لتشفير H.264 على CI،
`gstreamer1.0-plugins-ugly` (التي تحتوي على `x264enc`) إلزامي.

---

<a id="ci-deny"></a>
## 🧪 إجراء CI: `Run cargo-deny` (`cargo deny check`)

**العرض:**
```
advisories FAILED, bans FAILED, licenses ok, sources ok
##[error]Process completed with exit code 3.
```

**السبب:**
فحص التراخيص ينجح (قائمة السماح GPL-3.0-or-later تشمل LGPL-2.1 الخاصة
بـ `libevdi`). يفشل فحص الـ bans عند `wildcards = "deny"` يعلّم على مسار
يحتوي على `*`. يفشل فحص التنبيهات على تبعيات مهجورة (ولاحظ `derivative`
عبر `evdi`).

**الحل:**
للإعداد الحالي:
- هذا **تنبيه إعلامي فقط** - `cargo-deny` ليس من فحوصات الحالة المطلوبة
  في `ci.yml`، لذا فإن فشل فحص deny لا يمكن أن يمنع دمج PR.

لجعل `cargo-deny` ينجح بنظافة:
1. حدّث قسم `[advisories]` في `deny.toml` لتجاهل تنبيه `derivative` المهجور.
2. تأكد من حل جميع الـ globs في المسارات قبل الالتزام.

---

<a id="runtime-evdi"></a>
## 🚀 تشغيل: فشل `orbiscreen start` - النواة غير مثبتة

**العرض:**
```
Error: evdi kernel module is not installed
```

**السبب:**
وحدة النواة `evdi` من DisplayLink لم يتم تحميلها على المضيف. يحتاج الـ
daemon لكشف شاشة افتراضية عبر DRM/KMS.

**الحل:**
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
2. تحقق من تحميلها:
   ```bash
   lsmod | grep evdi   # يجب أن يعرض وحدة evdi
   ls /dev/dri/card*  # يجب أن يعرض بطاقة واحدة على الأقل
   ```

دليل التثبيت الكامل موجود في `scripts/setup-dev-env.sh`
و `scripts/test-evdi.sh`.

---

<a id="runtime-wayland"></a>
## 🚀 تشغيل: خلفية الالتقاط غير متاحة على Wayland

**العرض:**
```
Error: capture backend unavailable: Wayland capture requires open_async
```

**السبب:**
لا يمكن لـ `CaptureSession::open()` المتزامن قيادة بوابة Wayland
ScreenCast. على جلسة Wayland، يجب استخدام المسار غير المتزامن.

**الحل:**
استخدم `CaptureSession::open_async()` بدلاً من `open()`:
```rust
// ❌ خطأ على Wayland
let session = CaptureSession::open(width, height)?;

// ✅ صحيح
let session = CaptureSession::open_async(width, height).await?;
```

يستخدم ثنائي الـ daemon (`orbiscreen-daemon`) داخلياً `open_async` بالفعل،
لذا هذا مهم فقط إذا استدعيت المكتبة مباشرة.

---

<a id="runtime-lints"></a>
## 🚀 تشغيل: تحذيرات `unsafe_op_in_unsafe_fn` / `missing_debug_implementations`

**العرض:**
```
warning: unnecessary `unsafe` block
  --> crates/orbiscreen-display/src/lib.rs:142:9
warning: type `VirtualDisplay` does not implement `Debug`
```

**السبب:**
مساحة العمل مفعّلة فيها تنبيهات clippy التالية على مستوى التحذير:
- `unsafe_code = "warn"`
- `missing_debug_implementations = "warn"`

**الحل:**
بالنسبة للأنواع التي تحتوي على مقابض (handles) غير-`Debug` (مثل
`evdi::Handle` أو خط أنابيب `GStreamer`)، قم بإلغاء الـ lint صراحة:
```rust
#[allow(missing_debug_implementations)]
pub struct VirtualDisplay { ... }
```

بالنسبة لـ `unsafe_op_in_unsafe_fn`:
```rust
#[allow(unsafe_code)]
pub async fn open_at(...) -> ... {
    let unconnected = unsafe { node.open() }?;
    ...
}
```

**لماذا توجد هذه الـ lints:**
- `missing_debug_implementations` يفرض معالجة صريحة للأنواع المعتمة
  حتى لا تطبع عن طريق الخطأ مؤشرات داخلية في السجلات.
- `unsafe_code = warn` يلتقط كتل `unsafe` شاردة خارج مواقع الاستدعاء
  القليلة المحددة عمداً.

---

<a id="still-stuck"></a>
## 🛟 لا تزال عالقاً؟

<a id="re-run-job"></a>
### إعادة تشغيل مهمة CI واحدة

في صفحة الـ PR الفاشلة:
1. افتح قسم **Checks**.
2. انقر على اسم الفحص الفاشل (مثل `Build, test & lint (workspace)`).
3. انقر على **Re-run jobs** → **Re-run failed jobs**.

### تحقق من سجلات الإجراء

بالنسبة لكل مهمة فاشلة، يعرض قسم **Run logs** مخرجات `cargo` الدقيقة.
قارنها مع الأقسام أعلاه.

### افتح issue

إذا لم ينطبق أي مما سبق، افتح issue باستخدام
`.github/ISSUE_TEMPLATE/bug.yml`. قم بتضمين:
- مخرجات خطأ `cargo` الدقيقة
- رابط تشغيل CI
- نظام التشغيل / المركّب للمضيف (إذا كان متعلقاً بالتشغيل)

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
<a href="TROUBLESHOOTING.md">English</a> ·
[Back to README](../README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>