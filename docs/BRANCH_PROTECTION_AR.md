<div align="center">

# سياسة حماية الفروع - Orbiscreen

[![الإصدار](https://img.shields.io/badge/الإصدار-0.1.0-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/الرخصة-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![اللغة](https://img.shields.io/badge/rust-edition_2021-16a34a?style=flat-square&logo=rust)
![المنصة](https://img.shields.io/badge/منصة-Linux-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 اللغة

<a href="BRANCH_PROTECTION.md">🇬🇧 English</a> · <a href="BRANCH_PROTECTION_AR.md">🇸🇦 العربية</a>

---

## 📋 فهرس المحتويات

- [السياسة](#السياسة)
- [حقل `restrictions`](#حقل-restrictions)
- [فحص الحالة المطلوب: `workspace`](#فحص-الحالة-المطلوب-workspace)
- [التحقق](#التحقق)
- [لماذا هذه القواعد](#لماذا-هذه-القواعد)
- [رقصة التخفيف والاستعادة للمطور الوحيد](#رقصة-التخفيف-والاستعادة-للمطور-الوحيد)
- [إعادة التطبيق بعد نقل المستودع](#إعادة-التطبيق-بعد-نقل-المستودع)
- [استكشاف الأخطاء وإصلاحها](#استكشاف-الأخطاء-وإصلاحها)

---

<a id="السياسة"></a>
## 🛡️ السياسة

تصف هذه الوثيقة سياسة حماية الفروع المطبقة على
`shadow-x78/orbiscreen:main` و `shadow-x78/orbiscreen:dev`.

تُرسَل السياسة كما هي عبر GitHub REST API:

```bash
gh api -X PUT /repos/shadow-x78/orbiscreen/branches/main/protection \
    --input branch-protection.json
```

### الإعدادات

| الإعداد | القيمة | السبب |
|---------|--------|-------|
| `required_status_checks.strict` | `true` | يجب أن تكون الفروع محدّثة قبل الدمج |
| `required_status_checks.contexts[]` | `["workspace"]` | يجب أن يكون CI أخضر |
| `enforce_admins` | `true` | حتى المشرفون يلتزمون بالقواعد |
| `required_pull_request_reviews.required_approving_review_count` | `1` | موافقة واحدة على الأقل |
| `required_pull_request_reviews.dismiss_stale_reviews` | `true` | تُلغى الموافقات القديمة |
| `required_linear_history` | `true` | لا توجد merge commits على الفروع المحمية؛ squash فقط |
| `allow_force_pushes` | `false` | ممنوع force push |
| `allow_deletions` | `false` | لا يمكن حذف `main` / `dev` |
| `block_creations` | `false` | يمكن لأي شخص إنشاء feature branches |
| `required_conversation_resolution` | `true` | يجب حل جميع تعليقات الـ PR |
| `lock_branch` | `false` | (افتراضي؛ لا تغلق) |
| `allow_fork_syncing` | `false` | الفروع الفرعية تبقى مستقلة |

مفاتيح التبديل على المستوى الأعلى هي **boolean literals** في JSON body -
يرفض GitHub `"enforce_admins": {"enabled": true}` مع خطأ تحقق 422.

---

<a id="حقل-restrictions"></a>
## 🔧 حقل `restrictions`

بالنسبة للمستودعات **الشخصية** (هذا المستودع)، يجب أن تكون القيمة
`"restrictions": null`. الحقل مطلوب هيكلياً في الـ schema لكن GitHub يرفض:

- حذفه → `422 "restrictions" wasn't supplied`
- أي كائن مع مصفوفات `users[]` / `teams[]` → `422 Only organization
  repositories can have users and team restrictions`

الصيغة المقبولة هي `"restrictions": null` - يتجاهل GitHub بصمت قيد الدفع،
وهذا بالضبط ما يحتاجه المستودع الشخصي. بالنسبة لمستودعات **المؤسسات**،
املأ `users[]` / `teams[]` / `apps[]`.

---

<a id="فحص-الحالة-المطلوب-workspace"></a>
## ✅ فحص الحالة المطلوب: `workspace`

فحص الحالة المطلوب الوحيد هو مهمة `workspace` من
`.github/workflows/ci.yml`. مهمة `android` من سير عمل Android مُبلّغة
ولكنها غير مطلوبة (عميل Android لم يصل إلى الإنتاج بعد).

مهمة `Verify dependency licenses` (`cargo deny check`) مُبلّغة أيضاً
ولكنها **غير مطلوبة** - تفشل أحياناً بسبب التبعية المهملة
`derivative` (RUSTSEC-2024-0388) التي تأتي عبر `evdi v0.8.0`. انظر
`docs/TROUBLESHOOTING.md` → "إجراء CI: `Run cargo-deny`" لمسار الحل الكامل.

---

<a id="التحقق"></a>
## 🔍 التحقق

```bash
gh api /repos/shadow-x78/orbiscreen/branches/main/protection | head -20
gh api /repos/shadow-x78/orbiscreen/branches/dev/protection   | head -20
```

يجب أن يُرجع كلاهما نفس شكل JSON (الفرعان `workspace` + `dev` يتشاركان
سياسة متطابقة).

---

<a id="لماذا-هذه-القواعد"></a>
## ❓ لماذا هذه القواعد

- **`strict = true`** يمنع سباق PR سريع-الإعادة ضد إصلاح تم إرساؤه
  بعد فتح الـ PR.
- **`enforce_admins = true`** يمنع المالك الوحيد من التحايل بالقواعد
  تحت ضغط النشر.
- **`required_approving_review_count = 1`** هو الحد الأدنى لمشروع ذو
  مالك واحد؛ يفرض عيناً ثانية حتى لو كان المالك يراجع عمله الخاص
  عبر حساب منفصل.
- **`required_linear_history = true`** يحافظ على نظافة `git log
  --first-parent` لتوليد ملاحظات الإصدار.
- **`allow_force_pushes = false`** يحمي الإصدارات الموسومة بالفعل من
  إعادة الكتابة.
- **`allow_deletions = false`** يمنع فقدان `main` أو `dev` بالخطأ عند
  إعادة تعيين واجهة حماية الفروع.

---

<a id="رقصة-التخفيف-والاستعادة-للمطور-الوحيد"></a>
## 🔄 رقصة التخفيف والاستعادة للمطور الوحيد

لأن المستودع مملوك لمطور واحد، **لا يمكن للمراجع الوحيد أن يكون مؤلف
الـ PR**. لدمج PRs الخاصة بك، استخدم نمط التخفيف والاستعادة:

```bash
# 1. طبّق السياسة الصارمة (الافتراضي)
gh api -X PUT /repos/shadow-x78/orbiscreen/branches/<branch>/protection \
    --input branch-protection.json

# 2. لدمج PR الخاص بك: خفف القاعدتين المحظورتين مؤقتاً
gh api -X PATCH \
    /repos/shadow-x78/orbiscreen/branches/<branch>/protection/required_pull_request_reviews \
    -f required_approving_review_count=0
gh api -X PUT /repos/shadow-x78/orbiscreen/branches/<branch>/protection \
    --input policy-relaxed.json   # enforce_admins: false

# 3. ادمج الـ PR
gh pr merge <N> --squash --admin --delete-branch

# 4. أعد تطبيق السياسة الصارمة
gh api -X PUT /repos/shadow-x78/orbiscreen/branches/<branch>/protection \
    --input branch-protection.json
```

بمجرد وجود حساب GitHub ثانٍ للعمل كمراجع، لن تحتاج إلى هذه الرقصة -
يمكن استيفاء شرط الموافقة الواحدة الافتراضي من خلال الحساب الثاني.

---

<a id="إعادة-التطبيق-بعد-نقل-المستودع"></a>
## 🔁 إعادة التطبيق بعد نقل المستودع

السياسة مرتبطة باسم الفرع (`main`, `dev`). بعد النقل:

```bash
gh api -X PUT /repos/<new-owner>/orbiscreen/branches/main/protection \
    --input branch-protection.json
gh api -X PUT /repos/<new-owner>/orbiscreen/branches/dev/protection \
    --input branch-protection.json
```

---

<a id="استكشاف-الأخطاء-وإصلاحها"></a>
## 🛟 استكشاف الأخطاء وإصلاحها

انظر `TROUBLESHOOTING.md` للحصول على جدول استكشاف الأخطاء وإصلاحها
الكامل (متوفر باللغتين الإنجليزية والعربية):
- تطبيق حماية الفرع - `"enabled"` ليس boolean
- تطبيق حماية الفرع - `restrictions` مرفوض على المستودعات الشخصية
- تطبيق حماية الفرع - `"restrictions" wasn't supplied`
- دمج PR محظور - `enforce_admins` وموافقة النفس يتعارضان
- الدفع مرفوض - `protected branch hook declined`

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
<a href="BRANCH_PROTECTION.md">English</a> ·
[الرجوع إلى README](../README.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
