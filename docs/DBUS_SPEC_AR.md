<div align="center">

# مواصفات واجهة D-Bus - Orbiscreen

[![الإصدار](https://img.shields.io/badge/version-0.20.0-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![المنصّة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 اللغة

<a href="DBUS_SPEC.md">🇬🇧 English</a> · <a href="DBUS_SPEC_AR.md">🇸🇦 العربية</a>

---

يكشف Orbiscreen واجهة D-Bus Session Service تتيح للوحات تحكم سطح المكتب وسكربتات CLI، ومؤشرات شريط النظام فحص الحالة الحية والتحكم في عملية الـ daemon. التطبيق الفعلي موجود في `crates/orbiscreen-daemon/src/dbus.rs` - توثّق هذه المواصفات بالضبط ما يعرضه ذلك الكود.

- **نوع الناقل:** Session Bus (ناقل *المستخدم* وليس ناقل النظام)
- **اسم الخدمة:** `com.orbiscreen.Daemon`
- **مسار الكائن:** `/com/orbiscreen/Daemon`
- **اسم الواجهة:** `com.orbiscreen.Daemon`

تُسجَّل الخدمة بواسطة عملية daemon *قيد التشغيل*. إذا كان اسم الخدمة غائباً (`ServiceUnknown`)، فالدايمن ببساطة غير شغال - لا يوجد تفعيل أو إطلاق عبر D-Bus.

---

## 🛰 سطح تحكم HTTP المرافق

يتواصل عميلا Android والويب مع الـ daemon عبر HTTP وليس D-Bus. جدول التوجيه الحالي (راجع `orbiscreen-transport`):

| النقطة | المصادقة | الغرض |
|--------|----------|-------|
| `GET /health` | عامة | فحص الحيوية |
| `GET /api/info` | عامة | أبعاد الشاشة والمُرمّز والإصدار |
| `GET /stream?token=…` | توكن | بث فيديو MPEG-TS ‏(H.264)‏ |
| `POST /input` | توكن | أحداث المؤشر / لوحة المفاتيح / القلم |
| `POST /api/control` | توكن | إجراءات المضيف `lock`، `blank`، `unblank`، `ctrl_alt_del` |
| `GET /client/config.json` | عامة | تمهيد عميل الويب: `{token, display_width, display_height}` |
| `GET /` | عامة | إعادة توجيه إلى عميل الويب المضمّن |
| `GET /client/*` | عامة | ملفات عميل الويب الساكنة ‏(MSE عبر mpegts.js المُورَّدة محليا)‏ |

**نموذج التوكن:** يولَّد توكن عشوائي جديد مع كل تشغيل للدايمن (32 بايت، base64url). المسارات المحمية تشترطه عبر ترويسة `Authorization: Bearer <token>` أو معامل `?token=<token>`. يحصل العملاء عليه من سجل mDNS TXT أو من `/client/config.json`. وبما أن التوكن قابل للقراءة من أي طرف يستطيع الوصول إلى المنفذ، فهذه حماية من الاستخدام العرضي وليست مصادقة قوية - راجع `SECURITY.md`.

يبقى D-Bus الواجهة المرجعية لعملاء Linux الأصليين (سكربتات CLI، `orbiscreen stop`). يتشارك السطحان مصدر الحقيقة نفسه في `orbiscreen-transport` (‏`Stats`).

---

## 🛠 توابع D-Bus

جميع التوابع معروضة على ناقل الجلسة. يحوّل zbus أسماء Rust إلى PascalCase على السلك.

### 1. `GetStatus() -> String` (التوقيع `s`)

تُرجع حالة الـ daemon الحية كنص **كائن JSON**:

```json
{
  "running": true,
  "frames_forwarded": 184320,
  "active_clients": 2,
  "total_clients": 5,
  "auth_failures": 0,
  "usb_devices": 1,
  "encoder": "x264",
  "capture_backend": "evdi"
}
```

| الحقل | النوع | المعنى |
|-------|-------|--------|
| `running` | bool | علم تشغيل خط الأنابيب (‏`false` لفترة قصيرة بعد طلب `Stop`)‏ |
| `frames_forwarded` | u64 | عدد الإطارات المسلّمة إلى النقل منذ البدء |
| `active_clients` | u64 | عملاء `/stream` المتصلون حالياً |
| `total_clients` | u64 | إجمالي اتصالات `/stream` منذ البدء |
| `auth_failures` | u64 | الطلبات غير المصرّ بها المرفوضة منذ البدء (يظهر أيضاً في `GET /health`) |
| `usb_devices` | u64 | أجهزة Android ذات نفق `adb reverse` نشط الآن (يُحدَّث لحظياً أثناء عمل الدامن؛ يظهر أيضاً في `GET /health`) |
| `encoder` | string | المُرمّز الفعلي قيد الاستخدام ‏(`x264`، `vaapi`، `nvenc`)‏ |
| `capture_backend` | string | `evdi` للشاشة الافتراضية؛ `x11-portal-fallback` / `wayland-portal-fallback` عند غياب وحدة evdi |

### 2. `Stop() -> String` (التوقيع `s`)

تطلب إطفاءاً رشيقا للدايمن. يقلب المعالج علم التشغيل ويرسل إشارة إلى الحلقة الرئيسية عبر قناة watch داخلية؛ ثم يهدم الـ daemon الالتقاط والترميز والنقل ويخرج.

- **القيمة المُرجعة:** `"Orbiscreen daemon shutting down"`
- **إن كان متوقفاً أصلاً:** `"Orbiscreen is not running"`

أمر `orbiscreen stop` عميل رقيق لهذا التابع تحديداً: يستدعي `Stop()` عبر ناقل الجلسة، يطبع الرد، ويخرج بالرمز 1 مع تلميح `systemctl --user stop orbiscreen` عندما لا يُعثر على اسم الخدمة.

لا يمكن تشغيل الدامن عبر D-Bus: لا يوجد تفعيل للواجهة ولا تابع `Start()`؛ تُدار خدمة systemd بدل ذلك (`systemctl --user start orbiscreen`).

### 3. `ListClients() -> Array of String` (التوقيع `as`)

عدادات حية للعملاء عبر نقل البث الوحيد (كانت سابقا تعيد سلاسل ثابتة):

```json
["HTTP MPEG-TS /stream: 2 active client(s), 5 total connection(s)"]
```

### 4. `GetConfig() -> String` (التوقيع `s`)

تُرجع الإعدادات المعقّمة التي بُدئ بها الـ daemon، مسلسلة بصيغة **TOML** (وليس JSON) عبر `orbiscreen-core::dump_config`:

```toml
[display]
width = 1920
height = 1080
refresh_rate_hz = 60

[capture]
preferred = "auto"

[encode]
bitrate_kbps = 8000
preferred_encoder = "x264"

[transport]
signaling_port = 8788
mdns_advertise = true
```

عند فشل التسلسل تعيد `config serialize error: <تفاصيل>`.

### غير منفّذ

كان `SetScreenState` مقترحاً سابقاً لكنه **غير منفّذ**. حالة شاشة المضيف (`blank` / `unblank`) لا يمكن الوصول إليها إلا عبر نقطة HTTP المصادَقة `POST /api/control`.

---

## 💻 مثال استخدام من CLI ‏(`busctl`)

```bash
# فحص واجهة Orbiscreen على D-Bus
busctl --user introspect com.orbiscreen.Daemon /com/orbiscreen/Daemon

# الحصول على حالة الـ daemon (سلسلة JSON)
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetStatus

# سرد العملاء المتصلين (مصفوفة سلاسل)
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon ListClients

# طباعة الإعدادات الجارية (سلسلة TOML)
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetConfig

# إيقاف الـ daemon رشيقاً
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon Stop
```

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[العودة إلى README](../README_AR.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
