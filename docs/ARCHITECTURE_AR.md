# مواصفات معمارية النظام - Orbiscreen

---

## 🌐 اللغة

<a href="ARCHITECTURE.md">🇬🇧 English</a> · <a href="ARCHITECTURE_AR.md">🇸🇦 العربية</a>

---

## 🏛 الهيكل المعماري للنظام

بُني مشروع **Orbiscreen** كمنظومة برمجية قياسية مقسمة على عدة حزم (Rust Workspace) لفصل محركات الشاشات، والالتقاط، والترميز، والتواصل عبر D-Bus، وبروتوكولات النقل الشبكي:

```mermaid
graph TD
    subgraph Control_Layer [طبقة التحكم والإدارة]
        GTK["واجهة GTK4 / Libadwaita (orbiscreen-gtk)"]
        TRAY["أيقونة شريط النظام (System Tray)"]
        CLI["واجهة السطر الأوامر (orbiscreen)"]
        DBUS["ناقل D-Bus Session Bus (com.orbiscreen.Daemon)"]
    end

    subgraph Service_Layer [المحرك الأساسي والخادم]
        DAEMON["الخادم الأساسي (orbiscreen-daemon)"]
        CORE["المحرك والإعدادات (orbiscreen-core)"]
    end

    subgraph Display_Capture_Layer [طبقة العرض والالتقاط]
        DRM["الشاشة الافتراضية EVDI DRM (orbiscreen-display)"]
        PORTAL["التقاط Wayland ScreenCast Portal (orbiscreen-capture)"]
        X11_CAP["التقاط X11 (orbiscreen-capture)"]
    end

    subgraph Encode_Transport_Layer [طبقة الترميز والبث]
        GSTREAMER["مرمز H.264 (orbiscreen-encode)"]
        TRANSPORT["شبكة النقل Axum HTTP / WebRTC (orbiscreen-transport)"]
        MDNS["اكتشاف الأجهزة mDNS SD (mdns-sd)"]
    end

    subgraph Input_Layer [طبقة حقن اللمس والإدخال]
        UINPUT["محاقن اللمس والقلم uinput (orbiscreen-input)"]
    end

    GTK <-->|D-Bus Session IPC| DBUS
    TRAY <-->|D-Bus Session IPC| DBUS
    CLI <-->|D-Bus Session IPC| DBUS
    DBUS <--> DAEMON

    DAEMON --> CORE
    DAEMON --> DRM
    DAEMON --> PORTAL
    DAEMON --> GSTREAMER
    DAEMON --> TRANSPORT
    DAEMON --> UINPUT

    TRANSPORT <-->|HTTP /stream & WebRTC| Clients[تطبيق الأندرويد ومشغل الويب]
    Clients -->|أحداث اللمس والقلم ولوحة المفاتيح| TRANSPORT
    TRANSPORT --> UINPUT
```

---

## 📦 توزيع الحزم ومسؤولياتها

| الحزمة | المسؤولية | المكتبات الرئيسية |
|-------|----------------|------------------|
| `orbiscreen-core` | الإعدادات المشتركة، أنواع الأخطاء، الترميز | `serde`, `toml` |
| `orbiscreen-display` | إنشاء الشاشات الافتراضية EVDI وتوليد EDID | `evdi`, `libc` |
| `orbiscreen-capture` | التقاط الشاشة عبر Wayland Portal و X11 | `ashpd`, `x11rb` |
| `orbiscreen-encode` | خطوط ترميز الفيديو H.264 بالعتاد والبرمجيات | `gstreamer`, `gstreamer-app` |
| `orbiscreen-input` | حقن أحداث اللمس والقلم ولوحة المفاتيح في النواة | `evdevil`, `nix` |
| `orbiscreen-transport` | بث `/stream` عبر Axum و WebRTC و ADB reverse | `axum`, `webrtc`, `tokio` |
| `orbiscreen-daemon` | الملف التنفيذي للخدمة وتوفير واجهة D-Bus | `zbus`, `clap`, `tokio` |
| `orbiscreen-gtk` | لوحة التحكم الرسومية بسطح المكتب GTK4 / Libadwaita | `gtk4`, `libadwaita`, `zbus` |
