# مواصفات واجهة D-Bus - Orbiscreen

## 🌐 اللغة

<a href="DBUS_SPEC.md">🇬🇧 English</a> · <a href="DBUS_SPEC_AR.md">🇸🇦 العربية</a>

---

> ينطبق على **v0.10.3** والإصدارات الأحدث.

يكشف Orbiscreen واجهة D-Bus Session Service تتيح للوحات تحكم سطح المكتب (واجهة GTK4)، وسكربتات CLI، ومؤشرات شريط النظام فحص الحالة وضبط إعدادات الشاشة والتحكم في عملية الـ daemon.

- **نوع الناقل:** Session Bus
- **اسم الخدمة:** `com.orbiscreen.Daemon`
- **مسار الكائن:** `/com/orbiscreen/Daemon`
- **اسم الواجهة:** `com.orbiscreen.Daemon`

---

## 🛰 سطح تحكم HTTP المرافق

يتواصل عميل Android (v0.10.3) مع الـ daemon عبر HTTP بسيط وليس D-Bus. النقاط التي يستخدمها هي:

| النقطة | الغرض |
|----------|---------|
| `GET /health` | فحص الحيوية |
| `GET /api/info` | أبعاد الشاشة والمُرمّز والإصدار |
| `POST /api/control` | إجراءات القفل والتعتيم وctrl-alt-del والفتح |
| `GET /stream` | بث فيديو MPEG-TS |

يبقى D-Bus الواجهة المرجعية لعملاء Linux الأصليين (واجهة GTK وسكربتات CLI). يتشارك السطحان مصدر الحقيقة نفسه في `orbiscreen-transport`.

---

## 🛠 توابع D-Bus

### 1. `GetStatus() -> String`
تُرجع حالة تنفيذ الـ daemon الحالية.
- **القيمة المُرجعة:** `"Running"` أو `"Stopped"`

### 2. `Start() -> String`
تبدأ محرك التقاط الشاشة والترميز والنقل في Orbiscreen.
- **القيمة المُرجعة:** `"Orbiscreen daemon started via D-Bus"`

### 3. `Stop() -> String`
توقف التقاط الشاشة وتفصل البثوث النشطة.
- **القيمة المُرجعة:** `"Orbiscreen daemon stopped via D-Bus"`

### 4. `ListClients() -> Vec<String>`
تُرجع قائمة بعملاء الويب وAndroid المتصلين حالياً.
- **القيمة المُرجعة:** `["HTTP Direct /stream", "NSD / WebRTC Signaling Active"]`

### 5. `GetConfig() -> String`
تُرجع الإعدادات الفعلية منسّقةً كنص JSON.
- **القيمة المُرجعة:** `{"width":1920,"height":1080,"refresh_rate":60,"encoder":"auto"}`

---

## 🔄 تابع مرافق: `SetScreenState(state: String) -> String` (مقترح لـ v0.10.3)

يعكس إجراءات `/api/control` عبر D-Bus بحيث تستطيع لوحة تحكم طرف المضيف أيضاً تبديل حالة الشاشة دون المرور بـ HTTP.

التوقيع المقترح:

```dbus
SetScreenState(IN String state) -> String
```

حيث `state ∈ {"on", "off", "lock"}`.

---

## 💻 مثال استخدام من CLI (`busctl`)

```bash
# فحص واجهة Orbiscreen على D-Bus
busctl --user introspect com.orbiscreen.Daemon /com/orbiscreen/Daemon

# الحصول على حالة الـ daemon
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetStatus

# سرد العملاء المتصلين
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon ListClients
```

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[العودة إلى README](../README_AR.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
