<div align="center">

# دعم بيئات سطح المكتب - Orbiscreen

[![الإصدار](https://img.shields.io/badge/version-0.23.4-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الإصدار](https://img.shields.io/badge/version-0.23.5-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
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
| KDE Plasma (Wayland) | ‏`portal-virtual` ← `kwin-virtual` ← `portal` |
| GNOME (Wayland) | ‏`portal-virtual` ← `evdi` ← `portal` |
| COSMIC (Wayland) | ‏`evdi` ← `portal` |
| Sway / Hyprland / باقي wlroots | ‏`wlroots-virtual` ← `wlr-screencopy` ← `portal` ← `evdi` |
| X11 (أي بيئة) | ‏`evdi` ← `x11-root` (عكس عبر XShm) |
| جلسة غير معروفة | ‏`portal-virtual` ← `portal` ← `x11-root` (أو `evdi` ← `x11-root` عندما لا تُعلن أي بيئة) |

- ‏`wlroots-virtual` يفشل فوراً وينتقل للتالي عندما لا يكون IPC الـ compositor
  متاحاً، لذا تغطي الخطة أعلاه أيضاً compositors عائلة wlroots بلا دعم
  للمخرجات الافتراضية.
- ‏`portal-virtual` و`wlroots-virtual` و`kwin-virtual` و`evdi` تنشئ **شاشة ثانية حقيقية** تبدأ
  فارغة؛ اسحب النوافذ إليها.
- ‏`wlr-screencopy` و`x11-root` و`portal` **تعكس شاشة موجودة**.

## KDE Plasma (Wayland)

- **الشاشة الافتراضية:** أصلية عبر `zkde_screencast_unstable_v1` أو XDG Portal ScreenCast `SourceType::Virtual`. بلا root،
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

بدءاً من إصدار GNOME 46+، تدعم Mutter إنشاء شاشات افتراضية بدون صلاحيات root عبر واجهة XDG Desktop Portal ScreenCast API:

- **الشاشة الافتراضية:**
  - **مخرج افتراضي عبر Portal (GNOME 46+):** يطلب Orbiscreen نوع `SourceType::Virtual` عبر `ashpd::desktop::screencast`. عند دعم Mutter لها، تُنشأ شاشة افتراضية مستقلة فورياً بلا حاجة لصلاحيات root أو وحدات نواة أو تعديل إعدادات مدير العرض.
  - **التراجع إلى EVDI (ما قبل GNOME 46 أو الأنظمة غير المتوافقة):** مشغل DRM افتراضي على مستوى النواة (يساعدك أمر `orbiscreen doctor --fix` في إعداده).
- **الالتقاط:** portal ScreenCast (عبر PipeWire). **يُحفظ تصريح** المشاركة تلقائياً (restore token في `$XDG_STATE_HOME/orbiscreen/portal.json`): تظهر نافذة المشاركة في أول تشغيل فقط، ولا تظهر مجدداً (إلا إذا أُلغي الإذن).
- **الإدخال:** portal RemoteDesktop، ويُحفظ بالمثل.
- **حلول الأعطال:**
  - يظهر الحوار في كل تشغيل -> الـ backend لا يحترم restore tokens، أو حُذف ملف الحالة. يطبع `doctor`: ‏`screencast grant saved: yes/no`.
  - `portal: org.freedesktop.portal.Desktop NOT on the session bus` -> ثبّت `xdg-desktop-portal` مع backend الخاص بـ GNOME.

## بيئة COSMIC ‏(Wayland / cosmic-comp)

بيئة سطح المكتب COSMIC من تطوير System76 ‏(`cosmic-comp` المبني على Smithay) مدعومة في Orbiscreen كالتالي:

- **الشاشة الافتراضية:** عبر **EVDI** (مشغل DRM الافتراضي للنواة). عند تحميل وحدة نواة EVDI، ينشئ Orbiscreen منفذ DRM افتراضياً على مستوى العتاد (`/dev/dri/card*`) يكتشفه `cosmic-comp` تلقائياً عبر أحداث DRM uevents. يمكنك ضبط الدقة ومعدل التحديث وترتيب الشاشات مباشرة من إعدادات COSMIC Settings. شغّل `orbiscreen doctor --fix` لتثبيت وحدة نواة EVDI تلقائياً.
- **الالتقاط:** بث PipeWire عبر `xdg-desktop-portal-cosmic`. في حال عدم تثبيت EVDI، يتراجع `auto` إلى عكس شاشة موجودة عبر الـ portal. يحفظ Orbiscreen توكن تصريح مشاركة الشاشة في ملف الحالة (`$XDG_STATE_HOME/orbiscreen/portal.json`)، وبذلك تمنح الإذن لمرة واحدة فقط.
- **الإدخال:** جهاز حقن `/dev/uinput` بدون root يوفّر إيماءات اللمس المتعدد الأصلية، الفأرة، لوحة المفاتيح، ولوح الرسم بالقلم مع 4095 مستوى من حساسية الضغط والميلان. يتعرف مكدس `libinput` في `cosmic-comp` على أجهزة الإدخال الافتراضية فوراً.
- **حلول الأعطال:**
  - ‏`virtual display: kernel module missing` ← شغّل `orbiscreen doctor --fix` (على Pop!_OS وأوبونتو: `sudo apt install evdi-dkms`، فيدورا: `sudo dnf install evdi`، آرتش: `sudo pacman -S evdi`).
  - نافذة مشاركة الشاشة تظهر في كل مرة ← تأكد من تثبيت `xdg-desktop-portal-cosmic` وأن ملف `$XDG_STATE_HOME/orbiscreen/portal.json` قابل للكتابة.

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

## أجهزة Chromebook و ChromeOS (ASUS CM3001 و ARC++)

عند تشغيل تطبيق Orbiscreen على نظام ChromeOS (مثل جهاز ASUS Chromebook CM3001 أو أي جهاز لوحي يدعم تطبيقات الأندرويد عبر ARC++):

- **عزل شبكة الأندرويد:** يعمل نظام أندرويد داخل حاوية معزولة (ARC++) خلف جسر NAT افتراضي، ويُعيّن لها عنوان IP ضمن النطاق `100.115.92.0/28` (البوابة الافتراضية `100.115.92.2`).
- **استكشاف نفق ADB الداخلي تلقائياً:** يقوم Orbiscreen بفحص المنفذ الداخلي `100.115.92.2:5555` تلقائياً إلى جانب `localhost:5555` لربط نفق البث السلكي فورياً داخل ChromeOS.
- **تكامل القلم الذكي (Stylus):** تدعم أقلام USI تتبع الحركة أثناء التحليق بالهواء ومستويات الضغط والميلان. وتتم معالجة أحداث القلم في خيوط خلفية عبر `Dispatchers.IO` لمنع تجميد واجهة التطبيق.
- **الإعداد عبر لينكس داخل ChromeOS (Crostini):**
  1. فعّل **Linux development environment** من إعدادات ChromeOS.
  2. فعّل **Develop Android apps** -> **Enable ADB debugging**.
  3. شغّل `orbiscreen start` داخل طرفية لينكس.

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
