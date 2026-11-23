// ─────────────────────────────────────────────
// Orbiscreen - Landing page content (EN / AR)
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import type { Lang } from '../lib/site';
import { RUST_VERSION, VERSION_TAG } from '../lib/version';

export type IconKey = 'display' | 'chip' | 'pen' | 'usb' | 'lock';

export interface Tile {
  icon: IconKey;
  title: string;
  body: string;
}

export interface Step {
  num: string;
  title: string;
  body: string;
  mono: string;
}

export interface Landing {
  hero: {
    pills: string[];
    title: string;
    sub: string;
    primary: string;
    secondary: string;
    trust: string;
  };
  capabilities: {
    eyebrow: string;
    title: string;
    tiles: Tile[];
  };
  howItWorks: {
    eyebrow: string;
    title: string;
    steps: Step[];
    footnote: string;
  };
  platform: {
    eyebrow: string;
    title: string;
    sub: string;
    doctorLabel: string;
    doctorCmd: string;
    doctorOut: string;
    rows: { name: string; status: string }[];
  };
  performance: {
    eyebrow: string;
    title: string;
    stats: { value: string; label: string }[];
  };
  install: {
    eyebrow: string;
    title: string;
    hostLabel: string;
    clientsLabel: string;
  };
  oss: {
    title: string;
    body: string;
    primary: string;
    secondary: string;
    stats: { key: string; value: string }[];
  };
}

const en: Landing = {
  hero: {
    pills: [VERSION_TAG, 'GPL-3.0', `Rust ${RUST_VERSION}+`],
    title: 'Your tablet is your second monitor.',
    sub: `Orbiscreen turns any Android tablet or phone into a real extended display for Linux. H.264 over LAN or USB, 25-40 ms latency, no root on KDE and wlroots.`,
    primary: 'Install Orbiscreen',
    secondary: 'View on GitHub',
    trust: 'Runs on KDE, GNOME, COSMIC, Sway and X11 · no root on KDE and wlroots',
  },
  capabilities: {
    eyebrow: 'Core capabilities',
    title: 'What it does.',
    tiles: [
      {
        icon: 'display',
        title: 'True extended display',
        body: 'A real virtual monitor up to 8K, created by your compositor on KDE Plasma and wlroots, or by EVDI elsewhere. Never a mirror.',
      },
      {
        icon: 'chip',
        title: 'Hardware H.264',
        body: 'NVENC, VA-API, or x264, tuned for LAN: CBR 8 Mbps, no B-frames, IDR every 120 frames.',
      },
      {
        icon: 'pen',
        title: 'Reverse input',
        body: 'Multi-touch, mouse, keyboard, and a 4095-level stylus your apps see as a real drawing tablet.',
      },
      {
        icon: 'usb',
        title: 'LAN or USB',
        body: 'UDP with Reed-Solomon FEC, WebTransport for browsers, USB AOA. No adb reverse required.',
      },
      {
        icon: 'lock',
        title: 'Rootless where it counts',
        body: 'No root and no share dialog on KDE Plasma and wlroots. A portal consent is all it asks.',
      },
    ],
  },
  howItWorks: {
    eyebrow: 'How it works',
    title: 'How it streams',
    steps: [
      {
        num: '01',
        title: 'Capture',
        body: 'Frames come from PipeWire on KWin and portals, wlr-screencopy on wlroots, or XShm on X11.',
        mono: 'pipewire · wlr-screencopy · XShm',
      },
      {
        num: '02',
        title: 'Encode',
        body: 'H.264 with hardware acceleration where available, tuned for low latency rather than bitrate.',
        mono: 'NVENC · VA-API · x264',
      },
      {
        num: '03',
        title: 'Transport',
        body: 'Packets cross the LAN with forward error correction, or travel over USB without any adb bridge.',
        mono: 'UDP+FEC · WebTransport · USB AOA',
      },
      {
        num: '04',
        title: 'Display & input',
        body: 'The app decodes, and every touch, key and pen stroke is injected back as real Linux input.',
        mono: 'Android · Web · uinput',
      },
    ],
    footnote: 'orbiscreen doctor  →  shows the detected compositor and capture plan',
  },
  platform: {
    eyebrow: 'Desktop support',
    title: 'Runs on your desktop.',
    sub: 'KDE Plasma, GNOME 46+, COSMIC, Sway and Hyprland, plus the classic X11 desktops. Each has its own capture path.',
    doctorLabel: '# check what your compositor can do',
    doctorCmd: '$ orbiscreen doctor',
    doctorOut: '✓ compositor: KWin (Wayland) · virtual display: portal · root: not needed',
    rows: [
      { name: 'KDE Plasma · Wayland', status: 'native · no root' },
      { name: 'COSMIC', status: 'EVDI · doctor --fix' },
      { name: 'Sway / Hyprland', status: 'headless via IPC · no root' },
      { name: 'GNOME 46+', status: 'Portal virtual / EVDI' },
      { name: 'XFCE / MATE / LXQt / Cinnamon', status: 'EVDI' },
    ],
  },
  performance: {
    eyebrow: 'Performance',
    title: 'Built for latency.',
    stats: [
      { value: '2-4 ms', label: 'transport delay' },
      { value: '25-40 ms', label: 'glass to glass' },
      { value: '4095', label: 'stylus pressure levels' },
      { value: '8K', label: 'max resolution' },
    ],
  },
  install: {
    eyebrow: 'Installation',
    title: 'Install in one command.',
    hostLabel: 'Host / desktop · packages',
    clientsLabel: 'Client apps · receiver',
  },
  oss: {
    title: 'Open source, end to end.',
    body: 'GPL-3.0, written in Rust. Every Linux and Android release artifact is cryptographically signed.',
    primary: 'Star on GitHub',
    secondary: 'Contributing guide',
    stats: [
      { key: 'License', value: 'GPL-3.0' },
      { key: 'Language', value: `Rust ${RUST_VERSION}+` },
      { key: 'Release', value: VERSION_TAG },
    ],
  },
};

