// ─────────────────────────────────────────────
// Orbiscreen - Inner page copy (EN / AR)
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import type { Lang } from '../lib/site';
import { RUST_VERSION } from '../lib/version';

export interface Section {
  id: string;
  title: string;
  body: string[];
  points?: string[];
}

export interface FeaturesPage {
  eyebrow: string;
  title: string;
  lede: string;
  sections: Section[];
  cta: { title: string; body: string; primary: string; secondary: string };
}

export interface InstallPage {
  eyebrow: string;
  title: string;
  lede: string;
  requirements: { title: string; items: string[] };
  after: { title: string; steps: { title: string; text: string }[] };
  platformTitle: string;
  platformLede: string;
  cta: { title: string; body: string; primary: string; secondary: string };
}

export interface ComparePage {
  eyebrow: string;
  title: string;
  lede: string;
  methodology: { title: string; body: string[] };
  differences: { title: string; body: string }[];
  cta: { title: string; body: string; primary: string; secondary: string };
}

export interface FaqPage {
  eyebrow: string;
  title: string;
  lede: string;
  cta: { title: string; body: string; primary: string; secondary: string };
}

const featuresEn: FeaturesPage = {
  eyebrow: 'FEATURES',
  title: 'Everything a second screen needs.',
  lede:
    'Orbiscreen is not a mirror and not a remote desktop. It creates a real virtual monitor on the Linux host, streams it as H.264, and injects the tablet input back as native Linux events.',
  sections: [
    {
      id: 'virtual-display',
      title: 'A virtual monitor, three ways',
      body: [
        'On KDE Plasma and wlroots compositors the output is created by the compositor itself through its own IPC, with no kernel module and no root. GNOME 46+ and KDE also expose a portal virtual monitor. Everywhere else, the EVDI kernel module provides the display.',
        'The result is an independent output: windows can be moved onto it, the resolution is configurable up to 7680x4320, and the main display keeps its own layout, scale, and refresh rate.',
      ],
      points: [
        'KDE Plasma and wlroots: compositor-native output, no root',
        'GNOME 46+: portal virtual monitor',
        'COSMIC, X11, and other desktops: EVDI kernel display',
        'Resolution up to 8K, portrait and landscape',
      ],
    },
    {
      id: 'encoding',
      title: 'H.264 tuned for latency, not bitrate',
      body: [
        'Frames are encoded with NVENC, VA-API, or x264 depending on what the host exposes. The default profile is built for a LAN: constant bitrate around 8 Mbps, no B-frames, and an IDR every 120 frames so a lost packet recovers quickly.',
        'Hardware encoding keeps the CPU free and the encode step short. The encoder is selected in the config file, not on the command line, so a working setup survives restarts.',
      ],
      points: [
        'NVENC, VA-API, or x264, chosen automatically',
        'CBR 8 Mbps, no B-frames, IDR every 120 frames',
        'Encoder preference persisted in orbiscreen.toml',
      ],
    },
    {
      id: 'input',
      title: 'Input that applications believe',
      body: [
        'Touch, mouse, keyboard, and pen travel back to the host and are injected as real Linux input events. The stylus is exposed as a tablet through uinput with 4095 pressure levels, tilt, and in-air hover, so Krita, GIMP, Blender, and Inkscape see a drawing device rather than a mouse.',
        'Touchpad mode turns the tablet into a relative pointer with double-tap drag, constrained to the virtual display bounds so the cursor cannot escape onto another monitor.',
      ],
      points: [
        'Multi-touch, mouse, and keyboard',
        '4095-level pressure, tilt, and hover through uinput',
        'Touchpad mode with double-tap drag',
        'Pointer constrained to the virtual display',
      ],
    },
    {
      id: 'transport',
      title: 'Transport that survives a LAN',
      body: [
        'The default video path is UDP with Annex-B framing and Reed-Solomon forward error correction, so a lost datagram costs a slice rather than a frame. Browsers use WebTransport, and USB uses Android Open Accessory bulk endpoints with no adb bridge at all.',
        'HTTP MPEG-TS remains available as a fallback for clients or networks that cannot use the UDP path. Discovery is mDNS first, with manual entry and an optional subnet scanner behind it.',
      ],
      points: [
        'UDP Annex-B with Reed-Solomon FEC',
        'WebTransport for browsers',
        'USB AOA, no adb reverse',
        'HTTP MPEG-TS fallback',
        'mDNS discovery, manual entry, subnet scan',
      ],
    },
    {
      id: 'web-client',
      title: 'A web client with no install',
      body: [
        'Any Chromium browser can act as a receiver. The built-in web client uses WebTransport for the transport and WebCodecs for decoding, so the browser hands frames straight to the platform decoder.',
        'The web client is served by the host itself, which means there is no account, no cloud relay, and no third-party service in the path.',
      ],
      points: [
        'Runs in Chromium browsers',
        'WebTransport transport, WebCodecs decoding',
        'Served by the host on the local network',
      ],
    },
    {
      id: 'android-client',
      title: 'A native Android client',
      body: [
        'The Android app is written in Kotlin with Jetpack Compose and Material 3, with light and dark themes, hardware MediaCodec decoding, and both UDP and USB transports.',
        'Pairing is explicit: the host advertises a session token over mDNS, and the app can file a pairing request that you approve from the CLI.',
      ],
      points: [
        'Kotlin, Jetpack Compose, Material 3',
        'MediaCodec hardware decoding',
        'UDP and USB transports',
        'Explicit pairing with host approval',
      ],
    },
    {
      id: 'control',
      title: 'Control and automation',
      body: [
        'The daemon exposes a D-Bus session interface and a CLI. The CLI covers start, stop, doctor, pair, display, and update; the D-Bus interface covers status, resolution, configuration, and client listing, authorized for the daemon owner only.',
        'An authenticated host API adds lock, blank, unblank, and Ctrl+Alt+Del, and the daemon ships as a systemd user unit so it starts with the session.',
      ],
      points: [
        'CLI: start, stop, doctor, pair, display, update',
        'D-Bus: status, resolution, config, clients',
        'Host control: lock, blank, unblank, Ctrl+Alt+Del',
        'systemd user unit included',
      ],
    },
    {
      id: 'security',
      title: 'Security and signed releases',
      body: [
        'Every client is authenticated with a session token, a paired client cannot attach to another client\'s session, and input messages are capped in size. Credential-bearing paths are served over TLS when the host has a certificate.',
        'Linux and Android release artifacts are cryptographically signed, and the release workflow publishes the signatures alongside the binaries.',
      ],
      points: [
        'Per-client session tokens',
        'Session ownership enforced on stream attach',
        'TLS for credential-bearing endpoints',
        'Signed Linux and Android artifacts',
      ],
    },
  ],
  cta: {
    title: 'Ready to add a screen?',
    body: 'Install the host, open the app or the web client, and approve the pairing request. The doctor command tells you exactly what your desktop will use.',
    primary: 'Install Orbiscreen',
    secondary: 'Read the docs',
  },
};

