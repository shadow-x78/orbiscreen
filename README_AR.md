<div align="center">

<a href="https://github.com/shadow-x78/orbiscreen">
  <img src="https://raw.githubusercontent.com/shadow-x78/orbiscreen/main/assets/logo/orbiscreen-banner-ar.png" alt="شعار Orbiscreen" width="100%" />
</a>

# Orbiscreen

شاشة ثانية لنظام Linux باستخدام جهاز لوحي أو هاتف Android.

[![الإصدار](https://img.shields.io/badge/version-0.33.6-2563eb?style=for-the-badge)](CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=for-the-badge)](LICENSE)
![Rust](https://img.shields.io/badge/rust-1.92%2B-16a34a?style=for-the-badge)
![المنصة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=for-the-badge)

</div>

---

## اللغة

<a href="README.md">🇬🇧 English</a> · <a href="README_AR.md">🇸🇦 العربية</a>

---

## فهرس المحتويات

- [نظرة عامة](#overview)
- [المقارنة](#comparison)
- [حالات الاستخدام](#use-cases)
- [المزايا](#features)
- [دعم بيئات سطح المكتب](#desktop-support)
- [التثبيت](#installation)
- [الاستخدام](#usage)
- [المعمارية](#architecture)
- [هيكل المشروع](#project-structure)
- [الأسئلة الشائعة](#faq)
- [التوثيق](#documentation)
- [المساهمة](#contributing)
- [الرخصة](#license)

---

<a id="overview"></a>
## نظرة عامة

يحوّل Orbiscreen جهازا لوحيا أو هاتفا من Android إلى شاشة عرض ثانية لسطح مكتب Linux. ينشئ المضيف شاشة افتراضية إما كشاشة على مستوى النواة عبر مشغّل <code dir="ltr">evdi</code> (في COSMIC وGNOME وX11)، أو كمخرج أصيل لمدير النوافذ على KDE Plasma وبيئات wlroots، بدون صلاحيات root وبدون نوافذ مشاركة. تُرمَّز الشاشة بصيغة H.264 وتُبث إلى تطبيق Android أو إلى المتصفح، مع تحكم عكسي باللمس المتعدد والفأرة ولوحة المفاتيح وقلم بحساسية ضغط.

<a id="comparison"></a>
## المقارنة

<div dir="rtl" align="right">

| الإمكانية | Spacedesk | Deskreen | Weylus | Sidecar | **Orbiscreen** |
| :--- | :---: | :---: | :---: | :---: | :---: |
| مضيف Linux | ❌ ويندوز فقط | ✅ عبر المتصفح | ✅ عبر المتصفح | ❌ ماك فقط | ✅ |
| Wayland وX11 | ❌ | ⚠️ وصلة وهمية | ⚠️ تكرار فقط | ❌ | ✅ أصيل |
| شاشة ممتدة | ✅ ويندوز فقط | ❌ وصلة HDMI وهمية | ❌ تكرار فقط | ✅ أبل فقط | ✅ شاشة افتراضية |
| تطبيق أندرويد أصلي | ✅ | ❌ متصفح | ❌ متصفح | ❌ آيباد فقط | ✅ Compose |
| ترميز عتادي | ✅ | ❌ | ⚠️ | ✅ | ✅ NVENC وVA-API |
| زمن الاستجابة المقاس | ‏~50-80 ms | ‏~150-300 ms | ‏~80-120 ms | ‏~30 ms | ‏~25-40 ms |
| ضغط القلم وميلهانه | ❌ | ❌ | ⚠️ قلم فقط | ✅ | ✅ ‏4095 مستوى |
| لمس وفأرة عكسيان | ✅ | ❌ | ⚠️ قلم فقط | ✅ | ✅ متعدد اللمس |
| بلا root على KDE وwlroots | غير متاح | ✅ | ❌ | غير متاح | ✅ |
| مفتوح المصدر | ❌ | ✅ ‏GPL-3.0 | ✅ ‏AGPL-3.0 | ❌ | ✅ ‏GPL-3.0 |

</div>

<a id="use-cases"></a>
## حالات الاستخدام

<ul dir="rtl" style="padding-right:1.2em">
<li>شاشة ثانية لجهازك اللوحي (Samsung Galaxy Tab، Xiaomi Pad، Lenovo Tab) دون شراء شاشة محمولة.</li>
<li>لوح رسم رقمي بحساسية ضغط وميلان في Krita وGIMP وBlender وInkscape.</li>
<li>شاشة رأسية للقراءة والبرمجة، تتبدل تلقائيا ً مع تدوير الجهاز.</li>
<li>تشغيل سلكي: ينقل Android Open Accessory البث على كابل USB دون واي فاي ودون <code dir="ltr">adb reverse</code>.</li>
</ul>

<a id="features"></a>
## المزايا

<ul dir="rtl">
<li>إنشاء الشاشة الافتراضية عبر XDG Desktop Portal على GNOME 46+ وKDE Plasma 6+، وعبر IPC لمدير النوافذ في بيئات wlroots، وعبر EVDI على X11 وCOSMIC.</li>
<li>دعم القلم: 4095 مستوى ضغط، وزوايا ميلان، وتتبع التحليق في الهواء، تظهر كجهاز لوح رقمي في نواة Linux عبر <code dir="ltr">uinput</code>.</li>
<li>وضع لوحة اللمس مع إيماءة النقر المزدوج والسحب، مع تقييد المؤشر داخل حدود الشاشة الافتراضية.</li>
<li>ترميز منخفض الكمون: H.264 عبر NVENC أو VA-API أو x264، مع ضبط GOP والتحكم بالمعدل للشبكات المحلية.</li>
<li>مسارات نقل متعددة: UDP Annex-B مع تصحيح Reed-Solomon، وWebTransport للمتصفحات، وUSB AOA على قنوات Bulk، وHTTP MPEG-TS للتراجع.</li>
<li>اكتشاف المضيف عبر mDNS، أو بالإدخال اليدوي، أو بماسح اختياري للشبكة الفرعية.</li>
<li>عميل ويب يعمل في متصفحات Chromium دون تثبيت، عبر WebTransport وWebCodecs.</li>
<li>تطبيق Android بواجهة Material 3 مع سمتي فاتحة وداكنة.</li>
<li>تحكم بالمضيف عبر واجهة موثقة: قفل الشاشة، والتعتيم، وإلغاء التعتيم، وCtrl+Alt+Del.</li>
<li>خدمة D-Bus وواجهة أوامر (<code dir="ltr">orbiscreen start</code> و<code dir="ltr">stop</code> و<code dir="ltr">doctor</code>) مع وحدة systemd للمستخدم.</li>
<li>توقيع تشفيري لحزم Linux وAndroid الصادرة.</li>
</ul>

<a id="desktop-support"></a>
## دعم بيئات سطح المكتب

<div dir="rtl" align="right">

| البيئة | شاشة ثانية افتراضية | الالتقاط | الإدخال |
| :--- | :---: | :---: | :---: |
| KDE Plasma (Wayland) | ✅ أصيل عبر Portal أو zkde-screencast، بلا root | ✅ PipeWire | ✅ portal RemoteDesktop وuinput |
| COSMIC (Wayland) | ⚠️ عبر EVDI ‏(doctor --fix) | ✅ Portal ScreenCast (PipeWire) | ✅ uinput وRemoteDesktop |
| Sway / Hyprland / wlroots | ✅ مخرج headless عبر IPC، بلا root | ✅ wlr-screencopy | ✅ virtual-pointer وvirtual-keyboard |
| GNOME (Wayland) | ✅ Portal Virtual ‏(GNOME 46+) أو EVDI | ✅ Portal ScreenCast (PipeWire) | ✅ portal RemoteDesktop محفوظ |
| XFCE / MATE / LXQt / Cinnamon (X11) | ✅ عبر EVDI | ✅ عكس الجذر بـXShm | ✅ XTEST وuinput |
| غيرها | ✅ عبر EVDI | أفضل متوفر | أفضل متوفر |

</div>

يطبع <code dir="ltr">orbiscreen doctor</code> مدير النوافذ المكتشف وخطة الالتقاط وما ينقص النظام؛ ويثبّت <code dir="ltr">orbiscreen doctor --fix</code> وحدة نواة EVDI على التوزيعات المكتشفة. التفاصيل في [دعم بيئات سطح المكتب](docs/DE_SUPPORT_AR.md).

<a id="installation"></a>
## التثبيت

**أوبونتو وPop!_OS ولينكس مينت (PPA):**

<div dir="ltr" align="left">

```bash
sudo add-apt-repository ppa:shadow-x78/ppa -y
sudo apt update && sudo apt install orbiscreen -y
```

</div>

**فيدورا (COPR):**

<div dir="ltr" align="left">

```bash
sudo dnf copr enable shadow-x78/orbiscreen -y
sudo dnf install orbiscreen -y
```

</div>

**آرش لينكس ومانجارو (PKGBUILD):**

<div dir="ltr" align="left">

```bash
git clone https://github.com/shadow-x78/orbiscreen.git
cd orbiscreen
makepkg -si
```

</div>

**AppImage:** نزّل <code dir="ltr">orbiscreen-x86_64.AppImage</code> من [صفحة الإصدارات](https://github.com/shadow-x78/orbiscreen/releases) وشغّله.

**من المصدر:**

<div dir="ltr" align="left">

```bash
git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen
cd ~/Orbiscreen && ./scripts/install.sh
```

</div>

**Android:** ثبّت <code dir="ltr">orbiscreen-android-release.apk</code> من [صفحة الإصدارات](https://github.com/shadow-x78/orbiscreen/releases).

<a id="usage"></a>
## الاستخدام

شغّل الخدمة في المقدمة، أو فعّل وحدة systemd للمستخدم:

<div dir="ltr" align="left">

```bash
orbiscreen start
systemctl --user enable --now orbiscreen
```

</div>

الأوامر:

<div dir="ltr" align="left">

```bash
orbiscreen start                              # تشغيل باكتشاف البيئة تلقائيا ً
orbiscreen start --width 1920 --height 1080 --fps 60
orbiscreen display set 1920x1080@60            # تثبيت دقة العرض
orbiscreen doctor                             # التشخيص
orbiscreen doctor --fix                       # تثبيت الاعتماديات الناقصة
orbiscreen stop                               # إيقاف الخدمة الجارية
```

</div>

<div dir="ltr" align="left">

يُختار المرمِّز من ملف الإعدادات لا من سطر الأوامر:

```toml
[encode]
preferred_encoder = "auto"   # auto أو nvenc أو vaapi أو x264
```

</div>

افتح تطبيق Android واختر المضيف المكتشف، ثم أدخل توكن الجلسة (يصل عبر mDNS أو يُقرأ بأمر <code dir="ltr">orbiscreen doctor</code>).

<a id="architecture"></a>
## المعمارية

<div dir="ltr" align="left">

```
┌──────────────────────────────────────────────────────────────┐
│                      orbiscreen-daemon                       │
│  ┌────────────────────┐  ┌────────────────────────────────┐  │
│  │ orbiscreen-display │  │ orbiscreen-capture             │  │
│  │ (evdi kernel/DRM)  │  │ (kwin-virtual / wlr / portal)  │  │
│  └────────────────────┘  └────────────────────────────────┘  │
│             │                            │                   │
│             ▼                            ▼                   │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ orbiscreen-encode (GStreamer NVENC/VAAPI/x264)         │  │
│  └────────────────────────────────────────────────────────┘  │
│                              │                               │
│                              ▼                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ orbiscreen-transport: HTTP, UDP, WebTransport, AOA     │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

</div>

ترجع أحداث الإدخال العكسي عبر <code dir="ltr">orbiscreen-input</code> (يستخدم uinput أو XTEST أو أجهزة wlroots الافتراضية أو portal RemoteDesktop). المواصفات الكاملة في [docs/ARCHITECTURE_AR.md](docs/ARCHITECTURE_AR.md).

<a id="project-structure"></a>
## هيكل المشروع

<div dir="ltr" align="left">

```
orbiscreen/
├── crates/
│   ├── orbiscreen-core/        # shared types, config, errors
│   ├── orbiscreen-display/     # evdi-backed virtual displays
│   ├── orbiscreen-capture/     # KWin/wlroots virtual outputs, portal/X11 capture
│   ├── orbiscreen-encode/      # GStreamer pipeline (VAAPI / NVENC / x264)
│   ├── orbiscreen-input/       # uinput tablet and touch, RemoteDesktop portal
│   ├── orbiscreen-transport/   # axum, UDP/WebTransport, AOA USB, mDNS
│   ├── orbiscreen-daemon/      # daemon binary, D-Bus service, CLI
│   └── orbiscreen-gui/         # Tauri v2 desktop control center
├── clients/
│   ├── web/                    # browser client (WebTransport + WebCodecs)
│   └── android/                # Material 3 Compose app
├── assets/
│   └── logo/                   # SVG and PNG icons, banners
├── data/                       # desktop entry, RPM spec, systemd service
├── scripts/                    # install, packaging, dev tooling
└── docs/                       # bilingual guides (EN + AR)
```

</div>

<ul dir="rtl" style="padding-right:1.2em">
<li><code dir="ltr">crates/orbiscreen-core</code>: الأنواع المشتركة والإعدادات والأخطاء.</li>
<li><code dir="ltr">crates/orbiscreen-display</code>: الشاشات الافتراضية عبر evdi.</li>
<li><code dir="ltr">crates/orbiscreen-capture</code>: إنشاء المخارج الافتراضية في KWin وwlroots والتقاط الإطارات عبر portal وX11.</li>
<li><code dir="ltr">crates/orbiscreen-encode</code>: خطوط ترميز GStreamer بتسريع عتادي.</li>
<li><code dir="ltr">crates/orbiscreen-input</code>: حقن اللمس والقلم ولوحة المفاتيح.</li>
<li><code dir="ltr">crates/orbiscreen-transport</code>: كل واجهات الشبكة وUSB الخاصة بالعملاء.</li>
<li><code dir="ltr">crates/orbiscreen-daemon</code>: العملية الرئيسية وخدمة D-Bus وواجهة الأوامر.</li>
<li><code dir="ltr">crates/orbiscreen-gui</code>: مركز التحكم المكتبي (Tauri v2).</li>
</ul>

<a id="faq"></a>
## الأسئلة الشائعة

<div dir="rtl">

<details>
<summary><b>هل هي شاشة ممتدة حقيقية أم تكرار للشاشة؟</b></summary>
<br>
شاشة افتراضية مستقلة توضع بجوار شاشاتك الفعلية؛ يمكن سحب النوافذ إليها وضبط دقتها حتى 7680×4320 (‏8K).
</details>

<details>
<summary><b>هل يعمل على Wayland بدون صلاحيات root؟</b></summary>
<br>
على KDE Plasma وبيئات wlroots ينشئ مديرُ النوافذ الشاشةَ الافتراضية بنفسه، بلا root وبلا وحدة نواة. على GNOME وCOSMIC وX11 توفر وحدة النواة EVDI الشاشة الافتراضية.
</details>

<details>
<summary><b>هل يمكن الرسم بالقلم في Krita أو GIMP؟</b></summary>
<br>
يُحقن ضغط القلم (حتى 4095 مستوى) وزاوية الميلان والتحليق كلوح رقمي في نواة Linux عبر <code dir="ltr">uinput</code>، فترى التطبيقات جهاز رسم حقيقيا ً.
</details>

<details>
<summary><b>هل يمكن الاتصال عبر كابل USB بدل الشبكة؟</b></summary>
<br>
موّصل الكابل واقبل حوار إذن الملحق، فينتقل الجهاز إلى وضع Android Open Accessory ويبث H.264 على قنوات USB Bulk. لا حاجة لتفعيل تصحيح USB ولا <code dir="ltr">adb reverse</code>.
</details>

<details>
<summary><b>كم يبلغ زمن الاستجابة؟</b></summary>
<br>
على شبكة 5GHz مع ترميز عتادي يقيس المشروع 2 إلى 4 ms بين إرسال المضيف وتجميع العميل؛ أما زمن الشاشة الكامل فيتراوح عادة بين 25 و40 ms.
</details>

</div>

<a id="documentation"></a>
## التوثيق

<div dir="rtl" align="right">

| الوثيقة | الوصف |
| :--- | :--- |
| [ARCHITECTURE_AR.md](docs/ARCHITECTURE_AR.md) | معمارية النظام وخط أنابيب الإطارات وحزم المساحة |
| [UDP_TRANSPORT_AR.md](docs/UDP_TRANSPORT_AR.md) | مسار فيديو UDP Annex-B وأنواع الحزم وDPLPMTUD |
| [FRAME_TRANSPORT.md](docs/FRAME_TRANSPORT.md) | مصطلحات I/P/IDR/GOP وتدفق وحدات الوصول واستعادة الفقد |
| [DE_SUPPORT_AR.md](docs/DE_SUPPORT_AR.md) | دعم بيئات سطح المكتب وخطط الالتقاط |
| [PACKAGING_AR.md](docs/PACKAGING_AR.md) | مواصفات التغليف (deb وrpm وAppImage) |
| [DBUS_SPEC_AR.md](docs/DBUS_SPEC_AR.md) | واجهة D-Bus لخدمة الجلسة |
| [TROUBLESHOOTING_AR.md](docs/TROUBLESHOOTING_AR.md) | استكشاف الأخطاء الشائعة وإصلاحها |

</div>

<a id="contributing"></a>
## المساهمة

<ol dir="rtl">
<li>اعمل Fork للمستودع وأنشئ فرعا ً بمناسب: <code dir="ltr">feature/</code> أو <code dir="ltr">fix/</code> أو <code dir="ltr">docs/</code> أو <code dir="ltr">chore/</code>.</li>
<li>حافظ على نجاح <code dir="ltr">cargo fmt --all --check</code> و<code dir="ltr">cargo clippy --workspace --all-targets -- -D warnings</code> والاختبارات.</li>
<li>أضف قيدا ً في <code dir="ltr">CHANGELOG.md</code>.</li>
<li>التزم بصيغة: <code dir="ltr">orbiscreen | &lt;type&gt;: &lt;description&gt;</code> ثم افتح Pull Request نحو <code dir="ltr">main</code>.</li>
</ol>

راجع [إرشادات المساهمة](CONTRIBUTING.md) و[ميثاق السلوك](CODE_OF_CONDUCT.md)، وابلغ عن المشاكل عبر [GitHub Issues](https://github.com/shadow-x78/orbiscreen/issues).

<a id="license"></a>
## الرخصة

مرخّص تحت [رخصة GPL-3.0](LICENSE).

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[سجل التغييرات](CHANGELOG.md) ·
[الأمان](SECURITY.md)

<sub>&copy; 2026 Orbiscreen</sub>

</div>
