// ─────────────────────────────────────────────
// Orbiscreen - Frequently asked questions (EN / AR)
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import type { Lang } from '../lib/site';

export interface FaqItem {
  id: string;
  question: string;
  answer: string[];
}

const en: FaqItem[] = [
  {
    id: 'extended-or-mirror',
    question: 'Is this a true extended display or a mirror?',
    answer: [
      'It is an independent virtual monitor placed next to your physical displays. Windows can be moved onto it, and the resolution is configurable up to 7680x4320 (8K).',
      'Nothing is mirrored: the tablet shows only the virtual output, so your main screen keeps its own layout and refresh rate.',
    ],
  },
  {
    id: 'wayland-without-root',
    question: 'Does it work on Wayland without root?',
    answer: [
      'On KDE Plasma, wlroots compositors, and GNOME 46+ the virtual monitor is created by the compositor or its portal, with no root and no kernel module. A portal consent is the only prompt.',
      'Older GNOME, COSMIC, and X11 fall back to the EVDI kernel module. <code>orbiscreen doctor --fix</code> installs it on detected distributions.',
    ],
  },
  {
    id: 'stylus',
    question: 'Can I draw in Krita or GIMP with a stylus?',
    answer: [
      'Yes. Stylus pressure (up to 4095 levels), tilt, and in-air hover are injected as a Linux input tablet through <code>uinput</code>, so applications see a real drawing device rather than a mouse.',
    ],
  },
  {
    id: 'usb',
    question: 'Can I connect over USB instead of Wi-Fi?',
    answer: [
      'Plug in the cable and accept the accessory permission dialog. The host switches the device to Android Open Accessory mode and streams H.264 over USB bulk endpoints.',
      'USB debugging and <code>adb reverse</code> are not required.',
    ],
  },
  {
    id: 'latency',
    question: 'What latency should I expect?',
    answer: [
      'On a 5 GHz network with hardware encoding, transport delay measures 2 to 4 ms between host send and client assemble. End to end (glass to glass) is typically 25 to 40 ms on a tuned Wi-Fi connection.',
      'USB removes the network from the path entirely. Wired Ethernet on the host side removes the usual source of jitter.',
    ],
  },
  {
    id: 'desktops',
    question: 'Which Linux desktops are supported?',
    answer: [
      'KDE Plasma, GNOME 46+, COSMIC, Sway, Hyprland and other wlroots compositors, plus XFCE, MATE, LXQt and Cinnamon on X11. Each has its own capture path, and <code>orbiscreen doctor</code> prints the plan it will use on your machine.',
    ],
  },
  {
    id: 'clients',
    question: 'What do I need on the tablet?',
    answer: [
      'Either the Android app (Material 3, installed from GitHub Releases) or a Chromium browser using the built-in web client over WebTransport and WebCodecs. The browser client needs no install.',
    ],
  },
  {
    id: 'token',
    question: 'How do I find the session token?',
    answer: [
      'The host advertises the token over mDNS and the app offers a pairing request you can approve with <code>orbiscreen pair approve</code>. Running <code>orbiscreen doctor</code> prints the active token and the client URL as a fallback.',
    ],
  },
  {
    id: 'black-screen',
    question: 'Why is the picture black on COSMIC?',
    answer: [
      'COSMIC needs the EVDI kernel module for the virtual output; run <code>orbiscreen doctor --fix</code> once. If frames reach the encoder but the picture stays black, the daemon logs a capture probe and an encoder probe every five seconds that name the stage that fails.',
    ],
  },
  {
    id: 'multiple-clients',
    question: 'Can several devices connect at once?',
    answer: [
      'Yes, up to the configured client limit (8 by default for both the HTTP and UDP paths). Each client is authenticated separately, and a paired client cannot attach to another client\'s session.',
    ],
  },
  {
    id: 'price',
    question: 'Is it free?',
    answer: [
      'Yes. Orbiscreen is distributed under GPL-3.0-or-later, there is no paid tier, no account, and no telemetry. Every Linux and Android release artifact is cryptographically signed.',
    ],
  },
  {
    id: 'privacy',
    question: 'Does it send anything over the internet?',
    answer: [
      'No. The daemon listens on your local network and the video path stays between the host and the client. The update check is a single request you can disable with <code>ORBISCREEN_NO_UPDATE_CHECK=1</code>.',
    ],
  },
];