const featuresAr: FeaturesPage = {
  eyebrow: 'المزايا',
  title: 'كل ما تحتاجه شاشة ثانية.',
  lede:
    'Orbiscreen ليس نسخاً للشاشة ولا سطح مكتب بعيد. ينشئ شاشة افتراضية حقيقية على مضيف Linux، ويبثها بترميز H.264، ويعيد إدخال التابلت كأحداث Linux أصلية.',
  sections: [
    {
      id: 'virtual-display',
      title: 'شاشة افتراضية بثلاث طرق',
      body: [
        'على KDE Plasma ومديري نوافذ wlroots ينشئ المدير نفسه المخرج عبر IPC الخاص به، بلا وحدة نواة وبلا root. ويوفّر GNOME 46+ وKDE أيضاً شاشة افتراضية عبر البوابة. وفي بقية البيئات توفّر وحدة نواة EVDI الشاشة.',
        'الناتج مخرج مستقل: يمكن نقل النوافذ إليه، ودقته قابلة للضبط حتى 7680x4320، وتبقى الشاشة الأساسية بتخطيطها وتحجيمها ومعدل تحديثها.',
      ],
      points: [
        'KDE Plasma وwlroots: مخرج أصلي من المدير، بلا root',
        'GNOME 46+: شاشة افتراضية عبر البوابة',
        'COSMIC وX11 وغيرها: شاشة نواة EVDI',
        'دقة حتى 8K، طولية وأفقية',
      ],
    },
    {
      id: 'encoding',
      title: 'ترميز H.264 مضبوط لزمن الاستجابة لا لمعدل البتات',
      body: [
        'تُرمَّز الإطارات بـ NVENC أو VA-API أو x264 حسب ما يوفّره المضيف. والملف الافتراضي مبني للشبكة المحلية: معدل بتات ثابت نحو 8 Mbps، وبلا إطارات B، وIDR كل 120 إطاراً ليتعافى الفقد سريعاً.',
        'يبقي الترميز العتادي المعالج حراً وخطوة الترميز قصيرة. ويُختار المُرمِّز في ملف الإعدادات لا في سطر الأوامر، فيبقى الإعداد العامل بعد إعادة التشغيل.',
      ],
      points: [
        'NVENC أو VA-API أو x264، باختيار تلقائي',
        'CBR 8 Mbps، بلا إطارات B، وIDR كل 120 إطاراً',
        'تفضيل المُرمِّز محفوظ في orbiscreen.toml',
      ],
    },
    {
      id: 'input',
      title: 'إدخال تصدّقه التطبيقات',
      body: [
        'يعود اللمس والفأرة ولوحة المفاتيح والقلم إلى المضيف وتُحقن كأحداث إدخال Linux حقيقية. ويُعرض القلم كلوح رسم عبر uinput بـ4095 مستوى ضغط وميل وطوفان في الهواء، فترى Krita وGIMP وBlender وInkscape جهاز رسم لا فأرة.',
        'يحوّل وضع لوحة اللمس التابلت إلى مؤشر نسبي بسحب النقر المزدوج، مقيّداً بحدود الشاشة الافتراضية كي لا يهرب المؤشر إلى شاشة أخرى.',
      ],
      points: [
        'لمس متعدد وفأرة ولوحة مفاتيح',
        'ضغط 4095 مستوى وميل وطوفان عبر uinput',
        'وضع لوحة لمس بسحب النقر المزدوج',
        'مؤشر مقيّد بالشاشة الافتراضية',
      ],
    },
    {
      id: 'transport',
      title: 'نقل يصمد على الشبكة المحلية',
      body: [
        'المسار الافتراضي للفيديو هو UDP بتأطير Annex-B وتصحيح أخطاء أمامي Reed-Solomon، فيكلّف فقد حزمة شريحة لا إطاراً. وتستخدم المتصفحات WebTransport، ويستخدم USB منافذ Android Open Accessory بلا أي جسر adb.',
        'يبقى HTTP MPEG-TS بديلاً للعملاء أو الشبكات التي لا تستطيع مسار UDP. والاكتشاف عبر mDNS أولاً، ثم الإدخال اليدوي وفحص النطاق اختيارياً.',
      ],
      points: [
        'UDP Annex-B مع تصحيح Reed-Solomon',
        'WebTransport للمتصفحات',
        'USB AOA بلا adb reverse',
        'بديل HTTP MPEG-TS',
        'اكتشاف mDNS وإدخال يدوي وفحص نطاق',
      ],
    },
    {
      id: 'web-client',
      title: 'عميل ويب بلا تثبيت',
      body: [
        'أي متصفح Chromium يصلح مستقبلاً. يستخدم عميل الويب المدمج WebTransport للنقل وWebCodecs لفك الترميز، فيسلّم المتصفح الإطارات مباشرة إلى مفكّك المنصة.',
        'يقدّم المضيف عميل الويب بنفسه، فلا حساب ولا مرحّل سحابي ولا خدمة طرف ثالث في المسار.',
      ],
      points: [
        'يعمل في متصفحات Chromium',
        'نقل WebTransport وفك ترميز WebCodecs',
        'يقدّمه المضيف على الشبكة المحلية',
      ],
    },
    {
      id: 'android-client',
      title: 'تطبيق Android أصلي',
      body: [
        'تطبيق Android مكتوب بـ Kotlin وJetpack Compose وMaterial 3، بسمتين فاتحة وداكنة، وفك ترميز عتادي عبر MediaCodec، ومساري UDP وUSB.',
        'الاقتران صريح: يعلن المضيف رمز الجلسة عبر mDNS، ويستطيع التطبيق تقديم طلب اقتران توافق عليه من سطر الأوامر.',
      ],
      points: [
        'Kotlin وJetpack Compose وMaterial 3',
        'فك ترميز عتادي MediaCodec',
        'مساري UDP وUSB',
        'اقتران صريح بموافقة المضيف',
      ],
    },
    {
      id: 'control',
      title: 'التحكم والأتمتة',
      body: [
        'يعرض الخادم واجهة D-Bus على ناقل الجلسة وسطر أوامر. يغطي سطر الأوامر start وstop وdoctor وpair وdisplay وupdate، وتغطي واجهة D-Bus الحالة والدقة والإعدادات وسرد العملاء، بتفويض لمالك الخادم فقط.',
        'وتضيف واجهة المضيف الموثّقة القفل والتعطيم وإعادة الإظهار وCtrl+Alt+Del، ويعمل الخادم كوحدة systemd للمستخدم فيبدأ مع الجلسة.',
      ],
      points: [
        'سطر الأوامر: start وstop وdoctor وpair وdisplay وupdate',
        'D-Bus: الحالة والدقة والإعدادات والعملاء',
        'تحكم المضيف: قفل وتعطيم وإظهار وCtrl+Alt+Del',
        'وحدة systemd للمستخدم مضمّنة',
      ],
    },
    {
      id: 'security',
      title: 'الأمان والإصدارات الموقّعة',
      body: [
        'يُصادَق كل عميل برمز جلسة، ولا يستطيع عميل مقترن الارتباط بجلسة عميل آخر، ورسائل الإدخال محدودة الحجم. وتُقدَّم المسارات الحاملة للاعتمادات عبر TLS حين يملك المضيف شهادة.',
        'إصدارات Linux وAndroid موقّعة تشفيرياً، وينشر مسار الإصدار التوقيعات بجانب الملفات الثنائية.',
      ],
      points: [
        'رموز جلسة لكل عميل',
        'ملكية الجلسة مفروضة عند ارتباط البث',
        'TLS للمسارات الحاملة للاعتمادات',
        'ملفات Linux وAndroid موقّعة',
      ],
    },
  ],
  cta: {
    title: 'جاهز لإضافة شاشة؟',
    body: 'ثبّت المضيف، وافتح التطبيق أو عميل الويب، ووافق على طلب الاقتران. ويخبرك أمر doctor بما سيستخدمه سطح مكتبك بالضبط.',
    primary: 'ثبّت Orbiscreen',
    secondary: 'اقرأ التوثيق',
  },
};

