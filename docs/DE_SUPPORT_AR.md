<div align="center">

# دعم بيئات سطح المكتب - Orbiscreen

[![الإصدار](https://img.shields.io/badge/version-0.18.3-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![المنصّة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 اللغة

<a href="DE_SUPPORT.md">🇬🇧 English</a> · <a href="DE_SUPPORT_AR.md">🇸🇦 العربية</a>

---

يتكيّف Orbiscreen مع بيئة سطح المكتب التي يعمل عليها. تسرد هذه الوثيقة لكل
عائلة من الـ compositors مسار الشاشة الافتراضية المستخدم، والمتطلبات، وحلول
الأعطال الشائعة.

ابدأ بالفحص؛ يطبع كل ما في هذه الصفحة *لجهازك*:

```bash
orbiscreen doctor          # بصيغة مقروءة
orbiscreen doctor --json   # صيغة آلية
orbiscreen doctor --fix    # تثبيت/تحميل وحدة نواة EVDI عندما يكون ذلك ممكنا
```

## كيف يقرر `auto`

يقرأ الدامن البيئة (`XDG_SESSION_TYPE`, `XDG_CURRENT_DESKTOP`,
`WAYLAND_DISPLAY`, `SWAYSOCK`, `HYPRLAND_INSTANCE_SIGNATURE`, ...) ويبني خطة
التقاط مرتّبة. تُسجَّل الخطة في السجل عند كل `orbiscreen start`:

| البيئة | خطة التقاط `auto` (بالترتيب) |
|---|---|
| KDE Plasma (Wayland) | ‏`kwin-virtual` ← `portal` |
| Sway / Hyprland / باقي wlroots | ‏`wlroots-virtual` ← `wlr-screencopy` ← `portal` ← `evdi` |
| GNOME / Wayland آخر (غير wlroots) | ‏`portal` |
| X11 (أي بيئة) | ‏`evdi` ← `x11-root` (عكس عبر XShm) |
| جلسة غير معروفة | ‏`portal` ← `x11-root` (أو `evdi` ← `x11-root` عندما لا تُعلن أي بيئة) |

- ‏`wlroots-virtual` يفشل فوراً وينتقل للتالي عندما لا يكون IPC الـ compositor
  متاحاً، لذا تغطي الخطة أعلاه أيضاً compositors عائلة wlroots بلا دعم
  للمخرجات الافتراضية.
- ‏`wlroots-virtual` و`kwin-virtual` و`evdi` تنشئ **شاشة ثانية حقيقية** تبدأ
  فارغة؛ اسحب النوافذ إليها.
- ‏`wlr-screencopy` و`x11-root` و`portal` **تعكس شاشة موجودة**.

## KDE Plasma (Wayland)

- **الشاشة الافتراضية:** أصلية عبر `zkde_screencast_unstable_v1`. بلا root،
  بلا وحدة نواة، بلا نافذة مشاركة. تظهر الشاشة باسم `Virtual-ORBISCREEN`.
- **الالتقاط:** بث PipeWire من المونيتور الافتراضي.
- **الإدخال:** portal RemoteDesktop (يُحفظ الإذن بعد أول تشغيل، لا حوار بعد
  ذلك).
- **لا شيء يُثبَّت.**

## Sway وعائلة wlroots عموماً

- **الشاشة الافتراضية:** تُنشأ عبر IPC الـ compositor. يكشف Sway عن backend
  الـ headless المدمج: يرسل الدامن `create_output` عبر مقبس IPC ‏(`$SWAYSOCK`)،
  وينتظر ظهور المخرج (مثل `HEADLESS-2`)، ويطبّق الوضع المطلوب، ويعطّل المخرج
  عند توقف الدامن. لا يملك Sway أمر IPC لحذف مخرج headless أُنشئ ديناميكياً،
  لذلك يعطّله الدامن كأقرب تنظيف ممكن (يتوقف عن استقبال الإطارات) ويستعيده
  الـ compositor عند خروجه.
- **الالتقاط:** `zwlr_screencopy_manager_v1` (البروتوكول نفسه الذي يستخدمه
  ‏`grim`): بلا portal، بلا حوار، مدفوع بالـ damage، باسم المخرج.
- **الإدخال:** `virtual-keyboard-unstable-v1` + `wlr-virtual-pointer-unstable-v1`
  مباشرة على مقبس Wayland، لا حاجة إلى `xdg-desktop-portal-wlr`.
- **المتطلبات:** ‏Sway ≥ 1.6 (أو أي wlroots compositor يدعم `create_output`).
  إن لم يكن IPC متاحاً يتراجع `auto` إلى عكس مخرج موجود عبر screencopy.
- **حلول الأعطال:**
  - ‏`doctor` يطبع `virtual out: no compositor IPC reachable`؛ تحقق من أن
    ‏`SWAYSOCK` معرّف في بيئة الدامن (وحدات systemd للمستخدم ترثه من الجلسة).
  - الالتقاط يفشل بـ `zwlr_screencopy_manager_v1 is not available`؛ عطّل
    الـ compositor الـ screencopy؛ العكس عبر portal ما زال يعمل.

## Hyprland

- **الشاشة الافتراضية:** IPC بأسلوب `hyprctl` على
  ‏`$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`:
  ينشئ الدامن مخرجاً headless ويدمّره عند الإيقاف.
- **الالتقاط / الإدخال:** كما في Sway أعلاه (screencopy + أجهزة إدخال
  افتراضية أصلية).
- **المتطلبات:** ‏Hyprland ≥ 0.44.

## GNOME (Wayland / Mutter)

لا تملك Mutter واجهة عامة لإضافة مونيتور افتراضي داخل جلسة رسومية قائمة،
لذلك:

- **الشاشة الافتراضية:** عبر **EVDI** (وحدة نواة): المسار الموجَّه هو
  ‏`orbiscreen doctor --fix`.
- **الالتقاط:** ‏portal ScreenCast. منذ v0.13.0 **يُحفظ إذن** المشاركة
  (restore token في `$XDG_STATE_HOME/orbiscreen/portal.json`): تظهر نافذة
  المشاركة في أول تشغيل فقط، ولا تظهر مجدداً (إلا إذا أُلغي الإذن).
- **الإدخال:** ‏portal RemoteDesktop، ويُحفظ بالمثل.
- **حلول الأعطال:**
  - يظهر الحوار في كل تشغيل → الـ backend لا يحترم restore tokens، أو حُذف
    ملف الحالة. يطبع `doctor`: ‏`screencast grant saved: yes/no`.
  - ‏`portal: org.freedesktop.portal.Desktop NOT on the session bus` → ثبّت
    ‏`xdg-desktop-portal` مع backend الخاص بـ GNOME.

## X11 ‏(XFCE, MATE, LXQt, Cinnamon, Budgie, KDE-X11)

- **الشاشة الافتراضية:** ‏**EVDI** هو مسار التمديد الحقيقي الوحيد على X11
  (يثبّته `orbiscreen doctor --fix` على التوزيعات المكتشفة).
- **الالتقاط:** عكس الشاشة الجذرية عبر **MIT-SHM**: صورة مشتركة واحدة دائمة
  يكتب فيها خادم X مباشرة (بلا حمولة رد لكل إطار)، مع مجمّعات إطارات
  وتجاوز تلقائي للإطارات المطابقة للإطار السابق. يتراجع إلى `GetImage`
  العادي عند غياب MIT-SHM ≥ 1.2.
- **الإدخال:** حقن **XTEST**: بلا root، يعمل لأي مستخدم على أي X11. يبقى
  ‏uinput كخيار أقوى (يحتاج `/dev/uinput`).
- **حلول الأعطال:**
  - ‏`display: kernel module missing` → ‏`orbiscreen doctor --fix`، أو البناء
    من المصدر: ‏`bash scripts/install-evdi-module.sh`.
  - لا إدخال بلا root → كان يجب أن يعمل XTEST؛ راجع سطر واجهة الإدخال في
    مخرجات `doctor`.

## أي بيئة أخرى

يتراجع `auto` عبر المتاح: عكس عبر portal على Wayland، عكس XShm على X11،
وEVDI في كل مكان. يطبع `orbiscreen doctor` بالضبط أي خطوة ناقصة وكيف تُصلح.

## تفضيل واجهة الالتقاط (`orbiscreen.toml`)

يقرأ الخادم افتراضيًا `$XDG_CONFIG_HOME/orbiscreen/orbiscreen.toml`
(أو `~/.config/orbiscreen/orbiscreen.toml` عندما لا يكون `XDG_CONFIG_HOME` معرّفًا)،
وهو المسار نفسه الذي تستخدمه وحدة systemd للمستخدم. أنشئ الملف هناك، أو
حدّد موقعًا آخر عبر `--config /path/to/orbiscreen.toml`.

```toml
[capture]
preferred = "auto"
```

| القيمة | السلوك |
|--------|--------|
| `auto` | KDE Plasma Wayland: شاشة KWin الافتراضية. ‏Sway/Hyprland/wlroots: مخرج افتراضي من الـ compositor عبر IPC، وإلا عكس شاشة موجودة عبر wlr-screencopy، وإلا portal. ‏X11: ‏EVDI عند تحميل وحدتها، وإلا التقاط الشاشة الجذر. |
| `kwin-virtual` | شاشة KWin الافتراضية دائماً (فشل صريح على غير KDE). |
| `screencopy` | التقاط wlroots screencopy دائماً (يتطلب compositor من عائلة wlroots). |
| `evdi` | شاشة EVDI DRM الافتراضية دائماً (اختيارية، تتطلب وحدة نواة مثبتة بـ root). |
| `portal` | نافذة مشاركة portal دائماً؛ اختر أي شاشة. |
| `mirror` | اعرض **سطح مكتبك الحقيقي** بدل شاشة ثانية: اختر الشاشة المراد عكسها من نافذة المشاركة. |

> الشاشة الافتراضية تبدأ **فارغة** (خلفية سطح المكتب فقط)، هذا معنى الشاشة الثانية. اسحب النوافذ إلى `Virtual-ORBISCREEN`، أو استخدم `mirror` لبث شاشتك الفعلية.

## متغيرات البيئة التي يقرؤها Orbiscreen

| المتغير | الغرض |
|---|---|
| ‏`XDG_SESSION_TYPE`, `WAYLAND_DISPLAY`, `DISPLAY` | التمييز بين Wayland وX11 |
| ‏`XDG_CURRENT_DESKTOP`, `DESKTOP_SESSION`, `KDE_FULL_SESSION` | عائلة الـ compositor |
| ‏`SWAYSOCK` | ‏IPC الخاص بـ Sway للمخرجات الافتراضية |
| ‏`HYPRLAND_INSTANCE_SIGNATURE`, `XDG_RUNTIME_DIR` | مسار مقبس IPC الخاص بـ Hyprland |
| ‏`XDG_STATE_HOME` | موقع أذونات portal المحفوظة (`orbiscreen/portal.json`) |
