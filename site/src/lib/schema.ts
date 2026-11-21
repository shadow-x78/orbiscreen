// ─────────────────────────────────────────────
// Orbiscreen - JSON-LD builders
// https://github.com/shadow-x78/orbiscreen
// ─────────────────────────────────────────────

import {
  GITHUB_URL,
  OG_IMAGE,
  RELEASES_URL,
  SITE_NAME,
  SITE_URL,
  SHORT_DESCRIPTION,
} from './site';
import { VERSION } from './version';

const ORG_ID = `${SITE_URL}/#organization`;
const SITE_ID = `${SITE_URL}/#website`;
const APP_ID = `${SITE_URL}/#software`;

export function organizationSchema() {
  return {
    '@type': 'Organization',
    '@id': ORG_ID,
    name: SITE_NAME,
    url: `${SITE_URL}/`,
    logo: {
      '@type': 'ImageObject',
      url: `${SITE_URL}/icon-512.png`,
      width: 512,
      height: 512,
    },
    sameAs: [GITHUB_URL],
  };
}

export function websiteSchema() {
  return {
    '@type': 'WebSite',
    '@id': SITE_ID,
    url: `${SITE_URL}/`,
    name: SITE_NAME,
    description: SHORT_DESCRIPTION,
    inLanguage: ['en', 'ar'],
    publisher: { '@id': ORG_ID },
  };
}

export const APP_FEATURES = [
  'Independent virtual display up to 7680x4320',
  'Hardware H.264 encoding through NVENC, VA-API or x264',
  'Reverse multi-touch, mouse, keyboard and 4095-level stylus input',
  'UDP Annex-B transport with Reed-Solomon forward error correction',
  'WebTransport and WebCodecs browser client',
  'USB Android Open Accessory transport without adb',
  'mDNS discovery and session-token pairing',
  'D-Bus session interface and CLI control',
  'Rootless operation on KDE Plasma and wlroots',
];

export function softwareApplicationSchema(features: string[] = APP_FEATURES) {
  return {
    '@type': 'SoftwareApplication',
    '@id': APP_ID,
    name: SITE_NAME,
    url: `${SITE_URL}/`,
    description: SHORT_DESCRIPTION,
    applicationCategory: 'UtilitiesApplication',
    applicationSubCategory: 'Remote desktop and virtual display',
    operatingSystem: 'Linux, Android',
    softwareVersion: VERSION,
    softwareRequirements:
      'Linux host with GStreamer; KDE Plasma, GNOME 46+, COSMIC, Sway/Hyprland or X11; Android 8+ or a Chromium browser for the client.',
    license: 'https://www.gnu.org/licenses/gpl-3.0.html',
    isAccessibleForFree: true,
    offers: {
      '@type': 'Offer',
      price: '0',
      priceCurrency: 'USD',
    },
    downloadUrl: RELEASES_URL,
    installUrl: `${SITE_URL}/install`,
    codeRepository: GITHUB_URL,
    image: `${SITE_URL}${OG_IMAGE}`,
    screenshot: `${SITE_URL}${OG_IMAGE}`,
    featureList: features,
    inLanguage: ['en', 'ar'],
    publisher: { '@id': ORG_ID },
  };
}

export function breadcrumbSchema(items: { name: string; url: string }[]) {
  return {
    '@type': 'BreadcrumbList',
    itemListElement: items.map((item, index) => ({
      '@type': 'ListItem',
      position: index + 1,
      name: item.name,
      item: item.url,
    })),
  };
}

export function webPageSchema(input: {
  name: string;
  description: string;
  url: string;
  lang: 'en' | 'ar';
  type?: 'WebPage' | 'CollectionPage' | 'AboutPage';
}) {
  return {
    '@type': input.type ?? 'WebPage',
    '@id': `${input.url}#page`,
    url: input.url,
    name: input.name,
    description: input.description,
    inLanguage: input.lang,
    isPartOf: { '@id': SITE_ID },
    about: { '@id': APP_ID },
    publisher: { '@id': ORG_ID },
  };
}

export function faqSchema(items: { question: string; answer: string[] }[]) {
  return {
    '@type': 'FAQPage',
    mainEntity: items.map((item) => ({
      '@type': 'Question',
      name: item.question,
      acceptedAnswer: {
        '@type': 'Answer',
        text: item.answer.map((part) => part.replace(/<[^>]+>/g, '')).join(' '),
      },
    })),
  };
}

export function howToSchema(input: {
  name: string;
  description: string;
  url: string;
  lang: 'en' | 'ar';
  steps: { name: string; text: string }[];
}) {
  return {
    '@type': 'HowTo',
    name: input.name,
    description: input.description,
    inLanguage: input.lang,
    totalTime: 'PT3M',
    tool: [{ '@type': 'HowToTool', name: 'Terminal' }],
    step: input.steps.map((step, index) => ({
      '@type': 'HowToStep',
      position: index + 1,
      name: step.name,
      text: step.text,
      url: `${input.url}#${step.name.toLowerCase().replace(/[^a-z0-9]+/g, '-')}`,
    })),
  };
}

export function techArticleSchema(input: {
  headline: string;
  description: string;
  url: string;
  lang: 'en' | 'ar';
  dateModified: string;
}) {
  return {
    '@type': 'TechArticle',
    headline: input.headline,
    description: input.description,
    url: input.url,
    mainEntityOfPage: input.url,
    inLanguage: input.lang,
    dateModified: input.dateModified,
    proficiencyLevel: 'Expert',
    author: { '@id': ORG_ID },
    publisher: { '@id': ORG_ID },
    isPartOf: { '@id': SITE_ID },
    about: { '@id': APP_ID },
  };
}