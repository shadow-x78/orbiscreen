<div align="center">

# مواصفات المعمارية - Orbiscreen

[![الإصدار](https://img.shields.io/badge/version-0.45.3-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.92%2B-16a34a?style=flat-square&logo=rust)
![المنصة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## اللغة

<a href="ARCHITECTURE.md">🇬🇧 English</a> · <a href="ARCHITECTURE_AR.md">🇸🇦 العربية</a>

---

بني Orbiscreen كمساحة عمل Rust متعددة الحزم (crates) نمطية تفصل بين إنشاء الشاشات الافتراضية، ومحركات التقاط الإطارات، ومرمزات الفيديو المسرعة عتاديا، وحقن الإدخال العكسي، والنقل الشبكي متعدد البروتوكولات (HTTP و UDP و WebTransport و USB AOA).

---

## نظرة عامة على معمارية النظام

عملية واحدة (daemon) على مضيف Linux وعدد غير محدود من العملاء. الفيديو يتدفق في اتجاه واحد (من المضيف إلى العميل) والإدخال يعود في الاتجاه المعاكس:

```mermaid
flowchart TD
    DESK["سطح مكتب Linux"]
    CAP["orbiscreen-capture: إنشاء مخرج افتراضي والتقاط الإطارات (KWin، wlroots، portal، X11)"]
    EVDI["orbiscreen-display: شاشة افتراضية عبر EVDI (مسار اختياري)"]
    ENC["orbiscreen-encode: ترميز H.264 عبر GStreamer (NVENC، VA-API، x264)"]
    TR["orbiscreen-transport: HTTP و UDP و WebTransport و USB AOA؛ اكتشاف mDNS ومصادقة بالتوكن"]
    IN["orbiscreen-input: حقن اللمس والقلم ولوحة المفاتيح في سطح المكتب"]
    AND["تطبيق Android (MediaCodec)"]
    WEB["عميل الويب (WebCodecs)"]

    DESK --> CAP
    DESK --> EVDI
    CAP --> ENC
    EVDI --> ENC
    ENC --> TR
    TR --> AND
    TR --> WEB
    AND -->|أحداث الإدخال| TR
    WEB -->|أحداث الإدخال| TR
    TR --> IN
    IN --> DESK
```

نفس الدورة موصوفة بالكلمات:

1. تنشأ الشاشة الافتراضية عبر `orbiscreen-capture` (مخرج KWin الافتراضي، أو مخرج wlroots headless، أو مخرج portal الافتراضي) أو عبر `orbiscreen-display` (مسار نواة EVDI).
2. يلتقط `orbiscreen-capture` (أو مضخة إطارات EVDI) الإطارات من ذلك المخرج.
3. يحول `orbiscreen-encode` الإطارات من صيغة BGRA إلى H.264 بالمرمزات العتادية.
4. يوزع `orbiscreen-transport` البث على Android (UDP Annex-B، و USB AOA Annex-B، مع تراجع HTTP MPEG-TS) وعلى المتصفحات (WebTransport Annex-B).
5. ترسل العملاء أحداث الإدخال عائدة عبر `orbiscreen-transport`.
6. يحقن `orbiscreen-input` تلك الأحداث في سطح المكتب (uinput، أو XTEST، أو أجهزة wlroots الافتراضية، أو portal RemoteDesktop).

تربط عملية `orbiscreen-daemon` جميع الحزم ببعضها وتوفر خدمة D-Bus؛ واجهة سطح المكتب (`orbiscreen-gui` بتقنية Qt 6 Widgets وC++17) وواجهة الأوامر تتحدثان معها عبر D-Bus.

---

<h2 dir="rtl" align="right">&rlm;طوبولوجيا حزم مساحة العمل</h2>

<div dir="rtl" align="right">

| الحزمة | المسؤولية | التبعيات الرئيسية |
| :--- | :--- | :--- |
| `orbiscreen-core` | الإعدادات المشتركة، وخطة الجلسة/الالتقاط، وتحويل BGRA، ومسارات XDG والكتابة الذرية، والذاكرة المشتركة، وحساب المنافذ، وتقارير الحالة/التشخيص، وهوية المخارج الافتراضية | `serde`، `toml`، `thiserror`، `libc` |
| `orbiscreen-display` | شاشة EVDI الافتراضية على مستوى النواة: موصل DRM، وتوليف EDID، ومضخة الإطار من الذاكرة الإطارية إلى BGRA ‏(`EvdiFramePump`) | `evdi`، `drm-fourcc`، `tokio` |
| `orbiscreen-capture` | إنشاء المخارج الافتراضية (KWin عبر `zkde-screencast`، و wlroots عبر IPC) بالإضافة إلى التقاط الإطارات: portal ‏(ashpd/PipeWire)‏، و wlr-screencopy، و X11 ‏(x11rb)‏، ومضخة الـ damage؛ يفتح خطة `CaptureStep` صريحة ويشارك حوض PipeWire واحدا | `ashpd`، `x11rb`، `wayland-*`، `orbiscreen-core` |
| `orbiscreen-encode` | خطوط أنابيب ترميز H.264: ‏NVENC و VA-API و x264 البرمجي، مع ضبط VBV و GOP لزمن استجابة منخفض | `gstreamer`، `gstreamer-app`، `orbiscreen-core` |
| `orbiscreen-input` | حقن الإدخال العكسي: شاشة لمس وألواح مفاتيح عبر uinput، و XTEST، وأجهزة wlroots الافتراضية (virtual-pointer / virtual-keyboard)، و portal RemoteDesktop | `evdevil`، `ashpd`، `orbiscreen-core` |
| `orbiscreen-transport` | كل واجهات العملاء: نقاط HTTP ‏(`/stream` و `/input` و `/api/*` و `/health`)‏، و UDP Annex-B مع تصحيح أخطاء Reed-Solomon، و WebTransport Annex-B ‏(`wt_port`)‏، وبث USB عبر AOA، وإعلان mDNS، والاقتران، والمصادقة بالتوكن، وتوجيه جلسات العرض لكل عميل؛ طبقة جلسة موحدة ووحدة بروتوكول موحدة | `axum`، `mdns-sd`، `wtransport`، `gstreamer`، `orbiscreen-core`، `orbiscreen-input` |
| `orbiscreen-daemon` | الثنائي الرئيسي الذي يربط كل الحزم؛ تكامل systemd؛ خدمة D-Bus ‏(`com.orbiscreen.Daemon`)‏؛ أداة التشخيص `doctor`؛ مضخات خط الأنابيب المشتركة وتوزيع الإدخال | `zbus`، `clap`، `tokio` |
| `orbiscreen-gui` (C++/CMake، `gui/`) | مركز تحكم سطح المكتب (Qt 6 Widgets): الحالة، والتشغيل والإيقاف، والدقة، والفحص، وشريط النظام عبر D-Bus | `Qt6::Widgets`، `Qt6::DBus`، `Qt6::Network` |

</div>

<h3 dir="rtl" align="right">&rlm;واجهة سطح المكتب Qt</h3>

<div dir="rtl" align="right">

`gui/` مشروع CMake مستقل وليس حزمة Cargo. تطبيق C++17 خالص بـ Qt 6 Widgets، ولا يرتبط إلا بـ `Qt6::Widgets` و`Qt6::DBus` و`Qt6::Network` من `qt6-base`.

- `DaemonClient` يدير نداءات D-Bus غير المتزامنة (`GetStatus` و`Stop` و`SetResolution` و`RenderQr`)، واستدعاءات `QProcess` لـ `systemctl` وواجهة `orbiscreen`، ومسارات التوكن والإعداد والتشغيل التلقائي، وبناء روابط الجلسة وUSB.
- `report.{h,cpp}` يحلل `StatusReport` و`DoctorReport` من JSON؛ وتبقى أنواع Rust في النواة مرجع المخطط.
- `MainWindow` يوفر الترويسة (الشعار، حبة الحالة، زر بدء/إيقاف، وقائمة إضافية تحمل إعادة التشغيل والتحديث)، والتنقل، وشريط الحالة، وإجراءات شريط النظام. وتحت `pages/` صفحات الاتصال والشاشة والإدخال والفحص.
- `uikit` هو نظام التصميم: `Card` و`StatusPill` و`StatTile` و`Switch` و`GlyphWidget`/`LogoWidget` (ترسم المتجهات مباشرة فتبقى حادة عند أي كثافة بكسل) و`PageScroll`/`ContentColumn` (عمود محتوى واحد موسط تتقاسمه الترويسة وشريط التحديث والتنقل وكل البطاقات) و`ResponsiveColumns` (عمودان عند 780px فأكثر، وعمود واحد دونها). و`qrview` يرسم رمز QR. و`theme.h` يملك مقاييس المسافات والزوايا والخطوط، فلا تحمل أي صفحة قيمة هامش أو زاوية عشوائية.
- `I18n` يحفظ قاموسي الإنجليزية والعربية وملف `~/.config/orbiscreen/gui.json`، ويحول الاتجاه إلى RTL للعربية.
- `Theme` يبني لوحة الألوان وورقة الأنماط للوضع الفاتح أو الداكن من إعداد النظام (ويمكن فرضه بـ `ORBISCREEN_THEME=dark|light`)، ويفرض Fusion لثبات المظهر، ويرسم مجموعة الأيقونات المتجهية عبر `QIconEngine`، فلا تعتمد الواجهة على سمة أيقونات النظام. و`paintLogo` يطابق `data/orbiscreen.svg` على شبكة 64 وحدة، مع نسخة مثقلة للأحجام الصغيرة وشريط النظام. و`Theme::logoIcon()` يعيد `QIconEngine`، فيرسم شريط النظام وأيقونة النافذة وأيقونة الإشعار العلامة عند أي حجم يطلبه النظام؛ و`TrayIcon::showMessage` يمررها صراحة بدل `QSystemTrayIcon::MessageIcon` الذي يجعل خادم الإشعارات يرسم رمزا عاما خاصا به.
- أيقونات التطبيق المثبتة ترسم من ملفي SVG الرئيسيين عند كل حجم مطلوب عبر `scripts/generate-icons.sh` (لا بتصغير صورة واحدة كبيرة)، وهو يعيد بناء أيقونات أندرويد القديمة أيضا ويتحقق من تطابق نسختي الشعار ومن كون كل ملف مربعا ومن احتفاظ العلامة بإطارها البصري؛ و`--check` يتحقق دون كتابة. الملفان الرئيسيان هما `data/orbiscreen.svg` (من 48px فأكبر) و`assets/logo/orbiscreen-logo-small.svg` (16/22/24/32px).
- يولد الخادم رمز QR عبر `RenderQr` الذي يعيد `width:base64`. ويرسم `QrView` مصفوفة الوحدات مع هامش هادئ، فلا يحتاج العميل أي حزمة QR.
- يستخدم `QSystemTrayIcon` عند وجود مضيف StatusNotifier؛ وإلا فإن إغلاق النافذة ينهي التطبيق بدل ترك عملية خفية.

</div>

<h3 dir="rtl" align="right">&rlm;الأساسات المشتركة</h3>

<div dir="rtl" align="right">

نقلت إعادة الهيكلة في 0.45.0 كل تعريف مكرر إلى الطبقة الأدنى التي تملكه:

- `orbiscreen-core::{session, frame, paths, shm, ports, report, output}`: خطة الالتقاط وكشف الجلسة، وتحويل BGRA/الـ stride، ومسارات XDG والكتابة الذرية للأسرار، وتعيين الذاكرة المشتركة، وحساب المنافذ، وتقارير `StatusReport`/`DoctorReport`، وأسماء المخارج الافتراضية.
- `orbiscreen-transport::{protocol, session}`: أنواع الإطارات وأطوال الرؤوس ومسارات النقاط، وسياسة الاتصال والقطع وطلب IDR وانتظار الإطار المفتاحي المشتركة بين HTTP Annex-B و MPEG-TS و WebTransport.
- `orbiscreen-daemon::pipeline`: مضخة IDR، وتطبيع PTS، وتحجيم إحداثيات الإدخال، وتوزيع `IncomingInput` المشترك بين مساري العرض المشترك ولكل عميل.
- `clients/web/protocol.js` (يتحقق منه `protocol.golden.json`) و`net/Protocol.kt` في Android هما النظيران في العميل. و`net/HostSpec.kt` هو المرجع الوحيد لتحليل host:port والمخطط وعنوان loopback.

أزيلت جلسة العرض الثانوي المستقلة في 0.45.0 لأنها كانت تحجز `signaling_port + 2`، وهو نفس منفذ WebTransport، فلم تكن تعمل بجانب النقل العادي. تدعم عدة أجهزة لوحية عبر مركز جلسات KWin لكل عميل، بمخرج `Virtual-Orbi-<key>` لكل جهاز.

</div>

---

<h2 dir="rtl" align="right">&rlm;بنية حزم عميل Android</h2>

<div dir="ltr" align="left">

```
com.orbiscreen.android/
├── MainActivity.kt                # مستضيف Compose، يطبق السمة من PrefsStore
├── data/
│   ├── PrefsStore.kt              # السمة، المضيف الأخير، مفتاح الماسح
│   └── HostCredentialStore.kt     # تخزين توكنات المضيفين المحفوظة
├── net/
│   ├── DiscoveryService.kt        # اكتشاف NSD ‏(mDNS)‏ نحو تدفق StateFlow للمضيفين
│   ├── SubnetScanner.kt           # مسح شبكة /24 بتوازي محدد بـ Semaphore
│   ├── HostApi.kt                 # عميل HTTP لنقاط config.json و api/info و api/control و health
│   ├── ClientIdentity.kt          # معرف ثابت لكل جهاز (تجزئة ANDROID_ID)
│   ├── PinnedHostProxy.kt         # التعامل مع شهادات المضيفين الموثوقين
│   ├── UsbLoopback.kt             # مقبس محلي أمام قناة AOA
│   ├── WifiGatewayProvider.kt     # فحص بوابة الشبكة لحاوية ARC++ في ChromeOS
│   └── DiscoveryModel.kt          # تحليل وتعديل مواصفات المضيف
├── player/
│   ├── PlayerHolder.kt            # يختار مسار UDP أو USB أو ExoPlayer حسب النقل المتاح
│   ├── UdpPlayer.kt               # UDP Annex-B نحو MediaCodec
│   ├── UsbPlayer.kt               # AOA Annex-B نحو MediaCodec
│   ├── AuReorder.kt               # إعادة تجميع الحزم وقاعدة "التعليق حتى IDR"
│   ├── Fec.kt / PendingFecStore.kt # تصحيح الأخطاء Reed-Solomon
│   ├── UdpCrypto.kt               # التحقق من أصالة حزم UDP
│   ├── H264.kt / Idr.kt / IdrFrames.kt / AoaFrames.kt # أدوات معالجة تيار H.264
│   ├── StreamStats.kt             # إحصائيات البث (زمن الوصول، عمر الإطار، المعدلات)
│   ├── StreamUrl.kt               # بناء رابط البث مع التوكن
│   └── SurfaceTarget.kt           # إدارة أسطح العرض للمفكك
├── input/
│   └── InputDispatcher.kt         # إرسال الإدخال عبر WebSocket أولا مع تراجع HTTP
├── usb/
│   ├── UsbAccessoryManager.kt     # مصافحة AOA وقراءة/كتابة الملحق
│   ├── AoaAcceptedSocket.kt       # تعريض AOA كمقبض خادم محلي
│   └── UsbPermissionPrompt.kt     # تدفق إذن الملحق
├── updater/
│   └── UpdateManager.kt           # فحص الإصدارات الجديدة وتثبيت APK
└── ui/
    ├── theme/                     # ألوان وسمات Material 3 (لوحة Catppuccin Mocha / Latte)
    ├── nav/OrbiNav.kt             # موجه التنقل (الاستكشاف / البث / الإعدادات)
    ├── discovery/                 # شاشة الاستكشاف ونموذج العرض
    ├── stream/                    # شاشة البث، سطح المشغل، شريط التحكم، طبقة الإحصائيات،
    │                              # مراقب المضيف، شاشة دخول الشبكة المحلية
    └── settings/                  # شاشة الإعدادات (السمة، فك الترميز، الماسح، المضيفون)
```

</div>

---

<h2 dir="rtl" align="right">&rlm;بنية عميل الويب</h2>

<div dir="ltr" align="left">

```
clients/web/
├── index.html                     # هيكل الصفحة والكانفس وأزرار التحكم
├── app.js                         # جلسة WebTransport وإعادة الاتصال وإرسال الإدخال
├── annexb.js                      # بناء وحدات Annex-B لغرف VideoDecoder
└── stats.js                       # طبقة إحصائيات البث
```

</div>

---

## خط أنابيب البث

1. **تهيئة الشاشة الافتراضية:**
   - **واجهة XDG Desktop Portal ScreenCast Virtual:** على GNOME 46+ و KDE Plasma 6+، يطلب `orbiscreen-capture` نوع `SourceType::Virtual` لإنشاء مخرج افتراضي حقيقي بدون صلاحيات root عبر PipeWire.
   - **واجهات الـ IPC:** تنشئ بيئات Sway و Hyprland مخارج headless ديناميكية عبر مقابس التحكم (`$SWAYSOCK` / `hyprctl`).
   - **مشغل EVDI للنواة:** على X11 و COSMIC وجلسات Wayland السابقة، يهيئ `orbiscreen-display` شاشة DRM افتراضية عبر وحدة EVDI.
   - **التراجع للشاشة الرئيسية:** عند تعذر إنشاء شاشة افتراضية، يتراجع النظام تلقائيا  لالتقاط الشاشة الرئيسية عبر portal ScreenCast أو X11 `GetImage`.
2. **التقاط الإطارات:** إطارات BGRA من المخرج الافتراضي (عبر PipeWire، أو screencopy في wlroots، أو KWin virtual، أو ذاكرة evdi الإطارية).
3. **الترميز:**
   - يرمز `orbiscreen-encode` الإطارات إلى H.264 عبر خطوط أنابيب GStreamer المسرعة عتاديا (NVENC أو VA-API مع التراجع البرمجي إلى x264).
   - تولد الإطارات المفتاحية عند الطلب فقط (GOP غير منتهي) مع إعادة إرسال SPS/PPS مع كل إطار IDR ليستطيع العميل المتأخر فك الترميز خلال إطار واحد.
   - تقيد ذاكرة AppSink بـ `drop = true` و `max-buffers = 1` لمنع طوابير الانتظار.
4. **النقل والتشغيل:**
   - **أندرويد عبر Wi-Fi:** يرسل `orbiscreen-transport` وحدات H.264 كحزم UDP Annex-B مع تصحيح Reed-Solomon، ويفكها `UdpPlayer` على MediaCodec.
   - **أندرويد عبر USB:** ترسل نفس الوحدات عبر قناة AOA Bulk مباشرة إلى MediaCodec بدون شبكة.
   - **المتصفح:** يفتح عميل الويب جلسة WebTransport ويفك Annex-B عبر WebCodecs `VideoDecoder` على كانفس.
   - **التراجع:** إذا فشل المسار الأصلي، يبقى مسار HTTP MPEG-TS مع ExoPlayer متاحا.
   - **الانقطاع والاستعادة:** فحص فوري عبر `/health` خلال 500ms مع حد أقصى 3 محاولات إعادة اتصال.
5. **الإدخال العكسي:**
   - ترسل أحداث المؤشر والعجلة والقلم ولوحة المفاتيح عبر قناة WebSocket (مع تراجع HTTP) وتربط بدقة الشاشة الفعلية مع تقييد صارم داخل حدود الشاشة الافتراضية.
   - **لوح الرسم والقلم:** تتبع التحليق في الهواء، وزوايا ميلان مصححة، وحتى 4095 مستوى ضغط ترسل عبر `Dispatchers.IO`.
   - **لوحة اللمس والسحب:** النقر المزدوج مع السحب يحرر زر الفأرة الأيسر تلقائيا عند رفع الإصبع.
6. **التحكم بالمضيف:**
   - يرسل `HostApi.sendControl` إجراءات JSON إلى `/api/control` لطلبات القفل والتعتيم و ctrl-alt-del، مع التوكن المعتمد.

---

## الأمان والمصادقة

يولد لكل جلسة توكن عشوائي من 32 بايت بترميز base64url عند بدء التشغيل:

- **تمهيد العملاء:** توفر النقطة `/client/config.json` توكن الجلسة وأبعاد العرض للتمهيد التلقائي لعملاء الويب والشبكة المحلية.
- **مصادقة المتصفحات البعيدة:** تدعم المتصفحات المصادقة الآمنة عبر تجزئة الرابط (`#token=<SECRET>`) أو الاستعلام (`?token=`) دون تسريب التوكن في سجلات الخادم.
- **عميل Android:** يستقبل التوكن عبر سجلات mDNS TXT أو بإدخاله يدويا.
- **حماية الملفات:** يحفظ الـ daemon التوكن في `~/.config/orbiscreen/token`، وشهادة WebTransport في `wt-cert.pem` / `wt-key.pem` (صالحة 14 يوما وتجدد عند البدء)، وشهادة مستمع HTTPS في `https-cert.pem` / `https-key.pem` (صالحة 10 سنوات)، وكلها بصلاحيات `0o600` والمجلدات الأب بصلاحيات `0o700`. شهادة HTTPS هي التي تثبت عليها أجهزة Android المقترنة بصمة SHA-256، لذا لا تتغير عند تجديد شهادة WebTransport؛ ويسجل الـ daemon بصمتها عند البدء.
- **حماية النقاط:** يطلب التوكن إجباريا على `/stream` و `/input` و `/api/control` عبر ترويسة `Authorization: Bearer <token>` أو الاستعلام `?token=` (بمقارنة بزمن ثابت).
- تبقى `/health` و `/api/info` عامة لكي يعمل الاكتشاف وفحص الحيوية بدون بيانات اعتماد.

---

<h2 dir="rtl" align="right">&rlm;عقد واجهة HTTP API</h2>

<div dir="rtl" align="right">

| النقطة | الطريقة | المصادقة | الوصف |
| :--- | :---: | :---: | :--- |
| `/` | `GET` | عامة | إعادة التوجيه إلى عميل الويب المضمن |
| `/stream` | `GET` | توكن | بث فيديو MPEG-TS ‏(`video/mp2t`)‏ |
| `/input` | `POST` | توكن | أحداث المؤشر / المفاتيح / القلم بصيغة JSON، الاستجابة `144` |
| `/api/control` | `POST` | توكن | الإجراءات: `lock`، `blank`، `unblank`، `ctrl_alt_del`، `idr`؛ `144` عند النجاح |
| `/api/info` | `GET` | عامة | أبعاد الشاشة ومعدل التحديث والمرمز والإصدار بصيغة JSON |
| `/health` | `GET` | عامة | `200 OK "ok"` |

</div>

تقبل نقطة `/input` نفس مخطط الحمولة المستخدم لدى عميل الويب: `Pointer` ‏(‏`Move` / `Button` / `Wheel`‏)‏، و `Key` ‏(بترميز مفاتيح evdev‏)‏، و `Stylus` ‏(الإحداثيات والضغط وزوايا الميلان‏).

---

## الشيفرة غير الآمنة

يقتصر `unsafe` على وحدات FFI محدودة. الشروط أدناه هي العقد لكل كتلة، وهي موثقة هنا بدلًا من داخل الملفات لأن سياسة المشروع تمنع التعليقات في الشيفرة.

- `orbiscreen-core/src/shm.rs`: استدعاءات `memfd_create` و`ftruncate` و`mmap`؛ يملك `File::from_raw_fd` الواصف مرة واحدة، ويبقى التعيين صالحًا بطوله حتى `munmap`، ولا يوجد تعيين ثانٍ يتشارك النطاق نفسه.
- `orbiscreen-transport/src/aoa.rs`: واجهات `USBDEVFS_CONTROL` و`USBDEVFS_BULK` على واصف جهاز حي؛ تُهيّأ بنى الطلب وتطابق أطوالها، ويُفحص عدد البايتات المُعاد قبل استخدامه.
- `orbiscreen-capture/src/wlr_screencopy.rs` و`wlr_virtual_output.rs` و`kwin_virtual.rs` و`orbiscreen-input/src/wlroots.rs`: استدعاءات `libc::poll` و`BorrowedFd::borrow_raw` وبروتوكولات Wayland؛ يبقى الواصف وكائن البروتوكول حيين طوال الاستدعاء أو الاستعارة.
- `orbiscreen-transport/src/udp_stream.rs` و`orbiscreen-display/src/lib.rs`: `setsockopt` وفتح عقدة DRM؛ الواصف صالح ومخزن الخيار بالحجم الصحيح ويُحوّل كل خطأ إلى خطأ مُنمّط.

يجب أن يحافظ أي تعديل على إحدى هذه الكتل على الشروط أعلاه وأن يبقي اختبارات الحزمة ناجحة؛ `cargo test --workspace` هو بوابة التحقق.

## تحسينات النقل

- **GOP غير منتهي + IDR عند الطلب:** المرمزات لا تبذر عرضها بإطارات مفتاحية دورية؛ العميل الملتحق حديثا أو المتأخر يطلب إطار IDR فيستلم SPS/PPS معه ويفك الترميز خلال إطار واحد.
- **تعدد الإرسال لكل عميل:** كل طلب `/stream` يفتح مسار `appsrc → mpegtsmux → appsink` خاصا به مع `h264parse config-interval=1` لإعادة بث SPS/PPS مع كل إطار مفتاحي.
- **بوابة ARC++ في ChromeOS:** يفحص عميل Android البوابات الداخلية لحاوية ARC++ ‏(`100.115.92.2`، `192.168.233.1`) على منفذ الإشارة؛ أما بث USB نفسه فيتم عبر AOA وليس `adb reverse`.
- **OkHttpDataSource:** مهلة قراءة صفرية، ومقبس طويل العمر، وترويسة `User-Agent: Orbiscreen-Android/1.0` لسجلات خادم أوضح.
- **DefaultLoadControl:** تخزين مؤقت بين 40ms و120ms يبقي العرض عند أحدث إطار ويمنع التراكم.
- **قناة البث:** `video_tx` هي `tokio::sync::broadcast` مقيدة الحجم؛ العميل البطيء يتخطى إلى الإطار المفتاحي التالي بدل أن يضغط عكسيا على المرمز.
- **بلا Protobuf:** تستخدم الحمولات `org.json.JSONObject` في كلا الاتجاهين للحفاظ على عقد سلكي متماثل مع عميل الويب.

---

## دورة الحياة

- يمتلك `StreamViewModel` كائن `PlayerHolder`؛ يحدث التحرير داخل `onCleared()`.
- ينشأ `InputDispatcher` كسولا عند أول لمسة ويحرر مع المشغل.
- يبدأ `DiscoveryService` في `DiscoveryViewModel.init` ويفصل مع نطاق ViewModel.
- تتوقف مضخة الإطارات عند إسقاط المستقبل؛ ويلغي الـ daemon حلقة الالتقاط بأمان عند SIGINT أو إيقاف D-Bus.

---

<div align="center">

بني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[العودة إلى README](../README_AR.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
