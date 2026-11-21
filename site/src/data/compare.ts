// ─────────────────────────────────────────────
// Orbiscreen - Comparison table (EN / AR)
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import type { Lang } from '../lib/site';

export interface CompareRow {
  capability: string;
  spacedesk: string;
  deskreen: string;
  weylus: string;
  sidecar: string;
  orbiscreen: string;
}

const en: CompareRow[] = [
  { capability: 'Linux host', spacedesk: 'No', deskreen: 'Web based', weylus: 'Web based', sidecar: 'No', orbiscreen: 'Yes' },
  { capability: 'Wayland and X11', spacedesk: 'No', deskreen: 'Dummy plug', weylus: 'Mirror only', sidecar: 'No', orbiscreen: 'Native' },
  { capability: 'Extended display', spacedesk: 'Windows only', deskreen: 'No', weylus: 'No', sidecar: 'Apple only', orbiscreen: 'Virtual monitor' },
  { capability: 'Native Android client', spacedesk: 'Yes', deskreen: 'Browser', weylus: 'Browser', sidecar: 'iPad', orbiscreen: 'Compose app' },
  { capability: 'Hardware encoding', spacedesk: 'Yes', deskreen: 'No', weylus: 'Partial', sidecar: 'Yes', orbiscreen: 'NVENC, VA-API' },
  { capability: 'Measured latency', spacedesk: '~50-80 ms', deskreen: '~150-300 ms', weylus: '~80-120 ms', sidecar: '~30 ms', orbiscreen: '~25-40 ms' },
  { capability: 'Stylus pressure and tilt', spacedesk: 'No', deskreen: 'No', weylus: 'Pen only', sidecar: 'Yes', orbiscreen: '4095 levels' },
  { capability: 'Reverse touch and mouse', spacedesk: 'Yes', deskreen: 'No', weylus: 'Pen only', sidecar: 'Yes', orbiscreen: 'Multi-touch' },
  { capability: 'Rootless on KDE, wlroots', spacedesk: 'N/A', deskreen: 'Yes', weylus: 'No', sidecar: 'N/A', orbiscreen: 'Yes' },
  { capability: 'Open source', spacedesk: 'No', deskreen: 'GPL-3.0', weylus: 'AGPL-3.0', sidecar: 'No', orbiscreen: 'GPL-3.0' },
];

const ar: CompareRow[] = [
  { capability: 'مضيف Linux', spacedesk: 'لا', deskreen: 'عبر المتصفح', weylus: 'عبر المتصفح', sidecar: 'لا', orbiscreen: 'نعم' },
  { capability: 'Wayland وX11', spacedesk: 'لا', deskreen: 'وصلة وهمية', weylus: 'نسخ الشاشة فقط', sidecar: 'لا', orbiscreen: 'أصلي' },
  { capability: 'شاشة ممتدة', spacedesk: 'Windows فقط', deskreen: 'لا', weylus: 'لا', sidecar: 'Apple فقط', orbiscreen: 'شاشة افتراضية' },
  { capability: 'تطبيق Android أصلي', spacedesk: 'نعم', deskreen: 'متصفح', weylus: 'متصفح', sidecar: 'iPad', orbiscreen: 'تطبيق Compose' },
  { capability: 'ترميز عتادي', spacedesk: 'نعم', deskreen: 'لا', weylus: 'جزئي', sidecar: 'نعم', orbiscreen: 'NVENC وVA-API' },
  { capability: 'زمن الاستجابة المقاس', spacedesk: '~50-80 ms', deskreen: '~150-300 ms', weylus: '~80-120 ms', sidecar: '~30 ms', orbiscreen: '~25-40 ms' },
  { capability: 'ضغط القلم وميله', spacedesk: 'لا', deskreen: 'لا', weylus: 'القلم فقط', sidecar: 'نعم', orbiscreen: '4095 مستوى' },
  { capability: 'لمس وفأرة عكسيان', spacedesk: 'نعم', deskreen: 'لا', weylus: 'القلم فقط', sidecar: 'نعم', orbiscreen: 'لمس متعدد' },
  { capability: 'بلا root على KDE وwlroots', spacedesk: 'لا ينطبق', deskreen: 'نعم', weylus: 'لا', sidecar: 'لا ينطبق', orbiscreen: 'نعم' },
  { capability: 'مفتوح المصدر', spacedesk: 'لا', deskreen: 'GPL-3.0', weylus: 'AGPL-3.0', sidecar: 'لا', orbiscreen: 'GPL-3.0' },
];

export const comparison: Record<Lang, CompareRow[]> = { en, ar };

