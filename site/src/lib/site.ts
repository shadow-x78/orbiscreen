// ─────────────────────────────────────────────
// Orbiscreen - Site constants
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

export const SITE_URL = 'https://orbiscreen.com';
export const SITE_NAME = 'Orbiscreen';
export const SITE_LOCALES = ['en', 'ar'] as const;
export type Lang = (typeof SITE_LOCALES)[number];

export const SHORT_DESCRIPTION =
  'Orbiscreen turns an Android tablet or phone into a real extended display for Linux, with H.264 over LAN or USB and no root on KDE and wlroots.';

export const GITHUB_URL = 'https://github.com/shadow-x78/orbiscreen';
export const RELEASES_URL = `${GITHUB_URL}/releases`;
export const LATEST_RELEASE_URL = `${GITHUB_URL}/releases/latest`;
export const ISSUES_URL = `${GITHUB_URL}/issues`;
export const ADVISORY_URL = `${GITHUB_URL}/security/advisories/new`;
export const PPA_URL = 'https://launchpad.net/~shadow-x78/+archive/ubuntu/ppa';
export const COPR_URL = 'https://copr.fedorainfracloud.org/coprs/shadow-x78/orbiscreen/';
export const PLAY_URL = 'https://play.google.com/store/apps/details?id=com.orbiscreen.app';
export const SECURITY_EMAIL = 'shadow-x7@red-shadows-rs.com';

export const OG_IMAGE = '/og/orbiscreen.png';
export const OG_IMAGE_WIDTH = 1280;
export const OG_IMAGE_HEIGHT = 640;