const ar: Landing = {
  hero: {
    pills: [VERSION_TAG, 'GPL-3.0', `Rust ${RUST_VERSION}+`],
    title: 'تابلتك، شاشتك الثانية.',
    sub: 'يحول Orbiscreen أي جهاز Android إلى شاشة ممتدة حقيقية لـ Linux. ترميز H.264 عبر الشبكة أو USB، بزمن استجابة \u206625-40 ms\u2069، ودون صلاحيات root على KDE وwlroots.',
    primary: 'تثبيت Orbiscreen',
    secondary: 'شاهد على GitHub',
    trust: 'يعمل على KDE وGNOME وCOSMIC وSway وX11 · بلا root على KDE وwlroots',
  },
  capabilities: {
    eyebrow: 'القدرات الأساسية',
    title: 'ما يفعله.',
    tiles: [
      {
        icon: 'display',
        title: 'شاشة ممتدة حقيقية',
        body: 'شاشة افتراضية حتى 8K ينشئها مدير النوافذ على KDE Plasma وwlroots، أو عبر EVDI في غيرها. لا نسخة من شاشتك الأساسية.',
      },
      {
        icon: 'chip',
        title: 'ترميز H.264 عتادي',
        body: 'NVENC أو VA-API أو x264، مضبوط للشبكة المحلية: CBR 8 Mbps، وبلا إطارات B، وIDR كل 120 إطارا.',
      },
      {
        icon: 'pen',
        title: 'تحكم عكسي كامل',
        body: 'لمس متعدد وفأرة ولوحة مفاتيح وقلم بـ4095 مستوى ضغط، يراها النظام كلوح رسم حقيقي.',
      },
      {
        icon: 'usb',
        title: 'شبكة أو USB',
        body: 'UDP مع تصحيح Reed-Solomon، وWebTransport للمتصفحات، وUSB AOA دون adb reverse.',
      },
      {
        icon: 'lock',
        title: 'بلا root حيث يهم',
        body: 'بلا صلاحيات root وبلا نافذة مشاركة على KDE Plasma وwlroots. موافقة البوابة تكفي.',
      },
    ],
  },
  howItWorks: {
    eyebrow: 'كيف يعمل',
    title: 'كيف يبث',
    steps: [
      {
        num: '01',
        title: 'الالتقاط',
        body: 'تلتقط الإطارات من PipeWire على KWin والبوابات، أو wlr-screencopy على wlroots، أو XShm على X11.',
        mono: 'pipewire · wlr-screencopy · XShm',
      },
      {
        num: '02',
        title: 'الترميز',
        body: 'ترميز H.264 بعتاد متوفر، مضبوط لزمن استجابة أدنى لا لمعدل بتات أعلى.',
        mono: 'NVENC · VA-API · x264',
      },
      {
        num: '03',
        title: 'النقل',
        body: 'تعبر الحزم الشبكة المحلية مع تصحيح الأخطاء، أو تمر عبر USB دون أي جسر adb.',
        mono: 'UDP+FEC · WebTransport · USB AOA',
      },
      {
        num: '04',
        title: 'العرض والإدخال',
        body: 'يفك التطبيق الترميز، ويعاد كل لمس ومفتاح وخط قلم كإدخال Linux حقيقي.',
        mono: 'Android · Web · uinput',
      },
    ],
    footnote: 'orbiscreen doctor  ←  يعرض مدير النوافذ المكتشف وخطة الالتقاط',
  },
  platform: {
    eyebrow: 'دعم بيئات سطح المكتب',
    title: 'يعمل على سطح مكتبك.',
    sub: 'KDE Plasma وGNOME 46+ وCOSMIC وSway وHyprland، إضافة إلى بيئات X11 الكلاسيكية. لكل منها مسار التقاط خاص.',
    doctorLabel: '# افحص ما يستطيعه مدير النوافذ لديك',
    doctorCmd: '$ orbiscreen doctor',
    doctorOut: '✓ compositor: KWin (Wayland) · virtual display: portal · root: not needed',
    rows: [
      { name: 'KDE Plasma · Wayland', status: 'أصلي · بلا root' },
      { name: 'COSMIC', status: 'EVDI · doctor --fix' },
      { name: 'Sway / Hyprland', status: 'مخرج headless عبر IPC · بلا root' },
      { name: 'GNOME 46+', status: 'بوابة افتراضية / EVDI' },
      { name: 'XFCE / MATE / LXQt / Cinnamon', status: 'EVDI' },
    ],
  },
  performance: {
    eyebrow: 'الأداء',
    title: 'مبني لزمن استجابة أدنى.',
    stats: [
      { value: '2-4 ms', label: 'تأخير النقل' },
      { value: '25-40 ms', label: 'من الزجاج إلى الزجاج' },
      { value: '4095', label: 'مستوى ضغط للقلم' },
      { value: '8K', label: 'أقصى دقة' },
    ],
  },
  install: {
    eyebrow: 'التثبيت',
    title: 'ثبت بأمر واحد.',
    hostLabel: 'المضيف / سطح المكتب · الحزم',
    clientsLabel: 'تطبيقات العميل · المستقبل',
  },
  oss: {
    title: 'مفتوح المصدر من الأساس.',
    body: 'برخصة GPL-3.0 ومكتوب بـ Rust. كل إصدارات Linux وAndroid موقعة تشفيريا.',
    primary: 'أضف نجمة على GitHub',
    secondary: 'دليل المساهمة',
    stats: [
      { key: 'الرخصة', value: 'GPL-3.0' },
      { key: 'اللغة', value: `Rust ${RUST_VERSION}+` },
      { key: 'الإصدار', value: VERSION_TAG },
    ],
  },
};

export const landing: Record<Lang, Landing> = { en, ar };
