// ─────────────────────────────────────────────
// Orbiscreen - Install methods (EN / AR)
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import type { Lang } from '../lib/site';
import { COPR_URL, PPA_URL, RELEASES_URL } from '../lib/site';

export interface CodeLine {
  kind: 'comment' | 'cmd';
  text: string;
}

export interface InstallMethod {
  id: string;
  label: string;
  lines: CodeLine[];
  note: string;
  action?: { label: string; href: string };
}

const en: InstallMethod[] = [
  {
    id: 'ubuntu',
    label: 'Ubuntu / Mint',
    lines: [
      { kind: 'comment', text: '# Ubuntu / Pop!_OS / Linux Mint' },
      { kind: 'cmd', text: 'sudo add-apt-repository ppa:shadow-x78/ppa -y' },
      { kind: 'cmd', text: 'sudo apt update && sudo apt install orbiscreen -y' },
      { kind: 'cmd', text: 'orbiscreen start' },
    ],
    note: 'The PPA tracks every release for the current Ubuntu LTS and the one before it.',
    action: { label: 'Open the PPA', href: PPA_URL },
  },
  {
    id: 'fedora',
    label: 'Fedora',
    lines: [
      { kind: 'comment', text: '# Fedora (COPR)' },
      { kind: 'cmd', text: 'sudo dnf copr enable shadow-x78/orbiscreen -y' },
      { kind: 'cmd', text: 'sudo dnf install orbiscreen -y' },
      { kind: 'cmd', text: 'orbiscreen start' },
    ],
    note: 'The COPR repository also builds for RHEL and CentOS Stream derivatives.',
    action: {
      label: 'Open the COPR repository',
      href: COPR_URL,
    },
  },
  {
    id: 'arch',
    label: 'Arch',
    lines: [
      { kind: 'comment', text: '# Arch Linux / Manjaro' },
      { kind: 'cmd', text: 'git clone https://github.com/shadow-x78/orbiscreen.git' },
      { kind: 'cmd', text: 'cd orbiscreen' },
      { kind: 'cmd', text: 'makepkg -si' },
      { kind: 'cmd', text: 'orbiscreen start' },
    ],
    note: 'The PKGBUILD builds from the tagged source archive and installs the systemd user unit.',
    action: {
      label: 'PKGBUILD on GitHub',
      href: 'https://github.com/shadow-x78/orbiscreen/blob/main/PKGBUILD',
    },
  },
  {
    id: 'appimage',
    label: 'AppImage',
    lines: [
      { kind: 'comment', text: '# Download orbiscreen-<version>-x86_64.AppImage first' },
      { kind: 'cmd', text: 'chmod +x orbiscreen-*-x86_64.AppImage' },
      { kind: 'cmd', text: './orbiscreen-*-x86_64.AppImage' },
    ],
    note: 'AppImage assets are named with the release version, so the exact file name changes every release.',
    action: { label: 'All releases', href: RELEASES_URL },
  },
  {
    id: 'source',
    label: 'From source',
    lines: [
      { kind: 'comment', text: '# Requires a Rust toolchain and GStreamer' },
      { kind: 'cmd', text: 'git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen' },
      { kind: 'cmd', text: 'cd ~/Orbiscreen && ./scripts/install.sh' },
      { kind: 'cmd', text: 'orbiscreen start' },
    ],
    note: 'scripts/install.sh builds the workspace, installs the D-Bus service, and can build the Qt desktop app.',
    action: { label: 'Read the source', href: 'https://github.com/shadow-x78/orbiscreen' },
  },
  {
    id: 'android',
    label: 'Android',
    lines: [
      { kind: 'comment', text: '# Install orbiscreen-android-release.apk from GitHub Releases' },
      { kind: 'cmd', text: 'orbiscreen doctor   # prints the session token and the client URL' },
    ],
    note: 'The Android client is a Material 3 Compose app. The browser client needs no install at all.',
    action: { label: 'Android APK', href: RELEASES_URL },
  },
];