const installEn: InstallPage = {
  eyebrow: 'INSTALLATION',
  title: 'Install in one command.',
  lede:
    'Pick the package for your distribution. Every method installs the same daemon, CLI, D-Bus service, and systemd user unit; only the delivery differs.',
  requirements: {
    title: 'Before you start',
    items: [
      'A Linux host on Wayland or X11, with GStreamer and the encoder plugins for your GPU.',
      'KDE Plasma, GNOME 46+, COSMIC, Sway/Hyprland, or an X11 desktop such as XFCE, MATE, LXQt, or Cinnamon.',
      'An Android tablet or phone on the same network, or a Chromium browser for the web client.',
      `Rust ${RUST_VERSION} or newer only if you build from source.`,
    ],
  },
  after: {
    title: 'After the install',
    steps: [
      {
        title: 'Check the plan',
        text: 'Run orbiscreen doctor. It prints the detected compositor, the capture backend, the virtual display path, and anything missing. Add --fix to install the EVDI module on detected distributions.',
      },
      {
        title: 'Start the daemon',
        text: 'Run orbiscreen start in the foreground for the first run, or enable the user unit with systemctl --user enable --now orbiscreen.',
      },
      {
        title: 'Connect the client',
        text: 'Open the Android app or the web client, pick the discovered host, and approve the pairing request with orbiscreen pair approve when the app files one.',
      },
      {
        title: 'Tune the display',
        text: 'Persist a size with orbiscreen display set 1920x1080@60, or pass --width, --height, and --fps to orbiscreen start.',
      },
    ],
  },
  platformTitle: 'What each desktop uses',
  platformLede:
    'The doctor command prints this table for your machine. Root is only needed where EVDI provides the virtual display.',
  cta: {
    title: 'Something not working?',
    body: 'The troubleshooting guide maps symptoms to fixes for capture, encoding, transport, pairing, input, and packaging.',
    primary: 'Open troubleshooting',
    secondary: 'Ask on GitHub',
  },
};

