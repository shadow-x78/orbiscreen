<div align="center">

# استكشاف الأخطاء وإصلاحها - Orbiscreen

[![الإصدار](https://img.shields.io/badge/الإصدار-0.1.0-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/الرخصة-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![اللغة](https://img.shields.io/badge/rust-edition_2021-16a34a?style=flat-square&logo=rust)
![المنصة](https://img.shields.io/badge/منصة-Linux-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 اللغة

<a href="TROUBLESHOOTING.md">🇬🇧 English</a> · <a href="TROUBLESHOOTING_AR.md">🇸🇦 العربية</a>

---

## 📋 فهرس المحتويات

### واجهة برمجة التطبيقات GitHub / إعداد المستودع

- [تطبيق حماية الفرع - `enabled` ليس boolean](#gh-api-enabled)
- [تطبيق حماية الفرع - `restrictions` مرفوض على المستودعات الشخصية](#gh-api-restrictions)
- [تطبيق حماية الفرع - `"restrictions" wasn't supplied`](#gh-api-restrictions-missing)
- [تطبيق حماية الفرع - فشل `cargo-deny` بسبب `derivative` التبعي](#gh-api-deny)
- [دمج PR محظور - `enforce_admins` وموافقة النفس يتعارضان](#gh-api-self-approval)
- [الدفع مرفوض - `protected branch update failed`](#gh-api-push-rejected)

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

<a id="gh-api-enabled"></a>
## 🔧 تطبيق حماية الفرع - `enabled` ليس boolean

**العرض:**
```json
{
  "message": "Invalid request.",
  "errors": ["For 'allOf/0', {\"enabled\" => true} is not a boolean."],
  "status": 422
}
```

**السبب:**
تم تغليف مفاتيح التبديل على المستوى الأعلى (`enforce_admins`،
`required_linear_history`، `allow_force_pushes`، إلخ) داخل كائنات
`{ "enabled": true }`. الـ schema يقبل **boolean literals** على المستوى
الأعلى.

**الحل:**
قم بتسوية كل مفتاح:
```jsonc
// ❌ خطأ
{ "enforce_admins": { "enabled": true } }

// ✅ صحيح
{ "enforce_admins": true }
```

**التطبيق:**
```bash
gh api -X PUT /repos/shadow-x78/orbiscreen/branches/main/protection \
    --input admin/branch-protection.json
```

---

<a id="gh-api-restrictions"></a>
## 🔧 تطبيق حماية الفرع - `restrictions` مرفوض على المستودعات الشخصية

**العرض:**
```json
{
  "message": "Validation Failed",
  "errors": ["Only organization repositories can have users and team restrictions"],
  "status": 422
}
```

**السبب:**
على المستودعات **الشخصية**، يرفض GitHub أي قيمة ذات مصفوفات:
```json
{ "restrictions": { "users": [], "teams": [], "apps": [] } }   // ❌ مرفوض
```

**الحل:**
بالنسبة للمستودعات الشخصية، استخدم `"restrictions": null`. الحقل موجود
(يحقق متطلبات الـ schema) لكن قيمته null (يتجاهله GitHub بصمت - وهذا
بالضبط ما تحتاجه المستودعات الشخصية).

بالنسبة لمستودعات **المؤسسات**، يمكنك ملء `users[]` / `teams[]` / `apps[]`.

---

<a id="gh-api-restrictions-missing"></a>
## 🔧 تطبيق حماية الفرع - `"restrictions" wasn't supplied`

**العرض:**
```json
{ "message": "Invalid request.", "errors": ["\"restrictions\" wasn't supplied."], "status": 422 }
```

**السبب:**
لقد حذفت مفتاح `restrictions` تماماً. يتطلب الـ schema وجوده (حتى على
المستودعات الشخصية حيث يجب أن تكون قيمته `null`).

**الحل:**
أضف المفتاح بقيمة `null` - لا تحذفه:
```jsonc
// ❌ خطأ - الحقل محذوف تماماً
{ "enforce_admins": true, "required_linear_history": true, ... }

// ✅ صحيح - الحقل موجود، قيمته null
{ "enforce_admins": true, "restrictions": null, "required_linear_history": true, ... }
```

---

<a id="gh-api-deny"></a>
## 🔧 تطبيق حماية الفرع - فشل `cargo-deny` بسبب `derivative` التبعي

**العرض:**
```
error[unmaintained]: `derivative` is unmaintained; consider using an alternative
  ├─ ID: RUSTSEC-2024-0388
  ├─ derivative v2.2.0
  │   └── evdi v0.8.0
  │       └── orbiscreen-display v0.1.0
advisories FAILED, bans FAILED, licenses ok, sources ok
##[error]Process completed with exit code 3.
```

**السبب:**
يضع `cargo-deny` علامة على `derivative v2.2.0` كحزمة مهجورة. تصل هذه الحزمة
**تبعياً** عبر `evdi v0.8.0`، وهو ربط وحدة النواة الذي نستخدمه لإنشاء شاشات
افتراضية.

**الحل:**
سياسة حماية الفرع تتطلب فقط فحص `workspace`، لذا فإن `cargo-deny` لا يمنع
عمليات الدمج. ثلاث طرق للمضي قدماً:

1. **اتركه كتنبيه إعلامي** - يستمر `cargo-deny` في عرض التحذيرات في سجلات
   CI لكنه لا يمنع عمليات الدمج. هذا هو الإعداد الحالي.

2. **احذف `cargo-deny` من سير العمل** إذا أصبح ضوضاء مشكلة - احذف وظيفة
   `licenses` من `.github/workflows/ci.yml`.

3. **تجاهل التنبيه** في `deny.toml` (ملاذ أخير - يضعف الفحص):
   ```toml
   [advisories]
   ignore = ["RUSTSEC-2024-0388"]
   ```

---

<a id="gh-api-self-approval"></a>
## 🔧 دمج PR - `enforce_admins` وموافقة النفس يتعارضان

**العرض:**
```
GraphQL: At least 1 approving review is required by reviewers with write access.
GraphQL: Review Can not approve your own pull request
```

**السبب:**
عندما يكون كل من `enforce_admins: true` و
`required_approving_review_count: 1` مفعّلين، **لا يمكن أن يكون المراجع
الوحيد هو مؤلف الـ PR**. المستودعات ذات المطور الواحد تصطدم بهذا فوراً.

**الحل:**
سير العمل القياسي للمطور الواحد هو تخفيف مؤقت وإعادة التطبيق:

```bash
# 1. طبّق السياسة الصارمة (الإعداد الافتراضي)
gh api -X PUT /repos/<owner>/orbiscreen/branches/main/protection \
    --input branch-protection.json

# 2. لدمج PR الخاص بك: خفف القاعدتين المحظورتين مؤقتاً
gh api -X PATCH /repos/<owner>/orbiscreen/branches/main/protection/enforce_admins \
    -f enabled=false
gh api -X PATCH /repos/<owner>/orbiscreen/branches/main/protection/required_pull_request_reviews \
    -f required_approving_review_count=0

# 3. ادمج الـ PR
gh pr merge <N> --squash --admin --delete-branch

# 4. أعد تطبيق السياسة الصارمة
gh api -X PUT /repos/<owner>/orbiscreen/branches/main/protection \
    --input branch-protection.json
```

بمجرد أن يكون لديك حساب GitHub ثانٍ للعمل كمراجع، لن تحتاج إلى هذه الرقصة.

---

<a id="gh-api-push-rejected"></a>
## 🔧 الدفع - `protected branch hook declined`

**العرض:**
```
remote: error: GH006: Protected branch update failed for refs/heads/main.
remote:
remote: - Changes must be made through a pull request.
remote: - Required status check "workspace" is expected.
```

**السبب:**
حماية الفرع نشطة على `main` وتتطلب:
1. التغييرات تأتي عبر pull request (وليس دفعاً مباشراً).
2. فحص الحالة `workspace` يجب أن يمر.

**الحل:**
استخدم سير عمل فرع الميزة + PR القياسي:
```bash
git checkout -b chore/your-change
git commit -am "orbiscreen | v0.1.0 | chore: your change"
git push -u origin chore/your-change
gh pr create --base main --head chore/your-change
# انتظر حتى ينجح فحص CI لـ workspace
gh pr merge <N> --squash --delete-branch
```

إذا كنت بحاجة إلى التجاوز مؤقتاً، استخدم رقصة التخفيف والاستعادة من
القسم السابق.

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
git commit -m "orbiscreen | v0.1.0 | style: cargo fmt --all"
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
git commit -m "orbiscreen | v0.1.0 | chore: refresh Cargo.lock for webrtc 0.20.0-rc.3"
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
- هذا **تنبيه إعلامي فقط** - `cargo-deny` ليس في فحوصات الحالة المطلوبة
  (`"contexts": ["workspace"]`).
- انظر [واجهة GitHub API: فشل cargo-deny](#gh-api-deny) أعلاه.

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