const ar: FaqItem[] = [
  {
    id: 'extended-or-mirror',
    question: 'هل هذه شاشة ممتدة حقيقية أم نسخة من الشاشة؟',
    answer: [
      'هي شاشة افتراضية مستقلة تُوضع بجانب شاشاتك المادية. يمكنك نقل النوافذ إليها، ودقتها قابلة للضبط حتى 7680x4320 (8K).',
      'لا شيء يُنسخ: يعرض التابلت المخرج الافتراضي فقط، فتبقى شاشتك الأساسية بتخطيطها ومعدل تحديثها.',
    ],
  },
  {
    id: 'wayland-without-root',
    question: 'هل يعمل على Wayland دون root؟',
    answer: [
      'على KDE Plasma ومديري نوافذ wlroots وGNOME 46+ ينشئ المدير أو بوابته الشاشة الافتراضية، بلا root وبلا وحدة نواة. موافقة البوابة هي الطلب الوحيد.',
      'أما GNOME الأقدم وCOSMIC وX11 فتعتمد على وحدة نواة EVDI. ويثبّتها <code>orbiscreen doctor --fix</code> على التوزيعات المكتشفة.',
    ],
  },
  {
    id: 'stylus',
    question: 'هل أرسم في Krita أو GIMP بالقلم؟',
    answer: [
      'نعم. ضغط القلم (حتى 4095 مستوى) وميله وطوفانه في الهواء تُحقن كلوح إدخال Linux عبر <code>uinput</code>، فترى التطبيقات جهاز رسم حقيقياً لا فأرة.',
    ],
  },
  {
    id: 'usb',
    question: 'هل أتصل عبر USB بدل الشبكة؟',
    answer: [
      'وصّل الكابل واقبل نافذة إذن الملحق. يحوّل المضيف الجهاز إلى وضع Android Open Accessory ويبث H.264 عبر منافذ USB.',
      'لا حاجة إلى تصحيح USB ولا إلى <code>adb reverse</code>.',
    ],
  },
  {
    id: 'latency',
    question: 'ما زمن الاستجابة المتوقع؟',
    answer: [
      'على شبكة 5 GHz بترميز عتادي يقاس تأخير النقل بين 2 و4 ms من إرسال المضيف إلى تجميع العميل. الزمن الكلي من الزجاج إلى الزجاج يقع عادة بين 25 و40 ms على اتصال Wi-Fi مضبوط.',
      'يلغي USB الشبكة من المسار تماماً، وEthernet على جهة المضيف يزيل المصدر المعتاد للتذبذب.',
    ],
  },
  {
    id: 'desktops',
    question: 'أي بيئات سطح مكتب Linux مدعومة؟',
    answer: [
      'KDE Plasma وGNOME 46+ وCOSMIC وSway وHyprland وسائر مديري wlroots، إضافة إلى XFCE وMATE وLXQt وCinnamon على X11. لكل بيئة مسار التقاط خاص، ويطبع <code>orbiscreen doctor</code> الخطة التي سيستخدمها على جهازك.',
    ],
  },
  {
    id: 'clients',
    question: 'ماذا أحتاج على التابلت؟',
    answer: [
      'إما تطبيق Android (Material 3، من GitHub Releases) أو متصفح Chromium عبر عميل الويب المدمج بتقنيتي WebTransport وWebCodecs. عميل المتصفح لا يحتاج تثبيتاً.',
    ],
  },
  {
    id: 'token',
    question: 'كيف أجد رمز الجلسة؟',
    answer: [
      'يعلن المضيف الرمز عبر mDNS، ويعرض التطبيق طلب اقتران توافق عليه بـ<code>orbiscreen pair approve</code>. ويطبع <code>orbiscreen doctor</code> الرمز النشط وعنوان العميل كبديل.',
    ],
  },
  {
    id: 'black-screen',
    question: 'لماذا تظهر الصورة سوداء على COSMIC؟',
    answer: [
      'يحتاج COSMIC وحدة نواة EVDI للمخرج الافتراضي؛ شغّل <code>orbiscreen doctor --fix</code> مرة واحدة. وإذا وصلت الإطارات إلى المُرمِّز وبقيت الصورة سوداء، يسجّل الخادم كل خمس ثوانٍ فحصاً للالتقاط وفحصاً للترميز يسمّيان المرحلة التي تفشل.',
    ],
  },
  {
    id: 'multiple-clients',
    question: 'هل يمكن اتصال عدة أجهزة في وقت واحد؟',
    answer: [
      'نعم، حتى الحد المضبوط للعملاء (8 افتراضياً لمساري HTTP وUDP). يُصادَق كل عميل على حدة، ولا يستطيع عميل مقترن الارتباط بجلسة عميل آخر.',
    ],
  },
  {
    id: 'price',
    question: 'هل هو مجاني؟',
    answer: [
      'نعم. يُوزَّع Orbiscreen برخصة GPL-3.0-or-later، بلا طبقة مدفوعة ولا حساب ولا قياسات. وكل إصدارات Linux وAndroid موقّعة تشفيرياً.',
    ],
  },
  {
    id: 'privacy',
    question: 'هل يرسل شيئاً عبر الإنترنت؟',
    answer: [
      'لا. يستمع الخادم على شبكتك المحلية ويبقى مسار الفيديو بين المضيف والعميل. وفحص التحديث طلب واحد يمكن تعطيله بـ<code>ORBISCREEN_NO_UPDATE_CHECK=1</code>.',
    ],
  },
];

export const faq: Record<Lang, FaqItem[]> = { en, ar };
