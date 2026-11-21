// ─────────────────────────────────────────────
// Orbiscreen - Desktop support matrix (EN / AR)
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import type { Lang } from '../lib/site';

export interface PlatformRow {
  name: string;
  virtual: string;
  capture: string;
  input: string;
  root: 'no' | 'optional' | 'yes';
}

const en: PlatformRow[] = [
  {
    name: 'KDE Plasma (Wayland)',
    virtual: 'Native portal virtual output or zkde-screencast',
    capture: 'PipeWire',
    input: 'RemoteDesktop portal, uinput',
    root: 'no',
  },
  {
    name: 'COSMIC (Wayland)',
    virtual: 'EVDI kernel display',
    capture: 'Portal ScreenCast (PipeWire)',
    input: 'uinput, RemoteDesktop portal',
    root: 'optional',
  },
  {
    name: 'Sway / Hyprland / wlroots',
    virtual: 'Headless output over compositor IPC',
    capture: 'wlr-screencopy',
    input: 'virtual-pointer, virtual-keyboard',
    root: 'no',
  },
  {
    name: 'GNOME (Wayland)',
    virtual: 'Portal virtual monitor (GNOME 46+) or EVDI',
    capture: 'Portal ScreenCast (PipeWire)',
    input: 'RemoteDesktop portal, persisted',
    root: 'optional',
  },
  {
    name: 'XFCE / MATE / LXQt / Cinnamon (X11)',
    virtual: 'EVDI kernel display',
    capture: 'XShm root mirror',
    input: 'XTEST, uinput',
    root: 'optional',
  },
  {
    name: 'Anything else',
    virtual: 'EVDI kernel display',
    capture: 'Best available backend',
    input: 'Best available backend',
    root: 'optional',
  },
];

const ar: PlatformRow[] = [
  {
    name: 'KDE Plasma (Wayland)',
    virtual: 'مخرج افتراضي أصلي عبر البوابة أو zkde-screencast',
    capture: 'PipeWire',
    input: 'بوابة RemoteDesktop وuinput',
    root: 'no',
  },
  {
    name: 'COSMIC (Wayland)',
    virtual: 'شاشة نواة EVDI',
    capture: 'بوابة ScreenCast (PipeWire)',
    input: 'uinput وبوابة RemoteDesktop',
    root: 'optional',
  },
  {
    name: 'Sway / Hyprland / wlroots',
    virtual: 'مخرج headless عبر IPC مدير النوافذ',
    capture: 'wlr-screencopy',
    input: 'virtual-pointer وvirtual-keyboard',
    root: 'no',
  },
  {
    name: 'GNOME (Wayland)',
    virtual: 'شاشة افتراضية عبر البوابة (GNOME 46+) أو EVDI',
    capture: 'بوابة ScreenCast (PipeWire)',
    input: 'بوابة RemoteDesktop مع حفظ الإذن',
    root: 'optional',
  },
  {
    name: 'XFCE / MATE / LXQt / Cinnamon (X11)',
    virtual: 'شاشة نواة EVDI',
    capture: 'XShm لنسخ الجذر',
    input: 'XTEST وuinput',
    root: 'optional',
  },
  {
    name: 'أي بيئة أخرى',
    virtual: 'شاشة نواة EVDI',
    capture: 'أفضل واجهة متاحة',
    input: 'أفضل واجهة متاحة',
    root: 'optional',
  },
];

export const platforms: Record<Lang, PlatformRow[]> = { en, ar };

export const rootLabels: Record<Lang, Record<PlatformRow['root'], string>> = {
  en: { no: 'No root', optional: 'Root only for EVDI', yes: 'Root required' },
  ar: { no: 'بلا root', optional: 'root لـ EVDI فقط', yes: 'يتطلب root' },
};