const installAr: InstallPage = {
  eyebrow: 'التثبيت',
  title: 'ثبّت بأمر واحد.',
  lede:
    'اختر حزمة توزيعتك. كل الطرق تثبّت الخادم نفسه وسطر الأوامر وخدمة D-Bus ووحدة systemd للمستخدم؛ يتغيّر التوصيل فقط.',
  requirements: {
    title: 'قبل أن تبدأ',
    items: [
      'مضيف Linux على Wayland أو X11، مع GStreamer وإضافات الترميز المناسبة لبطاقتك.',
      'KDE Plasma أو GNOME 46+ أو COSMIC أو Sway/Hyprland، أو بيئة X11 مثل XFCE أو MATE أو LXQt أو Cinnamon.',
      'تابلت أو هاتف Android على الشبكة نفسها، أو متصفح Chromium لعميل الويب.',
      `Rust ${RUST_VERSION} أو أحدث فقط إن بنيت من المصدر.`,
    ],
  },
  after: {
    title: 'بعد التثبيت',
    steps: [
      {
        title: 'افحص الخطة',
        text: 'شغّل orbiscreen doctor. يطبع مدير النوافذ المكتشف وواجهة الالتقاط ومسار الشاشة الافتراضية وكل ما ينقص. أضف --fix لتثبيت وحدة EVDI على التوزيعات المكتشفة.',
      },
      {
        title: 'ابدأ الخادم',
        text: 'شغّل orbiscreen start في المقدمة للتشغيل الأول، أو فعّل وحدة المستخدم بـ systemctl --user enable --now orbiscreen.',
      },
      {
        title: 'وصّل العميل',
        text: 'افتح تطبيق Android أو عميل الويب، واختر المضيف المكتشف، ووافق على طلب الاقتران بـ orbiscreen pair approve حين يقدّمه التطبيق.',
      },
      {
        title: 'اضبط الشاشة',
        text: 'احفظ دقة بـ orbiscreen display set 1920x1080@60، أو مرّر --width و--height و--fps إلى orbiscreen start.',
      },
    ],
  },
  platformTitle: 'ما تستخدمه كل بيئة',
  platformLede: 'يطبع أمر doctor هذا الجدول لجهازك. لا يُحتاج root إلا حين توفّر EVDI الشاشة الافتراضية.',
  cta: {
    title: 'شيء لا يعمل؟',
    body: 'يربط دليل حل الأعطال كل عَرَض بحلّه في الالتقاط والترميز والنقل والاقتران والإدخال والتغليف.',
    primary: 'افتح حل الأعطال',
    secondary: 'اسأل على GitHub',
  },
};