const ar: InstallMethod[] = [
  {
    id: 'ubuntu',
    label: 'أوبونتو / منت',
    lines: [
      { kind: 'comment', text: '# أوبونتو / Pop!_OS / Linux Mint' },
      { kind: 'cmd', text: 'sudo add-apt-repository ppa:shadow-x78/ppa -y' },
      { kind: 'cmd', text: 'sudo apt update && sudo apt install orbiscreen -y' },
      { kind: 'cmd', text: 'orbiscreen start' },
    ],
    note: 'يتابع مستودع PPA كل إصدار لإصدار Ubuntu LTS الحالي والذي قبله.',
    action: { label: 'افتح PPA', href: PPA_URL },
  },
  {
    id: 'fedora',
    label: 'فيدورا',
    lines: [
      { kind: 'comment', text: '# Fedora (COPR)' },
      { kind: 'cmd', text: 'sudo dnf copr enable shadow-x78/orbiscreen -y' },
      { kind: 'cmd', text: 'sudo dnf install orbiscreen -y' },
      { kind: 'cmd', text: 'orbiscreen start' },
    ],
    note: 'يبني مستودع COPR أيضاً لتوزيعات RHEL وCentOS Stream.',
    action: {
      label: 'افتح مستودع COPR',
      href: COPR_URL,
    },
  },
  {
    id: 'arch',
    label: 'آرتش',
    lines: [
      { kind: 'comment', text: '# Arch Linux / Manjaro' },
      { kind: 'cmd', text: 'git clone https://github.com/shadow-x78/orbiscreen.git' },
      { kind: 'cmd', text: 'cd orbiscreen' },
      { kind: 'cmd', text: 'makepkg -si' },
      { kind: 'cmd', text: 'orbiscreen start' },
    ],
    note: 'يبني PKGBUILD من أرشيف المصدر الموسوم ويثبّت وحدة systemd للمستخدم.',
    action: {
      label: 'PKGBUILD على GitHub',
      href: 'https://github.com/shadow-x78/orbiscreen/blob/main/PKGBUILD',
    },
  },
  {
    id: 'appimage',
    label: 'AppImage',
    lines: [
      { kind: 'comment', text: '# نزّل orbiscreen-<version>-x86_64.AppImage أولاً' },
      { kind: 'cmd', text: 'chmod +x orbiscreen-*-x86_64.AppImage' },
      { kind: 'cmd', text: './orbiscreen-*-x86_64.AppImage' },
    ],
    note: 'يحمل ملف AppImage رقم الإصدار في اسمه، لذا يتغيّر الاسم مع كل إصدار.',
    action: { label: 'كل الإصدارات', href: RELEASES_URL },
  },
  {
    id: 'source',
    label: 'من المصدر',
    lines: [
      { kind: 'comment', text: '# يتطلب Rust وGStreamer' },
      { kind: 'cmd', text: 'git clone https://github.com/shadow-x78/orbiscreen.git ~/Orbiscreen' },
      { kind: 'cmd', text: 'cd ~/Orbiscreen && ./scripts/install.sh' },
      { kind: 'cmd', text: 'orbiscreen start' },
    ],
    note: 'يبني scripts/install.sh مساحة العمل، ويثبّت خدمة D-Bus، ويستطيع بناء تطبيق Qt لسطح المكتب.',
    action: { label: 'اقرأ الشيفرة', href: 'https://github.com/shadow-x78/orbiscreen' },
  },
  {
    id: 'android',
    label: 'أندرويد',
    lines: [
      { kind: 'comment', text: '# ثبّت orbiscreen-android-release.apk من GitHub Releases' },
      { kind: 'cmd', text: 'orbiscreen doctor   # prints the session token and the client URL' },
    ],
    note: 'تطبيق Android مبني بـ Material 3 Compose، وعميل المتصفح لا يحتاج تثبيتاً إطلاقاً.',
    action: { label: 'Android APK', href: RELEASES_URL },
  },
];

export const installMethods: Record<Lang, InstallMethod[]> = { en, ar };

export interface DownloadButton {
  icon: string;
  micro: string;
  main: string;
  href: string;
  external: boolean;
}

const hostEn: DownloadButton[] = [
  { icon: 'github', micro: 'All packages', main: 'GitHub Releases', href: RELEASES_URL, external: true },
  {
    icon: 'terminal',
    micro: 'Ubuntu / Debian',
    main: 'APT Repository',
    href: PPA_URL,
    external: true,
  },
  {
    icon: 'terminal',
    micro: 'Fedora / RHEL',
    main: 'COPR Repository',
    href: COPR_URL,
    external: true,
  },
  { icon: 'terminal', micro: 'Arch / Manjaro', main: 'PKGBUILD', href: `${RELEASES_URL}/latest`, external: true },
  { icon: 'display', micro: 'Portable binary', main: 'AppImage', href: RELEASES_URL, external: true },
  { icon: 'chip', micro: 'Cargo / Git', main: 'Build from source', href: 'https://github.com/shadow-x78/orbiscreen', external: true },
];

const hostAr: DownloadButton[] = [
  { icon: 'github', micro: 'كل الحزم', main: 'GitHub Releases', href: RELEASES_URL, external: true },
  {
    icon: 'terminal',
    micro: 'أوبونتو / ديبيان',
    main: 'APT Repository',
    href: PPA_URL,
    external: true,
  },
  {
    icon: 'terminal',
    micro: 'فيدورا / RHEL',
    main: 'COPR Repository',
    href: COPR_URL,
    external: true,
  },
  { icon: 'terminal', micro: 'آرتش / مانجارو', main: 'PKGBUILD', href: `${RELEASES_URL}/latest`, external: true },
  { icon: 'display', micro: 'ملف محمول', main: 'AppImage', href: RELEASES_URL, external: true },
  { icon: 'chip', micro: 'كارجو / GIT', main: 'البناء من المصدر', href: 'https://github.com/shadow-x78/orbiscreen', external: true },
];

const clientEn: DownloadButton[] = [
  { icon: 'android', micro: 'Direct download', main: 'Android APK', href: `${RELEASES_URL}/latest`, external: true },
  { icon: 'globe', micro: 'No install', main: 'Web Client', href: '/features#web-client', external: false },
];

const clientAr: DownloadButton[] = [
  { icon: 'android', micro: 'تحميل مباشر', main: 'Android APK', href: `${RELEASES_URL}/latest`, external: true },
  { icon: 'globe', micro: 'دون تثبيت', main: 'Web Client', href: '/ar/features#web-client', external: false },
];

export const downloads: Record<Lang, { host: DownloadButton[]; clients: DownloadButton[] }> = {
  en: { host: hostEn, clients: clientEn },
  ar: { host: hostAr, clients: clientAr },
};
