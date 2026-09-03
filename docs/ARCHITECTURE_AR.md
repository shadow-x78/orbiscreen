<div align="center">

# مواصفات المعمارية - Orbiscreen

[![الإصدار](https://img.shields.io/badge/version-0.18.1-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-16a34a?style=flat-square&logo=rust)
![المنصّة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## 🌐 اللغة

<a href="ARCHITECTURE.md">🇬🇧 English</a> · <a href="ARCHITECTURE_AR.md">🇸🇦 العربية</a>

---

بُني Orbiscreen كمساحة عمل Rust متعددة الحزم (crates) نمطية تفصل بين مشغّلات الشاشة الافتراضية (evdi أساساً مع تراجع portal)، ومُرمَّزات الفيديو المسرَّعة، والاتصال بين العمليات (D-Bus)، ونقل MPEG-TS عبر HTTP مع توكن جلسة.

---

## 🏛 نظرة عامة على معمارية النظام

```mermaid
graph TD
    subgraph Host Linux Machine
        A[evdi Kernel Module] -->|Virtual DRM Device| B(Display Server X11/Wayland)
        B -->|evdi framebuffer| C1(orbiscreen-display EvdiFramePump)
        B -.->|portal fallback only| C0(orbiscreen-capture portal/X11)
        C1 -->|Tight BGRA frames| D(orbiscreen-encode)
        C0 -.->|BGRA frames (primary desktop)| D
        D -->|GStreamer HW Encode| E(H.264 Stream)
        E --> F(orbiscreen-transport)
        F -->|MPEG-TS HTTP /stream| G((Network/USB))
        F -->|mDNS _orbiscreen._tcp.| G
        F -->|GET /api/info| G
        F -->|POST /api/control| G
        F -->|GET /health| G
    end

    subgraph Clients
        G -->|MPEG-TS + token| W(Web client - mpegts.js MSE)
        W -->|POST /input| F
        G -->|NSD discovery + token| H(Android DiscoveryService)
        H -->|onConnect| J(StreamViewModel)
        J -->|PlayerHolder.build| K(OkHttpDataSource)
        K -->|MPEG-TS + Bearer token| L(ExoPlayer + MediaCodec)
        L -->|Touch| N(InputDispatcher)
        N -->|POST /input + Bearer token| F
        J -->|POST /api/control| F
    end
```

---

## 📦 طوبولوجيا حزم مساحة العمل

| الحزمة | المسؤولية | التبعيات الرئيسية |
|-------|----------------|------------------|
| `orbiscreen-core` | الإعدادات المشتركة وأنواع الأخطاء والتسلسل | `serde`، `toml` |
| `orbiscreen-display` | إنشاء شاشة افتراضية EVDI DRM وتوليف EDID | `evdi`، `libc` |
| `orbiscreen-capture` | محركات التقاط Wayland Portal (ashpd) وX11 (x11rb) | `ashpd`، `x11rb` |
| `orbiscreen-encode` | خطوط أنابيب ترميز H.264 عتادية وبرمجية | `gstreamer`، `gstreamer-app` |
| `orbiscreen-input` | حقن اللمس العكسي والقلم ولوحة المفاتيح | `evdevil`، `nix` |
| `orbiscreen-transport` | Axum HTTP على `/stream`، وmDNS، وADB reverse، و`/api/info`، و`/api/control`، و`/health` | `axum`، `gstreamer`، `tokio` |
| `orbiscreen-daemon` | ثنائي الـ daemon الرئيسي، تكامل systemd وخدمة D-Bus | `zbus`، `clap`، `tokio` |

---

## 📱 بنية حزم عميل Android

```
com.orbiscreen.android/
├── MainActivity.kt                # Compose host, observes PrefsStore.themePrefFlow
├── data/
│   └── PrefsStore.kt              # SharedPreferences (recent host, theme, scanner toggle)
├── net/
│   ├── DiscoveryService.kt        # NsdManager wrapper -> StateFlow<Map<DiscoveredHost>>
│   ├── SubnetScanner.kt           # /24 sweep with Semaphore-bounded parallelism
│   ├── HostApi.kt                 # OkHttp client for /api/info, /api/control, /health
│   ├── WifiGatewayProvider.kt     # Reads WifiManager.dhcpInfo.gateway
│   └── DiscoveryModel.kt          # HostSpec regex validator
├── player/
│   ├── PlayerHolder.kt            # ExoPlayer + OkHttpDataSource + DefaultLoadControl
│   └── StreamUrl.kt               # Builds http://host:port/stream?token=...
├── input/
│   └── InputDispatcher.kt         # Absolute-coord pointer / wheel / keyboard / stylus
└── ui/
    ├── theme/                     # Material 3 Color.kt, Theme.kt, Type.kt
    ├── nav/OrbiNav.kt             # NavHost (Discovery / Stream / Settings)
    ├── discovery/                 # DiscoveryScreen + DiscoveryViewModel
    ├── stream/                    # StreamScreen, PlayerSurface, ControlToolbar
    └── settings/                  # SettingsScreen (theme, decoder, scanner, recent host)
```

---

## ⚡ خط أنابيب البث

1. **تهيئة الشاشة الافتراضية:** يفتح `orbiscreen-display` شاشة افتراضية EVDI DRM ويقرأ ذاكرتها الإطارية مباشرة (EvdiFramePump) - هذه هي الشاشة الثانية الحقيقية التي يرسم عليها المنشئ. إذا غابت وحدة النواة يتراجع الدامن إلى التقاط سطح المكتب الرئيسي عبر portal (Wayland) أو جذر X11.
2. **التقاط الإطارات:** إطارات BGRA خام من ذاكرة evdi الإطارية (أو PipeWire / X11 Shared Memory في وضع التراجع).
3. **الترميز:**
   - يرمَّز `orbiscreen-encode` الإطارات إلى H.264 عبر خطوط أنابيب GStreamer المسرَّعة عتادياً (أوالاً VAAPI ثم NVENC ثم التراجع إلى x264).
   - لكل عميل يغلّف `orbiscreen-transport` وحدات NAL المرمَّزة بـ H.264 في خط `h264parse + mpegtsmux` مستقل ويقدّمها عبر `http://host:port/stream?token=...`.
4. **التشغيل على Android:**
   - تعمل `PlayerHolder.build()` على الخيط الرئيسي (`withContext(Dispatchers.Main)`) لمنع انهيارات التنافس الخيطي عند الاتصال.
   - جميع تهيئات builder وdataSource داخل `PlayerHolder.build()` مغلّفة بكتلة try-catch لتظهر أخطاء البناء كبطاقات `StreamEvent.Error` قابلة لإعادة المحاولة.
   - يبني `PlayerHolder` كائن `MediaItem` مع `MimeTypes.VIDEO_MP2T` حتى يفك ExoPlayer ترميز MPEG-TS دون sniffing.
   - يُضبط `OkHttpDataSource` بمهلة قراءة صفرية (بث مباشر) و`DefaultLoadControl` مضبوط (تخزين 1.5 ثوانٍ أدنى / 5 ثوانٍ أقصى).
   - يكشف `PlayerHolder` واجهة `Player.Listener` تربط حالات `Player.STATE_*` بأحداث `StreamEvent` لواجهة Compose.
5. **الإدخال العكسي:**
   - يربط `InputDispatcher` أحداث المؤشر / العجلة / القلم / لوحة المفاتيح من مستطيل `PlayerView` في Android بإحداثيات مطلقة للمضيف باستخدام دقة الشاشة المُبلَّغ عنها من `/api/info`.
   - تُزال تكرارات الأحداث عبر `MutableSharedFlow` مع `BufferOverflow.DROP_OLDEST` لمنع التراكم أثناء السحب السريع.
6. **التحكم بالمضيف:**
   - يُرسل `HostApi.sendControl` إجراءات JSON إلى `/api/control` لطلبات القفل والتعتيم وctrl-alt-del، مع توكن الجلسة المنتزَع من الـ TXT المُعلن عبر mDNS.

---

## 🌐 عقد HTTP API

| النقطة | الطريقة | المصادقة | الاستجابة |
|----------|--------|----------|----------|
| `/stream` | GET | توكن | بث `video/mp2t` MPEG-TS |
| `/health` | GET | عامة | `200 OK "ok"` |
| `/api/info` | GET | عامة | `{"display_width":1920,"display_height":1080,"refresh_hz":60,"encoder":"x264","version":"0.11.0"}` |
| `/api/control` | POST | توكن | `200 OK`; الإجراءات: `lock`، `blank`، `unblank`، `ctrl_alt_del` (يُرفض `open` برفض 400) |
| `/client/config.json` | GET | عامة | `{"token":"...","display_width":1920,"display_height":1080}` - تمهيد عميل الويب |

أحداث الإدخال (`/input`، تتطلب توكن) تقبل مخطط الحمولة المستخدم لدى عميل الويب: `Move{x,y}`، `Button{button,pressed,x?,y?}`، `Wheel{delta_y}`، `Key{code,pressed}`، `Stylus{x,y,pressure,tilt_x_deg,tilt_y_deg}`. يُقدم التوكن عبر ترويسة `Authorization: Bearer <token>` أو معامل `?token=`.

---

## 🔌 تحسينات النقل

- **OkHttpDataSource:** مهلة قراءة صفرية، مقبس طويل العمر، و`User-Agent: Orbiscreen-Android/1.0` مخصص لسجلات خادم أوضح.
- **DefaultLoadControl:** يخزّن 1.5 ثانية كحد أدنى و5 ثوانٍ كحد أقصى لامتصاص اضطراب Wi-Fi دون استنزاف RAM.
- **قناة البث:** `video_tx` هو `tokio::sync::broadcast` بحيث يمكن لعدة عملاء HTTP الاشتراك في نفس البث المرمّز دون ضغط عكسي على المُرمّز.
- **بلا Protobuf:** تستخدم الحمولات `org.json.JSONObject` في كلا الاتجاهين للحفاظ على عقد سلكي متماثل مع عميل الويب.

---

## 🔁 دورة الحياة

- يمتلك `StreamViewModel` كائن `PlayerHolder`؛ يحدث التحرير داخل `onCleared()`.
- يُنشأ `InputDispatcher` كسولاً عند أول لمسة ويُحرَّر مع المشغّل.
- يبدأ `DiscoveryService` في `DiscoveryViewModel.init` ويُفصل مع نطاق ViewModel.

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[العودة إلى README](../README_AR.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