const compareEn: ComparePage = {
  eyebrow: 'COMPARISON',
  title: 'How Orbiscreen compares.',
  lede:
    'Second-screen tools differ mainly in whether they extend the desktop or mirror it, and whether they need root, a dummy HDMI plug, or a cloud service.',
  methodology: {
    title: 'How these numbers were measured',
    body: [
      'Latency figures are glass-to-glass on a 5 GHz Wi-Fi link with hardware encoding enabled, measured as the interval between a frame being presented on the host and the same frame being presented on the client. Transport delay is measured separately between host send and client assemble, and lands at 2 to 4 ms.',
      'The comparison table summarises each project\'s documented capabilities and its Linux host support. Where a project does not support Linux as a host, the row is marked accordingly rather than left blank.',
    ],
  },
  differences: [
    {
      title: 'Extended, not mirrored',
      body: 'Orbiscreen creates an independent virtual output. A dummy HDMI plug mirrors an existing screen; a compositor-native output adds a new one.',
    },
    {
      title: 'Rootless where the compositor allows it',
      body: 'KDE Plasma and wlroots create the output themselves. EVDI is the fallback for desktops that cannot, not the default.',
    },
    {
      title: 'Input, not just video',
      body: 'Touch, mouse, keyboard, and a 4095-level stylus come back as real Linux input events, so the tablet works as a drawing tablet too.',
    },
    {
      title: 'Local by design',
      body: 'Video and input stay between the host and the client on your network. There is no relay service and no account.',
    },
  ],
  cta: {
    title: 'Try it on your desktop',
    body: 'The doctor command tells you which path your compositor will use before you install anything else.',
    primary: 'Install Orbiscreen',
    secondary: 'Read the architecture',
  },
};

