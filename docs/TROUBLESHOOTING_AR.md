<div align="center">

# استكشاف الأخطاء وإصلاحها - Orbiscreen

[![الإصدار](https://img.shields.io/badge/version-0.35.8-2563eb?style=flat-square&logo=semver)](../CHANGELOG.md)
[![الرخصة](https://img.shields.io/badge/license-GPL--3.0-dc2626?style=flat-square)](../LICENSE)
![Rust](https://img.shields.io/badge/rust-1.92%2B-16a34a?style=flat-square&logo=rust)
![المنصّة](https://img.shields.io/badge/platform-Linux%20%7C%20Android-9333ea?style=flat-square&logo=linux)

</div>

---

## اللغة

<a href="TROUBLESHOOTING.md">🇬🇧 English</a> · <a href="TROUBLESHOOTING_AR.md">🇸🇦 العربية</a>

---

## المحتويات

- [فشل بناء حزمة Debian على PPA](#فشل-بناء-حزمة-debian-على-ppa)
- [فشل بناء حزمة RPM على COPR](#فشل-بناء-حزمة-rpm-على-copr)
- [فشل مسار الإصدار قبل النشر](#فشل-مسار-الإصدار-قبل-النشر)
- [لم يُعثر على Android SDK](#لم-يُعثر-على-android-sdk)
- [رفض صلاحية uinput](#رفض-صلاحية-uinput)
- [وحدة نواة evdi مفقودة في إعدادات X11 الأقدم](#وحدة-نواة-evdi-مفقودة-في-إعدادات-x11-الأقدم)
- [إضافة GStreamer مفقودة](#إضافة-gstreamer-مفقودة)
- [لا يعثر الاكتشاف على المضيف](#لا-يعثر-الاكتشاف-على-المضيف)
- [رفض الاتصال أو انتهاء المهلة](#رفض-الاتصال-أو-انتهاء-المهلة)
- [إشباك النطاق الترددي على وصلة لاسلكية](#إشباك-النطاق-الترددي-على-وصلة-لاسلكية)
- [فيديو UDP يتقطّع بينما فيديو HTTP سليم](#فيديو-udp-يتقطّع-بينما-فيديو-http-سليم)
- [لا يمكن الاقتران لأن الطلب يصل دون وسيلة لقبوله](#لا-يمكن-الاقتران-لأن-الطلب-يصل-دون-وسيلة-لقبوله)
- [GNOME / Wayland (Mutter) لا يعرض أي صورة](#gnome--wayland-mutter-لا-يعرض-أي-صورة)
- [sway / Hyprland يُبلّغ عن محرك التقاط لا يبدأ أبداً](#sway--hyprland-يُبلّغ-عن-محرك-التقاط-لا-يبدأ-أبداً)
- [COSMIC (cosmic-comp) لا يوفّر محرك التقاط](#cosmic-cosmic-comp-لا-يوفّر-محرك-التقاط)
- [X11: يهرب المؤشر من الشاشة الافتراضية](#x11-يهرب-المؤشر-من-الشاشة-الافتراضية)
- [لا تظهر أي شاشة على الحزمة](#لا-تظهر-أي-شاشة-على-الحزمة)
- [الفأرة أو اللمس لا يفعلان شيئاً](#الفأرة-أو-اللمس-لا-يفعلان-شيئاً)
- [خدمة المضيف تنتهي فوراً](#خدمة-المضيف-تنتهي-فوراً)
- [المُفكِّك يتراجع إلى البرمجي، أو لا تظهر الصورة أبداً](#المُفكِّك-يتراجع-إلى-البرمجي-أو-لا-تظهر-الصورة-أبداً)
- [التطبيق يتعطل فور تشغيله](#التطبيق-يتعطل-فور-تشغيله)
- [اتصال USB يعرض "Looking for host…" إلى الأبد](#اتصال-usb-يعرض-looking-for-host-إلى-الأبد)
- [فشل اتصال ADB على ASUS Chromebook CM3001](#فشل-اتصال-adb-على-asus-chromebook-cm3001)
- [سحب النوافذ أو تحديد النصوص في وضع لوحة اللمس](#سحب-النوافذ-أو-تحديد-النصوص-في-وضع-لوحة-اللمس)
- [اللمس مُدوَّر أو غير محاذٍ](#اللمس-مُدوَّر-أو-غير-محاذٍ)
- [إجراءات شريط التحكم تُرجع 404](#إجراءات-شريط-التحكم-تُرجع-404)
- [قائمة الاكتشاف فارغة رغم وجود مضيفين على نفس Wi-Fi](#قائمة-الاكتشاف-فارغة-رغم-وجود-مضيفين-على-نفس-wi-fi)
  - [ما زلت عالقاً؟](#ما-زلت-عالقاً)
- [إجراء CI: `Format (cargo fmt)`](#إجراء-ci-format-cargo-fmt)
- [إجراء CI: `Clippy (deny warnings)`](#إجراء-ci-clippy-deny-warnings)
- [إجراء CI: `Build` (`cargo build --workspace --locked`)](#إجراء-ci-build-cargo-build---workspace---locked)
- [إجراء CI: `Test` ‏(`cargo test --workspace --locked`)](#إجراء-ci-test-cargo-test---workspace---locked)
- [إجراء CI: `Run cargo-deny`](#إجراء-ci-run-cargo-deny)
- [إجراء CI: `Android assembleDebug` + `lintDebug`](#إجراء-ci-android-assembledebug--lintdebug)
- [وقت التشغيل: فشل `orbiscreen start` - `kernel module is not installed`](#وقت-التشغيل-فشل-orbiscreen-start---kernel-module-is-not-installed)
- [وقت التشغيل: KDE Plasma (شاشة افتراضية بدون evdi وبدون root)](#وقت-التشغيل-kde-plasma-شاشة-افتراضية-بدون-evdi-وبدون-root)
- [وقت التشغيل: واجهة الالتقاط غير متاحة على Wayland](#وقت-التشغيل-واجهة-الالتقاط-غير-متاحة-على-wayland)
- [وقت التشغيل: `unsafe_op_in_unsafe_fn` / `missing_debug_implementations`](#وقت-التشغيل-unsafe_op_in_unsafe_fn--missing_debug_implementations)
- [أجهزة وعميل Android](#أجهزة-وعميل-android)
  - [Android / ChromeOS: فشل اتصال ADB أو بقاء الرسالة "Looking for host" على ASUS Chromebook CM3001](#android--chromeos-فشل-اتصال-adb-أو-بقاء-الرسالة-looking-for-host-على-asus-chromebook-cm3001)
  - [Android: القلم لا يرسم، أو حساسية ضغط غير صحيحة، أو توقف التطبيق على Lenovo Tab](#android-القلم-لا-يرسم-أو-حساسية-ضغط-غير-صحيحة-أو-توقف-التطبيق-على-lenovo-tab)
  - [Android: سحب النوافذ وتحديد النصوص في وضع لوحة اللمس (Touchpad Drag-and-Drop)](#android-سحب-النوافذ-وتحديد-النصوص-في-وضع-لوحة-اللمس-touchpad-drag-and-drop)
  - [Android: التطبيق يتعطل أو تموت العملية عند النقر على Connect](#android-التطبيق-يتعطل-أو-تموت-العملية-عند-النقر-على-connect)
  - [Android: شاشة سوداء بعد Connect](#android-شاشة-سوداء-بعد-connect)
  - [Android: قائمة الاكتشاف فارغة رغم وجود مضيفين على نفس شبكة Wi-Fi](#android-قائمة-الاكتشاف-فارغة-رغم-وجود-مضيفين-على-نفس-شبكة-wi-fi)
  - [Android: اللمس مُدوَّر / غير محاذٍ](#android-اللمس-مُدوَّر--غير-محاذٍ)
  - [Android: إجراءات شريط التحكم تُرجع 404](#android-إجراءات-شريط-التحكم-تُرجع-404)
  - [Android: التطبيق يتعطل فوراً عند التشغيل](#android-التطبيق-يتعطل-فوراً-عند-التشغيل)
  - [Android: اتصال USB يعرض "Looking for host…"](#android-اتصال-usb-يعرض-looking-for-host)
- [البث: بطء شديد أو تقطيع في حركة الفأرة عبر شبكة 5GHz Wi-Fi](#البث-بطء-شديد-أو-تقطيع-في-حركة-الفأرة-عبر-شبكة-5ghz-wi-fi)
- [البث: وميض وإعادة اتصال لانهائية عند حدوث خطأ في البث بدل التعرف على انقطاع الاتصال](#البث-وميض-وإعادة-اتصال-لانهائية-عند-حدوث-خطأ-في-البث-بدل-التعرف-على-انقطاع-الاتصال)
- [تعدد الشاشات / X11: هروب مؤشر الفأرة من الشاشة الافتراضية إلى الشاشات المادية الأخرى](#تعدد-الشاشات--x11-هروب-مؤشر-الفأرة-من-الشاشة-الافتراضية-إلى-الشاشات-المادية-الأخرى)
- [العميل يعرض الشاشة الخطأ (سطح المكتب الرئيسي بدل الشاشة الافتراضية)](#العميل-يعرض-الشاشة-الخطأ-سطح-المكتب-الرئيسي-بدل-الشاشة-الافتراضية)
- [عميل الويب يُحمَّل لكن بلا صورة](#عميل-الويب-يُحمَّل-لكن-بلا-صورة)
- [لا يوجد مُرمَّز - البث يبدأ لكنه يفشل (غياب x264)](#لا-يوجد-مُرمَّز---البث-يبدأ-لكنه-يفشل-غياب-x264)
- [لا يوجد مُرمِّز - البث يبدأ لكنه يفشل (غياب x264)](#لا-يوجد-مُرمِّز---البث-يبدأ-لكنه-يفشل-غياب-x264)
- [رفض 401 من `/stream` أو `/input` أو `/api/control` (التوكن)](#رفض-401-من-stream-أو-input-أو-apicontrol-التوكن)
- [الـ daemon غير موجود على D-Bus](#الـ-daemon-غير-موجود-على-d-bus)
- [الـ Daemon: استهلاك 100% للمعالج أو تجمّد](#الـ-daemon-استهلاك-100-للمعالج-أو-تجمّد)

# إخفاقات بناء الحزم في CI

بناء الحزم يجري في مسارات عمل مخصّصة لا في `ci.yml`، لذا نجاح فحص `CI` لا يعني أن الحزم تُبنى. وتعيد بوابة الإصدار تشغيل `cargo deny check` و`cargo audit` قبل نشر أي شيء.

---

<a id="ci-deb"></a>
## فشل بناء حزمة Debian على PPA

**الأعراض:**
```
dpkg-buildpackage: error: unmet build-dependency: libwebkit2gtk-4.1-dev
```
أو من مهمة Ubuntu:
```
error: Package 'libjavascriptcoregtk-4.1-dev' has no installation candidate
```

**السبب:**
يبني `debian/rules` مساحة العمل كاملة، ومنها لوحة تحكم Tauri. وهي تحتاج حزم WebKitGTK للتطوير، وعلى `debian/control` أن يذكر كل واحدة منها وإلا لم يبدأ البناء أصلاً.

**الحل:**
يجب أن يحتوي `Build-Depends` على `libwebkit2gtk-4.1-dev` و`libjavascriptcoregtk-4.1-dev` و`libsoup-3.0-dev` و`libgtk-3-dev` و`librsvg2-dev` و`libayatana-appindicator3-dev` إلى جانب حزم GStreamer. يثبّتها المسار أيضاً قبل `debuild`؛ فإن كانت صورة PPA تفتقدها فشل البناء داخل المسار لا داخل `debian/rules`.

---

<a id="ci-rpm"></a>
## فشل بناء حزمة RPM على COPR

**الأعراض:**
```
error: Failed build dependencies:
/bin/sh: line 31: gstreamer1-devel: command not found
```

**السبب:**
يصرّف `data/orbiscreen-copr.spec` مساحة العمل نفسها، فيحتاج نظائر Fedora للحزم التي يحتاجها بناء Debian.

**الحل:**
تأكد أن `BuildRequires` في الملف يذكر `gstreamer1-devel` و`gstreamer1-plugins-base-devel` و`libdrm-devel` و`libxkbcommon-devel` و`webkit2gtk4.1-devel` و`gtk3-devel` و`libappindicator-gtk3-devel` و`librsvg2-devel`، وأن إصدار Rust المطلوب على الأقل بقدر `rust-version` في `Cargo.toml`. ويلتقط `scripts/check-versions.sh` وسم `Version:` قديماً، لكنه لا يلتقط متطلّب بناء ناقصاً.

---

<a id="ci-release"></a>
## فشل مسار الإصدار قبل النشر

**الأعراض:**
يفشل `Release Matrix` في الخطوة `Verify code integrity (release gate)`، ولا تُرفع أي ملفات.

**السبب:**
تمنع البوابة الإصدار عند فشل فحوص السلامة: التنسيق، أو clippy مع رفض التحذيرات، أو مجموعة الاختبارات، أو `cargo machete`، أو `cargo deny check`، أو `cargo audit`. وهي تعمل أيضاً على المراجع غير الموثوقة، فلا يستطيع وسم على commit لا يجتازها النشر.

**الحل:**
اقرأ أي خطوة فشلت في ملخّص التشغيل وأصلحها محلياً:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo machete && cargo deny check && cargo audit
```
تعمل البوابة على مساحة العمل بما فيها حزمة الواجهة الرسومية، لذا يحتاج `cargo clippy --workspace` إلى حزم WebKitGTK للتطوير التي يثبّتها `ci.yml`.

---

# مشاكل البناء المحلي

---

<a id="local-sdk"></a>
## لم يُعثر على Android SDK

**الأعراض:**
```
SDK location not found. Define a valid SDK location with an ANDROID_HOME environment variable
```

**السبب:**
يحتاج `clients/android` إلى SDK يحتوي المنصّة التي يُبنى التطبيق لها. ويجب أن يكون المجلد هو جذر الـSDK الذي يحوي `platforms/`، لا مجلد `cmdline-tools` الأعلى.

**الحل:**
وجّه `ANDROID_HOME` إلى جذر الـSDK واقبل التراخيص مرة واحدة:
```bash
export ANDROID_HOME="$HOME/Android/Sdk"
yes | "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --licenses
"$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --install \
  "platform-tools" "platforms;android-35" "build-tools;35.0.0"
```
ثم ابنِ باستخدام `./gradlew assembleDebug` من `clients/android`.

---

<a id="local-uinput"></a>
## رفض صلاحية uinput

**الأعراض:**
```
no usable input injector found (uinput and portal both failed)
```
أو عند بدء الخدمة:
```
orbiscreen_input: permission denied on /dev/uinput
```

**السبب:**
يحقن الإدخال عبر أجهزة `uinput` في النواة، و`/dev/uinput` ينتمي عادةً إلى `root:input` بصلاحية `0660`. ولا يستطيع مستخدم خارج مجموعة `input` فتحه.

**الحل:**
أضف المستخدم إلى المجموعة وابدأ جلسة دخول جديدة:
```bash
sudo usermod -aG input "$USER"
# سجّل الخروج ثم الدخول، ثم تحقق
id -nG | tr ' ' '\n' | grep -x input
ls -l /dev/uinput
```
يعتمد `scripts/install.sh` والوحدةopiّة على المجموعة نفسها، وتمنح قواعد udev صلاحية الوصول لعُقد USB للمستخدم الجالس. لا تشغّل الخدمة كـ root للالتفاف على هذا: يضع الإدخال المحقون خارج قوائم التحكم الخاصة بالجلسة.

---

<a id="local-evdi"></a>
## وحدة نواة evdi مفقودة في إعدادات X11 الأقدم

**الأعراض:**
```
Error: evdi kernel module is not installed
```

**السبب:**
يحتاج مستخدمو DisplayLink وحدة نواة مطابقة. انظر [وقت التشغيل: فشل `orbiscreen start`](#local-evdi) للتثبيت وإعداد تفضيل الالتقاط الذي يتجنّب الحاجة إليها أصلاً.

**الحل:**
ثبّت نسخة DKMS للنواة الجارية، ثم تأكد:
```bash
lsmod | grep evdi
```

---

<a id="local-gst"></a>
## إضافة GStreamer مفقودة

**الأعراض:**
```
ERROR ... no element found: nvh264enc
```
أو يتوقف البناء عند `vaapih264enc`.

**السبب:**
H.264 موجود في مجموعات الإضافات `good` و`bad` و`ugly`، والمُرمِّزات العتادية تحتاج `bad`. بدونها لا يمكن تركيب خط الأنابيب.

**الحل:**
```bash
# Fedora / Nobara
sudo dnf install gstreamer1-plugins-good gstreamer1-plugins-bad-free gstreamer1-plugins-ugly
# Ubuntu / Debian
sudo apt install gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly
gst-inspect-1.0 nvh264enc || gst-inspect-1.0 x264enc
```
انظر [لا يوجد مُرمِّز](#no-encoder) لقائمة الأعراض الكاملة.

---
---

# مشاكل الشبكة والاتصال

---

<a id="net-mdns"></a>
## لا يعثر الاكتشاف على المضيف

**الأعراض:**
تعرض حزمة Android قائمة مضيفين فارغة رغم أن الجهاز والمضيف على نفس الشبكة. وتسجّل الخدمة:
```
orbiscreen_transport::mdns: Advertised Orbiscreen service via mDNS instance=... port=8788
```
ولا يظهر شيء على الجهاز.

**السبب:**
يعتمد الاكتشاف على البث المتعدد، وهو لا يمرّ عبر شبكة موجَّهة، وكثير من نقاط الوصول اللاسلكية والشبكات الافتراضية والجسور البرمجية تمنعه أو ترشّحه. وجهازان على شبكتين مختلفتين لن يتعارفا بهذه الطريقة أبداً.

**الحل:**
تأكد أن الخدمة Thalعت نفسَها، ثم افحص المسار:
```bash
avahi-browse -rt _orbiscreen._tcp
ping -c 3 <host-lan-ip>
```
إذا كان البث المتعدد مرشَّحاً، فاتصل بالعنوان مباشرة: تقبل الحزمة مضيفاً مكتوباً يدوياً، أو عنوان IP على الشبكة المحلية مُمرَّراً من سطر الأوامر. لا يلزم غير ذلك. ويغطي `docs/DE_SUPPORT.md` محرك الالتقاط، وهو أمر منفصل عن الاكتشاف.

---

<a id="net-timeout"></a>
## رفض الاتصال أو انتهاء المهلة

**الأعراض:**
```
curl: (7) Failed to connect to 192.168.1.10 port 8788: Connection refused
```
أو تبقى الحزمة على `Looking for host…` دون رسالة خطأ.

**السبب:**
إما لا شيء يستمع، أو جدار ناري في المنتصف. تربط الخدمة كل الواجهات، لذا فإن الرفض يعني أن العملية لا تعمل.

**الحل:**
تحقق ممّا تسجّله الخدمة، من المضيف نفسه:
```bash
curl -s http://127.0.0.1:8788/health
ss -tlnp | grep -E ':(8788|8790)'
```
إذا أجاب فحص الـloopback بينما لا يفعل عميل، فالمسار محجوب:
```bash
sudo firewall-cmd --list-ports    # Fedora
sudo ufw status                   # Debian, Ubuntu
```
يجب أن تكون المنافذ `8788` (الإشارة والتحكم) و`8789` (فيديو UDP) و`8790` (عميل HTTPS وWebTransport) قابلة للوصول، وأنفذ `5353` عبر UDP متاح للاكتشاف.

---

<a id="net-bandwidth"></a>
## إشباك النطاق الترددي على وصلة لاسلكية

**الأعراض:**
البث يعمل على وصلة سلكية ويتقطّع فوق Wi-Fi. يتزايد التأخير بينما الصورة ما زالت تتحرك، وخفض الدقة يساعد.

**السبب:**
يحتاج بث بدقة 2560x1600 عند 90 إطاراً في الثانية أكثر من غيغابت في الثانية، وهو ما لا تقدر عليه وصلة 5GHz في طيف مشغول. يمتلئ عندها مخزن المُرمِّز الخاص، فيرى المشاهد تفريغ الطابور لا الشاشة الحيّة.

**الحل:**
اخفض الدقة أو معدل الإطارات من الحزمة. تفترض أرقام التأخير في `docs/UDP_TRANSPORT.md` وصلة تواكب؛ وعلى وصلة مشبعة قِس والصورة متوقفة لتفصل تأخير النقل عن تراكم الطابور.

---

<a id="net-udp"></a>
## فيديو UDP يتقطّع بينما فيديو HTTP سليم

**الأعراض:**
تبديل حزمة من HTTP Annex-B إلى فيديو UDP يزيل التقطّع، أو العكس.

**السبب:**
يشغّل مسار UDP بحثه عن MTU_PATH ومعالجة الازدحام الخاصة به، وخطافات الضبط في `docs/UDP_TRANSPORT.md` تنطبق عليه وحده. والمسار الذي لا يحمل حجم الداتغرام يُنزَع إلى الأسفل قسراً، فيظهر ذلك كاهتزاز لا كفشل.

**الحل:**
اضبط الخطافات من ذلك المستند وأعد القياس:
```bash
export ORBISCREEN_UDP_MAX_DATAGRAM=1200
export ORBISCREEN_UDP_MAX_CLIENTS=8
```

---

## لا يمكن الاقتران لأن الطلب يصل دون وسيلة لقبوله

**العَرَض:**
ينتظر عميل Android أو الويب على "بانتظار موافقة المضيف"، ويستلم الحاسوب طلب
الاقتران، لكن لا شيء على سطح المكتب يوفر وسيلة لقبوله.

**السبب:**
كانت نقطة الموافقة موجودة لكن لا شيء على المضيف يناديها: السطح الوحيد المتاح
كان نافذة موافقات المضيف في عميل الويب، وتتطلب لصق توكن مدير المضيف يدوياً.
الآن يعرض الـ daemon قائمة الانتظار عبر D-Bus ويديرها الأمر `orbiscreen pair`.

**الحل:**
1. راجع ما هو في الانتظار:
   ```bash
   orbiscreen pair list
   ```
   كما يسجّل الـ daemon كل طلب وارد باسم الجهاز وعنوان IP وتلميح الأمر نفسه.
2. اقبله:
   ```bash
   orbiscreen pair approve          # أحدث طلب في الانتظار
   orbiscreen pair approve <id>     # معيّن من القائمة
   ```
   يلتقط الجهاز بياناته تلقائياً في استقصاء الحالة التالي.
3. تنتهي الطلبات بعد 10 دقائق. يرفض `orbiscreen pair deny [id]` طلباً، ويزيل
   `orbiscreen pair revoke <client id>` جهازاً مقترناً بالفعل.

---

# مشاكل المُركِّبات

دعم الالتقاط لكل مُركِّب، وكيف يختار محرك `auto` موثّق في `docs/DE_SUPPORT.md`. تتناول هذه المداخلات ما يراه المستخدم فقط حين لا ينجح شيء.

---

<a id="comp-mutter"></a>
## GNOME / Wayland (Mutter) لا يعرض أي صورة

**الأعراض:**
تتصل الحزمة وتُنشأ الشاشة الافتراضية، لكن الحزمة لا تستلم أي إطار، وتسجّل الخدمة سطراً عن محرك الالتcapture الخاص بـGNOME دون إطار أول يطابقه.

**السبب:**
يحتاج GNOME إلى بوابة بث الشاشة، ونافذة الموافقة خاصة بكل جلسة. نافذةٌ أجيب عنها مرة واحدة تسري للجلسة، لكن خدمة انطلقت قبل الإجابة تحتفظ بمقبض غير ممنوح.

**الحل:**
ابدأ الخدمة بعد الموافقة على النافذة، وتأكد أن للجلسة عرض wayland خاص بها:
```bash
echo "$WAYLAND_DISPLAY" "$XDG_SESSION_TYPE"
```
انظر قسم GNOME في `docs/DE_SUPPORT.md` لمسار البوابة.

---

<a id="comp-wlroots"></a>
## sway / Hyprland يُبلّغ عن محرك التقاط لا يبدأ أبداً

**الأعراض:**
يبدأ `sway`، ويُنشأ المخرج، ثم لا يُرمَّز شيء.

**السبب:**
تحتاج مُركِّبات wlroots أن يعلن الجلسة عن بروتوكول نسخ الشاشة. وجلسة متداخلة أو بلا واجهة (`sway --headless` تحت مُركِّب آخر) لا توفّره عادةً.

**الحل:**
تأكد من وجود البروتوكول قبل لوم المُرمِّز:
```bash
swaymsg -t get_outputs | head -40
```
يصف قسم sway وHyprland في `docs/DE_SUPPORT.md` ما تُبلّغ عنه جلسة سليمة.

---

<a id="comp-cosmic"></a>
## COSMIC (cosmic-comp) لا يوفّر محرك التقاط

**الأعراض:**
تبدأ الخدمة، وتبلّغ `capture_backend` بقيمة `auto`، ولا تفتح التقاطاً أبداً.

**السبب:**
يعتمد دعم COSMIC على أن يعرض المُركِّب واجهة البث التي يبحث عنها `orbiscreen`. وحين لا يعرضها، لا يجد `auto` ما يختاره.

**الحل:**
تحقق مما يعلنه المُركِّب، ثم اقرأ قسم COSMIC في `docs/DE_SUPPORT.md` لمعرفة حالة الدعم الحالية قبل افتراض أنه خلل.

---

<a id="comp-x11"></a>
## X11: يهرب المؤشر من الشاشة الافتراضية

**الأعراض:**
تحريك الفأرة فوق الشاشة الافتراضية ينقلها إلى شاشة فعلية، والنقرات تصل إلى الشاشة الخطأ.

**السبب:**
تمرّ مسار إدخال X11 المؤشر المحقون عبر الخادم، والمؤشر الذي يتجاوز حدود الشاشة الافتراضية يلتقطه أي مخرج فعلي يقع تحت موضع المؤشر الفعلي.

**الحل:**
أعد موضع المخرج الافتراضي بحيث لا يتداخل مع مخرج فعلي، وتحقق من التحجيم الذي طبّقه الديمون:
```bash
xrandr --listmonitors
```
انظر [تعدد الشاشات / X11](#cursor-clamping) لسلوك المؤشر نفسه.

---

# سلوك وقت التشغيل

---

<a id="runtime-no-screen"></a>
## لا تظهر أي شاشة على الحزمة

**الأعراض:**
تتصل الحزمة، وتبلّغ عن جلسة، ولا تعرض شيئاً على الإطلاق.

**السبب:**
عادةً أحد ثلاثة أمور: محرك الالتقاط لم ينتج إطارات، أو لا يوجد عنصر للمُرمِّز، أو الحزمة تعرض المخرج الخطأ.

**الحل:**
مرّ عليها بالترتيب:
1. تأكد أن الإطارات موجودة: `busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetStatus`
   وابحث عن `frames_forwarded`
2. تأكد أن المُرمِّز قد حُسم: يبلّغ الخرج نفسه عن `encoder`
3. تأكد أن الحزمة تستهدف الشاشة الافتراضية: انظر
   [العميل يعرض الشاشة الخطأ](#wrong-screen)

---

<a id="runtime-input"></a>
## الفأرة أو اللمس لا يفعلان شيئاً

**الأعراض:**
الفيديو يعمل، لكن الإدخال لا يؤثر في المضيف.

**السبب:**
يحتاج الإدخال إلى `/dev/uinput`، ويُفتح الحقن مرة واحدة عند بدء الجلسة. والخدمة التي تبدأ بلا صلاحية لا تفتح حقناً أصلاً.

**الحل:**
ابحث عن سطر الحقن عند البدء:
```bash
journalctl --user -u orbiscreen | grep -i 'input injector'
```
إن كان غائباً، انظر [رفض صلاحية uinput](#local-uinput). وإن كان موجوداً والإدخال لا يزال لا يفعل شيئاً، فقد لاAddressing ترويسة الجلسة للشاشة التي تنقر عليها؛ انظر [رفض 401](#token-401).

---

<a id="runtime-exits"></a>
## خدمة المضيف تنتهي فوراً

**الأعراض:**
يطبع `orbiscreen start` الشعار ثم تختفي العملية قبل أن تتصل أي حزمة.

**السبب:**
وحدة `systemd` الفاشلة تُبلَّغ في السجل لا في الطرفية، ومحرك التقاط مفقود قاتل عند البدء.

**الحل:**
اسأل الخدمة عمّا سجّلته، واقرأ سجل الوحدة للسبب:
```bash
journalctl --user -u orbiscreen -n 40
busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetStatus
```
موصوف واجهة D-Bus في `docs/DBUS_SPEC.md`.

---
---

# ثغرات حزمة Android

---

<a id="android-decoder"></a>
## المُفكِّك يتراجع إلى البرمجي، أو لا تظهر الصورة أبداً

**الأعراض:**
تسجّل الحزمة:
```
AOA Annex-B video up 2560x1600
MediaCodec configured 2560x1600
codec input full; hold until IDR
```
ثم تبقى على رسالة الانتظار بدل عرض صورة.

**السبب:**
المُفكِّك مُهيّأ ومُغذّى، لكن لم تصل أي إطار مفتاحي بعد. ولا يستطيع أن يبدأ بدون واحد، فهذه عَرَض للنقل لا للمُفكِّك.

**الحل:**
تأكد أن المضيف أرسل واحداً فعلاً:
```bash
journalctl --user -u orbiscreen | grep -i 'IDR requested from encoder'
```
إن وصل طلبات IDR وبقي الجهاز ينتظر، فالإطارات لا تصله. وعلى USB يعني ذلك أن الوصلة لا تواكب:
```bash
for d in /sys/bus/usb/devices/*; do cat "$d/speed" 2>/dev/null && echo " <- $d"; done
```
الوصلة السريعة تبلّغ `12`. اخفض الدقة أو معدل الإطارات حتى تناسب؛ السقف هو الوصلة لا المُرمِّز.

---

<a id="android-app-crash-launch"></a>
## التطبيق يتعطل فور تشغيله

**الأعراض:**
الضغط على الأيقونة يعرض نافذة للحظة ثم لا شيء.

**السبب:**
غالباً تثبيت قديم مكتباته الأصلية لا تطابق الـAPK، أو بناء موقَّع بمفتاح مختلف عن النسخة المثبَّتة، وهو ما يرفض أندرويد استبداله.

**الحل:**
اقرأ التعطّل، ثم ثبّت نظيفاً:
```bash
adb logcat -c && adb shell am start -n com.orbiscreen.android/.MainActivity
adb logcat -d | grep -i orbiscreen
adb uninstall com.orbiscreen.android
adb install orbiscreen-android-release.apk
```

---

<a id="android-usb-host"></a>
## اتصال USB يعرض "Looking for host…" إلى الأبد

**الأعراض:**
يحتجز المضيف الجهاز ويسجّل:
```
AOA accessory claimed on "...", in_ep=0x81, out_ep=0x01
```
لكن الحزمة لا تغادر `Looking for host…`.

**السبب:**
يحوز المضيف دور المساعد، لكن لم تُمنح الحزمة صلاحية فتحه. يعرض أندرويد نافذته الخاصة بذلك، وإلى أن تُجاب لا تستطيع الحزمة سوى الانتظار.

**الحل:**
اقبل نافذة صلاحية USB على الجهاز، وتأكد أن المساعد مرفق فعلاً لا محتجَز فقط:
```bash
adb shell dumpsys usb | grep -i current_functions
```
قيمة `ACCESSORY` تعني أن الدور نشط. وإن لم تظهر النافذة أبداً، فالنسخة المثبَّتة ليست التي يطابق مرشِّح المساعد فيها ما يعلنه المضيف؛ قارن `usb-manufacturer` و`usb-model` و`usb-version` بما تسجّله الخدمة.

---

<a id="android-evdi-host"></a>
## فشل اتصال ADB على ASUS Chromebook CM3001

**الأعراض:**
لا يسرد `adb devices` شيئاً على CM3001 أو جهاز ChromeOS آخر، رغم أن الجهاز والمضيف على نفس الشبكة.

**السبب:**
يشغّل ChromeOS اتصال adb من جهة المضيف عبر نقل شبكي، ولكل نقل شبكة إجراء منفصل عن USB.

**الحل:**
استخدم صيغة الشبكة، بعد وصول الجهاز:
```bash
adb connect <device-lan-ip>:5555
adb devices
```
يوصف مسار Chromebook في `docs/DE_SUPPORT.md`.

---

<a id="android-touchpad-drag"></a>
## سحب النوافذ أو تحديد النصوص في وضع لوحة اللمس

**الأعراض:**
السحب والنقر يحدد نصاً أو يحرّك نافذة بدل أن يرسم.

**السبب:**
تسليمة قلم أو إصبع تُبلَّغ كلمس، ويحقنها المضيف كلمس. ووضع لوحة اللمس على طبقة الجهاز يُبلّغ أحداث أزرار يتصرّف بها المُركِّب أصلاً، فلتُلتقط الإيماءة قبل أن تصل الشاشة الافتراضية.

**الحل:**
استخدم أداة اللوح أو القلم المخصّصة في شريط الأدوات بدل ملامسة مباشرة بالإصبع، وتحقق من الجهاز الذي ربطه المضيف:
```bash
journalctl --user -u orbiscreen | grep 'bound KWin input device'
```
يربط الديمون أجهزة الفأرة واللمس والقلم علىNkseparation، فيخبرك السجل بأيها جاءت عبره الإيماءة.

---

<a id="android-touch-offset"></a>
## اللمس مُدوَّر أو غير محاذٍ

**الأعراض:**
الرسم يقع مُزاحاً عن الإصبع، أو في الجهة الخطأ، بينما الفيديو صحيح.

**السبب:**
يُقرَّر دوران الالتقاط ودوران اللمس المحقون على حدة. وحين يُحجَّم المخرج، تحتاج إحداثيات اللمس نفس التحويل الذي تحتاجه الصورة.

**الحل:**
تأكد من التحجيم الذي طبّقه الديمون، ثم غيّر شيئاً واحداً في كل مرة:
```bash
journalctl --user -u orbiscreen | grep 'Enabled and scaled KWin output'
```
إعادة تشغيل الحزمة بعد تغيير الدقة تعيد ربط التدفق وتعيد ربط أجهزة الإدخال، وهو ما يمسح عادةً تحويلاً قديماً.

---

<a id="android-control-404"></a>
## إجراءات شريط التحكم تُرجع 404

**الأعراض:**
الضغط على عنصر تحكم في الحزمة يُنتج 404 في سجل الديمون.

**السبب:**
يخبر شريط التحكم المسار `/api/control`، وهو يتطلب توكن الجلسة. والحزمة التي لم تجلب التوكن لا ترسل أي اعتماد، فيرفضه المسار.

**الحل:**
تأكد أن الحزمة حصلت على التوكن:
```bash
adb logcat -d | grep -i 'token fetch'
```
سطر ناجح يُبلّغ `available=true`. وإن فشل، فالجلسة لا تستطيع المصادقة أصلاً؛ انظر [رفض 401](#token-401).

---

<a id="android-discovery-empty"></a>
## قائمة الاكتشاف فارغة رغم وجود مضيفين على نفس Wi-Fi

**الأعراض:**
لا تظهر مضيفين في قائمة الحزمة، بينما يعمل مضيف مكتوب يدوياً.

**السبب:**
الاكتشاف يعتمد على البث المتعدد، وهو مرشَّح غالباً على Wi-Fi. والعنوان المكتوب يدوياً يسلك مساراً مختلفاً، ولهذا قد ينفع.

**الحل:**
تأكد من المضيف أن الخدمة معلنة:
```bash
avahi-browse -rt _orbiscreen._tcp
```
ثم انظر [لا يعثر الاكتشاف على المضيف](#net-mdns).

---

### ما زلت عالقاً؟

- [ما زال البناء يفشل؟ راجع سجلات الإجراء](#ما-زلت-عالقاً)
- [إعادة تشغيل مهمة CI واحدة](#إجراء-ci-build-cargo-build---workspace---locked)
- [التحقق من بث حي من طرف إلى طرف ‏(`scripts/verify-stream.sh`)](#رفض-401-من-stream-أو-input-أو-apicontrol-التوكن)
- [تجهيز بيئة تطوير ‏(`scripts/setup-dev-env.sh`)](#إجراء-ci-build-cargo-build---workspace---locked)

---

<a id="ci-fmt"></a>
## إجراء CI: `Format (cargo fmt)`

**العَرَض:**
```
Diff in /home/runner/work/orbiscreen/orbiscreen/crates/...:
```

**السبب:**
الكود البرمجي غير مطابق لمعايير تنسيق Rust القياسية (`rustfmt`).

**الحل:**
```bash
cargo fmt --all
git add -A
git commit -m "orbiscreen | v0.31.3 | style: cargo fmt --all"
```

**الوقاية:**
شغّل `./gradlew :app:lintDebug` و `cargo fmt --all` محلياً قبل الدفع.

---

<a id="ci-clippy"></a>
## إجراء CI: `Clippy (deny warnings)`

**العَرَض:**
```
error: this operation is not supported for derived errors
  --> src/lib.rs:42:5
```

**السبب:**
يعامل أمر `cargo clippy -D warnings` كل تحذيرات clippy كأخطاء توقف البناء.

**الحل:**
```bash
cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 | head -50
cargo clippy --workspace --all-targets --locked --fix
git add -A
git commit -m "orbiscreen | v0.31.3 | fix: resolve clippy warnings"
```

**الوقاية:**
شغّل `cargo clippy` محلياً قبل الدفع.

---

<a id="ci-build"></a>
## إجراء CI: `Build` (`cargo build --workspace --locked`)

**العَرَض:**
```
error[E0463]: can't find crate for `gstreamer`
```

**الحل:**
```bash
cargo update -p gstreamer
cargo build --workspace --locked
git add Cargo.lock
git commit -m "orbiscreen | v0.31.3 | chore: refresh Cargo.lock"
```

---

<a id="ci-test"></a>
## إجراء CI: `Test` ‏(`cargo test --workspace --locked`)

تفترض الاختبارات وجود إضافات GStreamer على المضيف (`x264enc`، `vaapih264enc`، `nvh264enc`). ثبّتها محلياً:
```bash
sudo dnf install gstreamer1.0-plugins-{good,bad,ugly,libav}
```

---

<a id="ci-deny"></a>
## إجراء CI: `Run cargo-deny`

هذا فحص **غير مانع** لأغراض معلوماتية. راجع `deny.toml` لقائمة السماح.

---

<a id="ci-android"></a>
## إجراء CI: `Android assembleDebug` + `lintDebug`

يشغّل سير عمل Android الأمر `./gradlew :app:assembleDebug :app:lintDebug`. الإخفاقات الشائعة:

- **خطأ lint الخاص بـ UnstableApi:** يشترك `clients/android/app/lint.xml` في `androidx.media3.common.util.UnstableApi`. إذا ناديت واجهة Media3 جديدة، تأكد من تعليم الصنف المحيط بـ `@OptIn(UnstableApi::class)`.
- **استيرادات Compose:** شغّل `./gradlew :app:compileDebugKotlin` لتحديد مكان الخطأ أولاً؛ فالـ lint أبطأ.

---

<a id="runtime-evdi"></a>
## وقت التشغيل: فشل `orbiscreen start` - `kernel module is not installed`

**العرَض:**
```
Error: evdi kernel module is not installed
```

**الإصلاح:**
1. ثبّت `evdi` (بناء DKMS) على المضيف:
   ```bash
   # Fedora / Nobara
   sudo dnf install dkms gcc make kernel-devel-$(uname -r) displaylink
   sudo modprobe evdi
   ```
   ```bash
   # Ubuntu / Pop!_OS
   sudo apt install dkms
   git clone https://github.com/DisplayLink/evdi.git
   cd evdi && sudo make dkms-install
   sudo modprobe evdi
   ```
2. تحقق:
   ```bash
   lsmod | grep evdi
   ls /dev/dri/card*
   ```

---

<a id="runtime-kwin"></a>
## وقت التشغيل: KDE Plasma (شاشة افتراضية بدون evdi وبدون root)

**الأعراض:** يسجّل `orbiscreen start` رسالة `EVDI kernel module not active` ولا تريد بناء وحدة نواة.

**الحل:** على KDE Plasma Wayland لا شيء إضافي مطلوب. مع الإعداد الافتراضي `[capture] preferred = "auto"` ينشئ الـ daemon مونيتوراً افتراضياً لكل عميل عبر بروتوكول Wayland‏ `zkde_screencast_unstable_v1` (الاسم الظاهر هو اسم الجهاز، والموصّل `Virtual-Orbi-<key>`) ويبثه عبر PipeWire مباشرة، بلا root وبلا نافذة مشاركة. KWin لا يعرض هذا البروتوكول إلا للتنفيذيات المصرّح لها، لذا يحافظ الـ daemon على الملف `~/.local/share/applications/orbiscreen.kwin.desktop` (قابل للكتابة من المستخدم) ويحدّث ذاكرة KService تلقائياً؛ قد يستغرق التشغيل الأول ثوانٍ إضافية حتى يصبح الترخيص مرئياً.

ملاحظات:
- يمكن فرض المسار عبر `[capture] preferred = "kwin-virtual"` (فشل صريح إن لم يتوفر) أو `"portal"` (إظهار نافذة المشاركة دائماً).
- يختفي مخرج العميل عند انقطاعه، وتختفي كل المخارج عند إيقاف الـ daemon.
- **ترى خلفية سطح المكتب فقط في البث؟** هذا صحيح: الشاشة الافتراضية هي *شاشة ثانية فارغة*. اسحب النوافذ إلى مخرج ذلك العميل (`Virtual-Orbi-<الجهاز>`)، أو اجعل `[capture] preferred = "mirror"` لبث شاشتك الحقيقية بدلاً منها.
- على GNOME / wlroots البروتوكول غير موجود ويرجع `auto` تلقائياً إلى نافذة مشاركة portal.
- أصبح EVDI اختيارياً (`preferred = "evdi"`)؛ لا يلمسه `auto` على Wayland إطلاقاً، لذا لن يظهر سطر `EVDI kernel module not active` القديم على KDE.

---

<a id="runtime-wayland"></a>
## وقت التشغيل: واجهة الالتقاط غير متاحة على Wayland

استخدم `CaptureSession::open_with_preference()` (الـ daemon يفعل ذلك مسبقاً).

---

<a id="runtime-lints"></a>
## وقت التشغيل: `unsafe_op_in_unsafe_fn` / `missing_debug_implementations`

استخدم `#[allow(missing_debug_implementations)]` أو `#[allow(unsafe_code)]` على النوع أو الدالة المعنية.

---

## أجهزة وعميل Android

<a id="android-chromebook-adb"></a>
### Android / ChromeOS: فشل اتصال ADB أو بقاء الرسالة "Looking for host" على ASUS Chromebook CM3001

**العَرَض:**
عند تشغيل تطبيق Orbiscreen على جهاز ASUS Chromebook CM3001 (أو أجهزة ChromeOS الأخرى)، يعلق التطبيق في وضع USB على "Looking for host" ولا يكتشف خادم لينكس.

**السبب:**
يعزل نظام ChromeOS تطبيقات أندرويد داخل حاوية ARC++ مع نطاق شبكة فرعي خاص (`100.115.92.0/28`). بث USB يستخدم Android Open Accessory، وليس `adb reverse` إلى Crostini.

**الحل:**
- شغّل `orbiscreen start` داخل حاوية لينكس، ثم اقبل حوار إذن ملحق USB على جانب Android (Orbiscreen Display Server).
- عندما لا تكون AOA جاهزة، يظل العميل يفحص بوابات ARC / مشاركة USB ‏(`100.115.92.2`، ...) على منفذ الإشارة.
- بث USB نفسه يتم عبر Android Open Accessory ولا يتطلب تفعيل تصحيح USB ولا `adb reverse`.

---

<a id="android-stylus"></a>
### Android: القلم لا يرسم، أو حساسية ضغط غير صحيحة، أو توقف التطبيق على Lenovo Tab

**العَرَض:**
عند استخدام القلم الذكي على أجهزة مثل Lenovo Tab (IdeaTab) أو Chromebook:
1. يتجمد التطبيق أو ينهار مع خطأ `NetworkOnMainThreadException` بمجرد تحريك القلم.
2. غياب مؤشر القلم عند التحليق في الهواء فوق الشاشة.
3. خطأ في زوايا الميلان أو بقاء الضغط معلقاً عند رفع القلم.

**السبب:**
في الإصدارات السابقة، كانت حزم شبكة القلم تُرسل مباشرة على الخيط الرسومي الرئيسي لتطبيق أندرويد. كما كانت واجهة الاستماع لحركة التحليق `ACTION_HOVER_MOVE` غير مفعلة، ولم تكن إشارة رفع القلم تُرسل عند وصول الضغط إلى صفر.

**الحل:**
- حدّث التطبيق إلى الإصدار **v0.31.3** أو أحدث.
- ينقل v0.31.3 معالجة حزم القلم بالكامل إلى خلفية غير متزامنة عبر `Dispatchers.IO` مع دمج الأحداث السريعة عبر `latestStylus` لمنع تجمد الواجهة أو انهيار التطبيق.
- يفعل `setOnGenericMotionListener` لتتبع حركة المؤشر في الهواء أثناء تحليق القلم.
- يصحح معادلة زاوية الميلان (`-altitudeDeg * cos(orientationRad)`) ويرسل إشارة `BTN_TOOL_PEN: RELEASED` عند رفع القلم وانعدام الضغط.

---

<a id="android-touchpad-drag"></a>
### Android: سحب النوافذ وتحديد النصوص في وضع لوحة اللمس (Touchpad Drag-and-Drop)

**العَرَض:**
في وضع لوحة اللمس (Touchpad)، عند النقر والسحب على شاشة التابلت يتحرك المؤشر فقط دون سحب النوافذ أو تحديد النصوص.

**السبب:**
كان وضع لوحة اللمس سابقاً يفسر جميع الحركات كمجرد تحريك للمؤشر دون دعم إيماءة السحب.

**الحل:**
- التحديث إلى **v0.31.3**.
- **النقر المزدوج مع السحب:** انقر مرتين سريعاً على الشاشة مع إبقاء إصبعك مضغوطاً في النقرة الثانية. أثناء تحريك إصبعك، يظل زر الفأرة الأيسر مضغوطاً لسحب النوافذ أو نقل الملفات أو تحديد النصوص بسلاسة.
- رفع إصبعك عن الشاشة يحرر زر الفأرة فوراً.

---

<a id="android-connect-crash"></a>
### Android: التطبيق يتعطل أو تموت العملية عند النقر على Connect

**العرَض:**
النقر على مضيف في شاشة Discovery يقتل فوراً عملية التطبيق أو يعيده إلى المشغّل (launcher).

**السبب:**
يجب إنشاء `PlayerHolder.build()` على الخيط الرئيسي. يتطلب ExoPlayer الإنشاء على الخيط الرئيسي؛ وإنشاء مكوّنات المشغّل على خيوط IO يرمي استثناءات وصول خيطي تنهي العملية.

**الإصلاح:**
- حدّث إلى `orbiscreen-android-release.apk` الإصدار **v0.31.3** أو أحدث.
- ينشئ `StreamViewModel` مشغّل ExoPlayer على `Dispatchers.Main` مع تحصين عبر try-catch لتظهر الأخطاء كبطاقة `StreamEvent.Error` قابلة لإعادة المحاولة بدل الانهيار.

---

<a id="android-black-screen"></a>
### Android: شاشة سوداء بعد Connect

**العرَض:**
النقر على مضيف مكتشف يعرض سطحاً أسود؛ لا فيديو؛ ولا يظهر شريط التحكم.

**السبب:**
محاولة ExoPlayer التعرف التلقائي على نوع الوسائط (MIME sniffing) لمسار `/stream` والتراجع لسطح أسود عند تعذر الكشف التلقائي.

**الإصلاح:**
- حدّث إلى `orbiscreen-android-release.apk` الإصدار **v0.31.3** أو أحدث.
- يقوم `PlayerHolder` بضبط `MediaItem` بنوع صريح `setMimeType(MimeTypes.VIDEO_MP2T)` لفك ترميز البث مباشرة دون الحاجة للتعرف التلقائي.
- تظهر الأخطاء كبطاقة إعادة محاولة واضحة بدل السطح الأسود.

إذا استمرت المشكلة بعد التحديث:
1. تأكد من إمكانية الوصول إلى المضيف عبر `curl http://host:8788/health` من نفس شبكة Wi-Fi.
2. تأكد من استجابة `/api/info`: `curl http://host:8788/api/info`.
3. افحص `adb logcat -s OrbiPlayer:*` بحثاً عن أسطر `player error:`.

---

<a id="android-no-hosts"></a>
### Android: قائمة الاكتشاف فارغة رغم وجود مضيفين على نفس شبكة Wi-Fi

**السبب:**
بروتوكول mDNS محظور على الشبكة (شبكات العمل المقيدة، جدار الحماية، إلخ).

**الإصلاح:**
1. افتح بطاقة **Add manually** وأدخل `host:port` (مثال `192.168.1.50:8788`).
2. اختياري: فعّل خيار **Scan subnet for hosts** في **Settings**. يفحص الماسح نطاق ‎/24 عبر TCP ويضيف أي جهاز يستجيب على المنفذ 8788.

---

<a id="android-touch-offset"></a>
### Android: اللمس مُدوَّر / غير محاذٍ

**السبب:**
يعتمد تعيين المؤشر إلى المضيف على دقة الشاشة المُبلَّغ عنها من `/api/info`. إذا كان المضيف مُدوَّراً (مثلاً شاشة افتراضية عمودية) لكن JSON ما زال يبلّغ عن اتجاه أفقي، فسيكون التعيين غير متطابق.

**الإصلاح:**
دوّر المضيف بدلاً من شاشة Android. يطبّق `PlayerView` ضبط النطاق تلقائياً للحفاظ على نسبة العرض إلى الارتفاع المحددة.

---

<a id="android-control-404"></a>
### Android: إجراءات شريط التحكم تُرجع 404

**السبب:**
المضيف يشغّل إصداراً أقدم من الخدمة لا يدعم نقطة `/api/control`.

**الإصلاح:**
أعد تشغيل الخدمة على المضيف:
```bash
orbiscreen stop
orbiscreen start
```

---

<a id="android-crash"></a>
### Android: التطبيق يتعطل فوراً عند التشغيل

**العرَض:**
تفتح تطبيق Orbiscreen على Android فيتعطل فوراً ويعود إلى الشاشة الرئيسية.

**السبب:**
مشاكل سابقة متعلقة بـ WebView في الإصدارات القديمة.

**الإصلاح:**
يعتمد تطبيق Orbiscreen على Jetpack Compose + `PlayerView` أصيلاً دون WebView. تأكد من تثبيت `orbiscreen-android-release.apk` الإصدار **v0.31.3** أو أحدث. إذا واجهت أي مشكلة، التقط السجل عبر `adb logcat *:E | grep orbiscreen` وافتح بلاغاً في GitHub.

---

<a id="android-usb"></a>
### Android: اتصال USB يعرض "Looking for host…"

**الإصلاح:**
USB يستخدم Android Open Accessory وليس `adb reverse`. تأكد من:
1. تشغيل الدامن (`orbiscreen start`).
2. توصيل الكابل والموافقة على حوار إذن الملحق (Orbiscreen Display Server).
3. تحقق مما يراه الدامن:
   ```bash
   orbiscreen doctor
   ```
4. إن أُلغي الحوار، انقر بطاقة USB في شاشة Discovery.

عدد الملحقات لدى الدامن مرئي عبر `GET /health`‏ (`usb_devices`) وفي `GetStatus` عبر D-Bus.

<a id="streaming-wifi-latency"></a>
## البث: بطء شديد أو تقطيع في حركة الفأرة عبر شبكة 5GHz Wi-Fi

**العَرَض:**
عند الاتصال عبر شبكة واي فاي 5GHz (مثل أجهزة Lenovo Tab أو الهواتف)، تبدو حركة الفأرة ثقيلة جداً أو متأخرة بفارق زمني ملحوظ، أو يتأخر بث الشاشة عن المضيف.

**السبب:**
1. استخدام ذاكرة تخزين مؤقت كبيرة لتشغيل الفيديو يؤدي إلى تراكم الإطارات وزيادة التأخير في البث التفاعلي المباشر.
2. تباعد الإطارات المفتاحية (GOP) يجبر المشغل على الانتظار طويلاً عند فقدان أي حزمة بيانات عبر الشبكة اللاسلكية.
3. تجميع أحداث الفأرة بفاصل زمني طويل نسبياً.

**الحل:**
- يقوم الإصدار v0.31.3 بضبط مسار البث بالكامل لأدنى كمون واستجابة فورية:
  - **إطار مفتاحي كل 6 إطارات (GOP 6):** ترسل المرمّزات العتادية إطاراً مفتاحياً كل 100ms، ما يسمح باستعادة سريعة عند التشويش أو فقد الحزم.
  - **توليف ذاكرة ExoPlayer (40-120ms):** تخفيض التخزين المؤقت إلى 40ms كحد أدنى و 120ms كحد أقصى لإبقاء العرض عند أحدث إطار.
  - **حلقة إرسال الفأرة 8ms:** تقليص نافذة تجميع الفأرة إلى 8ms لمواكبة معدّلات تحديث تصل إلى 120Hz.
- تأكد من ضبط راوتر Wi-Fi 5GHz على قناة غير مزدحمة وبعرض نطاق 80MHz.

---

<a id="stream-disconnect-retry"></a>
## البث: وميض وإعادة اتصال لانهائية عند حدوث خطأ في البث بدل التعرف على انقطاع الاتصال

**العَرَض:**
عند إيقاف خادم لينكس أو تعطل الشبكة، يظل تطبيق أندرويد يومض ويحاول إعادة الاتصال بلا نهاية دون إظهار شاشة توقف واضحة.

**السبب:**
كانت المشغلات تفتقر إلى حالة انقطاع صريحة، وتستمر في محاولات الاتصال دون حد أقصى.

**الحل:**
- في v0.31.3، أُضيفت حالة صريحة `StreamEvent.Disconnected`.
- فور حدوث خطأ في الشبكة، يُطلق التطبيق فحصاً سريعاً خلال 500ms لنقطة `/health` للتأكد من حالة الخادم.
- حُددت محاولات إعادة الاتصال بـ 3 محاولات فقط؛ وعند تعذر الوصول للخادم يعرض التطبيق بطاقة انقطاع الاتصال مع زر لإعادة المحاولة اليدوية.

---

<a id="cursor-clamping"></a>
## تعدد الشاشات / X11: هروب مؤشر الفأرة من الشاشة الافتراضية إلى الشاشات المادية الأخرى

**العَرَض:**
عند تحريك الفأرة أو القلم على التابلت، يقفز المؤشر خارج حدود الشاشة الافتراضية إلى شاشة اللابتوب أو الشاشات المادية الأخرى.

**السبب:**
حقن إحداثيات XTEST بدون تقييد أبعاد المخرج يمتد على كامل مساحة سطح المكتب المجمعة.

**الحل:**
- في v0.31.3، يستعلم Orbiscreen عن أبعاد الشاشة الافتراضية بدقة عبر XRandR ويقيد حركة المؤشر والقلم تماماً داخل مستطيل الشاشة الافتراضية (`InputProp::DIRECT`).
- ينحصر المؤشر داخل شاشة التابلت دون القفز إلى الشاشات الأخرى.

---

<a id="wrong-screen"></a>
## العميل يعرض الشاشة الخطأ (سطح المكتب الرئيسي بدل الشاشة الافتراضية)

**العرض:**
يتصل عميل Android/الويب ويعرض فيديو، لكنه يعكس سطح مكتب المضيف الرئيسي بدل شاشة ثانية نظيفة. سحب النوافذ إلى شاشة ثانية لا يفعل شيئاً.

**السبب:**
وحدة النواة `evdi` غير محملة، فيتراجع Orbiscreen إلى التقاط سطح المكتب الرئيسي (Wayland portal أو نافذة جذر X11). هذا الوضع المتدهور مقصود: يبلغ `GetStatus.capture_backend` عن `wayland-portal-fallback` أو `x11-portal-fallback` بدل `evdi`، ويسجل الدامن تحذير `EVDI kernel module missing/inactive ... Falling back` عند البدء.

**الإصلاح:**
1. ثبّت وحمّل `evdi` عبر DKMS - راجع [وقت التشغيل: فشل `orbiscreen start`](#وقت-التشغيل-kde-plasma-شاشة-افتراضية-بدون-evdi-وبدون-root)، ثم:
   ```bash
   sudo modprobe evdi && lsmod | grep evdi
   ```
2. أعد تشغيل الدامن (`orbiscreen stop && orbiscreen start`) وتحقق:
   ```bash
   busctl --user call com.orbiscreen.Daemon /com/orbiscreen/Daemon com.orbiscreen.Daemon GetStatus
   # "capture_backend":"evdi"
   ```
3. انقل نافذة إلى مخرج Orbiscreen (‏`EVDI-0`) من إعدادات الشاشات في المنشئ.

---

<a id="web-no-picture"></a>
## عميل الويب يُحمَّل لكن بلا صورة

**العرض:**
`https://<host>:8790/client/` يُحمل، وطبقة الحالة تظل على "Connecting"، أو تظهر رسالة "Unsupported browser" تطلب Chrome أو Brave أو Edge.

**السبب:**
عميل الويب يفتح جلسة WebTransport ويفك وحدات Annex-B عبر WebCodecs ‏`VideoDecoder`‏ على عنصر canvas. مسارا HTTP ‏`/` و `/client/`‏ على منفذ الإشارة يعيدان التوجيه إلى هناك. يجب قبول الشهادة الذاتية التوقيع مرة واحدة؛ ويعيد الدامن استخدام `$XDG_CONFIG_HOME/orbiscreen/wt-cert.pem` (مع `wt-key.pem`) بين عمليات إعادة التشغيل. المتصفحات بدون ‏`VideoDecoder`‏ (مثل Firefox Mobile) لا تفك البث إطلاقاً. لا يوجد مسار MSE ولا WebRTC.

**الإصلاح:**
1. افتح الصفحة في Chrome أو Brave أو Edge أو أي متصفح Chromium آخر. Firefox Mobile لا يدعم ‏WebCodecs `VideoDecoder`‏.
2. تأكد أن الصفحة خُدمت عبر HTTPS على منفذ WebTransport ‏(`signaling_port + 2`، أي 8790 عادةً)‏ وأنك قبلت الشهادة.
3. راجع وحدة التحكم/الشبكة في أدوات المطور: خطأ 401 على `/au` أو فشل رسالة Hello يعني أن مسار التوكن تعطّل - راجع [رفض 401](#رفض-401-من-stream-أو-input-أو-apicontrol-التوكن).

---

<a id="no-encoder"></a>
## لا يوجد مُرمَّز - البث يبدأ لكنه يفشل (غياب x264)

**العرض:**
يبدأ الدامن ويتصل العملاء، لكن الفيديو لا يصل أو يظهر في السجل خطأ ربط عناصر GStreamer يذكر `x264enc` / `no element found`.

**السبب:**
الترميز يمر عبر GStreamer. عنصر التراجع البرمجي `x264enc` يأتي في حزمة الإضافات `ugly`؛ والمرمزات العتادية تحتاج `vaapih264enc` أو `nvh264enc` (حزمة `bad`). بدونها لا يمكن إنتاج H.264.

**الإصلاح:**
```bash
# Fedora / Nobara
sudo dnf install gstreamer1-plugins-ugly gstreamer1-plugins-bad-free gstreamer1-plugins-good

# Ubuntu / Debian
sudo apt install gstreamer1.0-plugins-ugly gstreamer1.0-plugins-bad gstreamer1.0-plugins-good

# تحقق من وجود عنصر الترميز
gst-inspect-1.0 x264enc
```
ثم أعد تشغيل الدامن؛ يبلغ `GetStatus.encoder` عن المُرمَّز المستخدم فعلياً.

---

<a id="no-encoder"></a>
## لا يوجد مُرمِّز - البث يبدأ لكنه يفشل (غياب x264)

**الأعراض:**
الخدمة تبدأ، والعملاء يتصلون، لكن الفيديو لا يصل أبداً، أو تظهر في السجل أخطاء ربط عناصر GStreamer تشير إلى `x264enc` أو `no element found`.

**السبب:**
الترميز يمر عبر GStreamer. عنصر الاحتياطي البرمجي `x264enc` يأتي في مجموعة الإضافات `ugly`، والمُرمِّزات العتادية تحتاج `vaapih264enc` من `bad` أو `nvh264enc` من `bad`. بدونها لا يُنتَج أي H.264.

**الحل:**
```bash
# Fedora / Nobara
sudo dnf install gstreamer1-plugins-ugly gstreamer1-plugins-bad-free gstreamer1-plugins-good

# Ubuntu / Debian
sudo apt install gstreamer1.0-plugins-ugly gstreamer1.0-plugins-bad gstreamer1.0-plugins-good

# التحقق من وجود عنصر المُرمِّز
gst-inspect-1.0 x264enc
```
ثم أعد تشغيل الخدمة؛ ويعرض `GetStatus.encoder` المُرمِّز المستخدَم فعلياً.

---

<a id="token-401"></a>
## رفض 401 من `/stream` أو `/input` أو `/api/control` (التوكن)

**العرض:**
يحصل العملاء (Android أو الويب أو سكربتات مكتوبة يدوياً) على `401 Unauthorized`. ‏`curl http://host:8788/health` يعمل بشكل طبيعي، لكن `/stream` و`/input` و`/api/control` ترفض الطلب.

**السبب:**
تشترط هذه النقاط توكن الوصول الخاص بالجلسة الذي يتم توليده مع كل تشغيل للدامن.
- **التحقق من التوكن:** تأكد من أن العميل يمرر توكن الجلسة الصحيح المطابق لجلسة الدامن الحالية. عند الاتصال عبر المتصفح، تقوم النقطة `/client/config.json` بتمهيد التوكن تلقائياً، أو يمكن تمريره عبر `#token=` في الرابط.
- يجب على متصفحات الويب الخارجية تقديم التوكن في عنوان الرابط مباشرة.

**الإصلاح:**
1. لمتصفحات الويب الخارجية، أضف التوكن عبر تجزئة الرابط (Hash) أو الاستعلام:
   ```
   http://<host-ip>:8788/#token=<SECRET_TOKEN>
   ```
   أو:
   ```
   http://<host-ip>:8788/?token=<SECRET_TOKEN>
   ```
2. استخرج التوكن من جهاز المضيف:
   ```bash
   orbiscreen doctor
   # أو قراءة ملف التوكن المحمي بصلاحيات 0o600:
   cat ~/.config/orbiscreen/token
   ```
3. يستقبل تطبيق أندرويد التوكن تلقائياً عبر سجلات mDNS TXT. وفي حال الإضافة اليدوية لمضيف، أدخل التوكن في نافذة الإعدادات.
4. مرر التوكن في السكربتات عبر ترويسة المصادقة:
   ```bash
   curl -H "Authorization: Bearer $TOKEN" http://host:8788/stream --output - | head -c 1000
   ```

---

<a id="dbus-missing"></a>
## الـ daemon غير موجود على D-Bus

**العرض:**
يطبع `orbiscreen stop` الرسالة `daemon is not running (no com.orbiscreen.Daemon on the session bus)`

**السبب:**
خدمة D-Bus (‏`com.orbiscreen.Daemon`) تُسجل على **ناقل جلسة المستخدم** من طرف عملية الدامن طالما هي قيد التشغيل. أسباب غيابها الشائعة:
- الدامن لم يبدأ (أو انهار) في جلسة المستخدم الحالية.
- بدأ `orbiscreen start` بمستخدم آخر أو بـ `sudo` - ناقل النظام/المستخدم الآخر ليس ناقل جلستك.
- ‏`DBUS_SESSION_BUS_ADDRESS` غير مضبوط أو متجاوز في الصدفة التي يُشغَّل فيها `orbiscreen stop`.

**الإصلاح:**
1. تحقق من الخدمة والحالة:
   ```bash
   busctl --user status com.orbiscreen.Daemon 2>&1 || echo "not on the bus"
   systemctl --user status orbiscreen
   ```
2. شغله بمستخدمك العادي: `orbiscreen start` (دون `sudo`) أو `systemctl --user start orbiscreen`.
3. إذا شُغل تحت systemd فضل `systemctl --user stop orbiscreen` لإيقافه (يعمل `orbiscreen stop` أيضاً ويتراجع إلى تابع D-Bus `Stop`).

---

<a id="daemon-cpu"></a>
## الـ Daemon: استهلاك 100% للمعالج أو تجمّد

**السبب:**
كانت حلقة الالتقاط تعمل دون إخلاء للمعالج أو تراكم غير محدود في الطابور.

**الإصلاح:**
حدّث إلى الإصدار الأخير (v0.31.3 أو أحدث).

---

<a id="still-stuck"></a>
## ما زلت عالقاً؟

<a id="re-run-job"></a>
### إعادة تشغيل مهمة CI واحدة

في صفحة PR الفاشلة:
1. افتح قسم **Checks**.
2. انقر اسم الفحص الفاشل.
3. انقر **Re-run jobs** ← **Re-run failed jobs**.

### مراجعة سجلات الإجراء

يعرض قسم **Run logs** مخرجات `cargo` / `gradlew` الدقيقة. قارنها مع الأقسام أعلاه.

### فتح issue

استخدم `.github/ISSUE_TEMPLATE/bug.yml`. أرفق:
- مخرجات خطأ `cargo` / `gradlew` الدقيقة.
- رابط تشغيل CI.
- نظام التشغيل / المنشّئ (compositor) للمضيف (إن كانت المشكلة وقت تشغيل).
- مخرجات `adb logcat *:E` (إن كانت المشكلة متعلقة بـ Android).

<a id="verify-stream"></a>
### التحقق من بث حي من طرف إلى طرف

```bash
./scripts/verify-stream.sh [المنفذ] [مدة_بالثواني]
```

يسجّل بضع ثوانٍ من `/stream`، يتحقق من أن الحمولة تُفكّ ترميزها كـ H.264، ويقيس سطوع الإطارات (YAVG) لالتقاط تراجعات البث الأسود/الفارغ تلقائياً. يتطلب `curl` و`python3` و`ffmpeg` على المضيف.

<a id="setup-dev-env"></a>
### تجهيز بيئة تطوير

```bash
./scripts/setup-dev-env.sh
```

يثبّت سلسلة أدوات Rust واعتماديات البناء (GStreamer وWayland/X11 وlibevdev) لتوزيعات Fedora وDebian وArch المكتشفة من `/etc/os-release`.

---

<div align="center">

بُني بواسطة <a href="https://github.com/shadow-x78">shadow-x78</a> ·
[العودة إلى README](../README_AR.md)

<sub>&copy; 2026 Orbiscreen (shadow-x78)</sub>

</div>
