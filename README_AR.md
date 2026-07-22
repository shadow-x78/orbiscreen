<div align="center">

<pre align="center">
   ___  ___   ___ _  ___ ___ _ _
  / _ \/ -_) / _ `/ / -_) _ `/ _\
  \___/\__/  \_,_/  \__/\_,_/\__/
</pre>

# Orbiscreen

شاشات فرعية افتراضية مفتوحة المصدر للينكس، تُبَثّ إلى أجهزة أندرويد عبر Wi-Fi أو USB

[![الإصدار](https://img.shields.io/badge/version-0.1.0-2563eb?style=flat-square&logo=semver)](CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](LICENSE)
![اللغة](https://img.shields.io/badge/rust-edition_2021-9333ea?style=flat-square&logo=rust)
![المنصة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-16a34a?style=flat-square&logo=linux)
[![النجوم](https://img.shields.io/github/stars/shadow-x78/orbiscreen?style=flat-square&color=eab308&logo=github&label=النجوم)](https://github.com/shadow-x78/orbiscreen/stargazers)

</div>

---

## 🌐 اللغة

<a href="README.md">🇬🇧 English</a> · <a href="README_AR.md">🇸🇦 العربية</a>

---

## 📋 فهرس المحتويات

- [ما هو Orbiscreen؟](#what-is-orbiscreen)
- [لماذا Orbiscreen؟](#why)
- [أبرز المزايا](#highlights)
- [حالة المشروع](#status)
- [البدء السريع](#quick-start)
- [هيكل المشروع](#project-structure)
- [التوثيق](#documentation)
- [المساهمة](#contributing)
- [الرخصة](#license)

---

<a id="what-is-orbiscreen"></a>
## 🤔 ما هو Orbiscreen؟

**Orbiscreen** يحوّل جهاز أندرويد إضافي (تابلت أو هاتف) إلى شاشة ثانوية حقيقية لسطح مكتب لينكس. يُنشئ **شاشة افتراضية على مستوى النواة** عبر وحدة `evdi` من DisplayLink، فتظهر كشاشة حقيقية لكل من مركّبات X11 وWayland، ثم يبثّها عبر **WebRTC** مع إرجاع أحداث اللمس إلى المضيف.

<a id="why"></a>
## 💡 لماذا Orbiscreen؟

| المشكلة | المشاريع الأخرى | Orbiscreen |
|---------|----------------|------------|
| `spacedesk` بلا دعم لينكس كمضيف | ❌ أعلنوا أنه ليس مخططاً | ✅ لينكس كمضيف أصلي |
| `VirtScreen` على X11 فقط | ❌ متوقف منذ 2018 | ✅ X11 و Wayland |
| `Weylus` شاشة ثانية X11 فقط | ❌ Wayland بوضع المتصفح فقط | ✅ شاشة افتراضية حقيقية |
| لا مشروع يجمع الشاشة الافتراضية + USB + أندرويد | ❌ فجوة | ✅ حل متكامل |

<a id="highlights"></a>
## ⭐ أبرز المزايا

| الميزة | الوصف |
|--------|-------|
| **شاشة افتراضية حقيقية** | عبر `evdi` — تعمل على X11 و Wayland |
| **بث WebRTC** | يفتح من أي متصفح حديث، دون تثبيت تطبيق |
| **لمس عكسي** | الجهاز يرسل أحداث المؤشر ولوحة المفاتيح إلى المضيف |
| **اكتشاف mDNS** | عميل أندرويد يجد المضيف تلقائياً |
| **نقل USB** | عبر `adb reverse` دون تعريفات خاصة |
| **ترميز بالعتاد** | VAAPI لـ Intel/AMD، NVENC لـ NVIDIA، x264 احتياطياً |

<a id="status"></a>
## 📊 حالة المشروع

| المرحلة | الهدف | الحالة |
|---------|-------|--------|
| 0 | هيكل مساحة العمل + جدوى evdi | ✅ مكتملة |
| 1 | شاشة + التقاط + ترميز + إدخال (X11) | ⚠️ هيكل — بث WebRTC معلّق |
| 2 | عميل أندرويد + USB + mDNS | ⚠️ هيكل |
| 3 | التقاط + إدخال Wayland | ⚠️ هيكل — PipeWire DMA-BUF معلّق |
| 4 | التغليف + ميزات متقدمة | ⚠️ هيكل |

> انظر `CHANGELOG.md` للسجل الكامل للإصدارات.

<a id="quick-start"></a>
## 🚀 البدء السريع

```bash
# استنساخ المستودع
git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
cd ~/Orbiscreen

# البناء
. "$HOME/.cargo/env"
cargo build --workspace
cargo test  --workspace

# فحص النظام المحلي
cargo run -p orbiscreen-daemon -- probe

# تشغيل الخادم
cargo run -p orbiscreen-daemon -- start
```

> دليل التبعيات الكامل في `scripts/setup-dev-env.sh`، وفحص جدوى evdi في `scripts/test-evdi.sh`.

<a id="project-structure"></a>
## 🏗️ هيكل المشروع

```
orbiscreen/
├── crates/
│   ├── orbiscreen-core/        # أنواع، إعداد، أخطاء
│   ├── orbiscreen-display/     # شاشات افتراضية عبر evdi
│   ├── orbiscreen-capture/     # X11 (x11rb) + Wayland (ashpd + PipeWire)
│   ├── orbiscreen-encode/      # خط GStreamer (VAAPI / NVENC / x264)
│   ├── orbiscreen-input/       # evdevil + ashpd RemoteDesktop
│   ├── orbiscreen-transport/   # axum + mDNS + adb reverse + إشارة
│   └── orbiscreen-daemon/      # واجهة CLI تربط كل الطبقات
├── clients/
│   ├── web/                    # عميل المتصفح WebRTC (HTML / CSS / JS)
│   └── android/                # تطبيق Kotlin أندرويد WebView
├── packaging/{flatpak,appimage,debian}/
├── scripts/{setup-dev-env.sh,test-evdi.sh}
├── .github/{workflows/,ISSUE_TEMPLATE/,dependabot.yml}
└── .editorconfig, .gitignore, .gitattributes, deny.toml, rustfmt.toml
```

<a id="documentation"></a>
## 📚 التوثيق

| المستند | الوصف |
|---------|-------|
| [CHANGELOG.md](CHANGELOG.md) | سجل الإصدارات |
| [SECURITY.md](SECURITY.md) | سياسة الأمان والإبلاغ عن الثغرات |

<a id="contributing"></a>
## 🤝 المساهمة

1. افرد المستودع (Fork)
2. أنشئ فرعاً: `git checkout -b feature/my-feature`
3. أكد التغييرات (Commit)
4. ادفع إلى الفرع (Push)
5. افتح طلب دمج (Pull Request)

---

<a id="license"></a>
## 📜 الرخصة

موزّع تحت رخصة [GPL-3.0-or-later](LICENSE).

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[سجل التغييرات](CHANGELOG.md) ·
<a href="README.md">English</a>

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