const compareAr: ComparePage = {
  eyebrow: 'المقارنة',
  title: 'كيف يُقارَن Orbiscreen.',
  lede:
    'تختلف أدوات الشاشة الثانية أساساً في كونها تمدّ سطح المكتب أو تنسخه، وفي حاجتها إلى root أو وصلة HDMI وهمية أو خدمة سحابية.',
  methodology: {
    title: 'كيف قيست هذه الأرقام',
    body: [
      'أرقام زمن الاستجابة من الزجاج إلى الزجاج على وصلة Wi-Fi بتردد 5 GHz مع ترميز عتادي، وتُقاس بالمدة بين عرض الإطار على المضيف وعرض الإطار نفسه على العميل. ويُقاس تأخير النقل منفصلاً بين إرسال المضيف وتجميع العميل، فيقع بين 2 و4 ms.',
      'يلخّص جدول المقارنة القدرات الموثّقة لكل مشروع ودعمه لمضيف Linux. وحيث لا يدعم المشروع Linux كمضيف تُعلَّم الخانة بذلك بدل تركها فارغة.',
    ],
  },
  differences: [
    {
      title: 'ممتدة لا منسوخة',
      body: 'ينشئ Orbiscreen مخرجاً افتراضياً مستقلاً. وصلة HDMI الوهمية تنسخ شاشة قائمة، والمخرج الأصلي من مدير النوافذ يضيف شاشة جديدة.',
    },
    {
      title: 'بلا root حيث يسمح مدير النوافذ',
      body: 'ينشئ KDE Plasma وwlroots المخرج بنفسيهما. وEVDI بديل للبيئات التي لا تستطيع، لا خيار افتراضي.',
    },
    {
      title: 'إدخال لا فيديو فقط',
      body: 'يعود اللمس والفأرة ولوحة المفاتيح وقلم بـ4095 مستوى كأحداث Linux حقيقية، فيعمل التابلت كلوح رسم أيضاً.',
    },
    {
      title: 'محلي بالتصميم',
      body: 'يبقى الفيديو والإدخال بين المضيف والعميل على شبكتك. لا خدمة تمرير ولا حساب.',
    },
  ],
  cta: {
    title: 'جرّبه على سطح مكتبك',
    body: 'يخبرك أمر doctor بالمسار الذي سيستخدمه مدير نوافذك قبل تثبيت أي شيء آخر.',
    primary: 'ثبّت Orbiscreen',
    secondary: 'اقرأ المعمارية',
  },
};

const faqEn: FaqPage = {
  eyebrow: 'FAQ',
  title: 'Questions, answered.',
  lede:
    'The short answers to what people ask before installing. Each answer names the component involved, so you can verify it in the source.',
  cta: {
    title: 'Still stuck?',
    body: 'The troubleshooting guide covers capture, encoding, transport, pairing, input, packaging, and CI failures symptom by symptom.',
    primary: 'Open troubleshooting',
    secondary: 'Open an issue',
  },
};

const faqAr: FaqPage = {
  eyebrow: 'الأسئلة',
  title: 'أسئلة وأجوبة.',
  lede: 'أجوبة قصيرة عمّا يُسأل قبل التثبيت. كل جواب يسمّي المكوّن المعني كي تتحقق منه في الشيفرة.',
  cta: {
    title: 'ما زلت عالقاً؟',
    body: 'يغطي دليل حل الأعطال الالتقاط والترميز والنقل والاقتران والإدخال والتغليف وأعطال CI عَرَضاً بعَرَض.',
    primary: 'افتح حل الأعطال',
    secondary: 'افتح مشكلة',
  },
};

export const pages: {
  features: Record<Lang, FeaturesPage>;
  install: Record<Lang, InstallPage>;
  compare: Record<Lang, ComparePage>;
  faq: Record<Lang, FaqPage>;
} = {
  features: { en: featuresEn, ar: featuresAr },
  install: { en: installEn, ar: installAr },
  compare: { en: compareEn, ar: compareAr },
  faq: { en: faqEn, ar: faqAr },
};

