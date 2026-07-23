# مواصفات واجهة D-Bus البرمجية - Orbiscreen

---

## 🌐 اللغة

<a href="DBUS_SPEC.md">🇬🇧 English</a> · <a href="DBUS_SPEC_AR.md">🇸🇦 العربية</a>

---

## 🛰 نظرة عامة

يوفر Orbiscreen واجهة D-Bus Session Service تتيح للوحات التحكم الرسومية (GTK4 GUI) وسكربتات السطر الأوامر وأيقونة شريط النظام فحص حالة الخادم، وتعديل إعدادات الشاشة، والتحكم بالخادم الخلفي.

- **نوع الناقل:** Session Bus
- **اسم الخدمة:** `com.orbiscreen.Daemon`
- **مسار الكائن:** `/com/orbiscreen/Daemon`
- **اسم الواجهة:** `com.orbiscreen.Daemon`

---

## 🛠 الدوال المتاحة (Methods)

### 1. `GetStatus() -> String`
إرجاع حالة تشغيل الخادم الحالية.
- **القيمة المرجعة:** `"Running"` أو `"Stopped"`

### 2. `Start() -> String`
بدء محركات الالتقاط والترميز والبث لـ Orbiscreen.
- **القيمة المرجعة:** `"Orbiscreen daemon started via D-Bus"`

### 3. `Stop() -> String`
إيقاف التقاط الشاشة وفصل البث المباشر.
- **القيمة المرجعة:** `"Orbiscreen daemon stopped via D-Bus"`

### 4. `ListClients() -> Vec<String>`
إرجاع قائمة بالأجهزة وعملاء الويب/الأندرويد المتصلة حالياً.
- **القيمة المرجعة:** `["HTTP Direct /stream", "WebRTC Signaling Active"]`

### 5. `GetConfig() -> String`
إرجاع الإعدادات الحالية بتنسيق نصي JSON.
- **القيمة المرجعة:** `{"width":1920,"height":1080,"refresh_rate":60,"encoder":"auto"}`

---

## 💻 مثال الاستدعاء عبر الأوامر (`busctl`)

```bash
# فحص وتصفح واجهة Orbiscreen D-Bus
busctl --user introspect com.orbiscreen.Daemon /com/orbiscreen/Daemon

# جلب حالة الخادم
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetStatus

# عرض العملاء المتصلين
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon ListClients
```
